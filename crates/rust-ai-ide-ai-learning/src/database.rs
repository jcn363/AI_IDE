//! Database operations for the learning system

use super::models::{LearnedPattern, LearningPreferences, PatternStatistics};
use crate::types::{AIResult, AIServiceError, PrivacyMode};
use chrono::{DateTime, Utc};
use serde_json;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::task;

/// Async database wrapper using rusqlite with spawn_blocking
#[derive(Debug)]
pub struct AsyncDatabase {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl AsyncDatabase {
    /// Create a new database connection from file path
    pub async fn new<T: AsRef<Path>>(db_path: T) -> AIResult<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        let conn = task::spawn_blocking(move || rusqlite::Connection::open(db_path))
            .await
            .map_err(|e| AIServiceError::Database(format!("Async spawn error: {}", e)))?
            .map_err(|e| AIServiceError::Database(format!("Connection error: {}", e)))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create database connection in memory (for testing)
    pub async fn new_in_memory() -> AIResult<Self> {
        let conn = task::spawn_blocking(|| rusqlite::Connection::open_in_memory())
            .await
            .map_err(|e| AIServiceError::Database(format!("Async spawn error: {}", e)))?
            .map_err(|e| AIServiceError::Database(format!("Connection error: {}", e)))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Execute a SQL query with parameters
    pub async fn execute<P: rusqlite::Params + Send + 'static>(
        &self,
        sql: &str,
        params: P,
    ) -> AIResult<usize> {
        let conn = Arc::clone(&self.conn);
        let sql = sql.to_string();
        task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(&sql)?;
            stmt.execute(params)
        })
        .await
        .map_err(|e| AIServiceError::Database(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::Database(format!("Execute error: {}", e)))
    }

    /// Execute a SQL query and return a prepared statement for queries
    pub async fn prepare_statement(&self, sql: &str) -> AIResult<PreparedStatement> {
        let conn = Arc::clone(&self.conn);
        let sql = sql.to_string();
        Ok(PreparedStatement { conn, sql })
    }

    /// Execute multiple statements in a transaction
    pub async fn execute_batch(&self, sql: &str) -> AIResult<()> {
        let conn = Arc::clone(&self.conn);
        let sql = sql.to_string();
        task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute_batch(&sql)
        })
        .await
        .map_err(|e| AIServiceError::Database(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::Database(format!("Batch execute error: {}", e)))
    }

    /// Get rows from a query
    pub async fn query_map<P, F, T>(&self, sql: &str, params: P, mapper: F) -> AIResult<Vec<T>>
    where
        P: rusqlite::Params + Send + 'static,
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        let sql = sql.to_string();
        task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params, mapper)?;
            rows.collect::<rusqlite::Result<Vec<T>>>()
        })
        .await
        .map_err(|e| AIServiceError::Database(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::Database(format!("Query map error: {}", e)))
    }

    /// Get a single optional row from a query
    pub async fn query_row<P, F, T>(&self, sql: &str, params: P, mapper: F) -> AIResult<Option<T>>
    where
        P: rusqlite::Params + Send + 'static,
        F: FnOnce(&rusqlite::Row) -> rusqlite::Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        let sql = sql.to_string();
        task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(&sql)?;
            match stmt.query_row(params, mapper) {
                Ok(result) => Ok(Some(result)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
        .await
        .map_err(|e| AIServiceError::Database(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::Database(format!("Query row error: {}", e)))
    }
}

/// Wrapper for prepared statements
#[derive(Debug)]
pub struct PreparedStatement {
    conn: Arc<Mutex<rusqlite::Connection>>,
    sql: String,
}

impl PreparedStatement {
    /// Execute the prepared statement with parameters
    pub async fn execute<P: rusqlite::Params + Send + 'static>(
        &mut self,
        params: P,
    ) -> AIResult<usize> {
        let conn = Arc::clone(&self.conn);
        let sql = self.sql.clone();
        task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(&sql)?;
            stmt.execute(params)
        })
        .await
        .map_err(|e| AIServiceError::Database(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::Database(format!("Execute error: {}", e)))
    }

    /// Query with mapping function
    pub async fn query_map<F, T, P>(&mut self, params: P, mapper: F) -> AIResult<Vec<T>>
    where
        P: rusqlite::Params + Send + 'static,
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        let sql = self.sql.clone();
        task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params, mapper)?;
            rows.collect::<rusqlite::Result<Vec<T>>>()
        })
        .await
        .map_err(|e| AIServiceError::Database(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::Database(format!("Query map error: {}", e)))
    }

    /// Query optional row
    pub async fn query_row<F, T, P>(&mut self, params: P, mapper: F) -> AIResult<Option<T>>
    where
        P: rusqlite::Params + Send + 'static,
        F: FnOnce(&rusqlite::Row) -> rusqlite::Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        let sql = self.sql.clone();
        task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(&sql)?;
            match stmt.query_row(params, mapper) {
                Ok(result) => Ok(Some(result)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
        .await
        .map_err(|e| AIServiceError::Database(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::Database(format!("Query row error: {}", e)))
    }
}

/// Database manager for learning system operations
#[derive(Debug, Clone)]
pub struct LearningDatabase {
    pub db: Arc<AsyncDatabase>,
}

impl LearningDatabase {
    /// Create a new database connection
    pub async fn new(db_path: Option<PathBuf>) -> AIResult<Self> {
        let db_path = db_path.unwrap_or_else(|| {
            let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("rust-ai-ide");
            std::fs::create_dir_all(&path).ok();
            path.push("learning.db");
            path
        });

        let db = Arc::new(AsyncDatabase::new(db_path).await?);
        let learning_db = Self { db };
        learning_db.run_migrations().await?;

        Ok(learning_db)
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> AIResult<()> {
        self.create_learned_patterns_table().await?;
        self.create_pattern_applications_table().await?;
        self.create_user_preferences_table().await?;
        self.create_indexes().await?;

        Ok(())
    }

    /// Create learned_patterns table
    async fn create_learned_patterns_table(&self) -> AIResult<()> {
        self.db
            .execute(
                r#"
            CREATE TABLE IF NOT EXISTS learned_patterns (
                id TEXT PRIMARY KEY,
                description TEXT NOT NULL,
                error_pattern TEXT NOT NULL,
                error_code TEXT,
                context_patterns TEXT NOT NULL,
                fix_template TEXT NOT NULL,
                confidence REAL NOT NULL,
                success_count INTEGER NOT NULL DEFAULT 0,
                attempt_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                context_hash TEXT NOT NULL,
                tags TEXT NOT NULL,
                contributor_id TEXT
            )
        "#,
                [],
            )
            .await?;
        Ok(())
    }

    /// Create pattern_applications table
    async fn create_pattern_applications_table(&self) -> AIResult<()> {
        self.db
            .execute(
                r#"
            CREATE TABLE IF NOT EXISTS pattern_applications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pattern_id TEXT NOT NULL,
                applied_at TEXT NOT NULL,
                was_successful BOOLEAN NOT NULL,
                error_context TEXT NOT NULL,
                fix_result TEXT,
                FOREIGN KEY (pattern_id) REFERENCES learned_patterns (id)
            )
        "#,
                [],
            )
            .await?;
        Ok(())
    }

    /// Create user_preferences table
    async fn create_user_preferences_table(&self) -> AIResult<()> {
        self.db
            .execute(
                r#"
            CREATE TABLE IF NOT EXISTS user_preferences (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
        "#,
                [],
            )
            .await?;
        Ok(())
    }

    /// Create performance indexes
    async fn create_indexes(&self) -> AIResult<()> {
        self.db.execute("CREATE INDEX IF NOT EXISTS idx_patterns_error_code ON learned_patterns (error_code)", []).await?;
        self.db.execute("CREATE INDEX IF NOT EXISTS idx_patterns_context_hash ON learned_patterns (context_hash)", []).await?;
        self.db.execute("CREATE INDEX IF NOT EXISTS idx_patterns_confidence ON learned_patterns (confidence)", []).await?;
        self.db.execute("CREATE INDEX IF NOT EXISTS idx_applications_pattern_id ON pattern_applications (pattern_id)", []).await?;
        Ok(())
    }

    /// Store a learned pattern
    pub async fn store_pattern(&self, pattern: &LearnedPattern) -> AIResult<()> {
        let context_patterns_json = serde_json::to_string(&pattern.context_patterns)
            .map_err(|e| AIServiceError::SerializationError(e.to_string()))?;

        let fix_template_json = serde_json::to_string(&pattern.fix_template)
            .map_err(|e| AIServiceError::SerializationError(e.to_string()))?;

        let tags_json = serde_json::to_string(&pattern.tags)
            .map_err(|e| AIServiceError::SerializationError(e.to_string()))?;

        self.db
            .execute(
                r#"
            INSERT OR REPLACE INTO learned_patterns
            (id, description, error_pattern, error_code, context_patterns, fix_template,
             confidence, success_count, attempt_count, created_at, updated_at,
             context_hash, tags, contributor_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
                (
                    pattern.id.clone(),
                    pattern.description.clone(),
                    pattern.error_pattern.clone(),
                    pattern.error_code.clone(),
                    context_patterns_json,
                    fix_template_json,
                    pattern.confidence,
                    pattern.success_count,
                    pattern.attempt_count,
                    pattern.created_at.to_rfc3339(),
                    pattern.updated_at.to_rfc3339(),
                    pattern.context_hash.clone(),
                    tags_json,
                    pattern.contributor_id.clone(),
                ),
            )
            .await?;

        Ok(())
    }

    /// Get a pattern by ID
    pub async fn get_pattern_by_id(&self, pattern_id: &str) -> AIResult<Option<LearnedPattern>> {
        let pattern = self
            .db
            .query_row(
                "SELECT * FROM learned_patterns WHERE id = ?",
                (pattern_id.to_string(),),
                |row| LearningDatabase::row_to_learned_pattern(row),
            )
            .await?;
        Ok(pattern)
    }

    /// Get all patterns
    pub async fn get_all_patterns(&self) -> AIResult<Vec<LearnedPattern>> {
        let patterns = self
            .db
            .query_map(
                "SELECT * FROM learned_patterns ORDER BY updated_at DESC",
                [],
                |row| Ok(LearningDatabase::row_to_learned_pattern(row)?),
            )
            .await?;
        Ok(patterns)
    }

    /// Get patterns by error type
    pub async fn get_patterns_by_error_type(
        &self,
        error_code: &str,
        max_count: u32,
    ) -> AIResult<Vec<LearnedPattern>> {
        let patterns = self
            .db
            .query_map(
                r#"
            SELECT * FROM learned_patterns
            WHERE error_code = ?
            ORDER BY confidence DESC
            LIMIT ?
        "#,
                (error_code.to_string(), max_count),
                |row| Ok(LearningDatabase::row_to_learned_pattern(row)?),
            )
            .await?;
        Ok(patterns)
    }

    /// Update pattern success statistics
    pub async fn update_pattern_success(
        &self,
        pattern_id: &str,
        was_successful: bool,
        _user_id: &str,
    ) -> AIResult<()> {
        let query = if was_successful {
            "UPDATE learned_patterns SET success_count = success_count + 1, attempt_count = attempt_count + 1, updated_at = ? WHERE id = ?"
        } else {
            "UPDATE learned_patterns SET attempt_count = attempt_count + 1, updated_at = ? WHERE id = ?"
        };

        self.db
            .execute(query, (Utc::now().to_rfc3339(), pattern_id.to_string()))
            .await?;

        // Record the application
        self.db
            .execute(
                r#"
            INSERT INTO pattern_applications (pattern_id, applied_at, was_successful, error_context)
            VALUES (?, ?, ?, ?)
        "#,
                (
                    pattern_id.to_string(),
                    Utc::now().to_rfc3339(),
                    was_successful,
                    String::from(""),
                ),
            )
            .await?;

        Ok(())
    }

    /// Clear all patterns
    pub async fn clear_all_patterns(&self) -> AIResult<()> {
        self.db.execute("DELETE FROM learned_patterns", []).await?;
        self.db
            .execute("DELETE FROM pattern_applications", [])
            .await?;
        Ok(())
    }

    /// Get pattern statistics
    pub async fn get_pattern_statistics(&self) -> AIResult<PatternStatistics> {
        let total_patterns: i64 = self
            .db
            .query_row(
                "SELECT COUNT(*) as count FROM learned_patterns",
                [],
                |row| row.get(0),
            )
            .await?
            .unwrap_or(0);

        let successful_patterns: i64 = self
            .db
            .query_row(
                "SELECT COUNT(*) as count FROM learned_patterns WHERE success_count > 0",
                [],
                |row| row.get(0),
            )
            .await?
            .unwrap_or(0);

        let recent_patterns: i64 = self.db.query_row("SELECT COUNT(*) as count FROM learned_patterns WHERE updated_at > datetime('now', '-30 days')", [], |row| {
            row.get(0)
        }).await?.unwrap_or(0);

        Ok(PatternStatistics {
            total_patterns: total_patterns as u32,
            successful_patterns: successful_patterns as u32,
            recent_patterns: recent_patterns as u32,
            success_rate: if total_patterns > 0 {
                successful_patterns as f32 / total_patterns as f32
            } else {
                0.0
            },
        })
    }

    /// Load preferences from database
    pub async fn load_preferences(&self) -> AIResult<LearningPreferences> {
        let mut preferences = LearningPreferences::default();

        let prefs = self
            .db
            .query_map("SELECT key, value FROM user_preferences", [], |row| {
                let key: String = row.get(0)?;
                let value: String = row.get(1)?;
                Ok((key, value))
            })
            .await?;

        for (key, value) in prefs {
            match key.as_str() {
                "enable_learning" => {
                    preferences.enable_learning = value.parse().unwrap_or(true);
                }
                "privacy_mode" => {
                    preferences.privacy_mode =
                        serde_json::from_str(&value).unwrap_or(PrivacyMode::OptIn);
                }
                "confidence_threshold" => {
                    preferences.confidence_threshold = value.parse().unwrap_or(0.7);
                }
                "max_patterns_per_type" => {
                    preferences.max_patterns_per_type = value.parse().unwrap_or(100);
                }
                "enable_community_sharing" => {
                    preferences.enable_community_sharing = value.parse().unwrap_or(false);
                }
                "use_community_patterns" => {
                    preferences.use_community_patterns = value.parse().unwrap_or(true);
                }
                "auto_apply_threshold" => {
                    preferences.auto_apply_threshold = value.parse().unwrap_or(0.9);
                }
                _ => {} // Ignore unknown preferences
            }
        }

        Ok(preferences)
    }

    /// Save preferences to database
    pub async fn save_preferences(&self, preferences: &LearningPreferences) -> AIResult<()> {
        let prefs = vec![
            ("enable_learning", preferences.enable_learning.to_string()),
            (
                "privacy_mode",
                serde_json::to_string(&preferences.privacy_mode).unwrap(),
            ),
            (
                "confidence_threshold",
                preferences.confidence_threshold.to_string(),
            ),
            (
                "max_patterns_per_type",
                preferences.max_patterns_per_type.to_string(),
            ),
            (
                "enable_community_sharing",
                preferences.enable_community_sharing.to_string(),
            ),
            (
                "use_community_patterns",
                preferences.use_community_patterns.to_string(),
            ),
            (
                "auto_apply_threshold",
                preferences.auto_apply_threshold.to_string(),
            ),
        ];

        for (key, value) in prefs {
            self.db
                .execute(
                    r#"
                INSERT OR REPLACE INTO user_preferences (key, value, updated_at)
                VALUES (?, ?, ?)
            "#,
                    (key, value, Utc::now().to_rfc3339()),
                )
                .await?;
        }

        Ok(())
    }

    /// Import patterns
    pub async fn import_patterns(&self, patterns: &[LearnedPattern]) -> AIResult<usize> {
        let mut imported_count = 0;
        for pattern in patterns {
            // Check if pattern already exists
            if self.get_pattern_by_id(&pattern.id).await?.is_none() {
                self.store_pattern(pattern).await?;
                imported_count += 1;
            }
        }
        Ok(imported_count)
    }

    /// Export patterns
    pub async fn export_patterns(&self, privacy_mode: PrivacyMode) -> AIResult<String> {
        let patterns = self.get_all_patterns().await?;

        // Anonymize patterns if privacy mode requires it
        let export_patterns: Vec<LearnedPattern> = patterns
            .into_iter()
            .map(|mut pattern| {
                if matches!(privacy_mode, PrivacyMode::Anonymous | PrivacyMode::OptOut) {
                    pattern.contributor_id = None;
                }
                pattern
            })
            .collect();

        serde_json::to_string_pretty(&export_patterns).map_err(|e| AIServiceError::Serialization(e))
    }

    /// Convert database row to LearnedPattern
    fn row_to_learned_pattern(row: &rusqlite::Row) -> rusqlite::Result<LearnedPattern> {
        use super::models::FixTemplate;

        let context_patterns_json: String = row.get("context_patterns")?;
        let context_patterns: Vec<String> =
            serde_json::from_str(&context_patterns_json).unwrap_or_default();

        let fix_template_json: String = row.get("fix_template")?;
        let fix_template: FixTemplate = serde_json::from_str(&fix_template_json).unwrap();

        let tags_json: String = row.get("tags")?;
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

        let created_at: String = row.get("created_at")?;
        let updated_at: String = row.get("updated_at")?;

        let created_dt = DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    9,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);

        let updated_dt = DateTime::parse_from_rfc3339(&updated_at)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    10,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);

        Ok(LearnedPattern {
            id: row.get("id")?,
            description: row.get("description")?,
            error_pattern: row.get("error_pattern")?,
            error_code: row.get("error_code")?,
            context_patterns,
            fix_template,
            confidence: row.get("confidence")?,
            success_count: row.get("success_count")?,
            attempt_count: row.get("attempt_count")?,
            created_at: created_dt,
            updated_at: updated_dt,
            context_hash: row.get("context_hash")?,
            tags,
            contributor_id: row.get("contributor_id")?,
        })
    }
}
