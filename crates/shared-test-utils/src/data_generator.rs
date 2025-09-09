//! Test data generation utilities

use rand::{rng, Rng};

/// Generate a random string of specified length
pub fn random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rng();

    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate random test data
pub fn random_vec(len: usize, min: i32, max: i32) -> Vec<i32> {
    let mut rng = rng();
    (0..len).map(|_| rng.random_range(min..=max)).collect()
}

/// Generate random file content for testing
pub fn random_file_content(size_kb: usize) -> Vec<u8> {
    let mut rng = rng();
    (0..(size_kb * 1024)).map(|_| rng.random()).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_random_generation() {
        use super::*;

        let s = random_string(10);
        assert_eq!(s.len(), 10);

        let v = random_vec(5, 1, 10);
        assert_eq!(v.len(), 5);
    }
}