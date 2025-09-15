#!/usr/bin/env python3
"""
Security pre-commit hook to check for plaintext secrets in code.
Scans for common patterns that might indicate accidentally committed secrets.
"""

import re
import sys
import os
from pathlib import Path

# Patterns to detect potential secrets
SECRET_PATTERNS = [
    # API keys and tokens
    r'(?i)(api[_-]?key|apikey)\s*[=:]\s*["\']([^"\']{10,})["\']',
    r'(?i)(secret[_-]?key|secretkey)\s*[=:]\s*["\']([^"\']{10,})["\']',
    r'(?i)(access[_-]?token|accesstoken)\s*[=:]\s*["\']([^"\']{10,})["\']',
    r'(?i)(auth[_-]?token|authtoken)\s*[=:]\s*["\']([^"\']{10,})["\']',
    r'(?i)(bearer[_-]?token|bearertoken)\s*[=:]\s*["\']([^"\']{10,})["\']',

    # Passwords
    r'(?i)password\s*[=:]\s*["\']([^"\']{3,})["\']',
    r'(?i)passwd\s*[=:]\s*["\']([^"\']{3,})["\']',
    r'(?i)pwd\s*[=:]\s*["\']([^"\']{3,})["\']',

    # Database credentials
    r'(?i)(db[_-]?password|dbpassword)\s*[=:]\s*["\']([^"\']{3,})["\']',
    r'(?i)(database[_-]?password|databasepassword)\s*[=:]\s*["\']([^"\']{3,})["\']',

    # Generic secrets
    r'(?i)secret\s*[=:]\s*["\']([^"\']{8,})["\']',

    # AWS credentials
    r'(?i)(aws[_-]?access[_-]?key[_-]?id|awsaccesskeyid)\s*[=:]\s*["\']([^"\']{10,})["\']',
    r'(?i)(aws[_-]?secret[_-]?access[_-]?key|awssecretaccesskey)\s*[=:]\s*["\']([^"\']{10,})["\']',

    # JWT tokens (long base64-like strings)
    r'(?i)(jwt|token)\s*[=:]\s*["\']([A-Za-z0-9+/=]{50,})["\']',

    # Private keys
    r'-----BEGIN\s+(?:RSA\s+)?PRIVATE\s+KEY-----',
    r'-----BEGIN\s+(?:EC\s+)?PRIVATE\s+KEY-----',

    # Generic long hex strings (might be keys)
    r'(?i)(key|token|secret)\s*[=:]\s*["\']([a-f0-9]{32,})["\']',

    # OAuth client secrets
    r'(?i)(client[_-]?secret|clientsecret)\s*[=:]\s*["\']([^"\']{8,})["\']',
]

# Files to exclude from scanning
EXCLUDE_PATTERNS = [
    # Dependencies and build artifacts
    'target/',
    'node_modules/',
    '.git/',
    'dist/',
    'build/',

    # Configuration files that might legitimately contain secrets
    'deny.toml',  # Already handled by cargo-deny
    '.env.example',
    '.env.template',
    'config.example.toml',
    'config.template.toml',

    # Documentation and examples
    'docs/',
    'examples/',
    'README.md',
    'CHANGELOG.md',
    'CONTRIBUTING.md',

    # Test files
    'tests/',
    'integration-tests/',
    '*test*.rs',
    '*spec*.rs',

    # Generated files
    '*.lock',
    'Cargo.lock',
    'package-lock.json',
    'yarn.lock',

    # Web assets
    'web/dist/',
    'web/build/',
    '*.min.js',
    '*.min.css',

    # Security-related files that might contain test secrets
    'security-reports/',
    'security-dashboards/',
]

def should_exclude_file(file_path):
    """Check if file should be excluded from secret scanning."""
    path = Path(file_path)

    for pattern in EXCLUDE_PATTERNS:
        if pattern.endswith('/') and path.is_relative_to(pattern.rstrip('/')):
            return True
        elif path.match(pattern):
            return True

    return False

def scan_file_for_secrets(file_path):
    """Scan a single file for potential secrets."""
    findings = []

    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
            lines = content.splitlines()

        for line_num, line in enumerate(lines, 1):
            for pattern in SECRET_PATTERNS:
                matches = re.finditer(pattern, line)
                for match in matches:
                    # Extract the potential secret value
                    secret_value = match.group(2) if len(match.groups()) > 1 else match.group(1)

                    # Skip obviously fake/test values
                    if any(fake in secret_value.lower() for fake in [
                        'test', 'example', 'placeholder', 'dummy', 'sample', 'your_', 'xxx', '123456'
                    ]):
                        continue

                    findings.append({
                        'file': file_path,
                        'line': line_num,
                        'pattern': pattern,
                        'line_content': line.strip(),
                        'secret_type': 'potential_secret'
                    })

    except (IOError, UnicodeDecodeError) as e:
        print(f"Warning: Could not read {file_path}: {e}", file=sys.stderr)

    return findings

def main():
    """Main entry point for the secret scanner."""
    import argparse

    parser = argparse.ArgumentParser(description='Scan for plaintext secrets in code files')
    parser.add_argument('files', nargs='*', help='Files to scan (default: staged files)')
    parser.add_argument('--staged-only', action='store_true', help='Only scan staged files')
    args = parser.parse_args()

    if args.staged_only or not args.files:
        # Get staged files using git
        import subprocess
        try:
            result = subprocess.run(['git', 'diff', '--cached', '--name-only'],
                                  capture_output=True, text=True, cwd='.')
            if result.returncode == 0:
                args.files = result.stdout.strip().split('\n')
            else:
                print("Error: Could not get staged files", file=sys.stderr)
                sys.exit(1)
        except FileNotFoundError:
            print("Error: Git not found", file=sys.stderr)
            sys.exit(1)

    all_findings = []

    for file_path in args.files:
        if not file_path or not os.path.exists(file_path):
            continue

        if should_exclude_file(file_path):
            continue

        findings = scan_file_for_secrets(file_path)
        all_findings.extend(findings)

    if all_findings:
        print("🚨 Potential plaintext secrets found! 🚨", file=sys.stderr)
        print("=" * 60, file=sys.stderr)

        for finding in all_findings:
            print(f"File: {finding['file']}:{finding['line']}", file=sys.stderr)
            print(f"Content: {finding['line_content']}", file=sys.stderr)
            print(f"Pattern: {finding['pattern']}", file=sys.stderr)
            print("-" * 40, file=sys.stderr)

        print(f"\n❌ Found {len(all_findings)} potential secrets", file=sys.stderr)
        print("Please review and remove or properly secure these values.", file=sys.stderr)
        print("If these are false positives, consider excluding the file or updating the patterns.", file=sys.stderr)
        sys.exit(1)
    else:
        print("✅ No plaintext secrets detected")
        sys.exit(0)

if __name__ == '__main__':
    main()