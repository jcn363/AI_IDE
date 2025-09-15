# Security Automation Scripts

This directory contains automated security scanning and compliance checking scripts for the Rust AI IDE project.

## Overview

The security automation infrastructure provides comprehensive monitoring and enforcement of security policies, including:

- **Dependency Security**: Vulnerability scanning and license compliance
- **Code Security**: Unsafe code detection and secret scanning
- **Automated Updates**: Safe dependency updates with security validation
- **Notifications**: Multi-channel alerting for security issues

## Scripts

### Core Security Scanning

#### `run_owasp_scan.sh`
Comprehensive OWASP security scanner that combines multiple tools:
- **cargo-audit**: Known vulnerability scanning
- **cargo-deny**: Dependency and license compliance
- **Custom OWASP scanner**: Security pattern detection

**Usage:**
```bash
./run_owasp_scan.sh
```

**Features:**
- JUnit XML output for CI/CD integration
- Comprehensive security reporting
- Build failure on critical issues
- Integration with existing security scanners

### Security Validation

#### `check_plaintext_secrets.py`
Scans codebase for accidentally committed plaintext secrets and sensitive information.

**Usage:**
```bash
python3 check_plaintext_secrets.py [files...]
python3 check_plaintext_secrets.py --staged-only  # Only scan staged files
```

**Detection patterns:**
- API keys and tokens
- Database passwords
- Private keys
- AWS credentials
- Generic secret patterns

### Automated Updates

#### `dependency_updates.py`
Automated dependency security updates with safety validation.

**Usage:**
```bash
# Dry run to see what would be updated
python3 dependency_updates.py --dry-run

# Apply safe security updates
python3 dependency_updates.py

# Update specific package
python3 dependency_updates.py --package serde
```

**Safety features:**
- Checks deny.toml for banned packages
- Validates license compliance
- Verifies no known vulnerabilities in new versions
- Major version change protection
- Build verification after updates

### Notifications

#### `security_notifications.py`
Multi-channel security notification system.

**Usage:**
```bash
python3 security_notifications.py --results-file security-results.json
```

**Supported channels:**
- **Email**: SMTP-based notifications
- **Slack**: Webhook-based alerts
- **Microsoft Teams**: Adaptive cards
- **Configurable thresholds**: Only alert on significant issues

## Pre-commit Integration

The security scripts integrate with pre-commit hooks for early detection:

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: cargo-deny-quick
        name: Quick cargo-deny check
        entry: bash -c "cargo deny check bans --all-features --all-targets"
        language: system
        files: \.(toml|lock)$

      - id: check-plaintext-secrets
        name: Check for plaintext secrets
        entry: python3
        args: [scripts/security/check_plaintext_secrets.py]
        language: system
        files: .*
```

## CI/CD Integration

### GitHub Actions Workflow

The `.github/workflows/security-audit.yml` provides comprehensive CI/CD security scanning:

**Triggers:**
- Push to main/develop branches
- Pull requests
- Nightly security scans (2 AM UTC)
- Manual workflow dispatch

**Jobs:**
1. **security-scan**: Core vulnerability and code analysis
2. **compliance-check**: License and dependency compliance
3. **nightly-deep-scan**: Comprehensive analysis with dependency graphs
4. **summary**: Consolidated security report

**Tools used:**
- cargo-audit (vulnerability scanning)
- cargo-deny (dependency policy enforcement)
- cargo-geiger (unsafe code detection)
- cargo-outdated (dependency freshness)

## Configuration Files

### deny.toml
Cargo-deny configuration with security policies:
- Banned dependencies (openssl, md5, ring, quick-js)
- License restrictions (MIT/Apache-2.0 only)
- Registry restrictions (crates.io only)

### Notification Configuration
Security notification thresholds and channels can be configured via:
- Environment variables for webhook URLs
- JSON configuration files
- Default embedded configuration

## Security Policies Enforced

### Dependency Security
- **Banned packages**: openssl, md5, ring, quick-js
- **License compliance**: MIT/Apache-2.0/BSD only
- **Registry restrictions**: crates.io only
- **Version enforcement**: SQLite version consistency

### Code Security
- **Unsafe code monitoring**: Automated detection and alerting
- **Secret scanning**: Pre-commit and CI/CD validation
- **Input validation**: Sanitization requirements
- **Path traversal protection**: Secure path handling

### Compliance Validation
- **OWASP guidelines**: Automated compliance checking
- **Security best practices**: Pattern-based validation
- **Audit logging**: Sensitive operation tracking

## Reporting and Monitoring

### Security Reports
All scans generate comprehensive reports:
- JSON results for programmatic processing
- Markdown summaries for human review
- JUnit XML for CI/CD integration
- GitHub artifacts for download

### Alert Thresholds
Configurable alerting based on severity:
- **Critical**: Immediate notification, may block builds
- **High**: Alert stakeholders, review required
- **Medium**: Logged, periodic review
- **Low**: Informational, trend monitoring

### Dashboard Integration
Security metrics feed into monitoring dashboards:
- Vulnerability trends over time
- License compliance status
- Code quality metrics
- Automated remediation tracking

## Usage Examples

### Local Development
```bash
# Quick security check
./scripts/security/run_owasp_scan.sh

# Check specific files for secrets
python3 scripts/security/check_plaintext_secrets.py src/main.rs

# Preview dependency updates
python3 scripts/security/dependency_updates.py --dry-run
```

### CI/CD Integration
```yaml
# GitHub Actions
- name: Security Scan
  run: ./scripts/security/run_owasp_scan.sh

- name: Dependency Updates
  run: python3 scripts/security/dependency_updates.py --dry-run
```

### Pre-commit Setup
```bash
pip install pre-commit
pre-commit install
pre-commit run --all-files
```

## Maintenance

### Regular Updates
- Keep security tools updated: `cargo install --force cargo-audit cargo-deny`
- Update detection patterns as new threats emerge
- Review and update banned dependencies list
- Monitor false positive rates and adjust thresholds

### Monitoring Effectiveness
- Track security metrics over time
- Review automated remediation success rates
- Audit notification effectiveness
- Validate compliance with security policies

## Troubleshooting

### Common Issues
1. **Tools not found**: Ensure Rust nightly toolchain is installed
2. **Permission errors**: Make scripts executable with `chmod +x`
3. **Network issues**: Some tools require internet access for vulnerability databases
4. **False positives**: Adjust exclusion patterns in scripts

### Debug Mode
Most scripts support verbose output for debugging:
```bash
DEBUG=1 ./run_owasp_scan.sh
```

## Integration Points

The security automation integrates with:
- **Existing CI/CD**: GitHub Actions workflows
- **Development workflow**: Pre-commit hooks
- **Security policies**: deny.toml configuration
- **Monitoring systems**: External security dashboards
- **Notification systems**: Email, Slack, Teams

This comprehensive security automation ensures ongoing security monitoring, early detection of issues, and automated remediation where safe, maintaining the security posture of the Rust AI IDE project.