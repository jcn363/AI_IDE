# Environment Variables Documentation

This document provides comprehensive documentation for all environment variables used in the RUST AI IDE project. Environment variables are organized by category and include information about their purpose, format, default values, and environment-specific overrides.

## Table of Contents

1. [Notification Endpoints](#notification-endpoints)
2. [Security Alert Thresholds](#security-alert-thresholds)
3. [Monitoring Parameters](#monitoring-parameters)
4. [CI/CD Pipeline Variables](#cicd-pipeline-variables)
5. [Application Configuration](#application-configuration)
6. [Deployment Targets](#deployment-targets)
7. [Rollback Configuration](#rollback-configuration)
8. [Security Settings](#security-settings)

## Notification Endpoints

### Email Configuration

| Variable | Purpose | Format | Security Note |
|----------|---------|--------|---------------|
| `EMAIL_SMTP_HOST` | SMTP server hostname for email notifications | `string` (e.g., `smtp.gmail.com`) | Public |
| `EMAIL_SMTP_PORT` | SMTP server port | `number` (e.g., `587`) | Public |
| `EMAIL_FROM_ADDRESS` | Email address used as sender | `email` (e.g., `noreply@rust-ai-ide.com`) | Public |
| `EMAIL_USERNAME` | SMTP authentication username | `string` | **Secure - From Secret Manager** |
| `EMAIL_PASSWORD` | SMTP authentication password | `string` | **Secure - From Secret Manager** |

### Slack Configuration

| Variable | Purpose | Format | Security Note |
|----------|---------|--------|---------------|
| `SLACK_WEBHOOK_URL` | Slack webhook URL for notifications | `URL` (e.g., `https://hooks.slack.com/...`) | **Secure - From Secret Manager** |
| `SLACK_CHANNEL` | Slack channel for alerts | `string` (e.g., `#alerts`) | Public |

### Webhook Configuration

| Variable | Purpose | Format | Security Note |
|----------|---------|--------|---------------|
| `WEBHOOK_URL` | External webhook endpoint for notifications | `URL` | Public |
| `WEBHOOK_SECRET` | Secret key for webhook authentication | `string` | **Secure - From Secret Manager** |

## Security Alert Thresholds

### System Resource Thresholds

| Variable | Purpose | Format | Default | Range |
|----------|---------|--------|---------|-------|
| `CPU_THRESHOLD_HIGH` | CPU usage percentage threshold for alerts | `number` | `80` | 0-100 |
| `MEMORY_THRESHOLD_HIGH` | Memory usage percentage threshold for alerts | `number` | `85` | 0-100 |
| `DISK_THRESHOLD_HIGH` | Disk usage percentage threshold for alerts | `number` | `90` | 0-100 |

### Security Alert Levels

| Variable | Purpose | Format | Default | Range |
|----------|---------|--------|---------|-------|
| `SECURITY_ALERT_LEVEL` | Overall security alert sensitivity | `number` | `3` | 1-5 |
| `SECURITY_SCAN_INTERVAL_MINUTES` | Interval between security scans | `number` | `60` | 1-1440 |
| `MAX_FAILED_LOGIN_ATTEMPTS` | Maximum failed login attempts before lockout | `number` | `5` | 1-20 |
| `SECURITY_VIOLATION_COOLDOWN_MINUTES` | Cooldown period after security violation | `number` | `15` | 1-60 |

## Monitoring Parameters

### Monitoring Configuration

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `MONITORING_ENABLED` | Enable/disable system monitoring | `boolean` | `true` |
| `MONITORING_INTERVAL_SECONDS` | Interval between monitoring checks | `number` | `300` |
| `LOG_LEVEL` | Logging verbosity level | `string` | `info` |
| `LOG_RETENTION_DAYS` | Number of days to retain logs | `number` | `30` |

### Performance Monitoring

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `PERFORMANCE_MONITORING_ENABLED` | Enable performance monitoring | `boolean` | `true` |
| `PERFORMANCE_ALERT_THRESHOLD_MS` | Response time threshold for alerts | `number` | `5000` |

### Health Check Configuration

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `HEALTH_CHECK_ENDPOINT` | Health check API endpoint | `string` | `/health` |
| `HEALTH_CHECK_INTERVAL_SECONDS` | Interval between health checks | `number` | `60` |

## CI/CD Pipeline Variables

### CI Platform Configuration

| Variable | Purpose | Format | Example |
|----------|---------|--------|---------|
| `CI_PLATFORM` | CI/CD platform being used | `string` | `gitlab`, `github` |
| `CI_PROJECT_ID` | Project identifier in CI platform | `string` | `your_project_id` |
| `CI_COMMIT_REF_NAME` | Branch or tag name | `string` | `main` |

### Deployment Configuration

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `DEPLOYMENT_ENVIRONMENT` | Target deployment environment | `string` | `development` |
| `DEPLOYMENT_TARGET` | Deployment target type | `string` | `edge`, `lambda`, `container` |
| `DEPLOYMENT_STRATEGY` | Deployment strategy to use | `string` | `rolling`, `blue-green` |

### Build Configuration

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `BUILD_TIMEOUT_MINUTES` | Maximum build duration | `number` | `30` |
| `TEST_COVERAGE_THRESHOLD` | Minimum test coverage percentage | `number` | `80` |

### Security Scanning

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `SECURITY_SCAN_ENABLED` | Enable security scanning in CI | `boolean` | `true` |
| `DEPENDENCY_SCAN_ENABLED` | Enable dependency vulnerability scanning | `boolean` | `true` |
| `SECRET_SCAN_ENABLED` | Enable secret detection scanning | `boolean` | `true` |

## Application Configuration

### Runtime Configuration

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `NODE_ENV` | Application environment | `string` | `development` |
| `DATABASE_URL` | Database connection string | `URL` | `sqlite://./db.sqlite` |
| `SECRET_KEY` | Application secret key | `string` | **Secure - From Secret Manager** |

### AI Service Configuration

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `AI_SERVICE_ENDPOINT` | AI service API endpoint | `URL` | `https://api.ai-service.com` |
| `AI_MODEL_DEFAULT` | Default AI model to use | `string` | `gpt-4` |
| `AI_REQUEST_TIMEOUT_SECONDS` | Timeout for AI requests | `number` | `30` |

### LSP Configuration

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `LSP_ENABLED` | Enable Language Server Protocol | `boolean` | `true` |
| `LSP_PORT` | LSP server port | `number` | `3000` |

## Deployment Targets

### Cloud Provider Configuration

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `CLOUD_PROVIDER` | Primary cloud provider | `string` | `aws` |
| `AWS_REGION` | AWS region for resources | `string` | `us-east-1` |
| `GCP_PROJECT_ID` | Google Cloud project ID | `string` | `your_gcp_project` |

### Container Configuration

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `CONTAINER_PLATFORM` | Container orchestration platform | `string` | `docker` |
| `KUBERNETES_NAMESPACE` | Kubernetes namespace | `string` | `default` |

### Edge Runtime Configuration

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `EDGE_RUNTIME_ENABLED` | Enable edge runtime deployment | `boolean` | `true` |
| `EDGE_FUNCTIONS_ENDPOINT` | Edge functions API endpoint | `URL` | `https://edge.example.com` |

### Serverless Configuration

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `LAMBDA_FUNCTION_NAME` | AWS Lambda function name | `string` | `rust-ai-ide-lambda` |
| `LAMBDA_REGION` | AWS Lambda region | `string` | `us-east-1` |

## Rollback Configuration

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `ROLLBACK_ENABLED` | Enable automatic rollback on failures | `boolean` | `true` |
| `ROLLBACK_VERSIONS_TO_KEEP` | Number of versions to keep for rollback | `number` | `5` |
| `ROLLBACK_TIMEOUT_MINUTES` | Maximum time for rollback operation | `number` | `10` |
| `BLUE_GREEN_ENABLED` | Enable blue-green deployment | `boolean` | `true` |
| `BLUE_GREEN_PROMOTE_TIMEOUT_MINUTES` | Timeout for blue-green promotion | `number` | `5` |

## Security Settings

### Encryption Configuration

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `ENCRYPTION_ENABLED` | Enable data encryption | `boolean` | `true` |
| `ENCRYPTION_KEY_ROTATION_DAYS` | Key rotation interval | `number` | `90` |

### Rate Limiting

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `RATE_LIMIT_REQUESTS_PER_MINUTE` | Rate limit per minute | `number` | `100` |
| `RATE_LIMIT_BURST` | Burst capacity for rate limiting | `number` | `20` |

### Audit Logging

| Variable | Purpose | Format | Default |
|----------|---------|--------|---------|
| `AUDIT_LOG_ENABLED` | Enable audit logging | `boolean` | `true` |
| `AUDIT_LOG_RETENTION_DAYS` | Audit log retention period | `number` | `90` |

### TLS Configuration

| Variable | Purpose | Format | Example |
|----------|---------|--------|---------|
| `TLS_CERT_PATH` | Path to TLS certificate file | `path` | `/etc/ssl/certs/cert.pem` |
| `TLS_KEY_PATH` | Path to TLS private key file | `path` | `/etc/ssl/private/key.pem` |

## Environment-Specific Configurations

### Development (.env.development)

- `DEBUG_ENABLED=true`
- `TRACE_ENABLED=true`
- `LOG_LEVEL=debug`
- `SESSION_SECURE_COOKIE=false`
- Lower thresholds for quicker feedback

### Staging (.env.staging)

- Balanced configuration for testing
- `LOG_LEVEL=info`
- `SECURITY_ALERT_LEVEL=2`
- Blue-green deployments enabled
- TLS certificates configured

### Production (.env.production)

- Optimized for performance and security
- `LOG_LEVEL=warn`
- `SECURITY_ALERT_LEVEL=4`
- Full monitoring and alerting enabled
- Service mesh and edge runtime configured
- Strict rate limiting and audit logging

## Setup Instructions

1. **Copy the example file:**
   ```bash
   cp .env.example .env
   ```

2. **Configure sensitive values:**
   - Use secret managers for credentials
   - Never commit actual secrets to version control
   - Rotate secrets regularly

3. **Environment-specific overrides:**
   - Copy `.env.example` to `.env.{environment}`
   - Override only necessary variables
   - Use environment loading order: `.env` → `.env.{NODE_ENV}`

4. **Validation:**
   - Ensure all required variables are set
   - Validate URLs and email formats
   - Test configurations in staging before production

## Security Best Practices

- **Never hard-code secrets** in configuration files
- **Use managed secret services** (AWS Secrets Manager, HashiCorp Vault, etc.)
- **Rotate credentials regularly** as defined in `ENCRYPTION_KEY_ROTATION_DAYS`
- **Implement least privilege** access for all service accounts
- **Monitor secret usage** and access patterns
- **Backup encrypted secrets** securely

## Troubleshooting

### Common Issues

1. **Missing environment variables:**
   - Check `.env` file exists and is properly loaded
   - Verify variable names match exactly
   - Ensure no typos in variable declarations

2. **Security configuration failures:**
   - Validate secret manager access
   - Check TLS certificate paths and permissions
   - Verify encryption keys are properly formatted

3. **Notification failures:**
   - Test webhook endpoints manually
   - Verify SMTP credentials and server settings
   - Check Slack webhook URL validity

4. **Deployment failures:**
   - Validate cloud provider credentials
   - Check container registry access
   - Verify deployment target configurations

### Debugging

- Enable `DEBUG_ENABLED=true` in development
- Check application logs for configuration errors
- Use `LOG_LEVEL=debug` for detailed logging
- Validate configurations with dedicated test scripts

## Version History

- **v1.0** - Initial environment variable configuration
- Comprehensive documentation and examples
- Environment-specific configurations
- Security best practices implementation

---

For questions or issues with environment variable configuration, refer to the project documentation or contact the DevOps team.