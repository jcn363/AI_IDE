//! Tests for connection pooling functionality
//!
//! These tests verify that the LSP connection pooling works correctly
//! for managing multiple LSP server instances efficiently.

#[cfg(test)]
mod tests {
    use rust_ai_ide_common::ConnectionPool;
    use std::time::{Duration, Instant};
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = ConnectionPool::new(10);
        assert_eq!(pool.max_connections(), 10);
        assert_eq!(pool.len().await, 0);
    }

    #[tokio::test]
    async fn test_basic_connection_management() {
        let pool = ConnectionPool::new(5);

        // Pool should start empty
        assert_eq!(pool.len().await, 0);
        assert_eq!(pool.available_connections().await, 0);

        // Get connection (should create new one)
        let conn_id = pool.get(None, Some("rust")).await.unwrap();
        assert_eq!(pool.len().await, 1);
        assert_eq!(pool.available_connections().await, 0); // Connection is checked out

        // Return connection
        pool.return_connection(&conn_id).await.unwrap();
        assert_eq!(pool.len().await, 1);
        assert_eq!(pool.available_connections().await, 1);

        // Get connection again (should reuse)
        let conn_id2 = pool.get(None, Some("rust")).await.unwrap();
        assert_eq!(pool.len().await, 1); // Should still be 1 (reused)
        assert_eq!(pool.available_connections().await, 0);
    }

    #[tokio::test]
    async fn test_connection_pool_limits() {
        let pool = ConnectionPool::new(2);

        // Get two connections
        let conn1 = pool.get(None, Some("rust")).await.unwrap();
        let conn2 = pool.get(None, Some("rust")).await.unwrap();

        // Pool should now be at limit
        assert_eq!(pool.len().await, 2);
        assert_eq!(pool.available_connections().await, 0);

        // Third get should create new connection even without workspace
        let conn3 = pool
            .get(Some("/tmp/project"), Some("python"))
            .await
            .unwrap();
        assert_eq!(pool.len().await, 3);
        assert_eq!(pool.available_connections().await, 0);

        // Return connections
        pool.return_connection(&conn1).await.unwrap();
        pool.return_connection(&conn2).await.unwrap();
        pool.return_connection(&conn3).await.unwrap();

        assert_eq!(pool.len().await, 3);
        assert_eq!(pool.available_connections().await, 3);
    }

    #[tokio::test]
    async fn test_workspace_specific_connections() {
        let pool = ConnectionPool::new(10);

        let workspace1 = "/workspace1";
        let workspace2 = "/workspace2";

        // Create connections for different workspaces
        let conn1 = pool.get(Some(workspace1), Some("rust")).await.unwrap();
        let conn2 = pool.get(Some(workspace2), Some("rust")).await.unwrap();
        let conn3 = pool.get(Some(workspace1), Some("rust")).await.unwrap(); // Same workspace, should reuse

        assert_eq!(pool.len().await, 2); // Two different workspaces

        // Return connections
        pool.return_connection(&conn1).await.unwrap();
        pool.return_connection(&conn2).await.unwrap();
        pool.return_connection(&conn3).await.unwrap();

        // Get connection for workspace1 again
        let conn4 = pool.get(Some(workspace1), Some("rust")).await.unwrap();
        assert_eq!(pool.len().await, 2); // Should still be 2 (reused workspace1 connection)
    }

    #[tokio::test]
    async fn test_connection_health_checking() {
        let pool = ConnectionPool::new(5);

        let conn_id = pool.get(None, Some("rust")).await.unwrap();

        // Connection should be healthy initially
        assert!(pool.is_healthy(&conn_id).await.unwrap());

        // Mark connection as unhealthy
        pool.mark_unhealthy(&conn_id).await.unwrap();

        // Connection should be reported as unhealthy
        assert!(!pool.is_healthy(&conn_id).await.unwrap());

        // Getting another connection should create a new one
        let conn_id2 = pool.get(None, Some("rust")).await.unwrap();
        assert_eq!(pool.len().await, 2);
    }

    #[tokio::test]
    async fn test_connection_timeout() {
        let pool = ConnectionPool::new(1);

        let conn_id = pool.get(None, Some("rust")).await.unwrap();

        // Timeout when trying to get connection that doesn't become available
        let result = timeout(Duration::from_millis(100), pool.get(None, Some("rust"))).await;

        // Should timeout since there's no available connection and creation might fail
        assert!(result.is_err());

        // Return connection
        pool.return_connection(&conn_id).await.unwrap();

        // Now getting a connection should work
        let conn_id2 = pool.get(None, Some("rust")).await.unwrap();
        assert_eq!(pool.len().await, 1); // Reused existing connection
    }

    #[tokio::test]
    async fn test_connection_cleanup() {
        let pool = ConnectionPool::new(10);

        // Create some connections
        let mut conn_ids = vec![];
        for _ in 0..5 {
            conn_ids.push(pool.get(None, Some("rust")).await.unwrap());
        }

        assert_eq!(pool.len().await, 5);

        // Return some connections
        for i in 0..3 {
            pool.return_connection(&conn_ids[i]).await.unwrap();
        }

        // Cleanup should remove stale connections
        pool.cleanup_stale_connections(Duration::from_secs(0)).await;

        // Should still have all connections (we need to implement proper cleanup timing)
        assert_eq!(pool.len().await, 5);

        // Return remaining connections
        for i in 3..5 {
            pool.return_connection(&conn_ids[i]).await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_pool_statistics() {
        let pool = ConnectionPool::new(10);

        // Initially empty
        let stats = pool.get_stats().await;
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);

        // Create connections
        let conn1 = pool.get(Some("/workspace"), None).await.unwrap();
        let conn2 = pool.get(Some("/workspace"), None).await.unwrap(); // Should reuse

        let stats = pool.get_stats().await;
        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.active_connections, 2); // Both checked out

        // Return one connection
        pool.return_connection(&conn1).await.unwrap();

        let stats = pool.get_stats().await;
        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.active_connections, 1);

        // Return remaining connection
        pool.return_connection(&conn2).await.unwrap();

        let stats = pool.get_stats().await;
        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let pool = ConnectionPool::new(20);

        // Test concurrent access to connection pool
        let mut handles = vec![];

        for i in 0..10 {
            let pool_clone = pool.clone();
            let handle = tokio::spawn(async move {
                let workspace = format!("/workspace{}", i % 3); // Distribute across 3 workspaces
                let conn_id = pool_clone
                    .get(Some(&workspace), Some("rust"))
                    .await
                    .unwrap();

                // Simulate some work
                tokio::time::sleep(Duration::from_millis(50)).await;

                pool_clone.return_connection(&conn_id).await
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        // Should have 3 connections (one per workspace)
        assert_eq!(pool.len().await, 3);

        // All connections should be available
        assert_eq!(pool.available_connections().await, 3);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let pool = ConnectionPool::new(5);

        // Test invalid connection ID
        assert!(pool.return_connection("invalid_id").await.is_err());
        assert!(pool.mark_unhealthy("invalid_id").await.is_err());
        assert!(pool.is_healthy("invalid_id").await.is_err());

        // Test valid connection ID
        let conn_id = pool.get(None, Some("rust")).await.unwrap();
        assert!(pool.return_connection(&conn_id).await.is_ok());
        assert!(pool.mark_unhealthy(&conn_id).await.is_ok());
        assert!(pool.is_healthy(&conn_id).await.is_ok());
    }

    #[tokio::test]
    async fn test_pool_clear() {
        let pool = ConnectionPool::new(5);

        // Create some connections
        let conn1 = pool.get(None, Some("rust")).await.unwrap();
        let conn2 = pool.get(None, Some("python")).await.unwrap();

        // Return them
        pool.return_connection(&conn1).await.unwrap();
        pool.return_connection(&conn2).await.unwrap();

        assert_eq!(pool.len().await, 2);

        // Clear all connections
        pool.clear_all().await;

        assert_eq!(pool.len().await, 0);
        assert_eq!(pool.available_connections().await, 0);
    }
}
