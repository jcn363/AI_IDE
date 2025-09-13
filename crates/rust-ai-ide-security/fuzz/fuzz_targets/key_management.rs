#![no_main]
use libfuzzer_sys::fuzz_target;
use rust_ai_ide_security::key_management::{create_software_key_manager, KeyAlgorithm};
use tokio::runtime::Runtime;

// Fuzz target for key management operations
fuzz_target!(|data: &[u8]| {
    if data.len() < 16 {
        return;
    }

    // Create async runtime for testing async key operations
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let key_manager = create_software_key_manager().await;

        // Extract algorithm from input data
        let algorithm = match data[0] % 3 {
            0 => "aes256",
            1 => "chacha20",
            _ => "aes256-gcm",
        };

        let purpose = match data[1] % 5 {
            0 => "encryption",
            1 => "signing",
            2 => "authentication",
            3 => "key_exchange",
            _ => "general",
        };

        // Test key generation
        if let Ok(key_id) = key_manager.generate_key(purpose, algorithm).await {
            // Test encryption/decryption roundtrip
            let test_data = &data[2..];

            if let Ok(encrypted) = key_manager.encrypt_data(&key_id, test_data).await {
                if let Ok(decrypted) = key_manager.decrypt_data(&key_id, &encrypted).await {
                    // Verify roundtrip
                    assert_eq!(decrypted, test_data);
                } else {
                    panic!("Decryption failed for valid encrypted data");
                }
            }
            // Encryption failures are expected and not security issues

            // Test key rotation
            if let Ok(new_key_id) = key_manager.rotate_key(&key_id).await {
                // Verify old key still works (backwards compatibility)
                if let Ok(encrypted) = key_manager.encrypt_data(&key_id, test_data).await {
                    if let Ok(decrypted) = key_manager.decrypt_data(&key_id, &encrypted).await {
                        assert_eq!(decrypted, test_data);
                    }
                }

                // Test new key works
                if let Ok(encrypted) = key_manager.encrypt_data(&new_key_id, test_data).await {
                    if let Ok(decrypted) = key_manager.decrypt_data(&new_key_id, &encrypted).await {
                        assert_eq!(decrypted, test_data);
                    }
                }
            }
        }
        // Key generation failures are expected for invalid inputs
    });
});