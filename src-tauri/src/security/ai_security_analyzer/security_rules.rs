//! Security Rules Factory
//!
//! This module centralizes the initialization logic for SecurityRule and SecretPattern
//! definitions used by the AI Security Analyzer. It eliminates duplicated initialization
//! code and provides a clean interface for managing security rules and patterns.

use crate::security::*;
use regex::Regex;

/// Security Rules Factory
pub struct SecurityRulesFactory;

impl SecurityRulesFactory {
    /// Initialize security rules with advanced patterns
    pub fn initialize_security_rules() -> Result<Vec<SecurityRule>, Box<dyn std::error::Error>> {
        let rules = Self::create_security_rules_vec()?;
        Ok(rules)
    }

    /// Initialize secret patterns for hardcoded credential detection
    pub fn initialize_secret_patterns() -> Result<Vec<SecretPattern>, Box<dyn std::error::Error>> {
        let patterns = Self::create_secret_patterns_vec()?;
        Ok(patterns)
    }

    /// Create security rules vector - centralized for easy maintenance
    fn create_security_rules_vec() -> Result<Vec<SecurityRule>, Box<dyn std::error::Error>> {
        let rules = vec![
            // Command injection patterns - enhanced with more variants
            Self::create_security_rule(
                "direct_command_injection".to_string(),
                SecurityCategory::CommandInjection,
                r#"std::process::Command::new\([^)]*\b(stdin|stdout|stderr|arg|args)\s*[=,]\)"#,
                SecuritySeverity::High,
                "Potential command injection through unvalidated command arguments".to_string(),
                "Use whitelist of allowed commands and arguments. Validate and sanitize all user inputs.".to_string(),
                Some(78),
            )?,

            // Weak TLS configuration - expanded to include more variants
            Self::create_security_rule(
                "weak_tls_config".to_string(),
                SecurityCategory::CryptographicIssues,
                r#"(native_tls|rustls)::(TlsConnector|TlsAcceptor)::builder\(\)\.(danger_accept_invalid_certs|danger_accept_invalid_hostnames|danger_accept_invalid_cert_hashes|danger_accept_invalid_cert_subject_alt_names)\(true\)"#,
                SecuritySeverity::High,
                "Insecure TLS configuration that disables certificate validation".to_string(),
                "Always validate server certificates, use proper certificate pinning, and verify hostnames".to_string(),
                Some(295),
            )?,

            // Format string vulnerabilities - more comprehensive detection
            Self::create_security_rule(
                "format_string_vulnerability".to_string(),
                SecurityCategory::InputValidation,
                r#"\b((?:print|eprint|write|writeln|format|panic|unreachable|unimplemented|todo|assert|debug_assert)(?:!|_unchecked!)?)\(\s*[^"]*\{\s*\}\s*[^"]*\)"#,
                SecuritySeverity::Medium,
                "Potential format string vulnerability".to_string(),
                "Avoid using format strings with user-controlled input. Use format strings with explicit arguments or the Display trait.".to_string(),
                Some(134),
            )?,

            // SQL injection patterns
            Self::create_security_rule(
                "sql_injection".to_string(),
                SecurityCategory::SqlInjection,
                r#"(?:sqlx|diesel|rusqlite|postgres|mysql)::.*\.(?:query|execute|query_map|query_row|query_one|query_scalar|query_and_then|query_map_and_then|query_row_and_then|query_row_then|query_row_then_map|query_row_then_try_map|query_row_then_try_map_and_then|query_row_then_try_unwrap|query_row_then_try_unwrap_or_else|query_row_then_unwrap|query_row_then_unwrap_or_else|query_then|query_then_and_then|query_then_map|query_then_map_and_then|query_then_try_map|query_then_try_map_and_then|query_then_try_unwrap|query_then_try_unwrap_or_else|query_then_unwrap|query_then_unwrap_or_else)\s*\(\s*[^,]*\s*,\s*format!"#,
                SecuritySeverity::High,
                "Potential SQL injection through string formatting in SQL queries".to_string(),
                "Use parameterized queries or prepared statements instead of string formatting for SQL queries".to_string(),
                Some(89),
            )?,

            // Insecure deserialization
            Self::create_security_rule(
                "insecure_deserialization".to_string(),
                SecurityCategory::InputValidation,
                r#"serde_(json|yaml|toml|ron|pickle|bincode)::from_(?:str|reader|slice|read|deserialize)\([^)]*\buntrusted_"#,
                SecuritySeverity::High,
                "Potential insecure deserialization of untrusted data".to_string(),
                "Validate and sanitize all deserialized data. Use strongly-typed structs and avoid deserializing to generic types like Value.".to_string(),
                Some(502),
            )?,

            // Integer overflow patterns with more comprehensive detection
            Self::create_security_rule(
                "unchecked_arithmetic".to_string(),
                SecurityCategory::CryptographicIssues,
                r#"\b((?:u8|u16|u32|u64|u128|usize|i8|i16|i32|i64|i128|isize)\s*[+\-*/%]\s*[^\s;)]+)|([^\s;)]+\s*[+\-*/%]\s*(?:u8|u16|u32|u64|u128|usize|i8|i16|i32|i64|i128|isize))"#,
                SecuritySeverity::High,
                "Potential integer overflow in arithmetic operation".to_string(),
                "Use checked, saturating, or wrapping arithmetic operations to handle potential overflows".to_string(),
                Some(190),
            )?,

            // Unsafe transmute operations with more context
            Self::create_security_rule(
                "unsafe_transmute".to_string(),
                SecurityCategory::MemorySafety,
                r#"\bunsafe\s+\{\s*std::mem::transmute\s*\}"#,
                SecuritySeverity::Critical,
                "Unsafe memory transmutation can lead to undefined behavior".to_string(),
                "Avoid using std::mem::transmute. Use safer alternatives like std::mem::transmute_copy or proper type conversion methods".to_string(),
                Some(704),
            )?,

            // Potential SQL injection
            Self::create_security_rule(
                "sql_injection".to_string(),
                SecurityCategory::SqlInjection,
                r#"\.execute\(\s*\"(SELECT|INSERT|UPDATE|DELETE).*\{"#,
                SecuritySeverity::High,
                "Potential SQL injection through string formatting".to_string(),
                "Use prepared statements or an ORM to prevent SQL injection".to_string(),
                Some(89),
            )?,

            // Hardcoded credentials
            Self::create_security_rule(
                "hardcoded_credentials".to_string(),
                SecurityCategory::HardcodedSecrets,
                r#"\b(password|passwd|pwd|secret|token|key|credential|api[_-]?key|auth|apikey)[\s=:]+[\"'][^\s\"']+[\"']"#,
                SecuritySeverity::High,
                "Hardcoded sensitive information".to_string(),
                "Store sensitive information in environment variables or secure configuration files".to_string(),
                Some(798),
            )?,

            // Insecure random number generation
            Self::create_security_rule(
                "insecure_random".to_string(),
                SecurityCategory::CryptographicIssues,
                r#"\b(rand::thread_rng\(\s*\)|rand::random\s*\(\s*\)|rand::rngs::ThreadRng::default\(\s*\))"#,
                SecuritySeverity::Medium,
                "Insecure random number generation for security-sensitive operations".to_string(),
                "Use cryptographically secure random number generators like rand::rngs::OsRng or rand_core::OsRng".to_string(),
                Some(338),
            )?,

            // Potential XSS in web frameworks
            Self::create_security_rule(
                "potential_xss".to_string(),
                SecurityCategory::InputValidation,
                r#"(response\.set_body\([^)]*\b(innerHtml|outerHtml|document\.write|document\.writeln|eval|setTimeout|setInterval|Function|execScript)\s*\()"#,
                SecuritySeverity::High,
                "Potential Cross-Site Scripting (XSS) vulnerability".to_string(),
                "Use proper output encoding and avoid inserting untrusted data into HTML, JavaScript, or other interpreted contexts".to_string(),
                Some(79),
            )?,

            // Insecure deserialization
            Self::create_security_rule(
                "insecure_deserialization".to_string(),
                SecurityCategory::InputValidation,
                r#"\b(serde_json|bincode|rmp_serde|ron|toml|yaml)_(from_str|from_slice|from_reader|from_reader_unchecked|from_reader_buf|from_reader_buf_unchecked)\s*\([^)]*\b(File|std::fs::File|std::io::Cursor|Vec<u8>|\[u8\])"#,
                SecuritySeverity::High,
                "Potential insecure deserialization of untrusted data".to_string(),
                "Validate and sanitize all deserialized data. Consider using a safe deserialization library or implementing custom validation".to_string(),
                Some(502),
            )?,

            // Potential path traversal
            Self::create_security_rule(
                "path_traversal".to_string(),
                SecurityCategory::PathTraversal,
                r#"std::fs::(read|read_to_string|read_to_string|read_dir|read_link|canonicalize|metadata|symlink_metadata|remove_file|remove_dir|remove_dir_all|create_dir|create_dir_all|write|OpenOptions::new\(\s*)\([^)]*\b(Path|PathBuf|str|String)\s*,\s*[^)]*"#,
                SecuritySeverity::High,
                "Potential path traversal vulnerability".to_string(),
                "Validate and sanitize all file paths. Use std::path::Path::components() to normalize paths and check for traversal attempts".to_string(),
                Some(22),
            )?,

            // SQL injection patterns
            Self::create_security_rule(
                "raw_sql_query".to_string(),
                SecurityCategory::SqlInjection,
                r#"\.execute\(\s*\"(SELECT|INSERT|UPDATE|DELETE).*\{"#,
                SecuritySeverity::Critical,
                "Potential SQL injection through raw SQL query".to_string(),
                "Use prepared statements or an ORM to prevent SQL injection".to_string(),
                Some(89),
            )?,
        ];

        Ok(rules)
    }

    /// Create secret patterns vector - centralized for easy maintenance
    fn create_secret_patterns_vec() -> Result<Vec<SecretPattern>, Box<dyn std::error::Error>> {
        let patterns = vec![
            SecretPattern {
                name: "AWS Access Key".to_string(),
                pattern: Regex::new(r"AKIA[0-9A-Z]{16}")?,
                severity: SecuritySeverity::Critical,
                confidence: 0.95,
            },
            SecretPattern {
                name: "AWS Secret Key".to_string(),
                pattern: Regex::new(r"[A-Za-z0-9/+=]{40}")?,
                severity: SecuritySeverity::Critical,
                confidence: 0.7,
            },
            SecretPattern {
                name: "GitHub Token".to_string(),
                pattern: Regex::new(r"ghp_[A-Za-z0-9]{36}")?,
                severity: SecuritySeverity::High,
                confidence: 0.9,
            },
            SecretPattern {
                name: "API Key".to_string(),
                pattern: Regex::new(
                    r#"(?i)(api[_-]?key|apikey)["'\s]*[:=]["'\s]*[A-Za-z0-9]{20,}"#,
                )?,
                severity: SecuritySeverity::High,
                confidence: 0.8,
            },
            SecretPattern {
                name: "Password".to_string(),
                pattern: Regex::new(r#"(?i)(password|passwd|pwd)["'\s]*[:=]["'\s]*[^\s"']{8,}"#)?,
                severity: SecuritySeverity::Medium,
                confidence: 0.6,
            },
            SecretPattern {
                name: "Private Key".to_string(),
                pattern: Regex::new(r"-----BEGIN [A-Z ]+PRIVATE KEY-----")?,
                severity: SecuritySeverity::Critical,
                confidence: 0.95,
            },
            SecretPattern {
                name: "JWT Token".to_string(),
                pattern: Regex::new(r"eyJ[A-Za-z0-9_-]*\.[A-Za-z0-9_-]*\.[A-Za-z0-9_-]*")?,
                severity: SecuritySeverity::High,
                confidence: 0.85,
            },
        ];

        Ok(patterns)
    }

    /// Helper method to create a SecurityRule with proper initialization
    fn create_security_rule(
        name: String,
        category: SecurityCategory,
        pattern_str: &str,
        severity: SecuritySeverity,
        description: String,
        remediation: String,
        cwe_id: Option<u32>,
    ) -> Result<SecurityRule, Box<dyn std::error::Error>> {
        Ok(SecurityRule {
            name,
            category,
            pattern: Regex::new(pattern_str)?,
            severity,
            description,
            remediation,
            cwe_id,
        })
    }

    /// Helper method to create a SecretPattern with proper initialization
    fn create_secret_pattern(
        name: String,
        pattern_str: &str,
        severity: SecuritySeverity,
        confidence: f32,
    ) -> Result<SecretPattern, Box<dyn std::error::Error>> {
        Ok(SecretPattern {
            name,
            pattern: Regex::new(pattern_str)?,
            severity,
            confidence,
        })
    }
}
