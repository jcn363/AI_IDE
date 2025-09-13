//! Language detection and classification for multi-language LSP support
//!
//! This module provides intelligent detection of programming languages
//! based on file extensions, content analysis, and project structure.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use lru::LruCache;
use lsp_types::Url;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, instrument};

const DEFAULT_CACHE_SIZE: usize = 1000;
const DEFAULT_MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const DEFAULT_MIN_CONFIDENCE: f64 = 0.3;

/// Cache for storing language detection results
#[derive(Debug, Default)]
pub struct DetectionCache {
    extension_cache: RwLock<HashMap<String, Vec<LanguageDetection>>>,
    content_cache: RwLock<LruCache<String, Vec<LanguageDetection>>>,
    file_cache: RwLock<LruCache<PathBuf, Vec<LanguageDetection>>>,
}

impl DetectionCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            extension_cache: RwLock::new(HashMap::new()),
            content_cache: RwLock::new(LruCache::new(capacity)),
            file_cache: RwLock::new(LruCache::new(capacity)),
        }
    }

    pub async fn get_extension(&self, ext: &str) -> Option<Vec<LanguageDetection>> {
        self.extension_cache.read().await.get(ext).cloned()
    }

    pub async fn set_extension(&self, ext: String, detections: Vec<LanguageDetection>) {
        self.extension_cache.write().await.insert(ext, detections);
    }

    pub async fn get_content(&self, content: &str) -> Option<Vec<LanguageDetection>> {
        self.content_cache.write().await.get(content).cloned()
    }

    pub async fn set_content(&self, content: String, detections: Vec<LanguageDetection>) {
        self.content_cache.write().await.put(content, detections);
    }

    pub async fn get_file(&self, path: &Path) -> Option<Vec<LanguageDetection>> {
        self.file_cache.write().await.get(path).cloned()
    }

    pub async fn set_file(&self, path: PathBuf, detections: Vec<LanguageDetection>) {
        self.file_cache.write().await.put(path, detections);
    }

    pub async fn clear(&self) {
        self.extension_cache.write().await.clear();
        self.content_cache.write().await.clear();
        self.file_cache.write().await.clear();
    }
}

/// Detected programming language with confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone)]
pub struct LanguageDetector {
    extension_map: HashMap<String, Vec<(String, f64)>>,
    content_patterns: Vec<(Regex, String, f64)>,
    shebang_patterns: Vec<(Regex, String, f64)>,
    cache: Arc<DetectionCache>,
    min_confidence: f64,
    max_file_size: usize,
}

impl LanguageDetector {
    /// Create a new language detector with default settings
    pub fn new() -> Self {
        Self::with_cache(DEFAULT_CACHE_SIZE)
    }

    /// Create a new language detector with a specific cache size
    pub fn with_cache(cache_size: usize) -> Self {
        let mut detector = Self {
            extension_map: HashMap::new(),
            content_patterns: Vec::new(),
            shebang_patterns: Vec::new(),
            cache: Arc::new(DetectionCache::new(cache_size)),
            max_file_size: DEFAULT_MAX_FILE_SIZE,
        };

        // Initialize all pattern maps
        detector.init_extension_map();
        detector.content_patterns = Self::init_pattern_map();
        detector.shebang_patterns = Self::init_shebang_patterns();

        detector
    }

    /// Initialize the extension map with language mappings
    fn init_extension_map(&mut self) {
        let extension_map = &mut self.extension_map;

        // Clear existing mappings
        extension_map.clear();

        // Programming languages
        extension_map.insert("rs".to_string(), vec![("rust".to_string(), 1.0)]);
        extension_map.insert("ts".to_string(), vec![("typescript".to_string(), 1.0)]);
        extension_map.insert("tsx".to_string(), vec![("typescript".to_string(), 1.0)]);
        extension_map.insert("js".to_string(), vec![("javascript".to_string(), 1.0)]);
        extension_map.insert("jsx".to_string(), vec![("javascript".to_string(), 1.0)]);
        extension_map.insert("py".to_string(), vec![("python".to_string(), 1.0)]);
        extension_map.insert("go".to_string(), vec![("go".to_string(), 1.0)]);

        // Web technologies
        extension_map.insert("html".to_string(), vec![("html".to_string(), 1.0)]);
        extension_map.insert("htm".to_string(), vec![("html".to_string(), 1.0)]);
        extension_map.insert("css".to_string(), vec![("css".to_string(), 1.0)]);
        extension_map.insert("scss".to_string(), vec![("scss".to_string(), 1.0)]);
        extension_map.insert("sass".to_string(), vec![("sass".to_string(), 1.0)]);
        extension_map.insert("less".to_string(), vec![("less".to_string(), 1.0)]);

        // Data formats
        extension_map.insert("json".to_string(), vec![("json".to_string(), 1.0)]);
        extension_map.insert("yaml".to_string(), vec![("yaml".to_string(), 1.0)]);
        extension_map.insert("toml".to_string(), vec![("toml".to_string(), 1.0)]);
        extension_map.insert("xml".to_string(), vec![("xml".to_string(), 1.0)]);

        // Shell scripts
        extension_map.insert("sh".to_string(), vec![("shell".to_string(), 1.0)]);
        extension_map.insert("bash".to_string(), vec![("bash".to_string(), 1.0)]);
        extension_map.insert("zsh".to_string(), vec![("zsh".to_string(), 1.0)]);

        // Other languages
        extension_map.insert("java".to_string(), vec![("java".to_string(), 1.0)]);
        extension_map.insert("kt".to_string(), vec![("kotlin".to_string(), 1.0)]);
        extension_map.insert("swift".to_string(), vec![("swift".to_string(), 1.0)]);
        extension_map.insert("rb".to_string(), vec![("ruby".to_string(), 1.0)]);
        extension_map.insert("php".to_string(), vec![("php".to_string(), 1.0)]);
    }

    /// Set minimum confidence threshold for language detection
    pub fn with_min_confidence(mut self, confidence: f64) -> Self {
        self.min_confidence = confidence.max(0.0).min(1.0);
        self
    }

    /// Set maximum file size for content analysis
    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Detect language from file path and optional content
    #[instrument(skip(self, content))]
    pub async fn detect_language(&self, path: &Path, content: Option<&str>) -> Vec<LanguageDetection> {
        let start_time = Instant::now();

        // Check file cache first
        if let Some(cached) = self.cache.get_file(path).await {
            debug!("Cache hit for file: {:?}", path);
            return cached;
        }

        let mut detections = Vec::new();

        // Try to detect by extension first
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if let Some(cached) = self.cache.get_extension(ext).await {
                detections.extend(cached);
            } else if let Some(langs) = self.extension_map.get(ext) {
                let mut ext_detections = Vec::with_capacity(langs.len());
                for (lang, confidence) in langs {
                    ext_detections.push(LanguageDetection {
                        language: lang.clone(),
                        confidence: *confidence,
                        source: DetectionSource::Extension,
                    });
                }
                self.cache.set_extension(ext.to_string(), ext_detections.clone()).await;
                detections.extend(ext_detections);
            }
        }

        // If we have content, try to detect by content
        if let Some(content) = content {
            // Skip very large files
            if content.len() <= self.max_file_size {
                detections.extend(self.detect_by_content(content).await);
            }
        }

        // Sort by confidence (highest first)
        detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        // Filter by minimum confidence
        detections.retain(|d| d.confidence >= self.min_confidence);

        // Cache the result
        if !detections.is_empty() {
            self.cache.set_file(path.to_path_buf(), detections.clone()).await;
        }

        debug!("Detected languages for {:?} in {:?}", path, start_time.elapsed());
        detections
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
    shebang_map.insert(
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
    extension_map.insert("htm".to_string(), vec![("html".to_string(), 1.0)]);
    extension_map.insert("xhtml".to_string(), vec![("html".to_string(), 1.0)]);
    extension_map.insert("css".to_string(), vec![("css".to_string(), 1.0)]);
    extension_map.insert("scss".to_string(), vec![("css".to_string(), 0.95)]);
    extension_map.insert("sass".to_string(), vec![("css".to_string(), 0.95)]);
    extension_map.insert("less".to_string(), vec![("css".to_string(), 0.95)]);
    extension_map.insert("sql".to_string(), vec![("sql".to_string(), 1.0)]);
    extension_map.insert("mysql".to_string(), vec![("sql".to_string(), 0.95)]);
    extension_map.insert("pgsql".to_string(), vec![("sql".to_string(), 0.95)]);
    extension_map.insert("postgres".to_string(), vec![("sql".to_string(), 0.95)]);
    extension_map.insert("psql".to_string(), vec![("sql".to_string(), 0.95)]);
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
    let mut patterns = Vec::new();

    // Rust patterns
    patterns.push((
        Regex::new(r"#\[[a-zA-Z_][a-zA-Z0-9_]*]").unwrap(),
        "rust".to_string(),
        0.9,
    ));
    patterns.push((
        Regex::new(r"fn\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\(").unwrap(),
        "rust".to_string(),
        0.8,
    ));
    patterns.push((
        Regex::new(r"impl\s+[a-zA-Z_][a-zA-Z0-9_]*").unwrap(),
        "rust".to_string(),
        0.8,
    ));
    patterns.push((
        Regex::new(r"match\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\{").unwrap(),
        "rust".to_string(),
        0.8,
    ));
    // TypeScript/JavaScript patterns
    patterns.push((
        Regex::new(r#"import\s+(.*)\s+from\s+['"](.*)['"]"#).unwrap(),
        "javascript".to_string(),
        0.9,
    ));
    patterns.push((
        Regex::new(r"function\s+[a-zA-Z_$][a-zA-Z0-9_$]*\s*\(").unwrap(),
        "javascript".to_string(),
        0.7,
    ));
    patterns.push((
        Regex::new(r"class\s+[a-zA-Z_$][a-zA-Z0-9_$]*").unwrap(),
        "javascript".to_string(),
        0.8,
    ));
    patterns.push((
        Regex::new(r"<[a-zA-Z][a-zA-Z0-9]*[^>]*>").unwrap(),
        "typescript".to_string(),
        0.6,
    )); // TSX
    // Python patterns
    patterns.push((
        Regex::new(r"def\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\(").unwrap(),
        "python".to_string(),
        0.8,
    ));
    patterns.push((
        Regex::new(r"class\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\(").unwrap(),
        "python".to_string(),
        0.8,
    ));
    patterns.push((
        Regex::new(r"import\s+[a-zA-Z_][a-zA-Z0-9_]*").unwrap(),
        "python".to_string(),
        0.7,
    ));
    // Go patterns
    patterns.push((
        Regex::new(r"package\s+[a-zA-Z_][a-zA-Z0-9_]*").unwrap(),
        "go".to_string(),
        0.9,
    ));
    patterns.push((
        Regex::new(r"func\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\(").unwrap(),
        "go".to_string(),
        0.8,
    ));
    patterns.push((
        Regex::new(r"var\s+[a-zA-Z_][a-zA-Z0-9_]*\s+[a-zA-Z]").unwrap(),
        "go".to_string(),
        0.8,
    ));

    // HTML patterns
    patterns.push((
        Regex::new(r#"(?i)<!DOCTYPE\s+html|</?html"#).unwrap(),
        "html".to_string(),
        0.95,
    ));
    patterns.push((
        Regex::new(r#"(?i)<!DOCTYPE\s+html\s+PUBLIC\s+"[^>]*XHTML"|"http://www\.w3\.org/TR/xhtml"#).unwrap(),
        "html".to_string(),
        0.99,
    ));
    patterns.push((
        Regex::new(r#"(?i)<[a-z][\s\S]*?/?>"#).unwrap(),
        "html".to_string(),
        0.8,
    ));
    patterns.push((
        Regex::new(r#"(?i)<(html|head|body|div|span|p|a|img|ul|ol|li|table|tr|td|th|form|input|button|script|style)"#).unwrap(),
        "html".to_string(),
        0.9,
    ));

    // CSS patterns
    patterns.push((
        Regex::new(r#"(?i)\{\s*[\w-]+\s*:|@(?:import|media|keyframes|font-face|page|charset)"#).unwrap(),
        "css".to_string(),
        0.95,
    ));
    patterns.push((
        Regex::new(r#"(?i)\.?[\w-]+\s*\{[^}]*\}"#).unwrap(),
        "css".to_string(),
        0.85,
    ));
    patterns.push((
        Regex::new(r#"(?i):(?:hover|active|focus|nth-child|first-child|last-child|not)\(?"#).unwrap(),
        "css".to_string(),
        0.9,
    ));

    // SQL patterns
    patterns.push((
        Regex::new(r#"(?i)\b(?:SELECT\s+.*?\s+FROM|INSERT\s+INTO|UPDATE\s+\w+|DELETE\s+FROM)\b"#).unwrap(),
        "sql".to_string(),
        0.95,
    ));
    patterns.push((
        Regex::new(r#"(?i)\b(?:CREATE\s+TABLE|ALTER\s+TABLE|DROP\s+TABLE|CREATE\s+INDEX|CREATE\s+VIEW)\b"#).unwrap(),
        "sql".to_string(),
        0.95,
    ));
    patterns.push((
        Regex::new(r#"(?i)\b(?:JOIN|INNER\s+JOIN|LEFT\s+JOIN|RIGHT\s+JOIN|FULL\s+JOIN|ON|WHERE|GROUP\s+BY|HAVING|ORDER\s+BY|LIMIT|OFFSET)\b"#).unwrap(),
        "sql".to_string(),
        0.9,
    ));
    patterns.push((
        Regex::new(r#"(?i)\b(?:PRIMARY\s+KEY|FOREIGN\s+KEY|REFERENCES|UNIQUE|CHECK|DEFAULT|NOT\s+NULL|AUTO_INCREMENT|SERIAL|BIGSERIAL)\b"#).unwrap(),
        "sql".to_string(),
        0.9,
    ));

    patterns
}

/// Initialize shebang pattern mapping
fn init_shebang_patterns() -> Vec<(Regex, String)> {
    vec![
        (Regex::new(r"#!/bin/bash").unwrap(), "bash".to_string()),
        (Regex::new(r"#!/usr/bin/bash").unwrap(), "bash".to_string()),
        (Regex::new(r"#!/bin/sh").unwrap(), "shell".to_string()),
        (Regex::new(r"#!/usr/bin/sh").unwrap(), "shell".to_string()),
        (Regex::new(r"#!/bin/zsh").unwrap(), "zsh".to_string()),
        (Regex::new(r"#!/usr/bin/zsh").unwrap(), "zsh".to_string()),
        (Regex::new(r"#!/usr/bin/env\s+python").unwrap(), "python".to_string()),
        (Regex::new(r"#!/usr/bin/env\s+python3").unwrap(), "python".to_string()),
        (Regex::new(r"#!/usr/bin/env\s+node").unwrap(), "javascript".to_string()),
        (Regex::new(r"#!/usr/bin/env\s+node").unwrap(), "typescript".to_string()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_temp_file(content: &str, extension: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        let path = file.path().with_extension(extension);
        file.persist(&path).unwrap();
        file
    }

    #[tokio::test]
    async fn test_detect_by_extension() {
        let detector = LanguageDetector::new();

        // Test Rust files
        let rust_file = PathBuf::from("test.rs");
        let detections = detector.detect_language(&rust_file, None).await;
        assert!(!detections.is_empty());
        assert_eq!(detections[0].language, "rust");

        // Test Python files
        let py_file = PathBuf::from("script.py");
        let detections = detector.detect_language(&py_file, None).await;
        assert!(!detections.is_empty());
        assert_eq!(detections[0].language, "python");
    }

    #[tokio::test]
    async fn test_detect_by_content() {
        let detector = LanguageDetector::new();

        // Test Rust content
        let rust_content = r#"
        fn main() {
            println!("Hello, world!");
        }
        "#;
        let detections = detector.detect_language(&PathBuf::from("unknown"), Some(rust_content)).await;
        assert!(!detections.is_empty());
        assert_eq!(detections[0].language, "rust");

        // Test Python content
        let py_content = r#"
        def hello():
            print("Hello, Python!")

        if __name__ == "__main__":
            hello()
        "#;
        let detections = detector.detect_language(&PathBuf::from("unknown"), Some(py_content)).await;
        assert!(!detections.is_empty());
        assert_eq!(detections[0].language, "python");
    }

    #[tokio::test]
    async fn test_detect_by_shebang() {
        let detector = LanguageDetector::new();

        // Test Python shebang
        let py_content = "#!/usr/bin/env python3\nprint('Hello')";
        let detections = detector.detect_language(&PathBuf::from("script"), Some(py_content)).await;
        assert!(!detections.is_empty());
        assert_eq!(detections[0].language, "python");

        // Test shell shebang
        let sh_content = "#!/bin/bash\necho 'Hello'";
        let detections = detector.detect_language(&PathBuf::from("script"), Some(sh_content)).await;
        assert!(!detections.is_empty());
        assert!(detections.iter().any(|d| d.language == "bash" || d.language == "shell"));
    }

    #[tokio::test]
    async fn test_detection_confidence() {
        let detector = LanguageDetector::new()
            .with_min_confidence(0.8);

        // This should be detected with high confidence
        let rust_content = "fn main() {}";
        let detections = detector.detect_language(&PathBuf::from("test.rs"), Some(rust_content)).await;
        assert!(!detections.is_empty());
        assert!(detections[0].confidence >= 0.8);

        // This might be detected with lower confidence
        let ambiguous_content = "function test() {}";
        let detections = detector.detect_language(&PathBuf::from("test"), Some(ambiguous_content)).await;
        assert!(detections.is_empty() || detections[0].confidence >= 0.8);
    }

    #[test]
    fn test_init_pattern_map() {
        let patterns = LanguageDetector::init_pattern_map();
        assert!(!patterns.is_empty());

        // Check if we have patterns for major languages
        let has_rust = patterns.iter().any(|(_, lang, _)| lang == "rust");
        let has_python = patterns.iter().any(|(_, lang, _)| lang == "python");
        let has_js = patterns.iter().any(|(_, lang, _)| lang == "javascript");

        assert!(has_rust);
        assert!(has_python);
        assert!(has_js);
    }

    #[test]
    fn test_init_shebang_patterns() {
        let patterns = init_shebang_patterns();
        assert!(!patterns.is_empty());

        // Check if we have common shebang patterns
        let has_python = patterns.iter().any(|(_, lang)| lang == "python");
        let has_bash = patterns.iter().any(|(_, lang)| lang == "bash" || lang == "shell");

        assert!(has_python);
        assert!(has_bash);
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
