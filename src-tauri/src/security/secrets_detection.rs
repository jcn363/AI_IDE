use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use regex::{Regex, RegexSet};

// Lazy static for regex patterns to avoid recompilation
static SECRET_PATTERNS: OnceLock<RegexSet> = OnceLock::new();
static SECRET_PATTERNS_BUILDER: OnceLock<Vec<Regex>> = OnceLock::new();

/// Initialize and return the regex set for secret patterns
fn get_secret_patterns() -> &'static RegexSet {
    SECRET_PATTERNS.get_or_init(|| {
        let patterns = vec![
            // API keys (various formats)
            r"(?i)api[_-]?key[_-]?[:=]\s*['""]?([A-Za-z0-9_-]{20,})['""]?",
            r"(?i)secret[_-]?key[_-]?[:=]\s*['""]?([A-Za-z0-9_-]{20,})['""]?",
            // AWS credentials
            r"(?i)aws[_-]?access[_-]?key[_-]?id[:=]\s*['""]?([A-Z0-9]{20})['""]?",
            r"(?i)aws[_-]?secret[_-]?access[_-]?key[:=]\s*['""]?([A-Za-z0-9/+=]{40})['""]?",
            // Database connection strings
            r"(?i)mongodb://([A-Za-z0-9_-]+:[A-Za-z0-9_-]+@)",
            r"(?i)postgres://([A-Za-z0-9_-]+:[A-Za-z0-9_-]+@)",
            r"(?i)mysql://([A-Za-z0-9_-]+:[A-Za-z0-9_-]+@)",
            // JWT tokens
            r"(?i)eyJ[A-Za-z0-9_-]*\.eyJ[A-Za-z0-9_-]*\.[A-Za-z0-9_-]*",
            // Generic high-entropy strings (fallback)
            r"\b[A-Za-z0-9+/=]{32,}\b", // Base64-like
        ];
        RegexSet::new(patterns).expect("Failed to compile secret patterns")
    })
}

/// Get individual regex builders for extracting matches
fn get_secret_patterns_builder() -> &'static Vec<Regex> {
    SECRET_PATTERNS_BUILDER.get_or_init(|| {
        vec![
            Regex::new(r"(?i)api[_-]?key[_-]?[:=]\s*['""]?([A-Za-z0-9_-]{20,})['""]?").unwrap(),
            Regex::new(r"(?i)secret[_-]?key[_-]?[:=]\s*['""]?([A-Za-z0-9_-]{20,})['""]?").unwrap(),
            Regex::new(r"(?i)aws[_-]?access[_-]?key[_-]?id[:=]\s*['""]?([A-Z0-9]{20})['""]?").unwrap(),
            Regex::new(r"(?i)aws[_-]?secret[_-]?access[_-]?key[:=]\s*['""]?([A-Za-z0-9/+=]{40})['""]?").unwrap(),
            Regex::new(r"(?i)mongodb://([A-Za-z0-9_-]+:[A-Za-z0-9_-]+@)").unwrap(),
            Regex::new(r"(?i)postgres://([A-Za-z0-9_-]+:[A-Za-z0-9_-]+@)").unwrap(),
            Regex::new(r"(?i)mysql://([A-Za-z0-9_-]+:[A-Za-z0-9_-]+@)").unwrap(),
            Regex::new(r"(?i)eyJ[A-Za-z0-9_-]*\.eyJ[A-Za-z0-9_-]*\.[A-Za-z0-9_-]*").unwrap(),
            Regex::new(r"\b[A-Za-z0-9+/=]{32,}\b").unwrap(),
        ]
    })
}

/// Calculate Shannon entropy for a string to detect potential secrets
fn calculate_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }

    let mut freq = HashMap::new();
    for c in s.chars() {
        *freq.entry(c).or_insert(0) += 1;
    }

    let len = s.len() as f64;
    let mut entropy = 0.0;

    for &count in freq.values() {
        let p = count as f64 / len;
        entropy -= p * p.log2();
    }

    entropy
}

/// Struct to represent a detected potential secret
#[derive(Debug, Clone)]
pub struct SecretFinding {
    pub file_path:     String,
    pub line_number:   usize,
    pub line_content:  String,
    pub secret_type:   String,
    pub entropy_score: f64,
    pub confidence:    String,
}

/// Scan a single file for potential secrets
pub fn scan_file_for_secrets(file_path: &Path) -> Result<Vec<SecretFinding>, std::io::Error> {
    let content = fs::read_to_string(file_path)?;
    let mut findings = Vec::new();

    // Read lines for line number tracking
    for (line_num, line) in content.lines().enumerate() {
        let line_str = line.trim();

        // Skip empty lines or comments
        if line_str.is_empty() || line_str.starts_with("//") || line_str.starts_with("#") {
            continue;
        }

        // Check regex patterns
        let patterns = get_secret_patterns_builder();
        for (idx, pattern) in patterns.iter().enumerate() {
            if let Some(captures) = pattern.captures(line_str) {
                if let Some(matched) = captures.get(1) {
                    let secret_candidate = matched.as_str();
                    let entropy = calculate_entropy(secret_candidate);

                    // High confidence if entropy > 4.0 and length > 10
                    let confidence = if entropy > 4.0 && secret_candidate.len() > 10 {
                        "high"
                    } else if secret_candidate.len() > 20 {
                        "medium"
                    } else {
                        "low"
                    };

                    let secret_type = match idx {
                        0 => "API Key",
                        1 => "Secret Key",
                        2 => "AWS Access Key ID",
                        3 => "AWS Secret Access Key",
                        4 => "MongoDB Connection",
                        5 => "PostgreSQL Connection",
                        6 => "MySQL Connection",
                        7 => "JWT Token",
                        _ => "High Entropy String",
                    };

                    findings.push(SecretFinding {
                        file_path:     file_path.to_string_lossy().to_string(),
                        line_number:   line_num + 1,
                        line_content:  line.to_string(),
                        secret_type:   secret_type.to_string(),
                        entropy_score: entropy,
                        confidence:    confidence.to_string(),
                    });
                }
            }
        }

        // Additional entropy-based detection for lines not caught by patterns
        for word in line_str.split_whitespace() {
            if word.len() > 20 {
                let entropy = calculate_entropy(word);
                if entropy > 4.5 {
                    findings.push(SecretFinding {
                        file_path:     file_path.to_string_lossy().to_string(),
                        line_number:   line_num + 1,
                        line_content:  line.to_string(),
                        secret_type:   "High Entropy String".to_string(),
                        entropy_score: entropy,
                        confidence:    "medium".to_string(),
                    });
                }
            }
        }
    }

    Ok(findings)
}

/// Scan a directory recursively for potential secrets, with path validation
pub fn scan_directory_for_secrets(dir_path: &Path, max_file_size: u64) -> Result<Vec<SecretFinding>, std::io::Error> {
    let mut all_findings = Vec::new();

    fn scan_recursive(
        current_path: &Path,
        max_file_size: u64,
        findings: &mut Vec<SecretFinding>,
    ) -> Result<(), std::io::Error> {
        if current_path.is_dir() {
            for entry in fs::read_dir(current_path)? {
                let entry = entry?;
                let path = entry.path();

                // Skip common non-sensitive directories and files
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                if file_name.starts_with('.')
                    || file_name == "target"
                    || file_name == "node_modules"
                    || file_name == "build"
                    || path
                        .extension()
                        .map_or(false, |ext| ext == "log" || ext == "tmp")
                {
                    continue;
                }

                scan_recursive(&path, max_file_size, findings)?;
            }
        } else if current_path.is_file() {
            // Check file size to avoid scanning large files
            if let Ok(metadata) = current_path.metadata() {
                if metadata.len() > max_file_size {
                    return Ok(());
                }
            }

            // Only scan text-based files
            if let Some(ext) = current_path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if matches!(
                    ext_str.as_str(),
                    "rs" | "js"
                        | "ts"
                        | "py"
                        | "java"
                        | "cpp"
                        | "h"
                        | "json"
                        | "yaml"
                        | "yml"
                        | "toml"
                        | "md"
                        | "txt"
                        | "env"
                ) {
                    findings.append(&mut scan_file_for_secrets(current_path)?);
                }
            }
        }

        Ok(())
    }

    scan_recursive(dir_path, max_file_size, &mut all_findings)?;
    Ok(all_findings)
}

/// Integration point with security manager - placeholder for future integration
pub fn integrate_with_security_manager(findings: Vec<SecretFinding>) -> Result<(), String> {
    // Placeholder implementation - in real integration, this would:
    // 1. Log findings using audit_logger
    // 2. Update security metrics
    // 3. Trigger alerts if high-confidence secrets found
    // 4. Store findings in secure storage

    if findings.is_empty() {
        return Ok(());
    }

    // Aggregate findings by confidence
    let mut high_confidence = 0;
    let mut medium_confidence = 0;
    let mut low_confidence = 0;

    for finding in &findings {
        match finding.confidence.as_str() {
            "high" => high_confidence += 1,
            "medium" => medium_confidence += 1,
            "low" => low_confidence += 1,
            _ => {}
        }
    }

    // Log summary (placeholder - would use actual audit_logger)
    println!("Secrets detection completed:");
    println!("High confidence findings: {}", high_confidence);
    println!("Medium confidence findings: {}", medium_confidence);
    println!("Low confidence findings: {}", low_confidence);

    if high_confidence > 0 {
        // Would trigger security alert here
        println!("WARNING: High-confidence secrets detected! Manual review required.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_calculate_entropy() {
        assert_eq!(calculate_entropy(""), 0.0);
        assert_eq!(calculate_entropy("a"), 0.0);
        assert!(calculate_entropy("abcdefghijklmnop") > 4.0);
    }

    #[test]
    fn test_scan_file_with_secrets() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "const API_KEY = 'sk-1234567890abcdef1234567890abcdef';"
        )
        .unwrap();
        writeln!(temp_file, "normal code here").unwrap();

        let findings = scan_file_for_secrets(temp_file.path()).unwrap();
        assert!(!findings.is_empty());
        assert_eq!(findings[0].secret_type, "High Entropy String");
    }
}
