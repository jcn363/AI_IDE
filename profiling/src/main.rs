//! Performance Profiling Test Program
//!
//! This standalone program tests performance-critical code paths
//! identified in the audit report without requiring full workspace dependencies.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Simple cache implementation for testing
struct SimpleCache<K, V> {
    entries: HashMap<K, V>,
    max_size: usize,
    ttl_seconds: u64,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> SimpleCache<K, V> {
    fn new(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
            ttl_seconds,
        }
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key)
    }

    fn put(&mut self, key: K, value: V) {
        if self.entries.len() >= self.max_size {
            // Simple eviction: remove oldest entry
            if let Some(first_key) = self.entries.keys().next().cloned() {
                self.entries.remove(&first_key);
            }
        }
        self.entries.insert(key, value);
    }
}

/// Simulate AI inference processing with caching
async fn simulate_ai_inference(cache: Arc<Mutex<SimpleCache<String, String>>>, iterations: u32) -> Duration {
    let start = Instant::now();

    for i in 0..iterations {
        let key = format!("inference_{}", i);

        // Check cache first
        let cached_result = {
            let cache_guard = cache.lock().await;
            cache_guard.get(&key).cloned()
        };

        if let Some(result) = cached_result {
            // Cache hit - process cached result
            let _processed = result.to_uppercase();
        } else {
            // Cache miss - simulate inference work
            let prompt = format!("Analyze this code: fn test_{}() {{}}", i);
            let result = format!("Analysis of: {}", prompt);

            // Simulate network/API delay
            tokio::time::sleep(Duration::from_millis(5)).await;

            // Cache the result
            let mut cache_guard = cache.lock().await;
            cache_guard.put(key, result);
        }

        // Simulate additional processing
        let mut vec = Vec::with_capacity(1000);
        for j in 0..1000 {
            vec.push(j as u64);
        }
        let _sum: u64 = vec.iter().sum();
    }

    start.elapsed()
}

/// Simulate LSP symbol resolution across multiple "language servers"
async fn simulate_lsp_symbol_resolution(num_servers: usize, symbols_per_server: usize) -> Duration {
    let start = Instant::now();

    let mut all_symbols = Vec::new();

    // Simulate querying multiple language servers concurrently
    let mut handles = Vec::new();

    for server_id in 0..num_servers {
        let handle = tokio::spawn(async move {
            let mut server_symbols = Vec::new();

            for symbol_id in 0..symbols_per_server {
                let symbol_name = format!("server_{}_symbol_{}", server_id, symbol_id);
                let symbol_info = format!("Symbol: {} at line {}", symbol_name, symbol_id);

                // Simulate some processing
                let processed_info = symbol_info.to_lowercase();
                server_symbols.push(processed_info);

                // Simulate cross-language matching
                if symbol_name.contains("common") {
                    let cross_ref = format!("Cross-reference to: {}", symbol_name);
                    server_symbols.push(cross_ref);
                }
            }

            server_symbols
        });

        handles.push(handle);
    }

    // Collect results from all servers
    for handle in handles {
        if let Ok(symbols) = handle.await {
            all_symbols.extend(symbols);
        }
    }

    // Simulate result consolidation and ranking
    all_symbols.sort_by(|a, b| a.len().cmp(&b.len()));
    let _top_results: Vec<_> = all_symbols.into_iter().take(10).collect();

    start.elapsed()
}

/// Simulate security policy evaluation
fn simulate_security_policy_evaluation(num_requests: usize) -> Duration {
    let start = Instant::now();

    let mut policies = vec![
        "read_file".to_string(),
        "write_file".to_string(),
        "execute_command".to_string(),
        "network_access".to_string(),
    ];

    for _ in 0..num_requests {
        // Simulate RBAC policy evaluation
        for policy in &policies {
            let _allowed = match policy.as_str() {
                "read_file" => true,
                "write_file" => false, // Simulate restricted access
                "execute_command" => true,
                "network_access" => false,
                _ => false,
            };
        }

        // Simulate ABAC attribute evaluation
        let user_attributes = HashMap::from([
            ("role", "developer"),
            ("department", "engineering"),
            ("clearance", "confidential"),
        ]);

        for (attr, value) in &user_attributes {
            let _processed_attr = format!("{}:{}", attr, value);
        }

        // Simulate cryptographic operation (simplified)
        let data = b"some sensitive data";
        let _encrypted = data.iter().map(|b| b.wrapping_add(1)).collect::<Vec<_>>();
    }

    start.elapsed()
}

/// Simulate SIMD-accelerated operations
fn simulate_simd_operations(iterations: usize) -> Duration {
    let start = Instant::now();

    let mut results = Vec::with_capacity(iterations);

    for i in 0..iterations {
        // Simulate vectorized operations that could benefit from SIMD
        let mut sum = 0.0;
        let factors = [0.75, 0.8, 1.2, 1.1, 0.9, 0.7]; // Confidence factors

        for factor in &factors {
            sum += factor;
        }

        results.push(sum);
    }

    start.elapsed()
}

/// Main performance profiling function
#[tokio::main]
async fn main() {
    println!("🔬 Performance Profiling Test Program");
    println!("=====================================");

    // Test 1: AI Inference with Caching
    println!("\n1. Testing AI Inference Performance:");
    let cache = Arc::new(Mutex::new(SimpleCache::new(1000, 3600)));
    let inference_time = simulate_ai_inference(cache, 1000).await;
    println!("   AI Inference (1000 iterations): {:.2}ms", inference_time.as_millis());
    println!("   Ops/second: {:.2}", 1000.0 / inference_time.as_secs_f64());

    // Test 2: LSP Symbol Resolution
    println!("\n2. Testing LSP Symbol Resolution:");
    let lsp_time = simulate_lsp_symbol_resolution(4, 1000).await;
    println!("   LSP Symbol Resolution (4 servers × 1000 symbols): {:.2}ms", lsp_time.as_millis());
    println!("   Total symbols processed: {}", 4 * 1000);

    // Test 3: Security Policy Evaluation
    println!("\n3. Testing Security Policy Evaluation:");
    let security_time = simulate_security_policy_evaluation(10000);
    println!("   Security Policy Evaluation (10,000 requests): {:.2}ms", security_time.as_millis());
    println!("   Ops/second: {:.2}", 10000.0 / security_time.as_secs_f64());

    // Test 4: SIMD Operations
    println!("\n4. Testing SIMD-like Operations:");
    let simd_time = simulate_simd_operations(100000);
    println!("   SIMD Operations (100,000 iterations): {:.2}ms", simd_time.as_millis());
    println!("   Ops/second: {:.2}", 100000.0 / simd_time.as_secs_f64());

    println!("\n✅ Performance profiling completed!");
    println!("\n📊 Key Findings:");
    println!("   - AI Inference: {:.2} ops/sec", 1000.0 / inference_time.as_secs_f64());
    println!("   - Security Evaluation: {:.2} ops/sec", 10000.0 / security_time.as_secs_f64());
    println!("   - SIMD Operations: {:.2} ops/sec", 100000.0 / simd_time.as_secs_f64());
}