/// ! Platform-specific utilities for cross-platform compatibility
/// Provides abstractions for Windows, macOS, and Linux-specific functionality

#[cfg(target_os = "windows")]
use std::os::windows::prelude::*;
#[cfg(target_os = "macos")]
use std::os::unix::prelude::*;
use std::path::{Path, PathBuf};

/// Platform-specific path separator constants
#[cfg(target_os = "windows")]
pub const PATH_SEPARATOR: char = '\\';
#[cfg(not(target_os = "windows"))]
pub const PATH_SEPARATOR: char = '/';

/// Platform-specific path separator as string
#[cfg(target_os = "windows")]
pub const PATH_SEPARATOR_STR: &str = "\\";
#[cfg(not(target_os = "windows"))]
pub const PATH_SEPARATOR_STR: &str = "/";

/// Get platform-specific application data directory
pub fn get_app_data_dir(app_name: &str) -> std::io::Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        use std::env;
        if let Ok(appdata) = env::var("APPDATA") {
            Ok(PathBuf::from(appdata).join(app_name))
        } else {
            Ok(PathBuf::from("C:\\Users\\Default\\AppData\\Roaming").join(app_name))
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::env;
        if let Ok(home) = env::var("HOME") {
            Ok(PathBuf::from(home).join("Library/Application Support").join(app_name))
        } else {
            Ok(PathBuf::from("/Library/Application Support").join(app_name))
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::env;
        if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
            Ok(PathBuf::from(xdg_data_home).join(app_name))
        } else if let Ok(home) = env::var("HOME") {
            Ok(PathBuf::from(home).join(".local/share").join(app_name))
        } else {
            Ok(PathBuf::from("/usr/local/share").join(app_name))
        }
    }
}

/// Get platform-specific user home directory
pub fn get_user_home_dir() -> std::io::Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        use std::env;
        if let Ok(userprofile) = env::var("USERPROFILE") {
            Ok(PathBuf::from(userprofile))
        } else {
            Ok(PathBuf::from("C:\\Users\\Default"))
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        use std::env;
        if let Ok(home) = env::var("HOME") {
            Ok(PathBuf::from(home))
        } else {
            Ok(PathBuf::from("/tmp"))
        }
    }
}

/// Get platform-specific temporary directory
pub fn get_temp_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        use std::env;
        if let Ok(temp) = env::var("TEMP") {
            PathBuf::from(temp)
        } else if let Ok(tmp) = env::var("TMP") {
            PathBuf::from(tmp)
        } else {
            PathBuf::from("C:\\Windows\\Temp")
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        use std::env;
        if let Ok(tmpdir) = env::var("TMPDIR") {
            PathBuf::from(tmpdir)
        } else {
            PathBuf::from("/tmp")
        }
    }
}

/// Get platform-specific cache directory
pub fn get_cache_dir(app_name: &str) -> std::io::Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        // On Windows, cache typically goes in app data
        get_app_data_dir(app_name)
    }

    #[cfg(target_os = "macos")]
    {
        use std::env;
        if let Ok(home) = env::var("HOME") {
            Ok(PathBuf::from(home).join("Library/Caches").join(app_name))
        } else {
            Ok(PathBuf::from("/Library/Caches").join(app_name))
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::env;
        if let Ok(xdg_cache_home) = env::var("XDG_CACHE_HOME") {
            Ok(PathBuf::from(xdg_cache_home).join(app_name))
        } else if let Ok(home) = env::var("HOME") {
            Ok(PathBuf::from(home).join(".cache").join(app_name))
        } else {
            Ok(PathBuf::from("/tmp/cache").join(app_name))
        }
    }
}

/// Convert path separators to platform-specific format
pub fn normalize_path_separators<P: AsRef<Path>>(path: P) -> PathBuf {
    let path_str = path.as_ref().to_string_lossy();

    #[cfg(target_os = "windows")]
    {
        // On Windows, convert forward slashes to backslashes
        PathBuf::from(path_str.replace('/', "\\"))
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On Unix-like systems, convert backslashes to forward slashes
        PathBuf::from(path_str.replace('\\', "/"))
    }
}

/// Check if path is valid for the current platform
pub fn is_valid_path<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    #[cfg(target_os = "windows")]
    {
        // Windows-specific validation
        if let Some(s) = path.to_str() {
            // Check for invalid characters on Windows
            !s.chars().any(|c| matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
        } else {
            false
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        // Unix-like systems are more permissive
        path.to_str().is_some()
    }
}

/// Get platform-specific executable extension
pub fn get_executable_extension() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        ".exe"
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        ""
    }
}

/// Platform-specific memory management hints
pub struct PlatformMemoryHints;

impl PlatformMemoryHints {
    /// Get recommended memory allocation size for the platform
    pub fn recommended_allocation_size() -> usize {
        #[cfg(target_os = "windows")]
        {
            // Windows prefers larger allocations to reduce fragmentation
            64 * 1024 // 64KB
        }

        #[cfg(target_os = "macos")]
        {
            // macOS APFS is optimized for smaller allocations
            16 * 1024 // 16KB
        }

        #[cfg(target_os = "linux")]
        {
            // Linux ext4 is efficient with page-sized allocations
            4 * 1024 // 4KB
        }
    }

    /// Get platform-specific memory alignment
    pub fn memory_alignment() -> usize {
        #[cfg(target_os = "windows")]
        {
            8 // 8-byte alignment typical for Windows
        }

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            16 // 16-byte alignment for better SIMD performance
        }
    }
}

/// Platform-specific file system features
pub struct PlatformFileSystem;

impl PlatformFileSystem {
    /// Check if the file system supports case-sensitive operations
    pub fn is_case_sensitive() -> bool {
        #[cfg(target_os = "windows")]
        {
            false // Windows file systems are case-insensitive by default
        }

        #[cfg(target_os = "macos")]
        {
            true // macOS APFS is case-sensitive (though case-insensitive is common)
        }

        #[cfg(target_os = "linux")]
        {
            true // Most Linux file systems are case-sensitive
        }
    }

    /// Get optimal file buffer size for the platform
    pub fn optimal_buffer_size() -> usize {
        #[cfg(target_os = "windows")]
        {
            64 * 1024 // 64KB - Windows NTFS optimal
        }

        #[cfg(target_os = "macos")]
        {
            256 * 1024 // 256KB - macOS APFS optimal
        }

        #[cfg(target_os = "linux")]
        {
            128 * 1024 // 128KB - Linux ext4 optimal
        }
    }

    /// Check if the platform supports file permissions
    pub fn supports_file_permissions() -> bool {
        #[cfg(target_os = "windows")]
        {
            true // Windows NTFS supports permissions
        }

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            true // Unix-like systems always support permissions
        }
    }
}

/// Platform detection utilities
pub struct Platform;

impl Platform {
    /// Get the current platform as a string
    pub fn name() -> &'static str {
        #[cfg(target_os = "windows")]
        {
            "windows"
        }

        #[cfg(target_os = "macos")]
        {
            "macos"
        }

        #[cfg(target_os = "linux")]
        {
            "linux"
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            "unknown"
        }
    }

    /// Check if running on Windows
    pub fn is_windows() -> bool {
        cfg!(target_os = "windows")
    }

    /// Check if running on macOS
    pub fn is_macos() -> bool {
        cfg!(target_os = "macos")
    }

    /// Check if running on Linux
    pub fn is_linux() -> bool {
        cfg!(target_os = "linux")
    }
}