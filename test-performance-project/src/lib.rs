//! Test performance project library

use std::time::Duration;

/// A function that does some work to test performance
pub fn do_some_work(iterations: u32) -> u64 {
    let mut result = 0u64;
    
    // Simulate some CPU-bound work
    for i in 0..iterations {
        // Simple hash-like operation
        let x = i as u64 * 2654435761 % (1 << 31);
        result = result.wrapping_add(x);
        
        // Simulate some memory allocation
        let mut vec = Vec::with_capacity(1000);
        for j in 0..1000 {
            vec.push(j as u64);
        }
        
        // Use the vector to prevent optimization
        result = result.wrapping_add(vec[vec.len() - 1]);
    }
    
    result
}

/// An async function that does some I/O-bound work
pub async fn do_async_work(iterations: u32) -> u64 {
    let mut result = 0u64;
    
    for i in 0..iterations {
        // Simulate async I/O
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Some computation
        let x = i as u64 * 11400714819323198549u64;
        result = result.wrapping_add(x);
    }
    
    result
}
