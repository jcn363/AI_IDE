//! Language detection and classification for multi-language LSP support
//!
//! This module provides intelligent detection of programming languages
//! based on file extensions, content analysis, and project structure.

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Detected programming language with confidence score
#[derive(Debug, Clone)]
pub struct LanguageDetection {
    pub language: String,
    pub confidence: f64,
    pub source: DetectionSource,
    pub extra_metadata: HashMap<String, String>,
}

/// Source of language detection
#[derive(Debug, Clone, PartialEq)]
pub enum DetectionSource {
    Extension,
    Content,
    ProjectStructure,
    PackageFile,
    Shebang,
}

/// Language detector configuration
#[derive(Debug, Clone)]
pub struct LanguageDetectorConfig {
    pub enable_content_analysis: bool,
    pub enable_shebang_detection: bool,
    pub max_content_size_kb: usize,
    pub confidence_threshold: f64,
}

impl Default for LanguageDetectorConfig {
    fn default() -> Self {
        Self {
            enable_content_analysis: true,
            enable_shebang_detection: true,
            max_content_size_kb: 1024, // 1MB
            confidence_threshold: 0.7,
        }
    }
}

/// Language detector that analyzes files and determines programming language
pub struct LanguageDetector {
    config: LanguageDetectorConfig,
    extension_map: HashMap<String, Vec<(String, f64)>>,
    pattern_map: Vec<(Regex, String, f64)>,
    shebang_patterns: Vec<(Regex, String)>,
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl LanguageDetector {
    /// Create a new language detector with configuration
    pub fn new(config: LanguageDetectorConfig) -> Self {
        let mut extension_map = HashMap::new();

        // Initialize file extension mappings
        Self::init_extension_map(&mut extension_map);

        // Initialize content pattern matching
        let pattern_map = Self::init_pattern_map();

        // Initialize shebang patterns
        let shebang_patterns = Self::init_shebang_patterns();

        Self {
            config,
            extension_map,
            pattern_map,
            shebang_patterns,
        }
    }

    /// Detect language for a file based on path and content
    pub fn detect_language(
        &self,
        file_path: &Path,
        content: Option<&str>,
    ) -> Vec<LanguageDetection> {
        let mut detections = Vec::new();

        // Detect by file extension
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            detections.extend(self.detect_by_extension(ext));
        }

        // Detect by file name for special cases
        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
            detections.extend(self.detect_by_filename(file_name));
        }

        // Detect by shebang
        if let Some(content) = content {
            if self.config.enable_shebang_detection {
                detections.extend(self.detect_by_shebang(content));
            }
        }

        // Detect by content patterns
        if let Some(content) = content {
            if self.config.enable_content_analysis
                && content.len() <= self.config.max_content_size_kb * 1024
            {
                detections.extend(self.detect_by_content(content));
            }
        }

        // Filter detections by confidence threshold and sort
        detections
            .into_iter()
            .filter(|d| d.confidence >= self.config.confidence_threshold)
            .collect()
    }

    /// Get all supported languages
    pub fn supported_languages(&self) -> HashSet<String> {
        let mut languages = HashSet::new();

        // Add languages from extensions
        for extensions in self.extension_map.values() {
            for (lang, _) in extensions {
                languages.insert(lang.clone());
            }
        }

        // Add languages from patterns
        for (_, lang, _) in &self.pattern_map {
            languages.insert(lang.clone());
        }

        // Add languages from shebang
        for (_, lang) in &self.shebang_patterns {
            languages.insert(lang.clone());
        }

        languages
    }

    /// Detect language by file extension
    fn detect_by_extension(&self, extension: &str) -> Vec<LanguageDetection> {
        let extension_key = extension.to_lowercase();

        if let Some(languages) = self.extension_map.get(&extension_key) {
            languages
                .iter()
                .map(|(lang, confidence)| LanguageDetection {
                    language: lang.clone(),
                    confidence: *confidence,
                    source: DetectionSource::Extension,
                    extra_metadata: HashMap::new(),
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Detect language by filename for special cases
    fn detect_by_filename(&self, filename: &str) -> Vec<LanguageDetection> {
        let filename_lower = filename.to_lowercase();

        match filename_lower.as_str() {
            "dockerfile" => vec![LanguageDetection {
                language: "dockerfile".to_string(),
                confidence: 1.0,
                source: DetectionSource::ProjectStructure,
                extra_metadata: HashMap::new(),
            }],
            "makefile" => vec![LanguageDetection {
                language: "makefile".to_string(),
                confidence: 1.0,
                source: DetectionSource::ProjectStructure,
                extra_metadata: HashMap::new(),
            }],
            "cmakefile.txt" | "cmakelists.txt" => vec![LanguageDetection {
                language: "cmake".to_string(),
                confidence: 1.0,
                source: DetectionSource::ProjectStructure,
                extra_metadata: HashMap::new(),
            }],
            _ => Vec::new(),
        }
    }

    /// Detect language by shebang line
    fn detect_by_shebang(&self, content: &str) -> Vec<LanguageDetection> {
        let mut detections = Vec::new();

        // Get first line
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("#!") {
                for (pattern, language) in &self.shebang_patterns {
                    if pattern.is_match(first_line) {
                        detections.push(LanguageDetection {
                            language: language.clone(),
                            confidence: 0.9,
                            source: DetectionSource::Shebang,
                            extra_metadata: HashMap::new(),
                        });
                        break; // Only one shebang is possible
                    }
                }
            }
        }

        detections
    }

    /// Detect language by content analysis
    fn detect_by_content(&self, content: &str) -> Vec<LanguageDetection> {
        let mut detections = Vec::new();

        for (pattern, language, confidence) in &self.pattern_map {
            if pattern.is_match(content) {
                let mut metadata = HashMap::new();

                // Add pattern match metadata
                metadata.insert("pattern".to_string(), pattern.as_str().to_string());

                detections.push(LanguageDetection {
                    language: language.clone(),
                    confidence: *confidence,
                    source: DetectionSource::Content,
                    extra_metadata: metadata,
                });
            }
        }

        detections
    }

    /// Initialize the extension mapping
    fn init_extension_map(extension_map: &mut HashMap<String, Vec<(String, f64)>>) {
        // Programming languages
        extension_map.insert("rs".to_string(), vec![("rust".to_string(), 1.0)]);
        extension_map.insert("ts".to_string(), vec![("typescript".to_string(), 1.0)]);
        extension_map.insert(
            "tsx".to_string(),
            vec![
                ("typescript".to_string(), 1.0),
                ("javascript".to_string(), 0.8),
            ],
        );
        extension_map.insert("js".to_string(), vec![("javascript".to_string(), 1.0)]);
        extension_map.insert("jsx".to_string(), vec![("javascript".to_string(), 1.0)]);
        extension_map.insert("py".to_string(), vec![("python".to_string(), 1.0)]);
        extension_map.insert("go".to_string(), vec![("go".to_string(), 1.0)]);

        // Web technologies
        extension_map.insert("html".to_string(), vec![("html".to_string(), 1.0)]);
        extension_map.insert("css".to_string(), vec![("css".to_string(), 1.0)]);
        extension_map.insert("scss".to_string(), vec![("scss".to_string(), 1.0)]);
        extension_map.insert("sass".to_string(), vec![("sass".to_string(), 1.0)]);
        extension_map.insert("less".to_string(), vec![("less".to_string(), 1.0)]);

        // Data/Config formats
        extension_map.insert("json".to_string(), vec![("json".to_string(), 1.0)]);
        extension_map.insert("yaml".to_string(), vec![("yaml".to_string(), 1.0)]);
        extension_map.insert("yml".to_string(), vec![("yaml".to_string(), 1.0)]);
        extension_map.insert("toml".to_string(), vec![("toml".to_string(), 1.0)]);
        extension_map.insert("xml".to_string(), vec![("xml".to_string(), 1.0)]);

        // Shell scripts
        extension_map.insert(
            "sh".to_string(),
            vec![("bash".to_string(), 0.8), ("shell".to_string(), 0.7)],
        );
        extension_map.insert("bash".to_string(), vec![("bash".to_string(), 1.0)]);
        extension_map.insert("zsh".to_string(), vec![("zsh".to_string(), 1.0)]);
        extension_map.insert("fish".to_string(), vec![("fish".to_string(), 1.0)]);

        // Other languages
        extension_map.insert("java".to_string(), vec![("java".to_string(), 1.0)]);
        extension_map.insert("kt".to_string(), vec![("kotlin".to_string(), 1.0)]);
        extension_map.insert("kotlin".to_string(), vec![("kotlin".to_string(), 1.0)]);
        extension_map.insert("scala".to_string(), vec![("scala".to_string(), 1.0)]);
        extension_map.insert("cpp".to_string(), vec![("cpp".to_string(), 1.0)]);
        extension_map.insert("cc".to_string(), vec![("cpp".to_string(), 1.0)]);
        extension_map.insert("cxx".to_string(), vec![("cpp".to_string(), 1.0)]);
        extension_map.insert("c".to_string(), vec![("c".to_string(), 1.0)]);
        extension_map.insert("hpp".to_string(), vec![("cpp".to_string(), 0.8)]);
        extension_map.insert(
            "h".to_string(),
            vec![("c".to_string(), 0.7), ("cpp".to_string(), 0.7)],
        );
        extension_map.insert("cs".to_string(), vec![("csharp".to_string(), 1.0)]);
        extension_map.insert("php".to_string(), vec![("php".to_string(), 1.0)]);
        extension_map.insert("rb".to_string(), vec![("ruby".to_string(), 1.0)]);
        extension_map.insert("swift".to_string(), vec![("swift".to_string(), 1.0)]);
    }

    /// Initialize content pattern mapping
    fn init_pattern_map() -> Vec<(Regex, String, f64)> {
        vec![
            // Rust patterns
            (
                Regex::new(r"#\[[a-zA-Z_][a-zA-Z0-9_]*]").unwrap(),
                "rust".to_string(),
                0.9,
            ),
            (
                Regex::new(r"fn\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\(").unwrap(),
                "rust".to_string(),
                0.8,
            ),
            (
                Regex::new(r"impl\s+[a-zA-Z_][a-zA-Z0-9_]*").unwrap(),
                "rust".to_string(),
                0.8,
            ),
            (
                Regex::new(r"match\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\{").unwrap(),
                "rust".to_string(),
                0.8,
            ),
            // TypeScript/JavaScript patterns
            (
                Regex::new(r#"import\s+(.*)\s+from\s+['"](.*)['"]"#).unwrap(),
                "javascript".to_string(),
                0.9,
            ),
            (
                Regex::new(r"function\s+[a-zA-Z_$][a-zA-Z0-9_$]*\s*\(").unwrap(),
                "javascript".to_string(),
                0.7,
            ),
            (
                Regex::new(r"class\s+[a-zA-Z_$][a-zA-Z0-9_$]*").unwrap(),
                "javascript".to_string(),
                0.8,
            ),
            (
                Regex::new(r"<[a-zA-Z][a-zA-Z0-9]*[^>]*>").unwrap(),
                "typescript".to_string(),
                0.6,
            ), // TSX
            // Python patterns
            (
                Regex::new(r"def\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\(").unwrap(),
                "python".to_string(),
                0.8,
            ),
            (
                Regex::new(r"class\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\(").unwrap(),
                "python".to_string(),
                0.8,
            ),
            (
                Regex::new(r"import\s+[a-zA-Z_][a-zA-Z0-9_]*").unwrap(),
                "python".to_string(),
                0.7,
            ),
            // Go patterns
            (
                Regex::new(r"package\s+[a-zA-Z_][a-zA-Z0-9_]*").unwrap(),
                "go".to_string(),
                0.9,
            ),
            (
                Regex::new(r"func\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\(").unwrap(),
                "go".to_string(),
                0.8,
            ),
            (
                Regex::new(r"var\s+[a-zA-Z_][a-zA-Z0-9_]*\s+[a-zA-Z]").unwrap(),
                "go".to_string(),
                0.8,
            ),
        ]
    }

    /// Initialize shebang pattern mapping
    fn init_shebang_patterns() -> Vec<(Regex, String)> {
        vec![
            (Regex::new(r#"#!/bin/bash"#).unwrap(), "bash".to_string()),
            (Regex::new(r"#!/usr/bin/bash").unwrap(), "bash".to_string()),
            (Regex::new(r"#!/bin/sh").unwrap(), "shell".to_string()),
            (Regex::new(r"#!/usr/bin/sh").unwrap(), "shell".to_string()),
            (Regex::new(r"#!/bin/zsh").unwrap(), "zsh".to_string()),
            (Regex::new(r"#!/usr/bin/zsh").unwrap(), "zsh".to_string()),
            (
                Regex::new(r"#!/usr/bin/env\s+python").unwrap(),
                "python".to_string(),
            ),
            (
                Regex::new(r"#!/usr/bin/env\s+python3").unwrap(),
                "python".to_string(),
            ),
            (
                Regex::new(r"#!/usr/bin/env\s+node").unwrap(),
                "javascript".to_string(),
            ),
            (
                Regex::new(r"#!/usr/bin/env\s+node").unwrap(),
                "typescript".to_string(),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_rust_by_extension() {
        let detector = LanguageDetector::default();
        let path = PathBuf::from("test.rs");
        let detections = detector.detect_language(&path, None);

        assert!(!detections.is_empty());
        assert_eq!(detections[0].language, "rust");
        assert_eq!(detections[0].confidence, 1.0);
        assert_eq!(detections[0].source, DetectionSource::Extension);
    }

    #[test]
    fn test_detect_typescript_by_extension() {
        let detector = LanguageDetector::default();
        let path = PathBuf::from("component.tsx");
        let detections = detector.detect_language(&path, None);

        assert!(!detections.is_empty());
        assert_eq!(detections[0].language, "typescript");
    }

    #[test]
    fn test_detect_python_by_content() {
        let detector = LanguageDetector::default();
        let content =
            "def hello_world():\n    print(\"Hello, Python!\")\n\nclass TestClass:\n    pass";

        let detections = detector.detect_by_content(content);
        assert!(!detections.is_empty());

        let python_detections: Vec<&LanguageDetection> = detections
            .iter()
            .filter(|d| d.language == "python")
            .collect();

        assert!(!python_detections.is_empty());
        assert!(python_detections[0].confidence > 0.5);
    }

    #[test]
    fn test_supported_languages() {
        let detector = LanguageDetector::default();
        let languages = detector.supported_languages();

        assert!(languages.contains("rust"));
        assert!(languages.contains("typescript"));
        assert!(languages.contains("python"));
        assert!(languages.contains("go"));
        assert!(languages.contains("javascript"));
    }
}
