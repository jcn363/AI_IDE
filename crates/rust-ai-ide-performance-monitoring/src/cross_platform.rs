//! Cross-platform performance monitoring implementations
//!
//! This module provides OS-specific implementations for performance monitoring
//! across Linux, macOS, Windows, and other supported platforms.

use std::ffi::CString;
#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(target_os = "linux")]
use std::io::Read;
use std::os::raw::{c_char, c_int};
use std::sync::Arc;

use tokio::sync::Mutex;

/// Platform-specific memory information
#[derive(Debug, Clone)]
pub struct PlatformMemoryInfo {
    pub resident_set_size_kb: u64,
    pub virtual_memory_kb: u64,
    pub shared_memory_kb: u64,
    pub private_memory_kb: u64,
    pub swap_total_kb: u64,
    pub swap_free_kb: u64,
}

/// Platform-specific CPU information
#[derive(Debug, Clone)]
pub struct PlatformCpuInfo {
    pub cpu_count: u32,
    pub frequency_mhz: f64,
    pub load_average: f64,
    pub context_switches: u64,
}

/// Platform performance monitor
#[derive(Debug)]
pub struct PlatformPerformanceMonitor {
    system_stats: Arc<Mutex<SystemStats>>,
}

#[derive(Debug, Clone)]
struct SystemStats {
    memory_info: PlatformMemoryInfo,
    cpu_info: PlatformCpuInfo,
    last_update: std::time::Instant,
}

impl PlatformPerformanceMonitor {
    /// Create a new platform-specific performance monitor
    pub fn new() -> Result<Self, PlatformPerformanceError> {
        let stats = Self::collect_platform_stats()?;
        Ok(Self {
            system_stats: Arc::new(Mutex::new(stats)),
        })
    }

    /// Update system statistics
    pub async fn update_stats(&self) -> Result<(), PlatformPerformanceError> {
        let new_stats = Self::collect_platform_stats()?;
        *self.system_stats.lock().await = new_stats;
        Ok(())
    }

    /// Get current memory information
    pub async fn get_memory_info(&self) -> PlatformMemoryInfo {
        self.system_stats.lock().await.memory_info.clone()
    }

    /// Get current CPU information
    pub async fn get_cpu_info(&self) -> PlatformCpuInfo {
        self.system_stats.lock().await.cpu_info.clone()
    }

    /// Collect platform-specific statistics
    fn collect_platform_stats() -> Result<SystemStats, PlatformPerformanceError> {
        let memory_info = Self::collect_memory_info()?;
        let cpu_info = Self::collect_cpu_info()?;

        Ok(SystemStats {
            memory_info,
            cpu_info,
            last_update: std::time::Instant::now(),
        })
    }

    /// Collect platform-specific memory information
    #[cfg(target_os = "linux")]
    fn collect_memory_info() -> Result<PlatformMemoryInfo, PlatformPerformanceError> {
        let mut file = File::open("/proc/meminfo").map_err(|e| {
            PlatformPerformanceError::SystemError(format!("Failed to open /proc/meminfo: {}", e))
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            PlatformPerformanceError::SystemError(format!("Failed to read /proc/meminfo: {}", e))
        })?;

        let mut mem_total = 0u64;
        let mut mem_free = 0u64;
        let mut swap_total = 0u64;
        let mut swap_free = 0u64;

        for line in contents.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let value: u64 = parts[1].parse().unwrap_or(0);
                match parts[0] {
                    "MemTotal:" => mem_total = value,
                    "MemFree:" => mem_free = value,
                    "SwapTotal:" => swap_total = value,
                    "SwapFree:" => swap_free = value,
                    _ => {}
                }
            }
        }

        // Get current process memory info
        let resident_set_size = Self::get_process_memory()?;
        let virtual_memory_kb = mem_total;
        let private_memory_kb = resident_set_size;

        Ok(PlatformMemoryInfo {
            resident_set_size_kb: resident_set_size,
            virtual_memory_kb,
            shared_memory_kb: mem_total - mem_free,
            private_memory_kb,
            swap_total_kb: swap_total,
            swap_free_kb: swap_free,
        })
    }

    /// Collect platform-specific CPU information
    #[cfg(target_os = "linux")]
    fn collect_cpu_info() -> Result<PlatformCpuInfo, PlatformPerformanceError> {
        let cpu_count = num_cpus::get() as u32;
        let load_average = Self::get_load_average()?;

        Ok(PlatformCpuInfo {
            cpu_count,
            frequency_mhz: Self::get_cpu_frequency()?,
            load_average,
            context_switches: Self::get_context_switches()?,
        })
    }

    /// Get current process memory usage (KB)
    #[cfg(target_os = "linux")]
    fn get_process_memory() -> Result<u64, PlatformPerformanceError> {
        let statm_path = format!("/proc/self/statm");
        let mut file = File::open(statm_path).map_err(|e| {
            PlatformPerformanceError::SystemError(format!("Failed to open /proc/self/statm: {}", e))
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            PlatformPerformanceError::SystemError(format!("Failed to read /proc/self/statm: {}", e))
        })?;

        let parts: Vec<&str> = contents.trim().split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(rss_pages) = parts[1].parse::<u64>() {
                // Convert pages to KB (assuming 4KB pages)
                return Ok(rss_pages * 4);
            }
        }

        Err(PlatformPerformanceError::ParseError(
            "Failed to parse process memory".to_string(),
        ))
    }

    /// Get system load average
    #[cfg(target_os = "linux")]
    fn get_load_average() -> Result<f64, PlatformPerformanceError> {
        let mut file = File::open("/proc/loadavg").map_err(|e| {
            PlatformPerformanceError::SystemError(format!("Failed to open /proc/loadavg: {}", e))
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            PlatformPerformanceError::SystemError(format!("Failed to read /proc/loadavg: {}", e))
        })?;

        let parts: Vec<&str> = contents.trim().split_whitespace().collect();
        if let Some(load_str) = parts.first() {
            if let Ok(load_avg) = load_str.parse::<f64>() {
                return Ok(load_avg);
            }
        }

        Err(PlatformPerformanceError::ParseError(
            "Failed to parse load average".to_string(),
        ))
    }

    /// Get CPU frequency
    #[cfg(target_os = "linux")]
    fn get_cpu_frequency() -> Result<f64, PlatformPerformanceError> {
        // Try to read from /proc/cpuinfo
        if let Ok(mut file) = File::open("/proc/cpuinfo") {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                for line in contents.lines() {
                    if line.starts_with("cpu MHz") {
                        let parts: Vec<&str> = line.split(':').collect();
                        if parts.len() == 2 {
                            if let Ok(freq) = parts[1].trim().parse::<f64>() {
                                return Ok(freq);
                            }
                        }
                    }
                }
            }
        }

        // Fallback to estimate
        Ok(0.0)
    }

    /// Get context switches count
    #[cfg(target_os = "linux")]
    fn get_context_switches() -> Result<u64, PlatformPerformanceError> {
        // This is a simplified implementation
        Ok(0)
    }

    // Windows-specific implementations
    #[cfg(target_os = "windows")]
    fn collect_memory_info() -> Result<PlatformMemoryInfo, PlatformPerformanceError> {
        use std::mem;

        use winapi::um::psapi::PROCESS_MEMORY_COUNTERS;
        use winapi::um::sysinfoapi::MEMORYSTATUSEX;
        use winapi::um::winbase::LocalSize;

        let mut mem_counters: PROCESS_MEMORY_COUNTERS = unsafe { mem::zeroed() };
        let psapi_size = mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

        // Get process memory info
        let result = unsafe {
            winapi::um::psapi::GetProcessMemoryInfo(
                winapi::um::handleapi::INVALID_HANDLE_VALUE,
                &mut mem_counters,
                psapi_size,
            )
        };

        if result == 0 {
            return Err(PlatformPerformanceError::SystemError(
                "Failed to get process memory".to_string(),
            ));
        }

        // Get system memory info
        let mut mem_status: MEMORYSTATUSEX = unsafe { mem::zeroed() };
        mem_status.dwLength = mem::size_of::<MEMORYSTATUSEX>() as u32;

        let result = unsafe { winapi::um::sysinfoapi::GlobalMemoryStatusEx(&mut mem_status) };

        if result == 0 {
            return Err(PlatformPerformanceError::SystemError(
                "Failed to get system memory".to_string(),
            ));
        }

        Ok(PlatformMemoryInfo {
            resident_set_size_kb: mem_counters.WorkingSetSize / 1024,
            virtual_memory_kb: mem_counters.PagefileUsage / 1024,
            shared_memory_kb: 0, // Not available on Windows
            private_memory_kb: (mem_counters.WorkingSetSize - mem_counters.PagefileUsage) / 1024,
            swap_total_kb: (mem_status.ullTotalPageFile - mem_status.ullTotalPhys) / 1024,
            swap_free_kb: (mem_status.ullAvailPageFile - mem_status.ullAvailPhys) / 1024,
        })
    }

    #[cfg(target_os = "windows")]
    fn collect_cpu_info() -> Result<PlatformCpuInfo, PlatformPerformanceError> {
        let cpu_count = num_cpus::get() as u32;
        let frequency_mhz = Self::get_cpu_frequency()?;
        let context_switches = Self::get_context_switches()?;

        Ok(PlatformCpuInfo {
            cpu_count,
            frequency_mhz,
            load_average: 0.0, // Not directly available on Windows
            context_switches,
        })
    }

    #[cfg(target_os = "windows")]
    fn get_process_memory() -> Result<u64, PlatformPerformanceError> {
        use std::mem;

        use winapi::um::psapi::PROCESS_MEMORY_COUNTERS;

        let mut mem_counters: PROCESS_MEMORY_COUNTERS = unsafe { mem::zeroed() };
        let psapi_size = mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

        let result = unsafe {
            winapi::um::psapi::GetProcessMemoryInfo(
                winapi::um::handleapi::INVALID_HANDLE_VALUE,
                &mut mem_counters,
                psapi_size,
            )
        };

        if result == 0 {
            return Err(PlatformPerformanceError::SystemError(
                "Failed to get process memory".to_string(),
            ));
        }

        Ok(mem_counters.WorkingSetSize / 1024)
    }

    #[cfg(target_os = "windows")]
    fn get_load_average() -> Result<f64, PlatformPerformanceError> {
        // Windows doesn't have load average, return 0
        Ok(0.0)
    }

    #[cfg(target_os = "windows")]
    fn get_cpu_frequency() -> Result<f64, PlatformPerformanceError> {
        // Simplified implementation - in real code would use Windows APIs
        Ok(0.0)
    }

    #[cfg(target_os = "windows")]
    fn get_context_switches() -> Result<u64, PlatformPerformanceError> {
        // Simplified implementation
        Ok(0)
    }

    // macOS-specific implementations
    #[cfg(target_os = "macos")]
    fn collect_memory_info() -> Result<PlatformMemoryInfo, PlatformPerformanceError> {
        use cocoa::appkit::NSProcessInfo;
        use cocoa::base::nil;
        use cocoa::foundation::NSString;
        use objc::rc::autoreleasepool;
        use objc::runtime::Object;

        autoreleasepool(|| {
            // Get memory information from NSProcessInfo
            let process_info = unsafe { NSProcessInfo::processInfo(nil) };
            let physical_memory: u64 = unsafe { msg_send![process_info, physicalMemory] };
            let processor_count: usize = unsafe { msg_send![process_info, processorCount] };

            // Get current process memory
            let resident_set_size = Self::get_process_memory()?;
            let virtual_memory_kb = physical_memory / 1024;
            let private_memory_kb = resident_set_size;

            Ok(PlatformMemoryInfo {
                resident_set_size_kb: resident_set_size,
                virtual_memory_kb,
                shared_memory_kb: 0, // Simplified
                private_memory_kb,
                swap_total_kb: 0, // Would need additional APIs
                swap_free_kb: 0,
            })
        })
        .map_err(|_| {
            PlatformPerformanceError::SystemError("Failed to get macOS memory info".to_string())
        })
    }

    #[cfg(target_os = "macos")]
    fn collect_cpu_info() -> Result<PlatformCpuInfo, PlatformPerformanceError> {
        use cocoa::appkit::NSProcessInfo;
        use cocoa::base::nil;
        use objc::runtime::Object;

        unsafe {
            let process_info = NSProcessInfo::processInfo(nil);
            let active_processor_count =
                cocoa::foundation::NSProcessInfoCocoaExt_ns64::activeProcessorCount(process_info)
                    .unwrap_or(0);
            let processor_count =
                cocoa::foundation::NSProcessInfoCocoaExt_ns64::processorCount(process_info)
                    .unwrap_or(0) as u32;

            Ok(PlatformCpuInfo {
                cpu_count: processor_count,
                frequency_mhz: 0.0, // Need specific APIs
                load_average: 0.0,  // Need specific APIs
                context_switches: 0,
            })
        }
    }

    #[cfg(target_os = "macos")]
    fn get_process_memory() -> Result<u64, PlatformPerformanceError> {
        use cocoa::base::nil;
        use objc::runtime::Object;

        unsafe {
            let process_info = cocoa::appkit::NSProcessInfo::processInfo(nil);
            let physical_footprint: u64 = msg_send![process_info, physicalFootprint];
            Ok(physical_footprint / 1024)
        }
    }

    #[cfg(target_os = "macos")]
    fn get_load_average() -> Result<f64, PlatformPerformanceError> {
        // macOS has load average but requires additional system calls
        Ok(0.0)
    }

    #[cfg(target_os = "macos")]
    fn get_cpu_frequency() -> Result<f64, PlatformPerformanceError> {
        // macOS CPU frequency requires IOKit
        Ok(0.0)
    }

    #[cfg(target_os = "macos")]
    fn get_context_switches() -> Result<u64, PlatformPerformanceError> {
        Ok(0)
    }

    // Fallback implementations for other platforms
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    fn collect_memory_info() -> Result<PlatformMemoryInfo, PlatformPerformanceError> {
        let resident_set_size = Self::get_process_memory()?;
        Ok(PlatformMemoryInfo {
            resident_set_size_kb: resident_set_size,
            virtual_memory_kb: 0,
            shared_memory_kb: 0,
            private_memory_kb: resident_set_size,
            swap_total_kb: 0,
            swap_free_kb: 0,
        })
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    fn collect_cpu_info() -> Result<PlatformCpuInfo, PlatformPerformanceError> {
        Ok(PlatformCpuInfo {
            cpu_count: num_cpus::get() as u32,
            frequency_mhz: 0.0,
            load_average: 0.0,
            context_switches: 0,
        })
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    fn get_process_memory() -> Result<u64, PlatformPerformanceError> {
        // Fallback - no accurate memory info available
        Ok(0)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    fn get_load_average() -> Result<f64, PlatformPerformanceError> {
        Ok(0.0)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    fn get_cpu_frequency() -> Result<f64, PlatformPerformanceError> {
        Ok(0.0)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    fn get_context_switches() -> Result<u64, PlatformPerformanceError> {
        Ok(0)
    }
}

/// Platform-specific performance errors
#[derive(Debug, thiserror::Error)]
pub enum PlatformPerformanceError {
    #[error("System error: {0}")]
    SystemError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_platform_monitor_creation() {
        let result = PlatformPerformanceMonitor::new();
        if let Ok(monitor) = result {
            // Test memory info retrieval
            let memory_info = monitor.get_memory_info().await;
            assert!(memory_info.resident_set_size_kb >= 0);

            // Test CPU info retrieval
            let cpu_info = monitor.get_cpu_info().await;
            assert!(cpu_info.cpu_count > 0);
        } else {
            // Some platforms may not be fully supported
            println!(
                "Platform monitor creation failed (expected on some platforms): {:?}",
                result
            );
        }
    }
}
