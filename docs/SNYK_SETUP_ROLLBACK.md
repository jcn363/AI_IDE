# Snyk Setup Rollback Procedures

## Overview
This document outlines the procedures for rolling back Snyk API token and organization configuration changes.

## Rollback Scenarios

### 1. Rollback Snyk Secrets from AWS Secrets Manager

#### Command Line Rollback
```bash
# Delete the Snyk secrets from AWS Secrets Manager
aws secretsmanager delete-secret \
  --secret-id "/rust-ai-ide/snyk/credentials" \
  --force-delete-without-recovery
```

#### Verification
```bash
# Confirm secret is deleted
aws secretsmanager describe-secret \
  --secret-id "/rust-ai-ide/snyk/credentials" || echo "Secret successfully deleted"
```

### 2. Rollback CI/CD Pipeline Changes

#### GitLab CI/CD Rollback
Remove the `security:snyk_scan` job from `ci-cd/.gitlab-ci.yml`:

```yaml
# Remove this entire job block
security:snyk_scan:
  stage: security
  # ... entire job definition
```

#### Alternative: Disable the Job
```yaml
security:snyk_scan:
  # Add this to temporarily disable
  when: manual
  # Or add this to skip entirely
  only:
    - never
```

### 3. Rollback Scripts and Configuration

#### Remove Secrets Retrieval Script
```bash
# Delete the secrets retrieval script
rm scripts/ci/retrieve-snyk-secrets.sh
```

#### Remove Test Script
```bash
# Delete the test script
rm scripts/ci/test-snyk-setup.sh
```

### 4. Rollback Environment Variables

#### Local Development
Remove any local environment variable exports:

```bash
# Remove from .bashrc, .zshrc, or .env files
unset SNYK_TOKEN
unset SNYK_ORG
```

#### Container Environments
Remove environment variable declarations from Docker Compose files:

```yaml
# Remove from docker-compose.yml
environment:
  - SNYK_TOKEN=${SNYK_TOKEN}
  - SNYK_ORG=${SNYK_ORG}
```

### 5. Rollback Snyk CLI Installation

#### Local System
```bash
# Uninstall Snyk CLI globally
npm uninstall -g snyk

# Or using yarn
yarn global remove snyk
```

#### CI/CD Environment
Snyk CLI will be automatically reinstalled if the job runs, but to prevent this:

```yaml
script:
  # Remove the npm install step
  # - npm install -g snyk  # Remove this line
```

## Emergency Rollback Checklist

### Immediate Actions
- [ ] Stop all CI/CD pipelines using Snyk
- [ ] Revoke Snyk API tokens (via Snyk dashboard)
- [ ] Delete AWS Secrets Manager secret
- [ ] Remove environment variables from all systems
- [ ] Update CI/CD configuration to remove Snyk jobs

### Verification Steps
- [ ] Confirm no Snyk processes are running
- [ ] Verify secrets are deleted from AWS
- [ ] Check that CI/CD pipelines no longer reference Snyk
- [ ] Validate that local systems no longer have Snyk credentials

### Communication
- [ ] Notify development team of rollback
- [ ] Update security team about token revocation
- [ ] Document rollback reason for future reference

## Recovery After Rollback

### To Re-enable Snyk (if desired)
1. Obtain new API token from Snyk
2. Store in AWS Secrets Manager
3. Re-add CI/CD job configuration
4. Test the setup with test script
5. Monitor for successful scans

### Alternative Security Scanning
If rolling back Snyk permanently, consider:
- Continue using `cargo audit` for Rust dependencies
- Use `npm audit` for Node.js dependencies
- Implement `cargo-deny` for license compliance
- Consider other scanning tools like `OWASP Dependency-Check`

## Monitoring and Alerts

### Post-Rollback Monitoring
- Watch for any failed CI/CD jobs that reference removed scripts
- Monitor AWS CloudTrail for unauthorized secret access attempts
- Check application logs for Snyk-related errors

### Alert Configuration
Set up alerts for:
- Failed secret retrieval attempts
- Unauthorized AWS API calls to Secrets Manager
- CI/CD job failures related to missing dependencies

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0   | 2025-09-10 | Initial rollback procedures documentation |

## Contact Information

For questions about rollback procedures:
- DevOps Team: devops@company.com
- Security Team: security@company.com
- Infrastructure Team: infra@company.com