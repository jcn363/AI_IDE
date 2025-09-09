use rusqlite::{Connection, Result as SqliteResult, params};
use std::path::Path;
use tokio::task;
use crate::types::{AIServiceError, AIResult};

/// Async database wrapper using rusqlite with spawn_blocking
pub struct AsyncDatabase {
    conn: Connection,
}

impl AsyncDatabase {
    /// Create a new database connection from file path
    pub async fn new<T: AsRef<Path>>(db_path: T) -> AIResult<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        let conn = task::spawn_blocking(move || {
            Connection::open(db_path)
        }).await
        .map_err(|e| AIServiceError::DatabaseError(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::DatabaseError(format!("Connection error: {}", e)))?;

        Ok(Self { conn })
    }

    /// Create database connection in memory (for testing)
    pub async fn new_in_memory() -> AIResult<Self> {
        let conn = task::spawn_blocking(|| {
            Connection::open_in_memory()
        }).await
        .map_err(|e| AIServiceError::DatabaseError(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::DatabaseError(format!("Connection error: {}", e)))?;

        Ok(Self { conn })
    }

    /// Execute a SQL query with parameters
    pub async fn execute<P: rusqlite::Params>(&self, sql: &str, params: P) -> AIResult<usize> {
        let sql = sql.to_string();
        task::spawn_blocking(move || {
            let mut stmt = self.conn.prepare(&sql)?;
            stmt.execute(params)
        }).await
        .map_err(|e| AIServiceError::DatabaseError(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::DatabaseError(format!("Execute error: {}", e)))
    }

    /// Execute a SQL query and return a prepared statement for queries
    pub async fn prepare_statement(&self, sql: &str) -> AIResult<PreparedStatement> {
        let sql = sql.to_string();
        task::spawn_blocking(move || {
            let stmt = self.conn.prepare(&sql)?;
            Ok(PreparedStatement { stmt })
        }).await
        .map_err(|e| AIServiceError::DatabaseError(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::DatabaseError(format!("Prepare error: {}", e)))
    }

    /// Execute multiple statements in a transaction
    pub async fn execute_batch(&self, sql: &str) -> AIResult<()> {
        let sql = sql.to_string();
        task::spawn_blocking(move || {
            self.conn.execute_batch(&sql)
        }).await
        .map_err(|e| AIServiceError::DatabaseError(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::DatabaseError(format!("Batch execute error: {}", e)))
    }

    /// Get rows from a query
    pub async fn query_map<P, F, T>(&self, sql: &str, params: P, mapper: F)
        -> AIResult<Vec<T>>
    where
        P: rusqlite::Params,
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let sql = sql.to_string();
        task::spawn_blocking(move || {
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map(params, mapper)?;
            rows.collect::<rusqlite::Result<Vec<T>>>()
        }).await
        .map_err(|e| AIServiceError::DatabaseError(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::DatabaseError(format!("Query map error: {}", e)))
    }

    /// Get a single optional row from a query
    pub async fn query_row<P, F, T>(&self, sql: &str, params: P, mapper: F)
        -> AIResult<Option<T>>
    where
        P: rusqlite::Params,
        F: FnOnce(&rusqlite::Row) -> rusqlite::Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let sql = sql.to_string();
        task::spawn_blocking(move || {
            let mut stmt = self.conn.prepare(&sql)?;
            match stmt.query_row(params, mapper) {
                Ok(result) => Ok(Some(result)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        }).await
        .map_err(|e| AIServiceError::DatabaseError(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::DatabaseError(format!("Query row error: {}", e)))
    }
}

/// Wrapper for prepared statements
pub struct PreparedStatement {
    stmt: rusqlite::Statement,
}

impl PreparedStatement {
    /// Execute the prepared statement with parameters
    pub async fn execute<P: rusqlite::Params + Send + 'static>(&mut self, params: P) -> AIResult<usize> {
        task::spawn_blocking(move || {
            self.stmt.execute(params)
        }).await
        .map_err(|e| AIServiceError::DatabaseError(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::DatabaseError(format!("Execute error: {}", e)))
    }

    /// Query with mapping function
    pub async fn query_map<F, T, P>(&mut self, params: P, mapper: F) -> AIResult<Vec<T>>
    where
        P: rusqlite::Params,
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T> + Send + 'static,
        T: Send + 'static,
    {
        task::spawn_blocking(move || {
            let rows = self.stmt.query_map(params, mapper)?;
            rows.collect::<rusqlite::Result<Vec<T>>>()
        }).await
        .map_err(|e| AIServiceError::DatabaseError(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::DatabaseError(format!("Query map error: {}", e)))
    }

    /// Query optional row
    pub async fn query_row<F, T, P>(&mut self, params: P, mapper: F) -> AIResult<Option<T>>
    where
        P: rusqlite::Params,
        F: FnOnce(&rusqlite::Row) -> rusqlite::Result<T> + Send + 'static,
        T: Send + 'static,
    {
        task::spawn_blocking(move || {
            match self.stmt.query_row(params, mapper) {
                Ok(result) => Ok(Some(result)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        }).await
        .map_err(|e| AIServiceError::DatabaseError(format!("Async spawn error: {}", e)))?
        .map_err(|e| AIServiceError::DatabaseError(format!("Query row error: {}", e)))
    }
}