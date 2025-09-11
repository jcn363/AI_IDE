//! Database testing utilities and fixtures
//!
//! Provides unified database testing patterns for SQLite and PostgreSQL,
//! including in-memory databases, schema management, and data seeding.

use crate::error::TestError;
use crate::filesystem::TempWorkspace;
use std::path::PathBuf;

/// Database connection type
#[derive(Debug, Clone)]
pub enum DatabaseType {
    /// SQLite database (file-based or in-memory)
    Sqlite,
    /// PostgreSQL database
    Postgres,
}

/// Database configuration for test setup
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub db_type: DatabaseType,
    pub connection_string: String,
    pub schema_path: Option<PathBuf>,
    pub seed_data_path: Option<PathBuf>,
    pub migrations_path: Option<PathBuf>,
}

impl DatabaseConfig {
    /// Create a new SQLite in-memory database configuration
    pub fn sqlite_memory() -> Self {
        Self {
            db_type: DatabaseType::Sqlite,
            connection_string: ":memory:".to_string(),
            schema_path: None,
            seed_data_path: None,
            migrations_path: None,
        }
    }

    /// Create a new SQLite file-based database configuration
    pub fn sqlite_file(path: impl Into<PathBuf>) -> Self {
        Self {
            db_type: DatabaseType::Sqlite,
            connection_string: path.into().to_string_lossy().to_string(),
            schema_path: None,
            seed_data_path: None,
            migrations_path: None,
        }
    }

    /// Create a new PostgreSQL database configuration
    pub fn postgres(connection_string: impl Into<String>) -> Self {
        Self {
            db_type: DatabaseType::Postgres,
            connection_string: connection_string.into(),
            schema_path: None,
            seed_data_path: None,
            migrations_path: None,
        }
    }

    /// Set schema file path
    pub fn with_schema(mut self, path: impl Into<PathBuf>) -> Self {
        self.schema_path = Some(path.into());
        self
    }

    /// Set seed data file path
    pub fn with_seed_data(mut self, path: impl Into<PathBuf>) -> Self {
        self.seed_data_path = Some(path.into());
        self
    }

    /// Set migrations directory path
    pub fn with_migrations(mut self, path: impl Into<PathBuf>) -> Self {
        self.migrations_path = Some(path.into());
        self
    }
}

/// Database test fixture that manages database lifecycle
pub struct DatabaseFixture {
    config: DatabaseConfig,
    #[cfg(feature = "database")]
    connection: Option<rusqlite::Connection>,
    temp_files: Vec<PathBuf>,
}

impl DatabaseFixture {
    /// Create a new database fixture
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "database")]
            connection: None,
            temp_files: Vec::new(),
        }
    }

    /// Setup the database with schema and seed data
    pub async fn setup(&mut self) -> Result<(), TestError> {
        match self.config.db_type {
            DatabaseType::Sqlite => self.setup_sqlite().await,
            DatabaseType::Postgres => self.setup_postgres().await,
        }
    }

    #[cfg(feature = "database")]
    async fn setup_sqlite(&mut self) -> Result<(), TestError> {
        let conn = if self.config.connection_string == ":memory:" {
            rusqlite::Connection::open_in_memory()?
        } else {
            rusqlite::Connection::open(&self.config.connection_string)?
        };

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        // Apply schema if provided
        if let Some(schema_path) = &self.config.schema_path {
            let schema = tokio::fs::read_to_string(schema_path).await?;
            conn.execute_batch(&schema)?;
        }

        // Apply seed data if provided
        if let Some(seed_path) = &self.config.seed_data_path {
            let seed_data = tokio::fs::read_to_string(seed_path).await?;
            conn.execute_batch(&seed_data)?;
        }

        self.connection = Some(conn);
        Ok(())
    }

    #[cfg(not(feature = "database"))]
    async fn setup_sqlite(&mut self) -> Result<(), TestError> {
        Err(TestError::Validation(
            crate::ValidationError::invalid_setup(
                "Database feature not enabled. Add 'database' feature flag.",
            ),
        ))
    }

    async fn setup_postgres(&mut self) -> Result<(), TestError> {
        // TODO: Implement PostgreSQL setup with sqlx
        Err(TestError::Validation(
            crate::ValidationError::invalid_setup("PostgreSQL support not yet implemented"),
        ))
    }

    /// Execute a query and return results
    #[cfg(feature = "database")]
    pub fn execute_query(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
    ) -> Result<Vec<rusqlite::Row>, TestError> {
        let conn = self.connection.as_ref().ok_or_else(|| {
            TestError::Validation(crate::ValidationError::invalid_setup(
                "Database not initialized",
            ))
        })?;

        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params, |row| Ok(row.to_owned()))?;
        let collected = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(collected)
    }

    /// Get database connection (SQLite only for now)
    #[cfg(feature = "database")]
    pub fn sqlite_connection(&self) -> Option<&rusqlite::Connection> {
        self.connection.as_ref()
    }
}

impl Drop for DatabaseFixture {
    fn drop(&mut self) {
        #[cfg(feature = "database")]
        if let Some(conn) = &self.connection {
            // Close connection explicitly
            drop(conn);
        }

        // Clean up temp files
        for file in &self.temp_files {
            let _ = std::fs::remove_file(file);
        }
    }
}

/// Predefined database fixture builders
pub struct DatabaseFixtures;

impl DatabaseFixtures {
    /// Create an in-memory SQLite database fixture
    pub fn sqlite_memory() -> DatabaseFixture {
        DatabaseFixture::new(DatabaseConfig::sqlite_memory())
    }

    /// Create a file-based SQLite database fixture
    pub fn sqlite_file(workspace: &TempWorkspace) -> Result<DatabaseFixture, TestError> {
        let db_path = workspace.path().join("test.db");
        Ok(DatabaseFixture::new(DatabaseConfig::sqlite_file(db_path)))
    }

    /// Create a PostgreSQL database fixture from connection string
    pub fn postgres(connection_string: &str) -> DatabaseFixture {
        DatabaseFixture::new(DatabaseConfig::postgres(connection_string))
    }
}

/// Mock database for testing without real database dependencies
pub struct MockDatabase {
    tables: std::collections::HashMap<
        String,
        Vec<std::collections::HashMap<String, serde_json::Value>>,
    >,
}

impl MockDatabase {
    pub fn new() -> Self {
        Self {
            tables: std::collections::HashMap::new(),
        }
    }

    /// Insert mock data into a table
    pub fn insert(
        &mut self,
        table: &str,
        data: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<(), TestError> {
        self.tables
            .entry(table.to_string())
            .or_insert_with(Vec::new)
            .push(data);
        Ok(())
    }

    /// Query mock data from a table
    pub fn query(
        &self,
        table: &str,
        condition: Option<&str>,
    ) -> Result<Vec<std::collections::HashMap<String, serde_json::Value>>, TestError> {
        let table_data = self.tables.get(table).cloned().unwrap_or_default();

        if let Some(cond) = condition {
            // Simple mock filtering - in real implementation, this would parse SQL-like conditions
            Ok(table_data
                .into_iter()
                .filter(|row| {
                    // Simple ID-based filtering for demonstration
                    if let Some(serde_json::Value::String(id)) = row.get("id") {
                        cond.contains(id)
                    } else {
                        true
                    }
                })
                .collect())
        } else {
            Ok(table_data)
        }
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.tables.clear();
    }
}

impl Default for MockDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Transaction wrapper for test isolation
pub struct DatabaseTransaction {
    #[cfg(feature = "database")]
    tx: Option<rusqlite::Transaction<'static>>,
}

#[cfg(feature = "database")]
impl DatabaseTransaction {
    /// Begin a new transaction
    pub fn begin(conn: &rusqlite::Connection) -> Result<Self, TestError> {
        let tx = conn.unchecked_transaction()?;
        Ok(Self { tx: Some(tx) })
    }

    /// Commit the transaction
    pub fn commit(mut self) -> Result<(), TestError> {
        if let Some(tx) = self.tx.take() {
            tx.commit()?;
        }
        Ok(())
    }

    /// Rollback the transaction (happens automatically on drop if not committed)
    pub fn rollback(mut self) -> Result<(), TestError> {
        self.tx = None; // Transaction will rollback on drop
        Ok(())
    }
}

#[cfg(not(feature = "database"))]
impl DatabaseTransaction {
    pub fn begin(_conn: &()) -> Result<Self, TestError> {
        Err(TestError::Validation(
            crate::ValidationError::invalid_setup("Database feature not enabled"),
        ))
    }

    pub fn commit(self) -> Result<(), TestError> {
        Ok(())
    }

    pub fn rollback(self) -> Result<(), TestError> {
        Ok(())
    }
}

impl Drop for DatabaseTransaction {
    fn drop(&mut self) {
        #[cfg(feature = "database")]
        if let Some(tx) = self.tx.take() {
            // Rollback on drop if not explicitly committed
            let _ = tx.rollback();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "database")]
    #[test]
    fn test_sqlite_memory_fixture() {
        let mut fixture = DatabaseFixtures::sqlite_memory();

        // Setup should succeed
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(fixture.setup()).expect("Setup should succeed");

        // Should be able to execute queries
        let results = fixture
            .execute_query("SELECT 1 as test", &[])
            .expect("Query should succeed");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_mock_database() {
        let mut db = MockDatabase::new();

        // Insert data
        let mut data = std::collections::HashMap::new();
        data.insert("id".to_string(), serde_json::json!("test-id"));
        data.insert("name".to_string(), serde_json::json!("test-name"));

        db.insert("users", data).expect("Insert should succeed");

        // Query data
        let results = db.query("users", None).expect("Query should succeed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["id"], "test-id");
    }
}
