//! Security analysis tests for the Rust AI IDE

use rust_ai_ide_ai::analysis::security::{HardcodedSecretsDetector, InsecureCryptoDetector, SqlInjectionDetector};
use rust_ai_ide_ai::analysis::{AnalysisRegistry, AnalysisResult, AnalysisType, Severity};
use rust_ai_ide_ai::test_helpers::*;

/// Test detection of insecure cryptographic functions
#[test]
fn test_insecure_crypto_detection() {
    let code = r#"
        use md5::Md5;
        use sha1::Sha1;
        use openssl::hash::MessageDigest;

        fn insecure_hash() {
            // These should be flagged as insecure
            let md5 = md5::compute(b"password");
            let sha1 = sha1::Sha1::digest(b"password");
            let md4 = openssl::hash::hash(MessageDigest::md4(), b"password").unwrap();

            // This is secure and shouldn't be flagged
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(b"password");
            let _secure_hash = hasher.finalize();
        }
    "#;

    let mut registry = AnalysisRegistry::default();
    registry.register_security_analyzer(InsecureCryptoDetector::default());

    let result = registry.analyze_code(code, "crypto_test.rs").unwrap();

    // Should flag md5, sha1, and md4 usage
    assert_finding!(
        &result,
        AnalysisType::InsecureCrypto,
        Severity::High,
        "Insecure hashing function 'md5::compute' detected"
    );

    assert_finding!(
        &result,
        AnalysisType::InsecureCrypto,
        Severity::High,
        "Insecure hashing function 'sha1::Sha1' detected"
    );

    assert_finding!(
        &result,
        AnalysisType::InsecureCrypto,
        Severity::High,
        "Insecure hashing function 'openssl::hash::MessageDigest::md4' detected"
    );

    // Should not flag the secure SHA-256 usage
    assert!(
        !result.findings.iter().any(|f| f.message.contains("Sha256")),
        "Secure SHA-256 usage should not be flagged"
    );
}

/// Test detection of hardcoded secrets
#[test]
fn test_hardcoded_secrets_detection() {
    let code = r#"
        // These should be flagged as hardcoded secrets
        const API_KEY: &str = "sk_test_mock_api_key_placeholder_value";
        const PASSWORD: &str = "mock_password_placeholder_value";

        // These should not be flagged
        const MAX_RETRIES: u32 = 3;
        const TIMEOUT: &str = "30s";

        fn main() {
            // This should be flagged
            let db_password = "postgres:mock_password@localhost:5432/mydb";

            // This should not be flagged
            let url = "https://example.com/api";
        }
    "#;

    let mut registry = AnalysisRegistry::default();
    registry.register_security_analyzer(HardcodedSecretsDetector::default());

    let result = registry.analyze_code(code, "secrets_test.rs").unwrap();

    // Should flag hardcoded API key, password, and database URL
    assert_finding!(
        &result,
        AnalysisType::HardcodedSecret,
        Severity::High,
        "Potential hardcoded API key detected"
    );

    assert_finding!(
        &result,
        AnalysisType::HardcodedSecret,
        Severity::High,
        "Potential hardcoded password detected"
    );

    assert_finding!(
        &result,
        AnalysisType::HardcodedSecret,
        Severity::High,
        "Potential hardcoded database credentials detected"
    );

    // Should not flag the non-secret constants
    assert!(
        !result
            .findings
            .iter()
            .any(|f| f.message.contains("MAX_RETRIES") || f.message.contains("TIMEOUT")),
        "Non-secret constants should not be flagged"
    );
}

/// Test detection of SQL injection vulnerabilities
#[test]
fn test_sql_injection_detection() {
    let code = r#"
        use sqlx::PgPool;

        // Vulnerable to SQL injection
        async fn get_user_vulnerable(pool: &PgPool, username: &str) -> Result<(), sqlx::Error> {
            let query = format!("SELECT * FROM users WHERE username = '{}'", username);
            sqlx::query(&query).fetch_one(pool).await?;
            Ok(())
        }

        // Safe - uses parameterized queries
        async fn get_user_safe(pool: &PgPool, username: &str) -> Result<(), sqlx::Error> {
            sqlx::query!("SELECT * FROM users WHERE username = $1", username)
                .fetch_one(pool)
                .await?;
            Ok(())
        }

        // Safe - uses query builder
        async fn get_user_safe_builder(pool: &PgPool, username: &str) -> Result<(), sqlx::Error> {
            sqlx::query("SELECT * FROM users WHERE username = ?")
                .bind(username)
                .fetch_one(pool)
                .await?;
            Ok(())
        }
    "#;

    let mut registry = AnalysisRegistry::default();
    registry.register_security_analyzer(SqlInjectionDetector::default());

    let result = registry.analyze_code(code, "sql_test.rs").unwrap();

    // Should flag the vulnerable SQL query construction
    assert_finding!(
        &result,
        AnalysisType::SqlInjection,
        Severity::Critical,
        "Potential SQL injection vulnerability detected"
    );

    // Should not flag the safe queries
    assert!(
        !result
            .findings
            .iter()
            .any(|f| f.message.contains("get_user_safe") || f.message.contains("get_user_safe_builder")),
        "Safe SQL queries should not be flagged"
    );
}

/// Test that the security analyzers work together
#[test]
fn test_security_analyzers_together() {
    let code = r#"
        // Multiple security issues in one file
        const DB_PASSWORD: &str = "mock_db_password_placeholder";

        fn hash_password(password: &str) -> String {
            let hash = md5::compute(password.as_bytes());
            format!("{:x}", hash)
        }

        fn get_user(pool: &sqlx::PgPool, user_id: &str) -> Result<(), sqlx::Error> {
            let query = format!("SELECT * FROM users WHERE id = {}", user_id);
            sqlx::query(&query).fetch_one(pool).await?;
            Ok(())
        }
    "#;

    let mut registry = AnalysisRegistry::default();
    registry.register_security_analyzer(InsecureCryptoDetector::default());
    registry.register_security_analyzer(HardcodedSecretsDetector::default());
    registry.register_security_analyzer(SqlInjectionDetector::default());

    let result = registry.analyze_code(code, "security_test.rs").unwrap();

    // Should find all three types of security issues
    assert!(
        result
            .findings
            .iter()
            .any(|f| f.analysis_type == AnalysisType::InsecureCrypto),
        "Expected to find insecure crypto usage"
    );

    assert!(
        result
            .findings
            .iter()
            .any(|f| f.analysis_type == AnalysisType::HardcodedSecret),
        "Expected to find hardcoded secrets"
    );

    assert!(
        result
            .findings
            .iter()
            .any(|f| f.analysis_type == AnalysisType::SqlInjection),
        "Expected to find SQL injection vulnerability"
    );
}

/// Test that security findings have the correct severity levels
#[test]
fn test_security_finding_severity() {
    let code = r#"
        // Critical: SQL injection
        fn get_user(pool: &sqlx::PgPool, user_id: &str) -> Result<(), sqlx::Error> {
            let query = format!("SELECT * FROM users WHERE id = {}", user_id);
            sqlx::query(&query).fetch_one(pool).await?;
            Ok(())
        }

        // High: Insecure crypto
        fn hash_password(password: &str) -> String {
            let hash = md5::compute(password.as_bytes());
            format!("{:x}", hash)
        }

        // Medium: Hardcoded API key with test prefix (less critical)
        const TEST_API_KEY: &str = "test_mock_api_key_placeholder";
    "#;

    let mut registry = AnalysisRegistry::default();
    registry.register_security_analyzer(InsecureCryptoDetector::default());
    registry.register_security_analyzer(HardcodedSecretsDetector::default());
    registry.register_security_analyzer(SqlInjectionDetector::default());

    let result = registry.analyze_code(code, "severity_test.rs").unwrap();

    // Check severities
    let sql_injection = result
        .findings
        .iter()
        .find(|f| f.analysis_type == AnalysisType::SqlInversion);
    assert_eq!(
        sql_injection.unwrap().severity,
        Severity::Critical,
        "SQL injection should be Critical"
    );

    let insecure_crypto = result
        .findings
        .iter()
        .find(|f| f.analysis_type == AnalysisType::InsecureCrypto);
    assert_eq!(
        insecure_crypto.unwrap().severity,
        Severity::High,
        "Insecure crypto should be High"
    );

    let hardcoded_secret = result
        .findings
        .iter()
        .find(|f| f.analysis_type == AnalysisType::HardcodedSecret);
    assert_eq!(
        hardcoded_secret.unwrap().severity,
        Severity::Medium,
        "Test API key should be Medium"
    );
}
