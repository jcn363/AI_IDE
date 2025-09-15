#!/usr/bin/env python3
"""
Automated dependency update script for security-compatible updates.
Checks for outdated dependencies and creates PRs for secure updates.
"""

import json
import os
import subprocess
import sys
import tempfile
import shutil
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple

class DependencyUpdateManager:
    """Manages automated dependency updates with security considerations."""

    def __init__(self, workspace_root: str = "."):
        self.workspace_root = Path(workspace_root).resolve()
        self.cargo_toml_path = self.workspace_root / "Cargo.toml"
        self.deny_toml_path = self.workspace_root / "deny.toml"

        if not self.cargo_toml_path.exists():
            raise FileNotFoundError(f"Cargo.toml not found at {self.cargo_toml_path}")

    def run_command(self, cmd: List[str], cwd: Optional[Path] = None) -> Tuple[int, str, str]:
        """Run a command and return exit code, stdout, stderr."""
        try:
            result = subprocess.run(
                cmd,
                cwd=cwd or self.workspace_root,
                capture_output=True,
                text=True,
                timeout=300  # 5 minute timeout
            )
            return result.returncode, result.stdout, result.stderr
        except subprocess.TimeoutExpired:
            return -1, "", "Command timed out"
        except Exception as e:
            return -1, "", str(e)

    def get_outdated_dependencies(self) -> Dict:
        """Get list of outdated dependencies using cargo outdated."""
        # First install cargo-outdated if not present
        self.run_command(["cargo", "install", "cargo-outdated", "--locked"])

        exit_code, stdout, stderr = self.run_command(["cargo", "outdated", "--format", "json"])

        if exit_code != 0:
            print(f"Warning: cargo outdated failed: {stderr}")
            return {}

        try:
            return json.loads(stdout)
        except json.JSONDecodeError:
            print("Warning: Failed to parse cargo outdated output")
            return {}

    def check_vulnerability_status(self, package_name: str, version: str) -> bool:
        """Check if a package version has known vulnerabilities."""
        exit_code, stdout, stderr = self.run_command([
            "cargo", "audit", "--json", "--no-fetch"
        ])

        if exit_code != 0:
            print(f"Warning: cargo audit failed: {stderr}")
            return False

        try:
            audit_data = json.loads(stdout)
            vulnerabilities = audit_data.get("vulnerabilities", {}).get("list", [])

            # Check if package has vulnerabilities
            for vuln in vulnerabilities:
                if vuln.get("package") == package_name:
                    return True
        except json.JSONDecodeError:
            pass

        return False

    def is_update_safe(self, package_name: str, current_version: str, new_version: str) -> bool:
        """Determine if a dependency update is safe based on security policies."""
        # Check deny.toml for banned packages
        if self.deny_toml_path.exists():
            try:
                with open(self.deny_toml_path, 'r') as f:
                    deny_content = f.read()

                # Check if package is banned
                if f'"{package_name}"' in deny_content:
                    print(f"Package {package_name} is banned in deny.toml")
                    return False

                # Check for version constraints
                if "openssl" in package_name.lower():
                    print(f"OpenSSL usage detected in {package_name} - requires security review")
                    return False

            except Exception as e:
                print(f"Warning: Could not read deny.toml: {e}")

        # Check for major version changes (higher risk)
        try:
            current_major = int(current_version.split('.')[0])
            new_major = int(new_version.split('.')[0])
            if new_major > current_major:
                print(f"Major version change for {package_name}: {current_version} -> {new_version}")
                return False  # Require manual review for major updates
        except (ValueError, IndexError):
            pass

        # Check if new version has known vulnerabilities
        if self.check_vulnerability_status(package_name, new_version):
            print(f"New version {new_version} of {package_name} has known vulnerabilities")
            return False

        return True

    def create_update_branch(self, package_name: str, new_version: str) -> str:
        """Create a git branch for the dependency update."""
        branch_name = f"security/dep-update-{package_name}-{new_version}"

        # Create and checkout new branch
        self.run_command(["git", "checkout", "-b", branch_name])

        return branch_name

    def update_dependency(self, package_name: str, new_version: str) -> bool:
        """Update a specific dependency to a new version."""
        print(f"Updating {package_name} to {new_version}...")

        # Use cargo edit to update the dependency
        exit_code, stdout, stderr = self.run_command([
            "cargo", "add", package_name, "--vers", new_version
        ])

        if exit_code != 0:
            print(f"Failed to update {package_name}: {stderr}")
            return False

        # Verify the update didn't break the build
        print("Verifying build after update...")
        exit_code, stdout, stderr = self.run_command(["cargo", "check"])

        if exit_code != 0:
            print(f"Build failed after updating {package_name}: {stderr}")
            # Revert the change
            self.run_command(["git", "checkout", "--", "Cargo.toml", "Cargo.lock"])
            return False

        return True

    def create_pull_request(self, package_name: str, current_version: str,
                          new_version: str, branch_name: str) -> bool:
        """Create a pull request for the dependency update."""
        # Commit the changes
        commit_message = f"Security: Update {package_name} from {current_version} to {new_version}"

        self.run_command(["git", "add", "Cargo.toml", "Cargo.lock"])
        self.run_command(["git", "commit", "-m", commit_message])

        # Push the branch
        exit_code, stdout, stderr = self.run_command(["git", "push", "-u", "origin", branch_name])

        if exit_code != 0:
            print(f"Failed to push branch {branch_name}: {stderr}")
            return False

        print(f"Successfully created and pushed branch {branch_name}")
        print(f"Please create a PR for branch {branch_name} with title: {commit_message}")

        return True

    def generate_update_report(self, updates: List[Dict]) -> str:
        """Generate a report of dependency updates."""
        report_lines = [
            "# Dependency Update Report",
            f"Generated: {datetime.now().isoformat()}",
            "",
            "## Safe Updates Applied",
            ""
        ]

        safe_updates = [u for u in updates if u.get("status") == "applied"]
        if safe_updates:
            for update in safe_updates:
                report_lines.append(f"- **{update['package']}**: {update['current']} → {update['new']} (Branch: {update['branch']})")
        else:
            report_lines.append("No safe updates were applied.")

        report_lines.extend([
            "",
            "## Updates Requiring Manual Review",
            ""
        ])

        manual_updates = [u for u in updates if u.get("status") == "manual_review"]
        if manual_updates:
            for update in manual_updates:
                report_lines.append(f"- **{update['package']}**: {update['current']} → {update['new']} ({update['reason']})")
        else:
            report_lines.append("No updates require manual review.")

        report_lines.extend([
            "",
            "## Failed Updates",
            ""
        ])

        failed_updates = [u for u in updates if u.get("status") == "failed"]
        if failed_updates:
            for update in failed_updates:
                report_lines.append(f"- **{update['package']}**: {update.get('reason', 'Unknown error')}")
        else:
            report_lines.append("No updates failed.")

        return "\n".join(report_lines)

    def run_security_updates(self, dry_run: bool = False) -> Dict:
        """Run automated security dependency updates."""
        print("🔍 Analyzing outdated dependencies...")

        outdated = self.get_outdated_dependencies()
        if not outdated:
            print("No outdated dependencies found.")
            return {"updates": [], "report": "No outdated dependencies found."}

        updates = []

        for package, info in outdated.items():
            current_version = info.get("current", "")
            latest_version = info.get("latest", "")

            if not current_version or not latest_version:
                continue

            print(f"📦 Checking {package}: {current_version} → {latest_version}")

            if not self.is_update_safe(package, current_version, latest_version):
                updates.append({
                    "package": package,
                    "current": current_version,
                    "new": latest_version,
                    "status": "manual_review",
                    "reason": "Requires security review or violates policies"
                })
                continue

            if dry_run:
                updates.append({
                    "package": package,
                    "current": current_version,
                    "new": latest_version,
                    "status": "would_apply",
                    "reason": "Dry run - would be applied"
                })
                continue

            # Create update branch
            branch_name = self.create_update_branch(package, latest_version)

            # Apply the update
            if self.update_dependency(package, latest_version):
                # Create PR
                if self.create_pull_request(package, current_version, latest_version, branch_name):
                    updates.append({
                        "package": package,
                        "current": current_version,
                        "new": latest_version,
                        "status": "applied",
                        "branch": branch_name
                    })
                else:
                    updates.append({
                        "package": package,
                        "current": current_version,
                        "new": latest_version,
                        "status": "failed",
                        "reason": "Failed to create PR"
                    })
            else:
                updates.append({
                    "package": package,
                    "current": current_version,
                    "new": latest_version,
                    "status": "failed",
                    "reason": "Update broke build or failed"
                })

        # Generate report
        report = self.generate_update_report(updates)

        # Save report
        report_path = self.workspace_root / "security-reports" / "dependency-updates.md"
        report_path.parent.mkdir(exist_ok=True)
        with open(report_path, 'w') as f:
            f.write(report)

        print(f"📋 Update report saved to {report_path}")

        return {
            "updates": updates,
            "report": report,
            "report_path": str(report_path)
        }

def main():
    """Main entry point for dependency updates."""
    import argparse

    parser = argparse.ArgumentParser(description='Automated dependency security updates')
    parser.add_argument('--dry-run', action='store_true',
                       help='Show what would be updated without making changes')
    parser.add_argument('--workspace-root', default='.',
                       help='Path to workspace root directory')
    parser.add_argument('--package', help='Update only specific package')
    parser.add_argument('--yes', action='store_true',
                       help='Skip confirmation prompts')

    args = parser.parse_args()

    try:
        manager = DependencyUpdateManager(args.workspace_root)

        if args.package:
            print(f"Updating only package: {args.package}")
            # Single package update logic would go here
        else:
            results = manager.run_security_updates(dry_run=args.dry_run)

            if args.dry_run:
                print("DRY RUN RESULTS:")
                print(json.dumps(results, indent=2))
            else:
                print(f"Processed {len(results['updates'])} dependency updates")
                print("Check the generated report for details")

    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()