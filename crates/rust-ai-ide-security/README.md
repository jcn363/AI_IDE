# rust-ai-ide-security

A comprehensive enterprise-grade security and compliance framework for the Rust AI IDE, implementing zero-trust security, comprehensive audit logging, GDPR/CCPA compliance, encrypted configurations, and role-based access control.

Built for v2.4.0, this crate provides multi-layered security through orchestrated components including identity management, network security, data protection, and continuous verification, all with full audit trails.

## 📦 Features

- **Zero Trust Security**: Continuous verification for all AI operations
- **Role-Based Access Control (RBAC)**: Hierarchical permissions with multi-tenant support
- **Audit Logging**: Complete audit trails for compliance and forensics
- **Data Protection**: AES256-GCM encryption with key rotation policies
- **GDPR/CCPA Compliance**: Automated data handling and privacy controls
- **Key Management**: HSM-backed key lifecycle management and credential rotation
- **Network Security**: TLS enforcement and secure communication channels
- **Identity & Access Management**: JWT-based authentication with MFA support
- **Emergency Access Procedures**: Break-glass protocols for critical situations

## 🔗 Architecture Integration

The security framework integrates deeply with:

- `rust-ai-ide-core`: Core operations and context management
- `rust-ai-ide-cache`: Encrypted cache storage with access controls
- `rust-ai-ide-lsp`: Secure language server communication
- `rust-ai-ide-ai-codegen`: Authorized AI operation execution
- `rust-ai-ide-types`: Secure data type serialization

## 🚀 Usage

### Basic Usage - Security Engine Setup

```rust
use rust_ai_ide_security::{SecurityConfig, SecurityEngine, OperationContext, OperationType};

// Create default enterprise security configuration
let config = SecurityConfig::default();

// Initialize comprehensive security engine
let security_engine = SecurityEngine::new(config).await?;

// Create operation context
let ctx = OperationContext {
    user_context: UserContext::new("user123"),
    network_context: NetworkContext {
        ip_address: "192.168.1.100".to_string(),
        tls_version: "TLSv1.3".to_string(),
        ..Default::default()
    },
    resource_context: ResourceContext {
        resource_type: "ai-model".to_string(),
        resource_id: "codellama-7b".to_string(),
        action: "inference".to_string(),
        sensitivity_level: SensitivityLevel::Confidential,
    },
    ..Default::default()
};

// Secure operation execution
let result = security_engine.secure_operation(&ctx, async {
    // Your secure AI operation here
    process_ai_inference(&ai_request).await
}).await?;
```

### Role-Based Access Control

```rust
use rust_ai_ide_security::rbac::{RoleBasedAccessControl, Permission};

let rbac = RoleBasedAccessControl::new().await?;

// Create hierarchical roles
let developer_role = rbac.create_role(
    "developer",
    "Developer with project access",
    [Permission::UseModel("codellama-7b".to_string())].into(),
    vec![],
    None
).await?;

// Assign role with expiration
rbac.assign_role("user123", &developer_role, "admin", Some(chrono::Utc::now() + chrono::Duration::days(30)), None).await?;

// Check permissions
let user_ctx = UserContext::new("user123");
let authorized = rbac.check_permission_action(&user_ctx, "ai.model.use", Some("codellama-7b")).await?;
```

### Audit Logging

```rust
use rust_ai_ide_security::audit::AuditLogger;

let audit = AuditLogger::new(AuditConfig::default()).await?;

// Log security events
audit.log_event(AuditEvent {
    event_type: "ai_operation",
    user_id: "user123",
    resource_id: "codellama-7b",
    action: "inference",
    success: true,
    details: serde_json::json!({"tokens": 150, "model": "codellama-7b"}),
    ..Default::default()
}).await?;
```

### Key Management

```rust
use rust_ai_ide_security::key_management::{KeyManager, create_software_key_manager};

// Initialize key manager
let key_manager = create_software_key_manager().await;

// Generate encryption key with automatic rotation
let key_id = key_manager.generate_key("production", "aes256").await?;

// Encrypt sensitive data
let encrypted = key_manager.encrypt_data(&key_id, b"sensitive_configuration").await?;

// Rotate key after policy
let new_key_id = key_manager.rotate_key(&key_id).await?;
```

### Encryption & Compliance

```rust
use rust_ai_ide_security::{EncryptedConfigManager, ComplianceEngine};

// Encrypted configuration management
let config_manager = EncryptedConfigManager::new(EncryptionConfig::default()).await?;
let encrypted_config = config_manager.store_config("ai_secrets", &config_data).await?;

// GDPR compliance validation
let compliance = ComplianceEngine::new(ComplianceConfig {
    gdpr_compliance: true,
    data_retention_years: 7,
    anonymization_enabled: true,
    ..Default::default()
}).await?;

let compliant = compliance.validate_operation_compliance(&operation_context).await?;
```

## 📚 Integration Guide

### Multi-Layer Security Setup

```rust
use rust_ai_ide_security::{SecurityConfig, SecurityEngine};

// Configure all security layers
let config = SecurityConfig {
    zero_trust: ZeroTrustConfig {
        continuous_verification: true,
        mfa_required: true,
        session_timeout_minutes: 30,
        ..Default::default()
    },
    rbac: RBACConfig {
        permission_cache_size: 5000,
        session_validation_seconds: 300,
        ..Default::default()
    },
    audit: AuditConfig {
        retention_days: 2555, // 7 years for compliance
        real_time_monitoring: true,
        ..Default::default()
    },
    encryption: EncryptionConfig {
        algorithm: "aes256-gcm".to_string(),
        key_rotation_days: 90,
        master_key_secure_store: true,
        ..Default::default()
    },
    compliance: ComplianceConfig {
        gdpr_compliance: true,
        ccpa_compliance: true,
        data_retention_years: 7,
        ..Default::default()
    },
    network: NetworkSecurityConfig {
        tls_enforce: true,
        rate_limiting: true,
        ..Default::default()
    },
    ..Default::default()
};

let security_engine = SecurityEngine::new(config).await?;
```

### Health Monitoring

```rust
// Get comprehensive security health status
let health = security_engine.health_check().await?;
println!("RBAC Status: {:?}", health.rbac_status);
println!("Encryption Status: {:?}", health.encryption_status);
println!("Compliance Status: {:?}", health.compliance_status);
```

## 📈 Performance Characteristics

- **Efficient RBAC**: Cached permission evaluation with sub-millisecond lookups
- **Optimized Encryption**: AES256-GCM with minimized overhead on data operations
- **Background Auditing**: Asynchronous audit logging without blocking operations
- **Scalable Key Management**: Support for thousands of active keys with rotation
- **Memory Efficient**: Lazy loading of security contexts and cached configurations
- **Multi-Threaded**: Async operations support concurrent security evaluations

### Security Performance Best Practices

1. **Configure Appropriate Caching**:
   ```rust
   let rbac_config = RBACConfig {
       permission_cache_size: 10000, // Adjust based on user count
       session_validation_seconds: 300, // 5-minute sessions
       ..Default::default()
   };
   ```

2. **Enable Audit Compression**:
   ```rust
   let audit_config = AuditConfig {
       compression_enabled: true, // Reduce storage overhead
       retention_days: 2555,
       ..Default::default()
   };
   ```

3. **Key Rotation Schedule**:
   ```rust
   let key_policy = KeyRotationPolicy {
       rotation_interval_days: 90, // Balance security vs performance
       keep_versions: 5, // Limit version history
       rotation_schedule: Some(RotationSchedule {
           maintenance_window_start: "02:00",
           maintenance_window_end: "06:00",
           ..Default::default()
       }),
       ..Default::default()
   };
   ```

## 🔄 Migration Notes

### v2.4.0 Security Framework Introduction

This enterprise security framework replaces ad-hoc security implementations:

- **Before**: Fragmented security checks across modules
- **After**: Unified, orchestrated security engine with comprehensive coverage

### Migration Path

1. **Update Dependencies**:
   ```rust
   // Add security crate
   rust-ai-ide-security = { version = "2.4.0", features = ["rbac", "audit", "encryption"] }
   ```

2. **Replace Direct Security Checks**:
   ```rust
   // Before: Direct permission check
   if user_has_permission(user, "ai.use") { /* ... */ }

   // After: Integrated security engine
   let result = security_engine.secure_operation(&ctx, operation).await?;
   ```

3. **Migrate Audit Logging**:
   ```rust
   // Before: Custom audit logging
   log_audit_event(event);

   // After: Standardized audit trails
   audit.log_event(event).await?;
   ```

4. **Update Configuration Management**:
   ```rust
   // Before: Plain configuration storage
   save_config(config);

   // After: Encrypted configuration storage
   config_manager.store_config("app_config", &config).await?;
   ```

### Enterprise Compliance Setup

```rust
// Complete GDPR/CCPA compliance configuration
let compliance_config = ComplianceConfig {
    gdpr_compliance: true,
    ccpa_compliance: true,
    data_retention_years: 7,
    anonymization_enabled: true,
    consent_management: true,
};

// Network security for enterprise
let network_config = NetworkSecurityConfig {
    tls_enforce: true,
    certificate_validation: true,
    secure_headers: true,
    rate_limiting: true,
};
```

### Breaking Changes

- **Security-First Defaults**: All security features enabled by default
- **Required Contexts**: All operations now require security context
- **Encrypted Storage**: Configuration files now encrypted by default
- **Audit Requirements**: All operations generate audit logs

This migration ensures enterprise-grade security while providing backward compatibility through the unified SecurityEngine interface.