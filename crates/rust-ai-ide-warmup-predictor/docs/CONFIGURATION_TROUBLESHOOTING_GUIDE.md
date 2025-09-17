# Model Warmup Prediction System - Configuration Troubleshooting Guide

This guide provides comprehensive troubleshooting procedures for configuration issues in the Model Warmup Prediction System, covering setup problems, parameter validation, environment configuration, and configuration management across all 7 core components.

## Configuration Architecture

### Configuration Sources

1. **Environment Variables**: Runtime configuration via env vars
2. **Configuration Files**: YAML/JSON files for static config
3. **Database Configuration**: Dynamic configuration stored in SQLite
4. **Command-line Arguments**: Override configuration at startup
5. **Remote Configuration**: Centralized config management

### Configuration Hierarchy

```rust
// Configuration loading priority (highest to lowest)
const CONFIG_PRIORITY = [
    CommandLineArgs,        // Highest priority
    EnvironmentVariables,
    LocalConfigFiles,
    DatabaseConfig,
    RemoteConfig,
    DefaultValues,         // Lowest priority
];
```

## Common Configuration Issues

### Environment Variable Problems

#### Issue: Environment Variables Not Loaded

**Symptoms:**
- Configuration defaults being used
- "CONFIG_ENV_NOT_FOUND" warnings
- Features not working as expected

**Resolution Steps:**
```bash
# Check environment variables
echo $WARMUP_PREDICTOR_CONFIG_PATH
echo $WARMUP_DB_PATH
echo $WARMUP_MODEL_PATH

# Export required variables
export WARMUP_PREDICTOR_CONFIG_PATH=/etc/warmup/config.yaml
export WARMUP_DB_PATH=/var/lib/warmup/data.db
export WARMUP_MODEL_PATH=/opt/warmup/models/

# Verify variables are set
env | grep WARMUP
```

#### Issue: Environment Variable Parsing Errors

**Symptoms:**
- Configuration parsing failures
- "INVALID_ENV_VALUE" errors
- Type conversion errors

**Resolution:**
```rust
use rust_ai_ide_warmup_predictor::config::EnvironmentParser;

let parser = EnvironmentParser::new();

// Validate environment variables
let validation = parser.validate_environment().await?;
for error in validation.errors {
    println!("Env error: {} - {}", error.variable, error.message);
}

// Fix common parsing issues
let config = parser.parse_with_fallbacks().await?;
```

### Configuration File Issues

#### Issue: Configuration File Not Found

**Symptoms:**
- "CONFIG_FILE_NOT_FOUND" errors
- Default configuration used
- Missing custom settings

**Resolution:**
```bash
# Check configuration file paths
ls -la /etc/warmup/config.yaml
ls -la ~/.config/warmup/config.yaml
ls -la ./config/warmup.yaml

# Create default configuration
cat > /etc/warmup/config.yaml << EOF
warmup_predictor:
  max_warm_models: 5
  prediction_threshold: 0.7
  usage_window_seconds: 3600

components:
  usage_pattern_analyzer:
    enabled: true
    batch_size: 1000
  prediction_engine:
    enabled: true
    model_path: "/opt/warmup/models"
EOF
```

#### Issue: YAML/JSON Syntax Errors

**Symptoms:**
- Configuration parsing failures
- "INVALID_CONFIG_SYNTAX" errors
- Partial configuration loaded

**Resolution:**
```yaml
# Validate YAML syntax
yamllint /etc/warmup/config.yaml

# Common syntax fixes
warmup_predictor:
  max_warm_models: 5  # Correct: integer
  # Wrong: max_warm_models: "5"

  prediction_threshold: 0.7  # Correct: float
  # Wrong: prediction_threshold: "0.7"

  enabled: true  # Correct: boolean
  # Wrong: enabled: "true"
```

### Database Configuration Issues

#### Issue: Database Connection Failures

**Symptoms:**
- "DB_CONNECTION_FAILED" errors
- Configuration not persisting
- State not being saved

**Resolution:**
```rust
use rust_ai_ide_warmup_predictor::config::DatabaseConfig;

let db_config = DatabaseConfig::new();

// Test database connection
let connection_test = db_config.test_connection().await?;
println!("DB Connection: {}", connection_test.status);

// Initialize database schema
db_config.initialize_schema().await?;

// Migrate existing data if needed
db_config.migrate_data().await?;
```

#### Issue: Database Schema Mismatches

**Symptoms:**
- Configuration fields missing
- "SCHEMA_MISMATCH" errors
- Data corruption warnings

**Resolution:**
```sql
-- Check current schema
.schema warmup_config

-- Apply schema migration
ALTER TABLE warmup_config ADD COLUMN new_field TEXT DEFAULT 'default_value';

-- Verify data integrity
SELECT COUNT(*) FROM warmup_config WHERE config_data IS NOT NULL;
```

## Component-Specific Configuration

### UsagePatternAnalyzer Configuration

```yaml
usage_pattern_analyzer:
  enabled: true
  data_collection:
    interval_seconds: 60
    retention_days: 30
    batch_size: 1000
  pattern_analysis:
    min_pattern_confidence: 0.6
    analysis_window_hours: 24
    enable_seasonal_analysis: true
  storage:
    max_memory_mb: 256
    compression_enabled: true
    backup_interval_hours: 6
```

**Common Issues:**
- `interval_seconds` too high → stale data
- `batch_size` too large → memory issues
- `min_pattern_confidence` too low → false patterns

### PredictionEngine Configuration

```yaml
prediction_engine:
  enabled: true
  model:
    path: "/opt/warmup/models/prediction_model.bin"
    version: "1.2.0"
    auto_update: true
  prediction:
    threshold: 0.7
    max_concurrent_predictions: 10
    cache_enabled: true
    cache_ttl_seconds: 300
  performance:
    target_latency_ms: 100
    max_memory_mb: 512
```

**Common Issues:**
- Model path incorrect → loading failures
- Cache TTL too short → cache misses
- Memory limit too low → OOM errors

### ResourceManager Configuration

```yaml
resource_manager:
  enabled: true
  limits:
    max_memory_mb: 1024
    max_cpu_percent: 70.0
    max_disk_mb: 5120
  monitoring:
    interval_seconds: 30
    alert_threshold_percent: 80
    enable_predictive_scaling: true
  allocation:
    strategy: "fair_share"
    preemption_enabled: false
```

**Common Issues:**
- Limits too restrictive → allocation failures
- Monitoring interval too long → delayed alerts
- Strategy incompatible → unfair allocation

## Environment-Specific Configuration

### Development Environment

```yaml
environment: development
logging:
  level: debug
  file_enabled: true
  console_enabled: true
features:
  debug_mode: true
  performance_monitoring: true
  health_checks: true
```

### Production Environment

```yaml
environment: production
logging:
  level: info
  file_enabled: true
  console_enabled: false
security:
  audit_logging: true
  rate_limiting: true
  encryption: true
performance:
  optimization_level: maximum
  caching: aggressive
```

### Testing Environment

```yaml
environment: testing
features:
  mock_services: true
  fast_startup: true
  minimal_logging: true
data:
  use_test_database: true
  populate_sample_data: true
```

## Configuration Validation

### Automatic Validation

```rust
use rust_ai_ide_warmup_predictor::config::ConfigValidator;

let validator = ConfigValidator::new();

// Validate entire configuration
let validation_result = validator.validate_config(config).await?;
if !validation_result.is_valid {
    for error in validation_result.errors {
        println!("Config error: {}", error.message);
    }
}

// Validate specific component
let component_validation = validator.validate_component("PredictionEngine", config).await?;
println!("Component valid: {}", component_validation.is_valid);
```

### Manual Validation Checklist

- [ ] All required fields present
- [ ] Data types correct
- [ ] Value ranges valid
- [ ] File paths exist and accessible
- [ ] Database connections working
- [ ] Network endpoints reachable
- [ ] Security credentials valid
- [ ] Resource limits reasonable

## Configuration Hot Reloading

### Enabling Hot Reloading

```rust
use rust_ai_ide_warmup_predictor::config::HotReloader;

let reloader = HotReloader::new();

// Watch configuration files
reloader.watch_file("/etc/warmup/config.yaml").await?;

// Enable automatic reloading
reloader.enable_auto_reload(true).await?;

// Handle configuration changes
reloader.on_config_change(|new_config| async move {
    println!("Configuration reloaded");
    // Apply new configuration
    apply_new_config(new_config).await?;
    Ok(())
}).await?;
```

### Hot Reload Safety

```rust
// Safe configuration transitions
let transition = ConfigTransition::new(old_config, new_config);

// Validate transition safety
let safety_check = transition.validate_safety().await?;
if safety_check.is_safe {
    // Apply transition
    transition.apply().await?;
} else {
    println!("Unsafe transition: {}", safety_check.reason);
}
```

## Configuration Backup and Recovery

### Automatic Backups

```rust
use rust_ai_ide_warmup_predictor::config::ConfigBackup;

let backup = ConfigBackup::new();

// Schedule regular backups
backup.schedule_backup(Duration::from_secs(3600)).await?; // Hourly

// Backup on configuration changes
backup.enable_change_backup(true).await?;

// Store backups securely
backup.set_backup_location("/secure/backup/warmup-config").await?;
```

### Recovery Procedures

```bash
# List available backups
./scripts/list-config-backups.sh

# Restore from backup
./scripts/restore-config-backup.sh 2025-09-17-14-00-00

# Validate restored configuration
./scripts/validate-config.sh /etc/warmup/config.yaml

# Apply restored configuration
./scripts/apply-config.sh /etc/warmup/config.yaml
```

## Configuration Security

### Secure Configuration Storage

```rust
use rust_ai_ide_warmup_predictor::security::SecureConfig;

let secure_config = SecureConfig::new();

// Encrypt sensitive configuration
let encrypted = secure_config.encrypt_sensitive_fields(config).await?;

// Store securely
secure_config.store_secure_config(encrypted).await?;

// Retrieve and decrypt
let decrypted = secure_config.retrieve_secure_config().await?;
```

### Configuration Access Control

```rust
use rust_ai_ide_warmup_predictor::security::ConfigAccessControl;

let access_control = ConfigAccessControl::new();

// Define access policies
access_control.set_policy("admin", vec![
    Permission::ReadAll,
    Permission::WriteAll,
    Permission::Backup,
    Permission::Restore,
]);

// Enforce access control
access_control.enforce_access(user, operation).await?;
```

## Troubleshooting Tools

### Configuration Diagnostic Commands

```bash
# Validate configuration syntax
warmup-config validate /etc/warmup/config.yaml

# Check configuration completeness
warmup-config completeness /etc/warmup/config.yaml

# Compare configurations
warmup-config diff config1.yaml config2.yaml

# Generate configuration template
warmup-config generate-template > config-template.yaml

# Test configuration loading
warmup-config test-load /etc/warmup/config.yaml
```

### Configuration Health Dashboard

```rust
use rust_ai_ide_warmup_predictor::diagnostics::ConfigDashboard;

let dashboard = ConfigDashboard::new();

// Display configuration health
dashboard.display_health().await?;

// Show configuration issues
dashboard.show_issues().await?;

// Generate health report
let report = dashboard.generate_report().await?;
println!("{}", report);
```

## Best Practices

### Configuration Management

1. **Use Version Control**: Store configurations in Git
2. **Environment Separation**: Different configs per environment
3. **Documentation**: Document all configuration options
4. **Validation**: Always validate configuration changes
5. **Backup**: Regular configuration backups
6. **Security**: Encrypt sensitive configuration data
7. **Monitoring**: Monitor configuration health
8. **Testing**: Test configuration changes before deployment

### Configuration Change Process

1. **Plan Changes**: Document intended changes and rollback plan
2. **Validate**: Test configuration in staging environment
3. **Backup**: Create backup of current configuration
4. **Apply**: Apply changes with monitoring
5. **Verify**: Verify system behavior after changes
6. **Monitor**: Monitor for issues post-deployment
7. **Document**: Update configuration documentation

This configuration troubleshooting guide provides comprehensive procedures for diagnosing and resolving configuration issues in the Model Warmup Prediction System. Proper configuration management is crucial for system reliability and performance.