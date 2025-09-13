// Performance profiling module

use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ProfileResult {
    pub name:         String,
    pub duration:     Duration,
    pub calls:        u64,
    pub avg_duration: Duration,
    pub max_duration: Duration,
    pub min_duration: Duration,
}

pub struct PerformanceProfiler {
    profiles:        HashMap<String, Vec<Duration>>,
    active_profiles: HashMap<String, Instant>,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            profiles:        HashMap::new(),
            active_profiles: HashMap::new(),
        }
    }

    pub fn start_profiling(&mut self, name: &str) {
        self.active_profiles
            .insert(name.to_string(), Instant::now());
    }

    pub fn stop_profiling(&mut self, name: &str) -> Option<Duration> {
        if let Some(start_time) = self.active_profiles.remove(name) {
            let duration = start_time.elapsed();
            self.profiles
                .entry(name.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
            Some(duration)
        } else {
            None
        }
    }

    pub fn get_profile_results(&self) -> Vec<ProfileResult> {
        self.profiles
            .iter()
            .map(|(name, durations)| {
                if durations.is_empty() {
                    return ProfileResult {
                        name:         name.clone(),
                        duration:     Duration::new(0, 0),
                        calls:        0,
                        avg_duration: Duration::new(0, 0),
                        max_duration: Duration::new(0, 0),
                        min_duration: Duration::new(0, 0),
                    };
                }

                let total_duration = durations.iter().sum::<Duration>();
                let calls = durations.len() as u64;
                let avg_duration = total_duration / calls as u32;
                let max_duration = durations.iter().max().unwrap().clone();
                let min_duration = durations.iter().min().unwrap().clone();

                ProfileResult {
                    name: name.clone(),
                    duration: total_duration,
                    calls,
                    avg_duration,
                    max_duration,
                    min_duration,
                }
            })
            .collect()
    }

    pub fn clear_profiles(&mut self) {
        self.profiles.clear();
        self.active_profiles.clear();
    }

    pub fn get_call_count(&self, name: &str) -> Option<u64> {
        self.profiles.get(name).map(|calls| calls.len() as u64)
    }
}

// Scoped profiling helper
pub struct ScopedProfiler<'a> {
    profiler: &'a mut PerformanceProfiler,
    name:     String,
}

impl<'a> ScopedProfiler<'a> {
    pub fn new(profiler: &'a mut PerformanceProfiler, name: &str) -> Self {
        profiler.start_profiling(name);
        Self {
            profiler,
            name: name.to_string(),
        }
    }
}

impl<'a> Drop for ScopedProfiler<'a> {
    fn drop(&mut self) {
        self.profiler.stop_profiling(&self.name);
    }
}

// Memory profiling utilities
pub struct MemoryProfiler {
    allocation_count:     u64,
    deallocation_count:   u64,
    current_memory_usage: u64,
    peak_memory_usage:    u64,
}

impl MemoryProfiler {
    pub fn new() -> Self {
        Self {
            allocation_count:     0,
            deallocation_count:   0,
            current_memory_usage: 0,
            peak_memory_usage:    0,
        }
    }

    pub fn record_allocation(&mut self, size: u64) {
        self.allocation_count += 1;
        self.current_memory_usage += size;
        if self.current_memory_usage > self.peak_memory_usage {
            self.peak_memory_usage = self.current_memory_usage;
        }
    }

    pub fn record_deallocation(&mut self, size: u64) {
        self.deallocation_count += 1;
        self.current_memory_usage = self.current_memory_usage.saturating_sub(size);
    }

    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            allocation_count:     self.allocation_count,
            deallocation_count:   self.deallocation_count,
            current_memory_usage: self.current_memory_usage,
            peak_memory_usage:    self.peak_memory_usage,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub allocation_count:     u64,
    pub deallocation_count:   u64,
    pub current_memory_usage: u64,
    pub peak_memory_usage:    u64,
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;

    use super::*;

    #[test]
    fn test_performance_profiler() {
        let mut profiler = PerformanceProfiler::new();

        // Profile a simple operation
        profiler.start_profiling("test_op");
        sleep(Duration::from_millis(10));
        let duration = profiler.stop_profiling("test_op").unwrap();

        assert!(duration >= Duration::from_millis(10));
        assert_eq!(profiler.get_call_count("test_op"), Some(1));
    }

    #[test]
    fn test_scoped_profiler() {
        let mut profiler = PerformanceProfiler::new();

        {
            let _scoped = ScopedProfiler::new(&mut profiler, "scoped_test");
            sleep(Duration::from_millis(10));
        } // profiler stops automatically here

        assert_eq!(profiler.get_call_count("scoped_test"), Some(1));
        let results = profiler.get_profile_results();
        assert_eq!(results.len(), 1);
        assert!(results[0].avg_duration >= Duration::from_millis(10));
    }
}
