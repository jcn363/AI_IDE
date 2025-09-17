# Security Notifications Setup Guide

This document explains how to set up and manage security notifications for the Rust AI IDE project.

## Prerequisites

1. **Slack Webhook** (for team notifications):
   - Create an incoming webhook in your Slack workspace
   - Add the webhook URL as a repository secret named `SLACK_WEBHOOK`

2. **Email Notifications** (for critical alerts):
   - Set up an email service account (e.g., Gmail)
   - Add the following repository secrets:
     - `EMAIL_USERNAME`: Email address for sending notifications
     - `EMAIL_PASSWORD`: App password for the email account
     - `SECURITY_TEAM_EMAIL`: Comma-separated list of security team emails

## Notification Types

### 1. Slack Notifications

- **When**: New security alerts are detected
- **What's included**:
  - Alert type and severity
  - Brief description
  - Direct link to the alert in GitHub

### 2. Email Notifications

- **When**: Critical security alerts are detected
- **What's included**:
  - Detailed alert information
  - Severity level
  - Direct link to the alert

### 3. GitHub Issues

- **When**: New security alerts are opened or reopened
- **What's included**:
  - Full alert details
  - Affected files and locations
  - Automatic assignment to security team

## Customization

### Alert Severity Levels

Edit `.github/workflows/security-notifications.yml` to modify:

- Which severity levels trigger notifications
- Notification thresholds
- Message templates

### Notification Channels

To add more notification channels:

1. Add new steps in the workflow file
2. Use the appropriate GitHub Action for your channel
3. Format the message using the available alert data

## Testing

To test the notification system:

1. Create a test security alert (e.g., by adding a vulnerable dependency)
2. Push the changes to trigger the workflow
3. Verify notifications are received on all channels

## Maintenance

- Review and update notification recipients regularly
- Monitor notification delivery
- Update the workflow file when GitHub's alert API changes

## Troubleshooting

### Common Issues

1. **Missing Notifications**:
   - Check GitHub Actions workflow runs
   - Verify repository secrets are correctly set
   - Check spam folders for emails

2. **Authentication Failures**:
   - Verify API tokens and credentials
   - Check for rate limiting

3. **Incorrect Alert Information**:
   - Update the workflow file to include the correct alert fields
   - Check GitHub's API documentation for changes

## Security Considerations

- Never expose sensitive information in notification messages
- Use repository secrets for all credentials
- Regularly rotate access tokens and credentials
- Follow the principle of least privilege for service accounts
