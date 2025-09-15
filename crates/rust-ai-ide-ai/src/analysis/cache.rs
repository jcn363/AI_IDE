use anyhow::{Result, Context, anyhow};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Duration, UNIX_EPOCH};
use std::hash::{Hash, Hasher, DefaultHasher};
use std::fs;
use log::{debug, warn, trace};
use filetime::FileTime;
use std::sync::RwLock;
use lazy_static::lazy_static;

lazy_static! {
    static ref CACHE_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);
    static ref CACHE_TTL: RwLock<Option<Duration>> = RwLock::new(Some(Duration::from_secs(3600))); // Default 1 hour TTL
}

start_line:19
-------
/// Initialize the cache system
pub fn init_cache(cache_dir: Option<PathBuf>, default_ttl: Option<Duration>) -> Result<()> {
    let cache_dir = match cache_dir {
        Some(dir) => dir,
        None => dirs::cache_dir()
            .ok_or_else(|| anyhow!("Could not determine cache directory"))?
            .join("rust-ai-ide/analysis"),
    };

    // Create cache directory if it doesn't exist
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }

    let mut cache_dir_lock = CACHE_DIR.write()
        .map_err(|_| anyhow!("Failed to acquire write lock for cache directory during initialization"))?;
    *cache_dir_lock = Some(cache_dir);

    if let Some(ttl) = default_ttl {
        let mut cache_ttl_lock = CACHE_TTL.write()
            .map_err(|_| anyhow!("Failed to acquire write lock for cache TTL during initialization"))?;
        *cache_ttl_lock = Some(ttl);
    }

    let cache_dir_read = CACHE_DIR.read()
        .map_err(|_| anyhow!("Failed to acquire read lock for cache directory during initialization"))?;
    debug!("Cache initialized at: {:?}", cache_dir_read);
    Ok(())
}

/// Key for cache lookups with content-based hashing for incremental analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheKey {
    pub(super) file_path: PathBuf,
    pub(super) key: String,
    pub(super) version: u32,
    pub(super) content_hash: u64,
    pub(super) config_hash: u64,
}

impl CacheKey {
    pub fn new(file_path: &Path, key: &str, version: u32, content: &str, config: &impl Serialize) -> Result<Self> {
        let content_hash = Self::calculate_hash(content);
        let config_hash = Self::calculate_hash(&serde_json::to_string(config)?);

        Ok(Self {
            file_path: file_path.to_path_buf(),
            key: key.to_string(),
            version,
            content_hash,
            config_hash,
        })
    }

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    pub fn cache_file_path(&self) -> Result<PathBuf> {
        let cache_dir = CACHE_DIR.read()
            .map_err(|_| anyhow!("Cache not initialized"))?
            .as_ref()
            .ok_or_else(|| anyhow!("Cache directory not set"))?;

        let file_name = format!(
            "{}_{:x}_{:x}_{}.json",
            self.file_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown"),
            self.version,
            self.content_hash,
            self.key
        );

        Ok(cache_dir.join(file_name))
    }

    /// Get a reference to the key
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the content hash
    pub fn content_hash(&self) -> u64 {
        self.content_hash
    }

    /// Get the config hash
    pub fn config_hash(&self) -> u64 {
        self.config_hash
    }

    /// Get the file path
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Get the version
    pub fn version(&self) -> u32 {
        self.version
    }

start_line:116
-------
    pub fn is_stale(&self) -> bool {
        match self.cache_file_path() {
            Ok(path) => {
                let metadata = fs::metadata(path).ok();
                let ttl_result = CACHE_TTL.read();
                match ttl_result {
                    Ok(ttl_lock) => {
                        if let (Some(ttl), Some(modified)) = (*ttl_lock, metadata.and_then(|m| m.modified().ok())) {
                            if let Ok(elapsed) = modified.elapsed() {
                                return elapsed > ttl;
                            }
                        }
                        false
                    }
                    Err(_) => {
                        // Lock poisoned or unavailable - consider stale to be safe
                        debug!("Cache TTL lock unavailable, considering entry stale");
                        true
                    }
                }
            }
            Err(_) => true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    data: T,
    expires_at: u64,
    created_at: u64,
    version: u32,
    metadata: serde_json::Value,
}

/// Get a value from cache if it exists, is not expired, and not stale
pub fn get_cached<T: DeserializeOwned>(key: &CacheKey) -> Result<Option<T>> {
    if key.is_stale() {
        debug!("Cache miss: entry is stale for key: {}", key.key);
        return Ok(None);
    }

    let cache_file = key.cache_file_path()?;
    if !cache_file.exists() {
        trace!("Cache miss: file does not exist: {:?}", cache_file);
        return Ok(None);
    }

    let content = fs::read_to_string(&cache_file)
        .with_context(|| format!("Failed to read cache file: {:?}", cache_file))?;

    let entry: CacheEntry<T> = serde_json::from_str(&content)
        .with_context(|| format!("Failed to deserialize cache entry: {:?}", cache_file))?;

    // Check if entry is expired
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("System time is before UNIX EPOCH"))?
        .as_secs();

    if now > entry.expires_at {
        debug!("Cache miss: entry expired for key: {}", key.key);
        // Entry expired, clean it up
        let _ = fs::remove_file(cache_file);
        return Ok(None);
    }

    // Update last accessed time
    if let Err(e) = update_access_time(&cache_file) {
        warn!("Failed to update access time for {:?}: {}", cache_file, e);
    }

    debug!("Cache hit for key: {}", key.key);
    Ok(Some(entry.data))
}

fn update_access_time(path: &Path) -> Result<()> {
    let now = FileTime::now();
    filetime::set_file_handle_times(path, Some(now), Some(now))?;
    Ok(())
}

start_line:188
-------
/// Set a value in the cache with TTL
pub fn set_cached<T: Serialize>(
    key: &CacheKey,
    value: &T,
    ttl: Option<Duration>,
    metadata: Option<serde_json::Value>,
) -> Result<()> {
    let cache_file = key.cache_file_path()?;
    if let Some(parent) = cache_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let ttl = match ttl {
        Some(t) => Some(t),
        None => {
            let ttl_read = CACHE_TTL.read()
                .map_err(|_| anyhow!("Failed to acquire read lock for cache TTL"))?;
            ttl_read.clone()
        }
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("System time is before UNIX EPOCH"))?;

    let expires_at = ttl.map(|ttl| now + ttl).unwrap_or_else(|| now + Duration::from_secs(31536000)); // Default to 1 year if no TTL

    let entry = CacheEntry {
        data: value,
        expires_at: expires_at.as_secs(),
        created_at: now.as_secs(),
        version: key.version,
        metadata: metadata.unwrap_or_else(|| serde_json::json!({})),
    };

    let content = serde_json::to_vec_pretty(&entry)?;

    // Write to temp file first, then rename atomically
    let temp_file = cache_file.with_extension(".tmp");
    fs::write(&temp_file, &content)?;
    fs::rename(&temp_file, &cache_file)
        .or_else(|e| {
            // On Windows, if the target exists, we need to remove it first
            if cfg!(windows) && e.kind() == std::io::ErrorKind::PermissionDenied {
                fs::remove_file(&cache_file)?;
                fs::rename(&temp_file, &cache_file)
            } else {
                Err(e)
            }
        })
        .with_context(|| format!("Failed to write cache file: {:?}", cache_file))?;

    debug!("Cached result for key: {} (TTL: {}s)",
        key.key,
        ttl.map(|t| t.as_secs().to_string())
           .unwrap_or_else(|| "∞".to_string())
    );

    Ok(())
}

/// Invalidate all cache entries for a file or pattern
pub fn invalidate_cache(pattern: &str) -> Result<usize> {
    let cache_dir = CACHE_DIR.read()
        .map_err(|_| anyhow!("Cache not initialized"))?
        .clone()
        .ok_or_else(|| anyhow!("Cache directory not set"))?;

    if !cache_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;

    for entry in fs::read_dir(&cache_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name() {
            if let Some(name_str) = name.to_str() {
                if name_str.contains(pattern) {
                    if let Err(e) = fs::remove_file(&path) {
                        warn!("Failed to remove cache file {}: {}", path.display(), e);
                    } else {
                        count += 1;
                        debug!("Invalidated cache: {}", path.display());
                    }
                }
            }
        }
    }

    debug!("Invalidated {} cache entries matching: {}", count, pattern);
    Ok(count)
}

/// Clean up expired cache entries
pub fn cleanup_expired() -> Result<usize> {
    let cache_dir = CACHE_DIR.read()
        .map_err(|_| anyhow!("Cache not initialized"))?
        .clone()
        .ok_or_else(|| anyhow!("Cache directory not set"))?;

    if !cache_dir.exists() {
        return Ok(0);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("System time is before UNIX EPOCH"))?;

    let mut count = 0;

    for entry in fs::read_dir(&cache_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        // Skip if we can't read the file (might be locked by another process)
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                debug!("Skipping unreadable cache file {}: {}", path.display(), e);
                continue;
            }
        };

        // Try to parse the entry, skip if invalid
        if let Ok(entry) = serde_json::from_str::<CacheEntry<serde_json::Value>>(&content) {
            if (entry.expires_at as u128) < now.as_secs() as u128 {
                if let Err(e) = fs::remove_file(&path) {
                    warn!("Failed to remove expired cache file {}: {}", path.display(), e);
                } else {
                    count += 1;
                    debug!("Cleaned up expired cache: {}", path.display());
                }
            }
        }
    }

    debug!("Cleaned up {} expired cache entries", count);
    Ok(count)
}

/// Get cache statistics
pub fn get_stats() -> Result<CacheStats> {
    let cache_dir = CACHE_DIR.read()
        .map_err(|_| anyhow!("Cache not initialized"))?
        .clone()
        .ok_or_else(|| anyhow!("Cache directory not set"))?;

    if !cache_dir.exists() {
        return Ok(CacheStats::default());
    }

    let mut stats = CacheStats::default();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("System time is before UNIX EPOCH"))?;

    for entry in fs::read_dir(&cache_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        stats.total_entries += 1;
        stats.total_size += fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(entry) = serde_json::from_str::<CacheEntry<serde_json::Value>>(&content) {
                if (entry.expires_at as u128) < now.as_secs() as u128 {
                    stats.expired_entries += 1;
                }
            }
        }
    }

    Ok(stats)
}

/// Cache statistics
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub total_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use serde_json::json;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_cache_operations() -> Result<()> {
        let temp_dir = tempdir()?;
        let cache_dir = temp_dir.path().join("test_cache");

        // Initialize cache
        init_cache(Some(cache_dir.clone()), Some(Duration::from_secs(1)))?;

        // Create a test key
        let key = CacheKey::new(
            Path::new("test.rs"),
            "test_key",
            1,
            "test content",
            &json!({ "test": true }),
        )?;

        // Test set and get
        set_cached(&key, &42, None, None)?;
        let cached: Option<i32> = get_cached(&key)?;
        assert_eq!(cached, Some(42));

        // Test TTL expiration
        thread::sleep(StdDuration::from_secs(2));
        let cached: Option<i32> = get_cached(&key)?;
        assert_eq!(cached, None);

        // Test invalidation
        set_cached(&key, &42, None, None)?;
        assert_eq!(invalidate_cache("test")?, 1);
        let cached: Option<i32> = get_cached(&key)?;
        assert_eq!(cached, None);

        // Test cleanup
        set_cached(&key, &42, Some(Duration::from_millis(1)), None)?;
        thread::sleep(StdDuration::from_millis(10));
        let cleaned = cleanup_expired()?;
        assert!(cleaned > 0);

        Ok(())
    }
}
