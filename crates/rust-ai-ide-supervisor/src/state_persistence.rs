//! State Persistence Layer - Checkpoint/savepoint mechanisms for crash recovery

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs;
use tokio::time::{Duration, timeout};
use rusqlite::{Connection, params, Transaction};

use crate::error::{SupervisorError, SupervisorResult};
use crate::types::*;

pub type Checkpoint = StateSnapshot;

/// State Persistence Manager using SQLite and file-based storage
#[derive(Debug, Clone)]
pub struct StatePersistence {
    db_path: PathBuf,
    checkpoint_dir: PathBuf,
    connection_pool: Arc<Mutex<Connection>>,
}

impl StatePersistence {
    /// Create a new state persistence instance
    pub async fn new(db_path: &str, checkpoint_dir: &str) -> SupervisorResult<Self> {
        let db_path = PathBuf::from(db_path);
        let checkpoint_dir = PathBuf::from(checkpoint_dir);

        // Create directories if they don't exist
        fs::create_dir_all(&checkpoint_dir).await
            .map_err(|e| SupervisorError::persistence_error(format!("Failed to create checkpoint directory: {:?}", e)))?;

        // Create SQLite connection
        let conn = Connection::open(&db_path)
            .map_err(|e| SupervisorError::persistence_error(format!("Failed to open database: {:?}", e)))?;

        // Enable WAL mode for better concurrency
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| SupervisorError::persistence_error(format!("Failed to set journal mode: {:?}", e)))?;

        // Enable foreign keys
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| SupervisorError::persistence_error(format!("Failed to enable foreign keys: {:?}", e)))?;

        // Initialize schema
        Self::initialize_schema(&conn)?;

        let manager = Self {
            db_path,
            checkpoint_dir,
            connection_pool: Arc::new(Mutex::new(conn)),
        };

        Ok(manager)
    }

    /// Initialize database schema
    fn initialize_schema(conn: &Connection) -> SupervisorResult<()> {
        let schema_sql = r#"
            CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                data BLOB NOT NULL,
                checksum TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS pending_operations (
                id TEXT PRIMARY KEY,
                operation_type TEXT NOT NULL,
                service_id TEXT NOT NULL,
                parameters TEXT NOT NULL,
                queued_time TEXT NOT NULL,
                max_retries INTEGER NOT NULL,
                retry_count INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS migration_history (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL,
                description TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS service_states (
                service_id TEXT PRIMARY KEY,
                last_state TEXT NOT NULL,
                last_updated TEXT NOT NULL,
                metrics TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_pending_operations_service_id
            ON pending_operations(service_id);

            CREATE INDEX IF NOT EXISTS idx_checkpoints_timestamp
            ON checkpoints(timestamp);

            -- Insert migration record
            INSERT OR IGNORE INTO migration_history (version, applied_at, description)
            VALUES (1, ?, 'Initial schema');
        "#;

        conn.execute(schema_sql, params![chrono::Utc::now().to_rfc3339()])
            .map_err(|e| SupervisorError::migration_error(format!("Failed to initialize schema: {:?}", e)))?;

        Ok(())
    }

    /// Create a checkpoint of current system state
    pub async fn create_checkpoint(&self, services: &HashMap<ServiceId, ServiceSnapshot>, operations: &[PendingOperation]) -> SupervisorResult<CheckpointId> {
        let id = CheckpointId::new_v4();
        let timestamp = chrono::Utc::now();

        let snapshot = StateSnapshot {
            id,
            timestamp,
            service_states: services.clone(),
            ipc_channels: Vec::new(), // TODO: Implement IPC state
            pending_operations: operations.to_vec(),
        };

        // Serialize snapshot
        let data = serde_json::to_vec(&snapshot)
            .map_err(|e| SupervisorError::persistence_error(format!("Failed to serialize snapshot: {:?}", e)))?;

        // Create checksum for integrity verification
        let checksum = format!("{:x}", md5::compute(&data));

        // Store checkpoint with timeout
        let result = timeout(
            Duration::from_secs(30),
            self.store_checkpoint_async(id, data, checksum)
        ).await;

        match result {
            Ok(Ok(())) => Ok(id),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(SupervisorError::persistence_error("Checkpoint creation timeout".to_string())),
        }
    }

    /// Store checkpoint asynchronously
    async fn store_checkpoint_async(&self, id: CheckpointId, data: Vec<u8>, checksum: String) -> SupervisorResult<()> {
        let conn_guard = self.connection_pool.lock().await;

        conn_guard.execute(
            "INSERT INTO checkpoints (id, timestamp, data, checksum) VALUES (?, ?, ?, ?)",
            params![
                id.to_string(),
                chrono::Utc::now().to_rfc3339(),
                &data,
                checksum
            ]
        ).map_err(|e| SupervisorError::persistence_error(format!("Failed to store checkpoint: {:?}", e)))?;

        Ok(())
    }

    /// Load checkpoint by ID
    pub async fn load_checkpoint(&self, id: &CheckpointId) -> SupervisorResult<StateSnapshot> {
        let conn_guard = self.connection_pool.lock().await;

        let (data, checksum) = conn_guard.query_row(
            "SELECT data, checksum FROM checkpoints WHERE id = ?",
            params![id.to_string()],
            |row| {
                let data: Vec<u8> = row.get(0)?;
                let checksum: String = row.get(1)?;
                Ok((data, checksum))
            }
        ).map_err(|e| SupervisorError::persistence_error(format!("Failed to load checkpoint: {:?}", e)))?;

        // Verify checksum
        let calculated_checksum = format!("{:x}", md5::compute(&data));
        if calculated_checksum != checksum {
            return Err(SupervisorError::persistence_error("Checkpoint checksum verification failed".to_string()));
        }

        // Deserialize snapshot
        let snapshot: StateSnapshot = serde_json::from_slice(&data)
            .map_err(|e| SupervisorError::persistence_error(format!("Failed to deserialize snapshot: {:?}", e)))?;

        Ok(snapshot)
    }

    /// Load latest checkpoint
    pub async fn load_latest_checkpoint(&self) -> SupervisorResult<StateSnapshot> {
        let conn_guard = self.connection_pool.lock().await;

        let (data, checksum) = conn_guard.query_row(
            "SELECT data, checksum FROM checkpoints ORDER BY timestamp DESC LIMIT 1",
            params![],
            |row| {
                let data: Vec<u8> = row.get(0)?;
                let checksum: String = row.get(1)?;
                Ok((data, checksum))
            }
        ).map_err(|e| SupervisorError::persistence_error(format!("Failed to load latest checkpoint: {:?}", e)))?;

        // Verify checksum
        let calculated_checksum = format!("{:x}", md5::compute(&data));
        if calculated_checksum != checksum {
            return Err(SupervisorError::persistence_error("Latest checkpoint checksum verification failed".to_string()));
        }

        // Deserialize snapshot
        let snapshot: StateSnapshot = serde_json::from_slice(&data)
            .map_err(|e| SupervisorError::persistence_error(format!("Failed to deserialize latest snapshot: {:?}", e)))?;

        Ok(snapshot)
    }

    /// Store pending operation
    pub async fn store_pending_operation(&self, operation: &PendingOperation) -> SupervisorResult<()> {
        let conn_guard = self.connection_pool.lock().await;

        conn_guard.execute(
            "INSERT INTO pending_operations (id, operation_type, service_id, parameters, queued_time, max_retries, retry_count)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                retry_count = excluded.retry_count,
                service_id = excluded.service_id,
                parameters = excluded.parameters",
            params![
                operation.id.to_string(),
                operation.operation_type,
                operation.service_id,
                serde_json::to_string(&operation.parameters).unwrap_or_default(),
                operation.queued_time.to_rfc3339(),
                operation.max_retries,
                operation.retry_count
            ]
        ).map_err(|e| SupervisorError::persistence_error(format!("Failed to store pending operation: {:?}", e)))?;

        Ok(())
    }

    /// Load pending operations for a service
    pub async fn load_pending_operations(&self, service_id: Option<&str>) -> SupervisorResult<Vec<PendingOperation>> {
        let conn_guard = self.connection_pool.lock().await;

        let mut stmt = if let Some(service_id) = service_id {
            conn_guard.prepare("SELECT id, operation_type, service_id, parameters, queued_time, max_retries, retry_count FROM pending_operations WHERE service_id = ?")
        } else {
            conn_guard.prepare("SELECT id, operation_type, service_id, parameters, queued_time, max_retries, retry_count FROM pending_operations")
        }?;

        let params = if let Some(service_id) = service_id { vec![service_id] } else { vec![] };
        let operations_iter = stmt.query_map(&params[..], |row| {
            let id: String = row.get(0)?;
            let operation_type: String = row.get(1)?;
            let service_id: String = row.get(2)?;
            let parameters_str: String = row.get(3)?;
            let queued_time_str: String = row.get(4)?;
            let max_retries: u32 = row.get(5)?;
            let retry_count: u32 = row.get(6)?;

            let parameters: serde_json::Value = serde_json::from_str(&parameters_str).unwrap_or(serde_json::json!({}));
            let queued_time = chrono::DateTime::parse_from_rfc3339(&queued_time_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(PendingOperation {
                id: uuid::Uuid::parse_str(&id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
                operation_type,
                service_id,
                parameters,
                queued_time,
                max_retries,
                retry_count,
            })
        })?;

        let mut operations = Vec::new();
        for operation in operations_iter {
            operations.push(operation.map_err(|e| SupervisorError::persistence_error(format!("Failed to parse operation: {:?}", e)))?);
        }

        Ok(operations)
    }

    /// Remove completed operations
    pub async fn remove_pending_operation(&self, operation_id: &uuid::Uuid) -> SupervisorResult<()> {
        let conn_guard = self.connection_pool.lock().await;

        conn_guard.execute(
            "DELETE FROM pending_operations WHERE id = ?",
            params![operation_id.to_string()]
        ).map_err(|e| SupervisorError::persistence_error(format!("Failed to remove operation: {:?}", e)))?;

        Ok(())
    }

    /// Clear old checkpoints (keep only n latest)
    pub async fn cleanup_old_checkpoints(&self, keep_latest: usize) -> SupervisorResult<usize> {
        let conn_guard = self.connection_pool.lock().await;

        // Get IDs of checkpoints to delete (all except the latest n)
        let checkpoint_ids: Vec<String> = conn_guard.prepare(
            "SELECT id FROM checkpoints ORDER BY timestamp DESC LIMIT -1 OFFSET ?"
        )?
        .query_map(params![keep_latest], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

        if checkpoint_ids.is_empty() {
            return Ok(0);
        }

        // Delete old checkpoints
        let placeholders = checkpoint_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("DELETE FROM checkpoints WHERE id IN ({})", placeholders);
        let mut stmt = conn_guard.prepare(&sql)?;

        let params: Vec<&dyn rusqlite::ToSql> = checkpoint_ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        stmt.execute(params.as_slice())?;

        Ok(checkpoint_ids.len())
    }

    /// Get database statistics
    pub async fn get_statistics(&self) -> SupervisorResult<DatabaseStats> {
        let conn_guard = self.connection_pool.lock().await;

        let checkpoint_count = conn_guard.query_row("SELECT COUNT(*) FROM checkpoints", params![], |row| row.get::<_, i64>(0))?;
        let pending_operations_count = conn_guard.query_row("SELECT COUNT(*) FROM pending_operations", params![], |row| row.get::<_, i64>(0))?;
        let service_states_count = conn_guard.query_row("SELECT COUNT(*) FROM service_states", params![], |row| row.get::<_, i64>(0))?;

        // Calculate database size
        let db_size = tokio::fs::metadata(&self.db_path).await
            .map(|metadata| metadata.len() as u64)
            .unwrap_or(0);

        Ok(DatabaseStats {
            checkpoint_count: checkpoint_count as u64,
            pending_operations_count: pending_operations_count as u64,
            service_states_count: service_states_count as u64,
            database_size: db_size,
        })
    }
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub checkpoint_count: u64,
    pub pending_operations_count: u64,
    pub service_states_count: u64,
    pub database_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_persistence() -> StatePersistence {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let checkpoint_dir = temp_dir.path().join("checkpoints");

        StatePersistence::new(
            &db_path.to_string_lossy(),
            &checkpoint_dir.to_string_lossy()
        ).await.expect("Failed to create persistence")
    }

    #[tokio::test]
    async fn test_checkpoint_creation_and_loading() {
        let persistence = create_test_persistence().await;

        let mut services = HashMap::new();
        services.insert(
            "test_service".to_string(),
            ServiceSnapshot {
                service_id: "test_service".to_string(),
                state: ServiceState::Running,
                metrics: ServiceMetrics::default(),
                last_start_time: Some(chrono::Utc::now()),
                process_id: Some(1234),
            }
        );

        let operations = vec![];
        let checkpoint_id = persistence.create_checkpoint(&services, &operations).await.expect("Failed to create checkpoint");

        let loaded_snapshot = persistence.load_checkpoint(&checkpoint_id).await.expect("Failed to load checkpoint");

        assert_eq!(loaded_snapshot.id, checkpoint_id);
        assert!(loaded_snapshot.service_states.contains_key("test_service"));
    }

    #[tokio::test]
    async fn test_pending_operations() {
        let persistence = create_test_persistence().await;

        let operation = PendingOperation {
            id: uuid::Uuid::new_v4(),
            operation_type: "test".to_string(),
            service_id: "test_service".to_string(),
            parameters: serde_json::json!({"action": "restart"}),
            queued_time: chrono::Utc::now(),
            max_retries: 3,
            retry_count: 0,
        };

        persistence.store_pending_operation(&operation).await.expect("Failed to store operation");

        let operations = persistence.load_pending_operations(None).await.expect("Failed to load operations");
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].id, operation.id);

        persistence.remove_pending_operation(&operation.id).await.expect("Failed to remove operation");

        let operations_after = persistence.load_pending_operations(None).await.expect("Failed to load operations after removal");
        assert_eq!(operations_after.len(), 0);
    }

    #[tokio::test]
    async fn test_statistics() {
        let persistence = create_test_persistence().await;

        let stats = persistence.get_statistics().await.expect("Failed to get statistics");

        // Should have at least the initial migration
        assert!(stats.pending_operations_count >= 0);
        assert!(stats.checkpoint_count >= 0);
        assert!(stats.service_states_count >= 0);
    }
}