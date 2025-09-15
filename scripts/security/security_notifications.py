#!/usr/bin/env python3
"""
Security notification system for automated alerts and stakeholder communication.
Handles notifications for security scan results, vulnerabilities, and compliance issues.
"""

import json
import os
import sys
import smtplib
import argparse
from datetime import datetime
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from pathlib import Path
from typing import Dict, List, Optional

class SecurityNotificationManager:
    """Manages security notifications and alerts."""

    def __init__(self, config_file: Optional[str] = None):
        self.config = self.load_config(config_file)
        self.notification_history = []

    def load_config(self, config_file: Optional[str] = None) -> Dict:
        """Load notification configuration."""
        default_config = {
            "email": {
                "enabled": False,
                "smtp_server": "smtp.gmail.com",
                "smtp_port": 587,
                "sender_email": "security@rust-ai-ide.com",
                "sender_password": os.getenv("SECURITY_EMAIL_PASSWORD", ""),
                "recipients": [
                    "security-team@company.com",
                    "devops@company.com"
                ]
            },
            "slack": {
                "enabled": False,
                "webhook_url": os.getenv("SLACK_SECURITY_WEBHOOK", ""),
                "channel": "#security-alerts"
            },
            "teams": {
                "enabled": False,
                "webhook_url": os.getenv("TEAMS_SECURITY_WEBHOOK", "")
            },
            "thresholds": {
                "critical_vulns": 0,  # Alert immediately on any critical
                "high_vulns": 1,      # Alert if more than 1 high severity
                "unsafe_code_percent": 10.0,  # Alert if unsafe code > 10%
                "outdated_deps_days": 90      # Alert if deps outdated > 90 days
            }
        }

        if config_file and os.path.exists(config_file):
            with open(config_file, 'r') as f:
                user_config = json.load(f)
            self.deep_update(default_config, user_config)

        return default_config

    def deep_update(self, base_dict: Dict, update_dict: Dict) -> None:
        """Deep update dictionary."""
        for key, value in update_dict.items():
            if isinstance(value, dict) and key in base_dict and isinstance(base_dict[key], dict):
                self.deep_update(base_dict[key], value)
            else:
                base_dict[key] = value

    def should_alert(self, scan_results: Dict) -> bool:
        """Determine if notification should be sent based on results and thresholds."""
        thresholds = self.config["thresholds"]

        # Check vulnerability counts
        vulns = scan_results.get("vulnerabilities", {})
        if vulns.get("critical", 0) >= thresholds["critical_vulns"]:
            return True
        if vulns.get("high", 0) > thresholds["high_vulns"]:
            return True

        # Check unsafe code percentage
        unsafe_percent = scan_results.get("unsafe_code_percent", 0)
        if unsafe_percent > thresholds["unsafe_code_percent"]:
            return True

        # Check outdated dependencies
        outdated_days = scan_results.get("outdated_deps_max_days", 0)
        if outdated_days > thresholds["outdated_deps_days"]:
            return True

        return False

    def send_email_notification(self, subject: str, body: str, severity: str = "info") -> bool:
        """Send email notification."""
        if not self.config["email"]["enabled"]:
            print("Email notifications disabled")
            return False

        try:
            msg = MIMEMultipart()
            msg['From'] = self.config["email"]["sender_email"]
            msg['To'] = ", ".join(self.config["email"]["recipients"])
            msg['Subject'] = f"[{severity.upper()}] {subject}"

            msg.attach(MIMEText(body, 'html'))

            server = smtplib.SMTP(self.config["email"]["smtp_server"],
                                self.config["email"]["smtp_port"])
            server.starttls()
            server.login(self.config["email"]["sender_email"],
                        self.config["email"]["sender_password"])
            server.send_message(msg)
            server.quit()

            print(f"Email notification sent to {len(self.config['email']['recipients'])} recipients")
            return True
        except Exception as e:
            print(f"Failed to send email: {e}")
            return False

    def send_slack_notification(self, message: str, severity: str = "info") -> bool:
        """Send Slack notification."""
        if not self.config["slack"]["enabled"]:
            print("Slack notifications disabled")
            return False

        try:
            import requests

            payload = {
                "channel": self.config["slack"]["channel"],
                "username": "Security Monitor",
                "text": message,
                "icon_emoji": ":shield:"
            }

            response = requests.post(self.config["slack"]["webhook_url"],
                                   json=payload,
                                   timeout=10)

            if response.status_code == 200:
                print("Slack notification sent successfully")
                return True
            else:
                print(f"Slack notification failed: {response.status_code}")
                return False
        except ImportError:
            print("requests library not available for Slack notifications")
            return False
        except Exception as e:
            print(f"Slack notification error: {e}")
            return False

    def send_teams_notification(self, title: str, text: str, severity: str = "info") -> bool:
        """Send Microsoft Teams notification."""
        if not self.config["teams"]["enabled"]:
            print("Teams notifications disabled")
            return False

        try:
            import requests

            color_map = {
                "critical": "FF0000",
                "high": "FFA500",
                "medium": "FFFF00",
                "low": "00FF00",
                "info": "0000FF"
            }

            payload = {
                "@type": "MessageCard",
                "@context": "http://schema.org/extensions",
                "themeColor": color_map.get(severity, "0000FF"),
                "title": title,
                "text": text
            }

            response = requests.post(self.config["teams"]["webhook_url"],
                                   json=payload,
                                   timeout=10)

            if response.status_code == 200:
                print("Teams notification sent successfully")
                return True
            else:
                print(f"Teams notification failed: {response.status_code}")
                return False
        except ImportError:
            print("requests library not available for Teams notifications")
            return False
        except Exception as e:
            print(f"Teams notification error: {e}")
            return False

    def format_security_report(self, scan_results: Dict) -> tuple[str, str]:
        """Format security scan results for notifications."""
        severity = scan_results.get("overall_severity", "info")

        # Determine color/emojis based on severity
        severity_indicators = {
            "critical": "🚨 CRITICAL",
            "high": "⚠️ HIGH",
            "medium": "🟡 MEDIUM",
            "low": "🟢 LOW",
            "info": "ℹ️ INFO"
        }

        emoji = severity_indicators.get(severity, "ℹ️")

        subject = f"{emoji} Security Scan Results - {scan_results.get('timestamp', datetime.now().isoformat())}"

        # Build HTML body
        body_parts = [
            f"<h2>{emoji} Security Scan Alert</h2>",
            f"<p><strong>Timestamp:</strong> {scan_results.get('timestamp', datetime.now().isoformat())}</p>",
            f"<p><strong>Commit:</strong> {scan_results.get('commit_sha', 'N/A')}</p>",
            f"<p><strong>Branch:</strong> {scan_results.get('branch', 'N/A')}</p>",
            "<h3>Findings Summary</h3>",
            "<ul>"
        ]

        # Add vulnerability summary
        vulns = scan_results.get("vulnerabilities", {})
        if vulns:
            body_parts.extend([
                f"<li>Critical Vulnerabilities: <strong>{vulns.get('critical', 0)}</strong></li>",
                f"<li>High Vulnerabilities: <strong>{vulns.get('high', 0)}</strong></li>",
                f"<li>Medium Vulnerabilities: <strong>{vulns.get('medium', 0)}</strong></li>",
                f"<li>Low Vulnerabilities: <strong>{vulns.get('low', 0)}</strong></li>"
            ])

        # Add other findings
        if "unsafe_code_percent" in scan_results:
            body_parts.append(f"<li>Unsafe Code Usage: <strong>{scan_results['unsafe_code_percent']:.1f}%</strong></li>")

        if "compliance_issues" in scan_results:
            body_parts.append(f"<li>Compliance Issues: <strong>{len(scan_results['compliance_issues'])}</strong></li>")

        if "outdated_deps_count" in scan_results:
            body_parts.append(f"<li>Outdated Dependencies: <strong>{scan_results['outdated_deps_count']}</strong></li>")

        body_parts.extend([
            "</ul>",
            "<h3>Action Required</h3>",
            "<p>Please review the security scan results and address any critical or high-severity issues.</p>",
            "<p>Full report available in the CI/CD artifacts or security dashboard.</p>"
        ])

        body = "\n".join(body_parts)

        return subject, body

    def send_notifications(self, scan_results: Dict) -> bool:
        """Send notifications through all configured channels."""
        if not self.should_alert(scan_results):
            print("No alerts triggered - results within acceptable thresholds")
            return True

        subject, html_body = self.format_security_report(scan_results)

        success = True

        # Send email notification
        if self.send_email_notification(subject, html_body,
                                      scan_results.get("overall_severity", "info")):
            print("✓ Email notification sent")
        else:
            print("✗ Email notification failed")
            success = False

        # Send Slack notification
        slack_message = f"{subject}\n\n{html_body.replace('<br>', '\n').replace('</li>', '').replace('<li>', '• ').replace('<ul>', '').replace('</ul>', '').replace('<h3>', '*').replace('</h3>', '*').replace('<h2>', '*').replace('</h2>', '*').replace('<strong>', '*').replace('</strong>', '*').replace('<p>', '').replace('</p>', '\n')}"
        if self.send_slack_notification(slack_message,
                                       scan_results.get("overall_severity", "info")):
            print("✓ Slack notification sent")
        else:
            print("✗ Slack notification failed")

        # Send Teams notification
        if self.send_teams_notification(subject, html_body,
                                       scan_results.get("overall_severity", "info")):
            print("✓ Teams notification sent")
        else:
            print("✗ Teams notification failed")

        # Log notification
        self.notification_history.append({
            "timestamp": datetime.now().isoformat(),
            "subject": subject,
            "severity": scan_results.get("overall_severity", "info"),
            "channels": ["email", "slack", "teams"],
            "success": success
        })

        return success

def parse_scan_results(results_file: str) -> Dict:
    """Parse security scan results from JSON file."""
    try:
        with open(results_file, 'r') as f:
            return json.load(f)
    except Exception as e:
        print(f"Error parsing results file {results_file}: {e}")
        return {}

def main():
    """Main entry point for security notifications."""
    parser = argparse.ArgumentParser(description='Send security notifications')
    parser.add_argument('--results-file', '-r', required=True,
                       help='Path to security scan results JSON file')
    parser.add_argument('--config-file', '-c',
                       help='Path to notification configuration file')
    parser.add_argument('--test', action='store_true',
                       help='Test notification configuration without sending')

    args = parser.parse_args()

    # Load scan results
    scan_results = parse_scan_results(args.results_file)
    if not scan_results:
        print("No scan results to process")
        sys.exit(1)

    # Initialize notification manager
    manager = SecurityNotificationManager(args.config_file)

    if args.test:
        print("=== NOTIFICATION TEST ===")
        print(f"Configuration: {json.dumps(manager.config, indent=2)}")
        print(f"Should alert: {manager.should_alert(scan_results)}")

        subject, body = manager.format_security_report(scan_results)
        print(f"Subject: {subject}")
        print(f"Body preview: {body[:200]}...")
        print("=== TEST COMPLETE ===")
        return

    # Send notifications
    success = manager.send_notifications(scan_results)

    if success:
        print("All notifications sent successfully")
        sys.exit(0)
    else:
        print("Some notifications failed")
        sys.exit(1)

if __name__ == '__main__':
    main()