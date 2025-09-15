//! Cache Storage Backends
//!
//! This module defines storage backends for the cache, including
//! memory, file-based, and external storage options.

use std::error::Error;

use super::*;

#[derive(Debug, Clone)]
pub enum StorageBackend {
    Memory,
    File(std::path::PathBuf),
    #[cfg(feature = "persistent")]
    Database(std::path::PathBuf),
    #[cfg(feature = "persistent")]
    Redis(String),
}

/// File-based storage backend
pub struct FileStorage<K, V> {
    path: std::path::PathBuf,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> FileStorage<K, V> {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self {
            path,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// In-memory storage for fast access
pub struct MemoryStorage<K, V> {
    entries: dashmap::DashMap<K, CacheEntry<V>>,
}

impl<K, V> MemoryStorage<K, V>
where
    K: Send + Sync + Clone + Hash + Eq + serde::Serialize + 'static,
    V: Send + Sync + Clone + serde::Serialize + 'static,
{
    pub fn new() -> Self {
        Self {
            entries: dashmap::DashMap::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<CacheEntry<V>> {
        self.entries.get(key).map(|entry| entry.value().clone())
    }

    pub fn set(&self, key: K, entry: CacheEntry<V>) {
        self.entries.insert(key, entry);
    }

    pub fn remove(&self, key: &K) -> Option<CacheEntry<V>> {
        self.entries.remove(key).map(|(_, entry)| entry)
    }

    pub fn clear(&self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn iter(&self) -> dashmap::iter::Iter<'_, K, CacheEntry<V>> {
        self.entries.iter()
    }
}

/// Disk-based storage for large datasets
#[cfg(feature = "persistent")]
pub struct DiskStorage<K, V> {
    sled_db: sled::Db,
    _phantom: std::marker::PhantomData<(K, V)>,
}

#[cfg(feature = "persistent")]
impl<K, V> DiskStorage<K, V>
where
    K: Send + Sync + Clone + Hash + Eq + serde::Serialize + 'static,
    V: Send + Sync + Clone + serde::Serialize + 'static,
{
    pub fn new(path: std::path::PathBuf) -> Result<Self, Box<dyn Error>> {
        let db = sled::open(path)?;
        Ok(Self {
            sled_db: db,
            _phantom: std::marker::PhantomData,
        })
    }

    pub fn get(&self, key: &K) -> Result<Option<CacheEntry<V>>, Box<dyn Error>> {
        let key_bytes = bincode::serialize(key)?;
        if let Some(value_bytes) = self.sled_db.get(key_bytes)? {
            let entry: CacheEntry<V> = bincode::deserialize(&value_bytes)?;
            if entry.is_expired() {
                Ok(None)
            } else {
                Ok(Some(entry))
            }
        } else {
            Ok(None)
        }
    }

    pub fn set(&self, key: K, entry: CacheEntry<V>) -> Result<(), Box<dyn Error>> {
        let key_bytes = bincode::serialize(&key)?;
        let value_bytes = bincode::serialize(&entry)?;
        self.sled_db.insert(key_bytes, value_bytes)?;
        Ok(())
    }

    pub fn remove(&self, key: &K) -> Result<Option<CacheEntry<V>>, Box<dyn Error>> {
        let key_bytes = bincode::serialize(key)?;
        if let Some(value_bytes) = self.sled_db.remove(key_bytes)? {
            let entry: CacheEntry<V> = bincode::deserialize(&value_bytes)?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    pub fn clear(&self) -> Result<(), Box<dyn Error>> {
        let mut batch = sled::Batch::default();
        for result in self.sled_db.iter() {
            let (key_bytes, _) = result?;
            batch.remove(key_bytes);
        }
        self.sled_db.apply_batch(batch)?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.sled_db.len()
    }
}

/// Storage manager that handles multiple storage backends
pub struct StorageManager<K, V> {
    memory: MemoryStorage<K, V>,
    storage_backend: StorageBackend,
}

impl<K, V> StorageManager<K, V>
where
    K: Send + Sync + Clone + Hash + Eq + serde::Serialize + 'static,
    V: Send + Sync + Clone + serde::Serialize + 'static,
{
    pub fn new(backend: StorageBackend) -> Self {
        Self {
            memory: MemoryStorage::new(),
            storage_backend: backend,
        }
    }

    pub async fn get(&self, key: &K) -> Result<Option<CacheEntry<V>>, Box<dyn Error>> {
        // Try memory first
        if let Some(entry) = self.memory.get(key) {
            Ok(Some(entry))
        } else {
            // Fallback to persistent storage
            match &self.storage_backend {
                StorageBackend::Memory => Ok(None),
                StorageBackend::File(path) => {
                    // File-based loading would be implemented here
                    Ok(None)
                }
                #[cfg(feature = "persistent")]
                StorageBackend::Database(_path) => {
                    // Would use disk storage here
                    Ok(None)
                }
                #[cfg(feature = "persistent")]
                StorageBackend::Redis(_url) => {
                    // Would use Redis here
                    Ok(None)
                }
            }
        }
    }

    pub async fn set(&self, key: K, entry: CacheEntry<V>) -> Result<(), Box<dyn Error>> {
        // Always store in memory
        self.memory.set(key.clone(), entry.clone());

        // Also store in persistent backend if configured
        match &self.storage_backend {
            StorageBackend::Memory => {}
            StorageBackend::File(path) => {
                // File-based saving would be implemented here
            }
            #[cfg(feature = "persistent")]
            StorageBackend::Database(_path) => {
                // Disk saving here
            }
            #[cfg(feature = "persistent")]
            StorageBackend::Redis(_url) => {
                // Redis saving here
            }
        }

        Ok(())
    }

    pub async fn cleanup_expired(&self) -> Result<usize, Box<dyn Error>> {
        let mut count = 0;

        // Clean memory
        self.memory.entries.retain(|_, entry| {
            if entry.is_expired() {
                count += 1;
                false
            } else {
                true
            }
        });

        // Clean persistent storage if configured
        match &self.storage_backend {
            StorageBackend::Memory => {}
            StorageBackend::File(_path) => {
                // Would clean file backups here
            }
            #[cfg(feature = "persistent")]
            StorageBackend::Database(_path) => {
                // Would clean disk entries here
            }
            #[cfg(feature = "persistent")]
            StorageBackend::Redis(_url) => {
                // Would clean Redis entries here
            }
        }

        Ok(count)
    }
}

/// Compression support for large cache entries
#[cfg(feature = "compression")]
pub mod compressed {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CompressionConfig {
        pub algorithm: CompressionAlgorithm,
        pub level: i32,
        pub threshold_bytes: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum CompressionAlgorithm {
        Zstd,
        Gzip,
        None,
    }

    pub struct CompressedStorage<K, V> {
        config: CompressionConfig,
        storage: MemoryStorage<K, V>,
    }

    impl<K, V> CompressedStorage<K, V> {
        pub fn new(config: CompressionConfig) -> Self {
            Self {
                config,
                storage: MemoryStorage::new(),
            }
        }
    }
}

#[cfg(feature = "compression")]
impl<K, V> StorageManager<K, V> {
    pub fn with_compression(
        self,
        config: compressed::CompressionConfig,
    ) -> compressed::CompressedStorage<K, V> {
        compressed::CompressedStorage::new(config)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::RwLock;

    use super::*;

    #[tokio::test]
    async fn test_memory_storage_basic_operations() {
        let storage: MemoryStorage<String, String> = MemoryStorage::new();

        // Test set and get
        let entry = CacheEntry::new("value1".to_string());
        storage.set("key1".to_string(), entry);

        let result = storage.get(&"key1".to_string());
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, "value1");

        // Test remove
        let result = storage.remove(&"key1".to_string());
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, "value1");

        // Test get after remove
        let result = storage.get(&"key1".to_string());
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_memory_storage_clear() {
        let storage: MemoryStorage<String, String> = MemoryStorage::new();

        // Add multiple entries
        storage.set("key1".to_string(), CacheEntry::new("value1".to_string()));
        storage.set("key2".to_string(), CacheEntry::new("value2".to_string()));
        storage.set("key3".to_string(), CacheEntry::new("value3".to_string()));

        assert_eq!(storage.len(), 3);

        storage.clear();
        assert_eq!(storage.len(), 0);
    }

    #[cfg(feature = "persistent")]
    #[tokio::test]
    async fn test_disk_storage() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let storage = DiskStorage::<String, String>::new(temp_dir.path().to_path_buf()).unwrap();

        // Test set and get
        let entry = CacheEntry::new("disk_value".to_string());
        storage.set("disk_key".to_string(), entry).unwrap();

        let result = storage.get(&"disk_key".to_string()).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, "disk_value");

        // Test remove
        let result = storage.remove(&"disk_key".to_string()).unwrap();
        assert!(result.is_some());

        let result = storage.get(&"disk_key".to_string()).unwrap();
        assert!(result.is_none());
    }
}
