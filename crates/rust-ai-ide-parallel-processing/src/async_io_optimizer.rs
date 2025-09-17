/*!
 * Async I/O Optimizer for high-throughput file processing
 *
 * This module provides asynchronous file reading with parallel batching,
 * memory-mapped file operations, and intelligent prefetching to eliminate
 * I/O bottlenecks in code analysis processing.
 */

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use memmap2::Mmap;
use tokio::fs::{self, File as AsyncFile};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::{Semaphore, RwLock};
use tokio::task;

/// Configuration for async I/O operations
#[derive(Debug, Clone)]
pub struct AsyncIoConfig {
    /// Maximum concurrent file operations
    pub max_concurrent_files: usize,
    /// Buffer size for streaming reads
    pub buffer_size: usize,
    /// Maximum memory for memory-mapped files
    pub max_mmap_memory: usize,
    /// Prefetch window size
    pub prefetch_window: usize,
    /// I/O timeout in milliseconds
    pub io_timeout_ms: u64,
}

impl Default for AsyncIoConfig {
    fn default() -> Self {
        Self {
            max_concurrent_files: 64,
            buffer_size: 64 * 1024, // 64KB
            max_mmap_memory: 1024 * 1024 * 1024, // 1GB
            prefetch_window: 4,
            io_timeout_ms: 30000, // 30 seconds
        }
    }
}

/// Result of file read operation
#[derive(Debug)]
pub enum FileReadResult {
    /// File content as bytes
    Bytes(Vec<u8>),
    /// Memory-mapped file
    MemoryMapped(Mmap),
    /// Streaming reader for large files
    Streaming(StreamingReader),
}

/// Streaming reader for large files
pub struct StreamingReader {
    file: AsyncFile,
    buffer: Vec<u8>,
    buffer_size: usize,
    position: u64,
    file_size: u64,
}

impl StreamingReader {
    /// Read next chunk
    pub async fn read_chunk(&mut self) -> io::Result<Option<&[u8]>> {
        if self.position >= self.file_size {
            return Ok(None);
        }

        let remaining = self.file_size - self.position;
        let chunk_size = std::cmp::min(self.buffer_size as u64, remaining) as usize;

        self.buffer.resize(chunk_size, 0);
        self.file.seek(SeekFrom::Start(self.position)).await?;
        let bytes_read = self.file.read(&mut self.buffer).await?;

        if bytes_read == 0 {
            return Ok(None);
        }

        self.buffer.truncate(bytes_read);
        self.position += bytes_read as u64;

        Ok(Some(&self.buffer))
    }

    /// Get current position
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Get file size
    pub fn file_size(&self) -> u64 {
        self.file_size
    }
}

/// Async I/O optimizer with intelligent caching and prefetching
pub struct AsyncIoOptimizer {
    /// Configuration
    config: AsyncIoConfig,
    /// Semaphore for limiting concurrent operations
    concurrency_semaphore: Arc<Semaphore>,
    /// File cache for frequently accessed files
    file_cache: Arc<RwLock<HashMap<PathBuf, Arc<FileCacheEntry>>>>,
    /// Memory usage tracker
    memory_usage: Arc<RwLock<usize>>,
    /// Prefetch queue
    prefetch_queue: Arc<RwLock<Vec<PathBuf>>>,
}

#[derive(Debug)]
struct FileCacheEntry {
    content: Vec<u8>,
    last_accessed: std::time::Instant,
    access_count: u64,
}

/// Optimized file read operation
impl AsyncIoOptimizer {
    pub fn new(config: AsyncIoConfig) -> Self {
        Self {
            concurrency_semaphore: Arc::new(Semaphore::new(config.max_concurrent_files)),
            file_cache: Arc::new(RwLock::new(HashMap::new())),
            memory_usage: Arc::new(RwLock::new(0)),
            prefetch_queue: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Read file with intelligent strategy selection
    pub async fn read_file(&self, path: &Path) -> io::Result<FileReadResult> {
        let _permit = self.concurrency_semaphore.acquire().await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Check cache first
        if let Some(cached) = self.get_cached_file(path).await {
            return Ok(FileReadResult::Bytes(cached));
        }

        let metadata = fs::metadata(path).await?;
        let file_size = metadata.len();

        // Choose optimal read strategy
        if file_size <= self.config.buffer_size as u64 {
            // Small file - read entirely
            self.read_small_file(path).await
        } else if file_size <= (self.config.max_mmap_memory / 4) as u64 {
            // Medium file - memory map
            self.read_memory_mapped_file(path).await
        } else {
            // Large file - streaming
            self.create_streaming_reader(path).await
        }
    }

    /// Batch read multiple files with prefetching
    pub async fn batch_read_files(&self, paths: &[PathBuf]) -> Vec<io::Result<FileReadResult>> {
        let mut handles = Vec::with_capacity(paths.len());

        // Start prefetching for upcoming files
        self.prefetch_files(&paths[self.config.prefetch_window..]).await;

        // Read files concurrently with controlled parallelism
        for chunk in paths.chunks(self.config.max_concurrent_files) {
            let mut chunk_handles = Vec::with_capacity(chunk.len());

            for path in chunk {
                let path_clone = path.clone();
                let self_clone = self.clone();

                let handle = task::spawn(async move {
                    self_clone.read_file(&path_clone).await
                });
                chunk_handles.push(handle);
            }

            // Wait for this chunk to complete
            for handle in chunk_handles {
                handles.push(handle);
            }
        }

        // Collect results maintaining order
        let mut results = Vec::with_capacity(paths.len());
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(io::Error::new(io::ErrorKind::Other,
                    format!("Task panicked: {}", e)))),
            }
        }

        results
    }

    /// Read small file entirely
    async fn read_small_file(&self, path: &Path) -> io::Result<FileReadResult> {
        let mut file = AsyncFile::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        // Cache the result
        self.cache_file(path.to_path_buf(), buffer.clone()).await;

        Ok(FileReadResult::Bytes(buffer))
    }

    /// Read file using memory mapping
    async fn read_memory_mapped_file(&self, path: &Path) -> io::Result<FileReadResult> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // Check memory limits
        let mmap_size = mmap.len();
        {
            let mut memory_usage = self.memory_usage.write().await;
            if *memory_usage + mmap_size > self.config.max_mmap_memory {
                // Evict least recently used files
                self.evict_cache_to_fit(mmap_size).await;
            }
            *memory_usage += mmap_size;
        }

        Ok(FileReadResult::MemoryMapped(mmap))
    }

    /// Create streaming reader for large files
    async fn create_streaming_reader(&self, path: &Path) -> io::Result<FileReadResult> {
        let file = AsyncFile::open(path).await?;
        let metadata = file.metadata().await?;
        let file_size = metadata.len();

        let reader = StreamingReader {
            file,
            buffer: Vec::with_capacity(self.config.buffer_size),
            buffer_size: self.config.buffer_size,
            position: 0,
            file_size,
        };

        Ok(FileReadResult::Streaming(reader))
    }

    /// Get cached file if available
    async fn get_cached_file(&self, path: &Path) -> Option<Vec<u8>> {
        let mut cache = self.file_cache.write().await;
        if let Some(entry) = cache.get_mut(path) {
            entry.last_accessed = std::time::Instant::now();
            entry.access_count += 1;
            Some(entry.content.clone())
        } else {
            None
        }
    }

    /// Cache file content
    async fn cache_file(&self, path: PathBuf, content: Vec<u8>) {
        let mut cache = self.file_cache.write().await;
        let entry = FileCacheEntry {
            content,
            last_accessed: std::time::Instant::now(),
            access_count: 1,
        };
        cache.insert(path, Arc::new(entry));
    }

    /// Evict cache entries to fit new content
    async fn evict_cache_to_fit(&self, required_size: usize) {
        let mut cache = self.file_cache.write().await;
        let mut entries: Vec<_> = cache.iter().collect();

        // Sort by access frequency and recency
        entries.sort_by(|a, b| {
            let a_score = a.1.access_count as f64 / a.1.last_accessed.elapsed().as_secs_f64();
            let b_score = b.1.access_count as f64 / b.1.last_accessed.elapsed().as_secs_f64();
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut freed_memory = 0;
        let mut to_remove = Vec::new();

        for (path, entry) in entries {
            if freed_memory >= required_size {
                break;
            }
            freed_memory += entry.content.len();
            to_remove.push(path.clone());
        }

        for path in to_remove {
            cache.remove(&path);
        }

        // Update memory usage
        let mut memory_usage = self.memory_usage.write().await;
        *memory_usage -= freed_memory;
    }

    /// Prefetch files for future access
    async fn prefetch_files(&self, paths: &[PathBuf]) {
        let mut prefetch_queue = self.prefetch_queue.write().await;
        for path in paths.iter().take(self.config.prefetch_window) {
            if !prefetch_queue.contains(path) {
                prefetch_queue.push(path.clone());
            }
        }

        // Spawn prefetch tasks
        for path in prefetch_queue.drain(..) {
            let self_clone = self.clone();
            task::spawn(async move {
                let _ = self_clone.prefetch_file(&path).await;
            });
        }
    }

    /// Prefetch single file
    async fn prefetch_file(&self, path: &Path) -> io::Result<()> {
        // Only prefetch if not already cached
        if self.get_cached_file(path).await.is_none() {
            let metadata = fs::metadata(path).await?;
            if metadata.len() <= self.config.buffer_size as u64 {
                // Only prefetch small files
                self.read_small_file(path).await?;
            }
        }
        Ok(())
    }

    /// Get current memory usage
    pub async fn memory_usage(&self) -> usize {
        *self.memory_usage.read().await
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, usize) {
        let cache = self.file_cache.read().await;
        let total_files = cache.len();
        let total_memory = cache.values().map(|entry| entry.content.len()).sum();
        (total_files, total_memory)
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.file_cache.write().await;
        cache.clear();
        let mut memory_usage = self.memory_usage.write().await;
        *memory_usage = 0;
    }
}

impl Clone for AsyncIoOptimizer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            concurrency_semaphore: Arc::clone(&self.concurrency_semaphore),
            file_cache: Arc::clone(&self.file_cache),
            memory_usage: Arc::clone(&self.memory_usage),
            prefetch_queue: Arc::clone(&self.prefetch_queue),
        }
    }
}

impl Default for AsyncIoOptimizer {
    fn default() -> Self {
        Self::new(AsyncIoConfig::default())
    }
}

/// Batch file processor for high-throughput processing
pub struct BatchFileProcessor {
    optimizer: AsyncIoOptimizer,
    processing_semaphore: Arc<Semaphore>,
}

impl BatchFileProcessor {
    pub fn new(optimizer: AsyncIoOptimizer, max_concurrent_processing: usize) -> Self {
        Self {
            optimizer,
            processing_semaphore: Arc::new(Semaphore::new(max_concurrent_processing)),
        }
    }

    /// Process files in parallel with custom processor function
    pub async fn process_files<F, Fut, T>(
        &self,
        paths: &[PathBuf],
        processor: F,
    ) -> Vec<io::Result<T>>
    where
        F: Fn(FileReadResult) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = io::Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        let mut handles = Vec::with_capacity(paths.len());

        // Read files in optimized batches
        let read_results = self.optimizer.batch_read_files(paths).await;

        // Process files concurrently
        for (path, read_result) in paths.iter().zip(read_results) {
            let processor_clone = processor.clone();
            let _permit = self.processing_semaphore.clone().acquire_owned().await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e));

            let handle = task::spawn(async move {
                match read_result {
                    Ok(file_result) => processor_clone(file_result).await,
                    Err(e) => Err(e),
                }
            });

            handles.push(handle);
        }

        // Collect results
        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(io::Error::new(io::ErrorKind::Other,
                    format!("Processing task panicked: {}", e)))),
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_small_file_read() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = b"Hello, World!";
        temp_file.write_all(content).unwrap();

        let optimizer = AsyncIoOptimizer::default();
        let result = optimizer.read_file(temp_file.path()).await.unwrap();

        match result {
            FileReadResult::Bytes(data) => assert_eq!(data, content),
            _ => panic!("Expected bytes result"),
        }
    }

    #[tokio::test]
    async fn test_batch_read() {
        let mut temp_files = Vec::new();
        let mut paths = Vec::new();

        for i in 0..3 {
            let mut temp_file = NamedTempFile::new().unwrap();
            let content = format!("Content {}", i);
            temp_file.write_all(content.as_bytes()).unwrap();
            paths.push(temp_file.path().to_path_buf());
            temp_files.push(temp_file);
        }

        let optimizer = AsyncIoOptimizer::default();
        let results = optimizer.batch_read_files(&paths).await;

        assert_eq!(results.len(), 3);
        for result in results {
            assert!(result.is_ok());
        }
    }
}