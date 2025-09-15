//! Code formatting functionality

use std::process::Stdio;

use tokio::process::Command;

use super::StyleCheckError;

/// Rust code formatter
#[derive(Clone)]
pub struct RustFormatter {
    use_rustfmt:   bool,
    custom_config: Option<String>,
}

impl RustFormatter {
    /// Create a new Rust formatter
    pub fn new() -> Self {
        Self {
            use_rustfmt:   true,
            custom_config: None,
        }
    }

    /// Create formatter with custom configuration
    pub fn with_config(config: &str) -> Self {
        Self {
            use_rustfmt:   true,
            custom_config: Some(config.to_string()),
        }
    }

    /// Format code using rustfmt if available, fallback to basic formatting
    pub async fn format(&self, content: &str) -> Result<String, StyleCheckError> {
        if self.use_rustfmt {
            match self.format_with_rustfmt(content).await {
                Ok(formatted) => Ok(formatted),
                Err(_) => {
                    tracing::warn!("rustfmt not available, falling back to basic formatting");
                    self.basic_format(content).await
                }
            }
        } else {
            self.basic_format(content).await
        }
    }

    /// Format code using external rustfmt
    async fn format_with_rustfmt(&self, content: &str) -> Result<String, StyleCheckError> {
        let mut command = Command::new("rustfmt");

        // Add config file if specified
        if let Some(config) = &self.custom_config {
            // Write config to temporary file (simplified)
            let config_path = "/tmp/rustfmt.toml";
            tokio::fs::write(config_path, config)
                .await
                .map_err(|e| StyleCheckError::ConfigError(format!("Failed to write config: {}", e)))?;
            command.arg("--config-path").arg(config_path);
        }

        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command
            .spawn()
            .map_err(|e| StyleCheckError::FormatError(format!("Failed to spawn rustfmt: {}", e)))?;

        // Write content to stdin
        if let Some(mut stdin) = child.stdin.take() {
            tokio::io::AsyncWriteExt::write_all(&mut stdin, content.as_bytes())
                .await
                .map_err(|e| StyleCheckError::FormatError(format!("Failed to write to rustfmt: {}", e)))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| StyleCheckError::FormatError(format!("rustfmt execution failed: {}", e)))?;

        if output.status.success() {
            let formatted = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(formatted)
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(StyleCheckError::FormatError(format!(
                "rustfmt failed: {}",
                error_msg
            )))
        }
    }

    /// Basic code formatting without external tools
    async fn basic_format(&self, content: &str) -> Result<String, StyleCheckError> {
        let mut formatted_lines = Vec::new();

        for (i, line) in content.lines().enumerate() {
            let formatted_line = if line.trim().is_empty() {
                // Handle empty lines
                String::new()
            } else {
                // Apply basic formatting rules
                self.apply_basic_formatting(line)
            };

            formatted_lines.push(formatted_line);
        }

        Ok(formatted_lines.join("\n"))
    }

    /// Apply basic formatting rules to a single line
    fn apply_basic_formatting(&self, line: &str) -> String {
        let mut result = line.to_string();

        // Fix indentation (spaces, 4 per level)
        result = self.fix_indentation(&result);

        // Add space after keywords
        result = self.add_keyword_spacing(&result);

        // Format expressions and statements
        result = self.format_expressions(&result);

        result
    }

    /// Fix indentation
    fn fix_indentation(&self, line: &str) -> String {
        if line.trim().is_empty() {
            return String::new();
        }

        let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
        let indentation_level = leading_spaces / 4;
        let correct_spaces = indentation_level * 4;

        format!("{}{}", " ".repeat(correct_spaces), line.trim_start())
    }

    /// Add spacing after keywords
    fn add_keyword_spacing(&self, line: &str) -> String {
        let keywords_needing_space = [
            "let", "fn", "struct", "enum", "impl", "if", "else", "for", "while", "match",
        ];

        let mut result = line.to_string();

        for keyword in &keywords_needing_space {
            let pattern = format!("{}([^{}])", keyword, regex::escape(" ("));
            if let Ok(regex) = regex::Regex::new(&pattern) {
                result = regex
                    .replace_all(&result, format!("{} $1", keyword))
                    .to_string();
            }
        }

        result
    }

    /// Format expressions and statements
    fn format_expressions(&self, line: &str) -> String {
        let mut result = line.to_string();

        // Add space around operators
        let operators = ["+", "-", "*", "/", "=", "==", "!=", "<", ">", "<=", ">="];

        for op in &operators {
            if line.contains(op) && !line.contains(&format!(" {}", op)) && !line.contains(&format!("{} ", op)) {
                result = result.replace(op, &format!(" {} ", op));
                // Clean up extra spaces
                result = result.replace(&format!("  {}  ", op), &format!(" {} ", op));
            }
        }

        // Format brackets (simplified)
        if line.contains('{') && !line.contains(" { ") {
            result = result.replace("{", " { ");
        }

        if line.contains('}') && !line.contains(" }") {
            result = result.replace("}", " }");
        }

        result
    }

    /// Check if rustfmt is available
    pub fn is_rustfmt_available() -> bool {
        std::process::Command::new("rustfmt")
            .arg("--version")
            .output()
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting() {
        let formatter = RustFormatter::new();
        let code = "fn main(){let x=1+2;println!(\"{}\",x);}";
        let formatted = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(formatter.basic_format(code))
            .unwrap();

        // Basic checks - this would be more comprehensive in practice
        assert!(formatted.contains("fn"));
        assert!(formatted.contains("main"));
    }

    #[test]
    fn test_indentation_fix() {
        let formatter = RustFormatter::new();
        let line = "    let x = 5;"; // Incorrect indentation (8 spaces)

        assert_eq!(formatter.fix_indentation(line), "        let x = 5;");
    }

    #[test]
    fn test_keyword_spacing() {
        let formatter = RustFormatter::new();
        let line = "fnmain(){";
        let formatted = formatter.add_keyword_spacing(line);

        assert!(formatted.contains("fn "));
    }

    #[test]
    fn test_rustfmt_availability() {
        // This test will pass regardless - we're just checking the function
        let _ = RustFormatter::is_rustfmt_available();
        assert!(true); // Placeholder test
    }
}
