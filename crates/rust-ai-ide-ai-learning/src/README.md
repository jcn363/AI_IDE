# Learning System Module

## Overview

The Learning System provides intelligent pattern recognition and continuous improvement capabilities for the Rust AI IDE. This modular implementation offers SQLite-based learning with privacy controls, statistical analysis, and automated pattern recommendations.

## Architecture

The learning system is organized into specialized submodules:

```text
learning/
├── types.rs       # Core types and error definitions
├── models.rs      # Data structures (LearnedPattern, FixTemplate, etc.)
├── database.rs    # SQLite persistence, migrations, CRUD operations
├── similarity.rs  # Pattern matching and confidence scoring
├── preferences.rs # User preferences, privacy modes, validation
├── statistics.rs  # Analytics, reporting, performance metrics
├── system.rs      # Main orchestration layer
└── README.md      # Documentation (this file)
```

## Quick Start

```rust
use rust_ai_ide_ai_learning::{LearningSystem, LearningPreferences};

// Create learning system with default database path
let learning = LearningSystem::new().await?;

// Configure preferences
let mut prefs = LearningPreferences::default();
prefs.enable_learning = true;
prefs.privacy_mode = PrivacyMode::OptIn;
learning.update_preferences(prefs).await?;

// Record successful fixes for learning
learning.record_successful_fix(error_pattern, fix_suggestion).await?;

// Get similar patterns for new errors
let similar_patterns = learning.get_similar_patterns("unused variable").await?;
let recommended_fix = similar_patterns.first().unwrap();
```

## Core Features

### 🔍 Pattern Recognition

- **Similarity Matching**: Find similar error patterns based on error messages and context
- **Confidence Scoring**: Rate patterns based on success history and recency
- **Context Analysis**: Understand code structure and patterns for better matching

### 📚 Learning Persistence

- **SQLite Database**: Efficient local storage with optimized indexes
- **Incremental Learning**: Build knowledge base over time from user successes
- **Pattern Evolution**: Improve confidence scores as more examples are learned

### 🔒 Privacy Controls

- **Multiple Privacy Modes**: OptIn, OptOut, Anonymous data collection
- **User Consent**: Respect user preferences for data sharing
- **Data Anonymization**: Remove personally identifiable information

### 📊 Analytics & Statistics

- **Success Tracking**: Monitor effectiveness of learned patterns
- **Performance Metrics**: Track query times, cache hit rates, memory usage
- **Trend Analysis**: Identify patterns and improvement opportunities

## API Reference

### LearningSystem

Main orchestration class providing unified API:

```rust
pub struct LearningSystem {
    database: LearningDatabase,
    preferences_manager: PreferencesManager,
    similarity_calculator: SimilarityCalculator,
    pattern_cache: Arc<RwLock<HashMap<String, Vec<LearnedPattern>>>>,
    similarity_cache: Arc<RwLock<SimilarityCache>>,
    preferences: LearningPreferences,
    user_id: String,
}
```

#### Key Methods

##### Initialization

- `new() -> AIResult<Self>`: Create with default database path
- `new_with_path(path: PathBuf) -> AIResult<Self>`: Create with custom database path

##### Pattern Management

- `record_successful_fix(error_pattern, fix) -> AIResult<()>`: Learn from successful fixes
- `record_failed_fix(error_pattern, fix) -> AIResult<()>`: Learn from failed attempts
- `get_similar_patterns(error_context) -> AIResult<Vec<LearnedPattern>>`: Find similar patterns

##### Preferences

- `update_preferences(preferences) -> AIResult<()>`: Update user preferences
- `get_preferences() -> &LearningPreferences`: Get current preferences
- `validate_preferences(prefs) -> AIResult<()>`: Validate preference settings

##### Analytics

- `get_pattern_statistics() -> AIResult<LearningStatistics>`: Comprehensive statistics
- `get_insights() -> AIResult<Vec<String>>`: AI-generated insights and recommendations

#### Database Methods

- `clear_all_patterns() -> AIResult<()>`: Reset learning data
- `export_patterns() -> AIResult<String>`: Export as JSON (respecting privacy)
- `import_patterns(json_data) -> AIResult<usize>`: Import from JSON

### Error Patterns

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub id: String,
    pub description: String,
    pub error_pattern: String,
    pub error_code: Option<String>,
    pub context_patterns: Vec<String>,
    pub fix_template: FixTemplate,
    pub confidence: f32,
    pub success_count: u32,
    pub attempt_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub context_hash: String,
    pub tags: Vec<String>,
    pub contributor_id: Option<String>,
}
```

## Privacy Considerations

### Privacy Modes

1. **OptOut**: Most restrictive, no data collection by default
2. **Anonymous**: Data collected without user identification
3. **OptIn**: Full functionality with explicit user consent

### Data Handling

- **Local Storage**: All data stored locally in SQLite database
- **Privacy Filtering**: Patterns anonymized based on privacy mode
- **User Control**: Full deletion capabilities via `clear_all_patterns()`
- **No External Communication**: All learning happens locally

## Configuration Options

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearningPreferences {
    pub enable_learning: bool,
    pub privacy_mode: PrivacyMode,
    pub confidence_threshold: f32,
    pub max_patterns_per_type: u32,
    pub enable_community_sharing: bool,
    pub use_community_patterns: bool,
    pub auto_apply_threshold: f32,
}
```

### Template Configurations

The system provides pre-configured templates for common usage scenarios:

- `maximum_learning()`: Full learning functionality with community features
- `balanced()`: Good balance of privacy and functionality
- `privacy_first()`: Maximum privacy protection
- `development()`: Developer-friendly settings with lower thresholds

## Performance Characteristics

### Caching Strategy

- **Pattern Cache**: In-memory cache for frequently accessed patterns
- **Similarity Cache**: Cached similarity calculations keyed by hash
- **LRU Eviction**: Automatic cleanup of less-used cache entries

### Database Optimization

- **Indexes**: Optimized for common query patterns
- **Connection Pooling**: Efficient database connections
- **Bulk Operations**: Batch processing where possible
- **Memory Mapping**: SQLite WAL mode for concurrency

### Expected Performance (Estimated)

- **Pattern Matching**: <10ms for cached patterns
- **Similarity Calculation**: 50-100ms for complex patterns
- **Database Queries**: 5-20ms typical response time
- **Memory Usage**: ~50MB for typical usage patterns

## Error Handling

The learning system uses comprehensive error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum LearningError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid preferences: {0}")]
    InvalidPreferencesError(String),

    #[error("Pattern not found: {0}")]
    PatternNotFoundError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Privacy mode violation: {0}")]
    PrivacyModeError(String),
}
```

## Migration from Monolithic System

If migrating from the previous monolithic implementation:

1. **Database Migration**: Existing SQLite database is fully compatible
2. **API Compatibility**: Main LearningSystem API remains the same
3. **Performance**: Modular system may show improved startup times
4. **Memory Usage**: Slightly lower memory footprint due to lazy loading

## Troubleshooting

### Common Issues

**Database Connection Errors**

```rust
// Ensure database directory exists and is writable
.use_dirs() // Uses standard user data directory
```

**Low Pattern Quality**

```rust
// Adjust confidence thresholds
let mut prefs = LearningPreferences::default();
prefs.confidence_threshold = 0.7; // Default: 0.7
prefs.auto_apply_threshold = 0.9; // Default: 0.9
```

**Privacy Concerns**

```rust
// Use restrictive privacy mode
prefs.privacy_mode = PrivacyMode::OptOut;
prefs.enable_community_sharing = false;
```

### Logging and Debugging

Enable detailed logging to troubleshoot issues:

```rust
use tracing::*;
#[tracing::instrument(level = "debug", skip(self))]
async fn problematic_method(&self, ...) -> AIResult<()> {
    debug!("Processing with parameters: {:?}", params);
    // ... implementation
}
```

## Future Enhancements

Planned improvements include:

- Machine Learning-based pattern clustering
- Community pattern synchronization (optional)
- Enhanced similarity algorithms
- Real-time performance monitoring
- Plugin architecture for custom learning algorithms

---

*For more detailed API documentation, see individual module files and inline documentation.*
