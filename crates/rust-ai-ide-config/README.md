# Unified Security-First Configuration System for Rust AI IDE

[![Crates.io](https://img.shields.io/crates/v/rust-ai-ide-config.svg)](https://crates.io/crates/rust-ai-ide-config)
[![Documentation](https://docs.rs/rust-ai-ide-config/badge.svg)](https://docs.rs/rust-ai-ide-config)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

A comprehensive, security-first configuration management system for the Rust AI IDE ecosystem with multi-source support, audit trails, hot reloading, and intelligent caching.

## 🛡️ Key Features

- **Multi-Source Support**: Load from files, environment variables, and databases with priority-based merging
- **Security Validation**: Path traversal prevention, input sanitization, and threat detection
- **Audit Trails**: Encrypted logging of all configuration changes with tamper detection
- **Hot Reloading**: Zero-downtime configuration updates with file watching
- **Intelligent Caching**: Performance optimization using the unified cache system
- **Format Auto-Detection**: Automatic detection and parsing of TOML, YAML, and JSON files
- **Developer Experience**: Comprehensive validation feedback and migration support
- **Migration System**: Automatic configuration upgrades with backup and rollback capabilities

## 🚀 Quick Start

```rust
use rust_ai_ide_config::{ConfigurationManager, Config, FileSource, EnvironmentSource};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyAppConfig {
    api_key: String,
    max_connections: usize,
    debug: bool,
}

impl Config for MyAppConfig {
    const FILE_PREFIX: &'static str = "myapp";
    const DESCRIPTION: &'static str = "My Application Configuration";

    fn validate(&self) -> Result<Vec<String>, anyhow::Error> {
        let mut errors = Vec::new();
        if self.max_connections == 0 {
            errors.push("max_connections must be > 0".to_string());
        }
        Ok(errors)
    }

    fn default_config() -> Self {
        Self {
            api_key: "default_key".to_string(),
            max_connections: 100,
            debug: false,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize configuration system
    rust_ai_ide_config::init();

    // Create configuration manager
    let mut manager = ConfigurationManager::new().await?;

    // Add configuration sources
    manager.add_source(
        rust_ai_ide_config::ConfigSourcePriority::File,
        FileSource::new()
    );

    manager.add_source(
        rust_ai_ide_config::ConfigSourcePriority::Environment,
        EnvironmentSource::new()
    );

    // Load configuration with security validation
    let config: MyAppConfig = manager.load_secure("myapp").await?;

    println!("Loaded configuration: {:?}", config);

    Ok(())
}
```

## 📋 Usage Examples

### Environment Variables

Set configuration via environment variables using the `RUST_AI_IDE_` prefix:

```bash
export RUST_AI_IDE_MYAPP_API_KEY="your-secret-key"
export RUST_AI_IDE_MYAPP_DEBUG="true"
export RUST_AI_IDE_MYAPP_MAX_CONNECTIONS="50"
```

### Configuration Files

Create configuration files in supported formats:

**TOML** (`myapp.toml`):
```toml
api_key = "file-key"
debug = true
max_connections = 200
```

**YAML** (`myapp.yaml`):
```yaml
api_key: "file-key"
debug: true
max_connections: 200
```

**JSON** (`myapp.json`):
```json
{
  "api_key": "file-key",
  "debug": true,
  "max_connections": 200
}
```

### Security Validation

```rust
use rust_ai_ide_config::SecurityValidator;

// Create validator with security level
let validator = SecurityValidator::new(SecurityLevel::High);

// Validate file paths
let safe_path = validator.validate_path(Path::new("./config/safe.toml"), None)?;
assert!(safe_path.exists());

// Sanitize user input
let sanitized = validator.sanitize_input("safe<input>", "field_name")?;
assert_eq!(sanitized, "safe");
```

### Audit Trails

```rust
// Get audit trail statistics
let stats = manager.audit_trail.get_stats().await?;
println!("Total audit events: {}", stats.total_entries);

// Search audit events
let filter = AuditSearchFilter::new()
    .config_name("myapp")
    .time_range(start_time, end_time);

let events = manager.audit_trail.search_events(filter).await?;
for event in events {
    println!("{:?}: {}", event.event_type, event.config_name);
}
```

### Hot Reloading

```rust
// Subscribe to configuration changes
let mut reload_rx = manager.hot_reload.subscribe();

// Watch configuration file
manager.hot_reload.watch_file("myapp", PathBuf::from("./myapp.toml")).await?;

tokio::spawn(async move {
    while let Ok(event) = reload_rx.recv().await {
        match event {
            ReloadEvent::ConfigChanged { config_name, .. } => {
                println!("Configuration {} changed, reloading...", config_name);
                // Reload configuration
                let new_config: MyAppConfig = manager.load_secure("myapp").await?;
            }
            _ => {}
        }
    }
    Ok::<(), Box<dyn std::error::Error>>(())
});
```

## 🔒 Security Features

- **Path Traversal Prevention**: Blocks attempts to access files outside allowed directories
- **Command Injection Prevention**: Sanitizes inputs to prevent command execution
- **Input Validation**: Comprehensive validation with sanitization
- **Audit Logging**: Encrypted tamper-evident logs of all configuration operations
- **Threat Detection**: Pattern-based detection of suspicious configuration content
- **Access Control**: Role-based access control for configuration operations

## 📚 Configuration Sources Priority

Configuration sources are merged in priority order (higher numbers override lower):

1. **Override (100)**: Manual overrides
2. **Database (30)**: Database-stored configurations
3. **Environment (20)**: Environment variables (`RUST_AI_IDE_*`)
4. **File (10)**: Configuration files (TOML/YAML/JSON)
5. **Default (0)**: Default configuration values

## 🏗️ Architecture

The configuration system is built with these core components:

- **`ConfigurationManager`**: Main entry point for configuration operations
- **`SecurityValidator`**: Input validation and threat prevention
- **`AuditTrail`**: Encrypted audit logging with tamper detection
- **`HotReloadManager`**: File watching and zero-downtime updates
- **`ConfigCache`**: Intelligent caching with TTL and invalidation
- **`ValidationEngine`**: Configuration validation with developer feedback
- **`MigrationEngine`**: Automatic configuration upgrades

## 📊 Performance

- **Intelligent Caching**: Reduces I/O operations with configurable TTL
- **Hot Reloading**: Zero-downtime configuration updates
- **Background Cleanup**: Automatic cleanup of expired cache entries
- **Memory Optimization**: Configurable memory limits and compression
- **Async Operations**: Non-blocking configuration loading and validation

## 🔧 Configuration

### Manager Configuration

```rust
let config = ManagerConfig {
    enable_audit: true,
    enable_hot_reload: true,
    enable_cache: true,
    security_level: SecurityLevel::High,
    config_directories: vec![PathBuf::from("./config")],
    env_prefix: "RUST_AI_IDE_".to_string(),
};
```

### Security Configuration

Choose from four security levels:

- **`Basic`**: Syntax validation and basic security checks
- **`Standard`**: Basic + threat detection
- **`High`**: Standard + comprehensive security validation
- **`Paranoid`**: High + zero-trust approach

## 📖 Migration Guide

### From Legacy Configuration Systems

1. **Implement the `Config` trait** for your configuration structs
2. **Replace manual file loading** with `ConfigurationManager`
3. **Add security validation** using `SecurityValidator`
4. **Enable audit logging** for compliance
5. **Set up hot reloading** for dynamic updates

### Breaking Changes

- Environment variable prefix changed to `RUST_AI_IDE_`
- Configuration files now require explicit security validation
- Cache behavior may differ from previous implementations

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🙏 Acknowledgments

- Built on the unified Rust AI IDE infrastructure
- Leverages battle-tested security patterns
- Inspired by production configuration management systems

---

For more information and advanced usage, see the [API Documentation](https://docs.rs/rust-ai-ide-config).