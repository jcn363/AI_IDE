# Rust AI IDE - CI/CD Integration Scripts

This directory contains comprehensive CI/CD automation scripts for the Rust AI IDE project, providing automated documentation updates, code reviews, git operations, and stakeholder notifications.

## 📋 Overview

The CI/CD integration system consists of several modular scripts that work together to automate the development workflow:

- **`main-integration.sh`** - Main orchestration script
- **`documentation-update.sh`** - Automated documentation generation
- **`review-workflow.sh`** - Code review automation
- **`git-operations.sh`** - Git operations with detailed logging
- **`stakeholder-notifications.sh`** - Multi-channel notifications
- **`notification-templates/`** - Customizable notification templates

## 🚀 Quick Start

### 1. System Setup

```bash
# Make all scripts executable
chmod +x scripts/ci/*.sh

# Create logs directory
mkdir -p logs

# Create notification templates
bash scripts/ci/stakeholder-notifications.sh template create

# Check system status
bash scripts/ci/main-integration.sh status
```

### 2. Environment Configuration

Create a `.env.ci` file in the project root:

```bash
# Notification channels
SLACK_WEBHOOK="https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"
EMAIL_SMTP_SERVER="smtp.gmail.com"
EMAIL_SMTP_PORT="587"
EMAIL_USERNAME="your-email@gmail.com"
EMAIL_PASSWORD="your-app-password"
EMAIL_FROM="ci@rust-ai-ide.dev"
EMAIL_RECIPIENTS="team@company.com,manager@company.com"
WEBHOOK_URL="https://your-webhook-endpoint.com/hook"

# Git configuration
GIT_AUTHOR="CI Bot"
GIT_EMAIL="ci@rust-ai-ide.dev"
BRANCH="main"
REMOTE="origin"

# Review configuration
REVIEW_BRANCH="main"
REVIEWER_EMAIL="reviewer@company.com"

# Secret management (use managed secrets in production)
SECRET_MANAGER="env"
```

### 3. Basic Usage

```bash
# Run complete CI/CD workflow
bash scripts/ci/main-integration.sh full

# Run documentation updates only
bash scripts/ci/main-integration.sh docs

# Run code review only
bash scripts/ci/main-integration.sh review

# Run git operations
bash scripts/ci/main-integration.sh git commit "feat" "Add new feature"
bash scripts/ci/main-integration.sh git push
```

## 📚 Scripts Reference

### Main Integration Script (`main-integration.sh`)

Orchestrates all CI/CD workflows:

```bash
# Run complete workflow
./main-integration.sh full

# Individual workflows
./main-integration.sh docs        # Documentation updates
./main-integration.sh review      # Code review
./main-integration.sh git <op>    # Git operations

# Utility commands
./main-integration.sh status      # System status check
./main-integration.sh config      # Create configuration
./main-integration.sh help        # Show help
```

### Documentation Updates (`documentation-update.sh`)

Generates Rust docs and TypeScript types:

```bash
# Run documentation update
./documentation-update.sh

# Outputs:
# DOCUMENTATION_UPDATE_STATUS=success
# DOCUMENTATION_UPDATE_DURATION=45
# DOCUMENTATION_UPDATE_LOG=/path/to/log
```

### Code Review Workflow (`review-workflow.sh`)

Automated code review with checks:

```bash
# Run review workflow
./review-workflow.sh

# Outputs:
# REVIEW_WORKFLOW_STATUS=completed
# REVIEW_DURATION=30
# REVIEW_REPORT=/path/to/report
```

### Git Operations (`git-operations.sh`)

Advanced git operations with logging:

```bash
# Commit changes
./git-operations.sh commit "feat" "Add new feature" "Optional details"

# Push with conflict resolution
./git-operations.sh push

# Rollback to commit
./git-operations.sh rollback <commit-hash> "Reason for rollback"

# Check status
./git-operations.sh status

# Outputs:
# COMMIT_HASH=abc123
# COMMIT_MESSAGE_FILE=/path/to/log
# CHANGE_TRACKING_FILE=/path/to/json
```

### Stakeholder Notifications (`stakeholder-notifications.sh`)

Multi-channel notifications:

```bash
# Send notification
./stakeholder-notifications.sh notify <event> <status> <details> [template]

# Manage templates
./stakeholder-notifications.sh template create
./stakeholder-notifications.sh template list

# Test notifications
./stakeholder-notifications.sh test

# Examples:
./stakeholder-notifications.sh notify documentation_update success "Docs updated"
./stakeholder-notifications.sh notify deployment failure "Build failed" deployment
```

## 🔧 CI/CD Pipeline Integration

### GitLab CI Integration

Add to your `.gitlab-ci.yml`:

```yaml
stages:
  - build
  - test
  - review
  - deploy

documentation:
  stage: review
  script:
    - bash scripts/ci/main-integration.sh docs
  artifacts:
    paths:
      - target/doc/
      - web/src/types/generated.ts
    expire_in: 1 week
  only:
    - merge_requests

code_review:
  stage: review
  script:
    - bash scripts/ci/main-integration.sh review
  artifacts:
    reports:
      junit: review-report-*.md
    expire_in: 1 week
  only:
    - merge_requests

deploy:
  stage: deploy
  script:
    - bash scripts/ci/main-integration.sh git commit "ci" "Automated deployment"
    - bash scripts/ci/main-integration.sh git push
  environment:
    name: production
  only:
    - main
```

### GitHub Actions Integration

Create `.github/workflows/ci.yml`:

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  documentation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: nightly-2025-09-03
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
      - name: Update Documentation
        run: bash scripts/ci/main-integration.sh docs
      - name: Upload docs
        uses: actions/upload-artifact@v3
        with:
          name: documentation
          path: target/doc/

  review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Code Review
        run: bash scripts/ci/main-integration.sh review
      - name: Upload review report
        uses: actions/upload-artifact@v3
        with:
          name: review-report
          path: review-report-*.md
```

### Azure Pipelines Integration

Add to your `azure-pipelines.yml`:

```yaml
stages:
- stage: CI
  jobs:
  - job: Documentation
    steps:
    - script: bash scripts/ci/main-integration.sh docs
      displayName: 'Update Documentation'

  - job: Review
    steps:
    - script: bash scripts/ci/main-integration.sh review
      displayName: 'Code Review'

- stage: Deploy
  jobs:
  - job: Deploy
    steps:
    - script: |
        bash scripts/ci/main-integration.sh git commit "ci" "Deployment $(Build.BuildNumber)"
        bash scripts/ci/main-integration.sh git push
      displayName: 'Deploy Changes'
```

## 📊 Logging and Audit Trails

### Log Files

All scripts generate detailed logs in the `logs/` directory:

- `main-integration-YYYYMMDD-HHMMSS.log` - Main workflow logs
- `documentation-update-YYYYMMDD-HHMMSS.log` - Documentation logs
- `review-workflow-YYYYMMDD-HHMMSS.log` - Review logs
- `git-operations-YYYYMMDD-HHMMSS.log` - Git operation logs
- `notifications-YYYYMMDD-HHMMSS.log` - Notification logs

### Commit History

Git operations maintain a detailed commit history:

- `logs/commit-history-YYYYMMDD.log` - Daily commit log
- `logs/change-tracking-YYYYMMDD.json` - Detailed change analysis

### Audit Trail

Each operation includes:

- Timestamp with microsecond precision
- User/actor identification
- Operation details and parameters
- Success/failure status
- Duration metrics
- Related artifacts (reports, logs)

## 🔒 Security Considerations

### Secret Management

- Never hardcode credentials in scripts
- Use environment variables or secret managers
- Rotate credentials regularly
- Audit secret access

### Secure Configuration

```bash
# Use managed secrets instead of plain text
SECRET_MANAGER="vault"  # or "aws" or "azure"

# For local development only
echo "SLACK_WEBHOOK=https://hooks.slack.com/services/..." > .env.ci
```

### Access Controls

- Limit script execution to authorized users
- Use CI/CD platform secrets for production
- Implement approval workflows for sensitive operations

## 🧪 Testing and Validation

### System Tests

```bash
# Test all components
bash scripts/ci/main-integration.sh status

# Test notifications
bash scripts/ci/stakeholder-notifications.sh test

# Validate git operations
bash scripts/ci/git-operations.sh status
```

### Integration Tests

```bash
# Test complete workflow
bash scripts/ci/main-integration.sh full

# Verify outputs
echo "Last commit: $(cat logs/commit-history-$(date +%Y%m%d).log | tail -1)"
echo "Review report: $(ls review-report-*.md 2>/dev/null | tail -1)"
```

## 📈 Monitoring and Metrics

### Performance Metrics

Scripts output performance data:

```bash
# Documentation update metrics
DOCUMENTATION_UPDATE_DURATION=45
DOCUMENTATION_UPDATE_STATUS=success

# Review workflow metrics
REVIEW_DURATION=30
REVIEW_WORKFLOW_STATUS=completed

# Git operation metrics
COMMIT_DURATION=5
PUSH_ATTEMPTS=1
```

### Health Checks

```bash
# Check script health
bash scripts/ci/main-integration.sh status

# Check notification channels
bash scripts/ci/stakeholder-notifications.sh test

# Check git repository status
bash scripts/ci/git-operations.sh status
```

## 🆘 Troubleshooting

### Common Issues

1. **Permission denied**
   ```bash
   chmod +x scripts/ci/*.sh
   ```

2. **Missing dependencies**
   ```bash
   # Install required tools
   cargo install cargo-deny
   npm install -g markdown-link-check
   ```

3. **Git configuration issues**
   ```bash
   git config --global user.name "CI Bot"
   git config --global user.email "ci@rust-ai-ide.dev"
   ```

4. **Notification failures**
   ```bash
   # Test individual channels
   bash scripts/ci/stakeholder-notifications.sh test
   ```

### Debug Mode

Enable verbose logging:

```bash
# Set debug environment variable
export DEBUG=true

# Run with verbose output
bash -x scripts/ci/main-integration.sh full
```

## 📚 Additional Resources

- [AGENTS.md](../../AGENTS.md) - Project agent rules
- [docs/ci-cd/](../../docs/ci-cd/) - CI/CD documentation
- [scripts/README.md](../README.md) - Scripts overview
- [.gitlab-ci.yml](../../.gitlab-ci.yml) - GitLab CI configuration

## 🤝 Contributing

When adding new CI/CD features:

1. Follow the existing script patterns
2. Add comprehensive logging
3. Include error handling and notifications
4. Update this documentation
5. Test integration with existing workflows

---

*Built for enterprise-scale Rust development with comprehensive CI/CD automation.*