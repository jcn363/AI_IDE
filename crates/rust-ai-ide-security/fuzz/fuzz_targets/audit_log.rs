#![no_main]
use std::sync::Arc;

use libfuzzer_sys::fuzz_target;
use rust_ai_ide_config::audit::{AuditConfig, AuditTrail, HashAlgorithm};
use tokio::runtime::Runtime;

// Fuzz target for audit logging cryptographic operations
fuzz_target!(|data: &[u8]| {
    if data.len() < 32 {
        return;
    }

    // Create async runtime
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        // Create audit trail with encryption enabled
        let config = AuditConfig {
            max_entries:        1000,
            encryption_enabled: true,
            hash_algorithm:     HashAlgorithm::Sha256,
        };

        match AuditTrail::new_with_config(config).await {
            Ok(trail) => {
                let trail = Arc::new(trail);

                // Generate test data from fuzz input
                let user_id = format!("user_{}", data[0]);
                let client_ip = format!("192.168.1.{}", data[1]);
                let query = String::from_utf8_lossy(&data[2..]);

                // Test audit event logging (this involves cryptographic operations)
                let event_data = format!(
                    "Test audit event: {} from {} executing: {}",
                    user_id, client_ip, query
                );

                // Log the event (this exercises encryption and hashing)
                if let Ok(event_id) = trail.log_event("test_operation", &event_data).await {
                    // Test event retrieval (decrypts and verifies integrity)
                    if let Ok(Some(retrieved)) = trail.get_event(&event_id).await {
                        // Verify the event data integrity
                        if retrieved.message != event_data {
                            panic!("Audit log integrity check failed - data corruption detected");
                        }
                    } else {
                        panic!("Failed to retrieve logged audit event");
                    }

                    // Test tamper detection
                    if let Ok(entries) = trail.list_events(0, 10).await {
                        for entry in entries {
                            // Verify hash chain integrity
                            if !trail.verify_integrity(&entry.id).await.unwrap_or(false) {
                                panic!("Audit log tamper detection failed");
                            }
                        }
                    }
                }
                // Logging failures are expected for certain inputs
            }
            Err(_) => {
                // Configuration failures are expected and not security issues
            }
        }
    });
});
