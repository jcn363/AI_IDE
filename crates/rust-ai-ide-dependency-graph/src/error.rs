//! Error types for dependency graph operations

use std::fmt;

/// Errors that can occur during dependency graph operations
#[derive(Debug, Clone)]
pub enum DependencyError {
    /// Version conflict detected between dependencies
    VersionConflict {
        package: String,
        required_versions: Vec<String>,
        source_packages: Vec<String>,
    },

    /// Circular dependency detected in the graph
    CircularDependency(Vec<String>),

    /// Package not found in the registry or cache
    PackageNotFound(String),

    /// Network error when fetching package information
    NetworkError(String),

    /// IO error during file operations
    IoError(String),

    /// Parsing error for dependency specifications
    ParseError(String),

    /// Cache related errors
    CacheError(String),

    /// Resolution timeout
    ResolutionTimeout(String),

    /// Workspace configuration error
    WorkspaceError(String),

    /// Configuration validation error
    ConfigError(String),

    /// Generic dependency resolution error
    ResolutionError {
        package: String,
        reason: String,
    },

    /// Security-related error
    SecurityError(String),
}

impl fmt::Display for DependencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyError::VersionConflict { package, required_versions, source_packages } => {
                write!(f, "Version conflict for '{}': required versions {:?} from packages {:?}",
                       package, required_versions, source_packages)
            },
            DependencyError::CircularDependency(packages) => {
                write!(f, "Circular dependency detected: {:?}", packages)
            },
            DependencyError::PackageNotFound(package) => {
                write!(f, "Package '{}' not found", package)
            },
            DependencyError::NetworkError(msg) => {
                write!(f, "Network error: {}", msg)
            },
            DependencyError::IoError(msg) => {
                write!(f, "IO error: {}", msg)
            },
            DependencyError::ParseError(msg) => {
                write!(f, "Parse error: {}", msg)
            },
            DependencyError::CacheError(msg) => {
                write!(f, "Cache error: {}", msg)
            },
            DependencyError::ResolutionTimeout(msg) => {
                write!(f, "Resolution timeout: {}", msg)
            },
            DependencyError::WorkspaceError(msg) => {
                write!(f, "Workspace error: {}", msg)
            },
            DependencyError::ConfigError(msg) => {
                write!(f, "Configuration error: {}", msg)
            },
            DependencyError::ResolutionError { package, reason } => {
                write!(f, "Resolution error for '{}': {}", package, reason)
            },
            DependencyError::SecurityError(msg) => {
                write!(f, "Security error: {}", msg)
            },
        }
    }
}

impl std::error::Error for DependencyError {}

/// Result type alias for dependency operations
pub type DependencyResult<T> = Result<T, DependencyError>;

/// Error recovery suggestions
pub struct ErrorSuggestion {
    pub error_type: String,
    pub suggestions: Vec<String>,
    pub confidence: f32,
}

impl ErrorSuggestion {
    /// Create a new error suggestion
    pub fn new(error_type: String) -> Self {
        Self {
            error_type,
            suggestions: Vec::new(),
            confidence: 0.0,
        }
    }

    /// Add a suggestion with confidence
    pub fn with_suggestion(mut self, suggestion: String, confidence: f32) -> Self {
        self.suggestions.push(suggestion);
        self.confidence = confidence;
        self
    }
}

/// Error aggregator for collecting multiple errors during resolution
#[derive(Default)]
pub struct ErrorAggregator {
    pub errors: Vec<DependencyError>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<ErrorSuggestion>,
}

impl ErrorAggregator {
    /// Create new error aggregator
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error
    pub fn add_error(&mut self, error: DependencyError) {
        self.errors.push(error);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Add a suggestion
    pub fn add_suggestion(&mut self, suggestion: ErrorSuggestion) {
        self.suggestions.push(suggestion);
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get all errors
    pub fn get_errors(&self) -> &[DependencyError] {
        &self.errors
    }

    /// Get all warnings
    pub fn get_warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Get suggestions sorted by confidence
    pub fn get_suggestions(&self) -> Vec<&ErrorSuggestion> {
        let mut suggestions = self.suggestions.iter().collect::<Vec<_>>();
        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        suggestions
    }

    /// Convert to single error if there's only one, or combined error string
    pub fn into_result<T>(self) -> DependencyResult<T> {
        if self.errors.is_empty() {
            // No errors, return success with empty result
            Err(DependencyError::ResolutionError {
                package: "unknown".to_string(),
                reason: "Empty result".to_string(),
            })
        } else if self.errors.len() == 1 {
            Err(self.errors.into_iter().next().unwrap())
        } else {
            // Multiple errors - combine them
            let error_messages = self.errors.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");

            Err(DependencyError::ResolutionError {
                package: "multiple".to_string(),
                reason: format!("Multiple errors: {}", error_messages),
            })
        }
    }
}