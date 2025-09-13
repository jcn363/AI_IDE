#![no_main]
use libfuzzer_sys::fuzz_target;
use rust_ai_ide_security::encryption::{CryptoOps, EncryptionAlgorithm};

// Fuzz target for AES-256-GCM encryption/decryption operations
fuzz_target!(|data: &[u8]| {
    // Limit input size to prevent excessive memory usage
    if data.len() < 32 || data.len() > 65536 {
        return;
    }

    let crypto_ops = CryptoOps::new(EncryptionAlgorithm::Aes256Gcm);

    // Generate a key from the input data
    let mut key_bytes = [0u8; 32];
    let key_len = std::cmp::min(data.len(), 32);
    key_bytes[..key_len].copy_from_slice(&data[..key_len]);

    // Test encryption/decryption roundtrip
    if let Ok((ciphertext, nonce)) = crypto_ops.encrypt(&data[32..], &key_bytes, None) {
        // Test decryption
        if let Ok(decrypted) = crypto_ops.decrypt(&ciphertext, &key_bytes, &nonce, None) {
            // Verify roundtrip succeeds
            assert_eq!(decrypted, &data[32..]);
        } else {
            // If decryption fails, this is a potential security issue
            panic!("Decryption failed for valid ciphertext");
        }
    }
    // Encryption failures are expected for malformed inputs and are not security issues
});
