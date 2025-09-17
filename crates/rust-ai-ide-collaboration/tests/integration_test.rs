// Integration tests for collaboration features

use std::sync::Arc;
use tokio::sync::RwLock;
use rust_ai_ide_collaboration::{
    CollaborationService,
    websocket::CollaborationWebSocketServer,
    session_management::SessionManager,
    performance_monitoring::CollaborationPerformanceMonitor,
    crdt::{TextDocument, EditorOperation, LamportClock},
};

#[tokio::test]
async fn test_collaboration_service_integration() {
    // Initialize services
    let collab_service = Arc::new(RwLock::new(CollaborationService::new()));
    let session_manager = Arc::new(RwLock::new(SessionManager::new()));
    let performance_monitor = Arc::new(RwLock::new(CollaborationPerformanceMonitor::new(Default::default())));

    // Test session creation
    let session_id = "test_session_123".to_string();
    let document_id = "test_doc_456".to_string();

    {
        let mut collab = collab_service.write().await;
        collab.create_session(session_id.clone(), document_id.clone()).await.expect("Failed to create session");
    }

    // Verify session was created
    {
        let collab = collab_service.read().await;
        let sessions = collab.get_active_sessions().await.expect("Failed to get sessions");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, session_id);
        assert_eq!(sessions[0].document_id, document_id);
    }

    println!("✅ Collaboration service integration test passed");
}

#[tokio::test]
async fn test_crdt_operations() {
    // Test CRDT operations
    let mut document = TextDocument::with_content("test_client".to_string(), "Hello World".to_string());

    // Create an insert operation
    let clock = LamportClock::new("test_client".to_string());
    let operation = EditorOperation::Insert {
        position: 5,
        content: " Beautiful".to_string(),
        op_id: "op1".to_string(),
        clock,
    };

    // Apply operation
    let result = document.apply_operation(operation);
    assert!(result.success);
    assert_eq!(result.new_content, "Hello Beautiful World");

    // Verify operation was recorded
    assert!(document.has_operation("op1"));

    println!("✅ CRDT operations test passed");
}

#[tokio::test]
async fn test_session_management() {
    let session_manager = Arc::new(RwLock::new(SessionManager::new()));

    // Test user management
    {
        let mut manager = session_manager.write().await;
        manager.add_user_to_session("session1", "user1", "client1").await.expect("Failed to add user");
        manager.add_user_to_session("session1", "user2", "client2").await.expect("Failed to add user");
    }

    // Verify users were added
    {
        let manager = session_manager.read().await;
        let participants = manager.get_session_participants("session1").await.expect("Failed to get participants");
        assert!(participants.is_some());
        let participants = participants.unwrap();
        assert_eq!(participants.len(), 2);
        assert!(participants.contains(&"user1".to_string()));
        assert!(participants.contains(&"user2".to_string()));
    }

    // Test user removal
    {
        let mut manager = session_manager.write().await;
        manager.remove_user_from_session("session1", "user1").await.expect("Failed to remove user");
    }

    // Verify user was removed
    {
        let manager = session_manager.read().await;
        let participants = manager.get_session_participants("session1").await.expect("Failed to get participants");
        assert!(participants.is_some());
        let participants = participants.unwrap();
        assert_eq!(participants.len(), 1);
        assert!(participants.contains(&"user2".to_string()));
        assert!(!participants.contains(&"user1".to_string()));
    }

    println!("✅ Session management test passed");
}

#[tokio::test]
async fn test_performance_monitoring() {
    use std::time::Duration;

    let performance_monitor = Arc::new(RwLock::new(CollaborationPerformanceMonitor::new(Default::default())));

    // Record some operations
    {
        let monitor = performance_monitor.read().await;
        monitor.record_operation("test_session", Duration::from_millis(50), "insert").await;
        monitor.record_operation("test_session", Duration::from_millis(75), "delete").await;
    }

    // Get metrics
    {
        let monitor = performance_monitor.read().await;
        let metrics = monitor.get_metrics("test_session").await;

        assert!(metrics.is_some());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.session_id, "test_session");
        assert!(metrics.operation_count >= 2);
        assert!(metrics.average_response_time_ms > 0.0);
    }

    println!("✅ Performance monitoring test passed");
}

#[tokio::test]
async fn test_lamport_clock_consistency() {
    let mut clock1 = LamportClock::new("client1".to_string());
    let mut clock2 = LamportClock::new("client2".to_string());

    // Test initial state
    assert_eq!(clock1.counter, 0);
    assert_eq!(clock2.counter, 0);

    // Test increment
    let new_clock1 = clock1.increment();
    assert_eq!(new_clock1.counter, 1);
    assert_eq!(clock1.counter, 1);

    // Test merge
    clock2.merge(&new_clock1);
    assert_eq!(clock2.counter, 1);

    // Test ordering
    assert!(new_clock1 > clock2);
    let newer_clock1 = clock1.increment();
    assert!(newer_clock1 > new_clock1);

    println!("✅ Lamport clock consistency test passed");
}