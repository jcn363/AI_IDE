use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::sync::RwLock;
use ndarray::{ArrayD, ArrayViewD, IxDynImpl};
use memmap2::MmapMut;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use rust_ai_ide_shared_types::{
    VectorSearchRequest, VectorSearchResult, SearchFilter, FilterOperator
};
use rust_ai_ide_common::validation::validate_secure_path;
use rust_ai_ide_cache::strategies::AdaptiveCache;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndexConfig {
    pub max_vectors: usize,
    pub dimensionality: usize,
    pub index_type: IndexType,
    pub quantization_bits: Option<u8>,
    pub compression_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexType {
    HNSW,      // Hierarchical Navigable Small World
    IVF,       // Inverted File with Product Quantization
    PQ,        // Product Quantization
    LSH,       // Locality Sensitive Hashing
}

#[derive(Clone)]
pub struct VectorDocument {
    pub id: String,
    pub vector: Vec<f32>,
    pub content: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for VectorIndexConfig {
    fn default() -> Self {
        Self {
            max_vectors: 1_000_000,
            dimensionality: 768,
            index_type: IndexType::HNSW,
            quantization_bits: Some(8),
            compression_ratio: 0.8,
        }
    }
}

/// Memory-mapped vector database with zero-copy operations
pub struct VectorDatabase {
    config: VectorIndexConfig,
    vectors: Arc<Mutex<HashMap<String, VectorDocument>>>,
    index: Arc<RwLock<HNSWIndex>>,
    cache: Arc<AdaptiveCache<Vec<f32>>>,
    mmap_area: Option<MmapMut>,
}

impl VectorDatabase {
    pub async fn new(config: VectorIndexConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let index = match config.index_type {
            IndexType::HNSW => HNSWIndex::new(config.clone()),
            _ => return Err("Index type not implemented yet".into()),
        };

        Ok(Self {
            config,
            vectors: Arc::new(Mutex::new(HashMap::new())),
            index: Arc::new(RwLock::new(index)),
            cache: Arc::new(AdaptiveCache::new(100 * 1024 * 1024)), // 100MB cache
            mmap_area: None,
        })
    }

    /// Memory-map the vector storage file for zero-copy operations
    pub async fn load_from_disk(&mut self, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        validate_secure_path(file_path.to_str().unwrap())?;

        if file_path.exists() {
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(file_path)?;

            let mmap = unsafe { MmapMut::map_mut(&file)? };
            self.mmap_area = Some(mmap);

            // Deserialize vectors from memory-mapped area
            self.deserialize_from_mmap()?;
        }

        Ok(())
    }

    /// Persist vectors to disk using memory mapping
    pub async fn save_to_disk(&mut self, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        validate_secure_path(file_path.to_str().unwrap())?;

        let serialized = self.serialize_vectors()?;
        std::fs::write(file_path, serialized)?;

        // Create memory mapping for future operations
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(file_path)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };
        self.mmap_area = Some(mmap);

        Ok(())
    }

    /// Add multiple vectors in batch for optimal performance
    pub async fn add_batch(&self, documents: Vec<VectorDocument>) -> Result<(), Box<dyn std::error::Error>> {
        let mut vectors_lock = self.vectors.lock();
        let mut index_lock = self.index.write().await;

        for doc in documents {
            if vectors_lock.contains_key(&doc.id) {
                return Err(format!("Document with ID {} already exists", doc.id).into());
            }

            if doc.vector.len() != self.config.dimensionality {
                return Err(format!("Vector dimension mismatch: expected {}, got {}",
                            self.config.dimensionality, doc.vector.len()).into());
            }

            vectors_lock.insert(doc.id.clone(), doc.clone());
            index_lock.add_vector(doc.id.clone(), doc.vector.clone());
        }

        Ok(())
    }

    /// Single vector addition with deduplication
    pub async fn add(&self, document: VectorDocument) -> Result<(), Box<dyn std::error::Error>> {
        self.add_batch(vec![document]).await
    }

    /// Perform vector similarity search
    pub async fn search(&self, request: VectorSearchRequest) -> Result<Vec<VectorSearchResult>, Box<dyn std::error::Error>> {
        if request.query_vector.len() != self.config.dimensionality {
            return Err(format!("Query vector dimension mismatch: expected {}, got {}",
                        self.config.dimensionality, request.query_vector.len()).into());
        }

        let index = self.index.read().await;
        let candidates = index.search(&request.query_vector, request.top_k * 3)?; // Get more for filtering

        let vectors = self.vectors.lock();

        // Apply filters and compute final results
        let mut results: Vec<VectorSearchResult> = candidates
            .into_iter()
            .filter_map(|(id, score)| {
                let doc = vectors.get(&id)?;
                if self.apply_filters(doc, &request.filters) {
                    Some(VectorSearchResult {
                        id: doc.id.clone(),
                        score,
                        content: request.config.include_content.then(|| doc.content.clone()).flatten(),
                        metadata: doc.metadata.clone(),
                        ..Default::default()
                    })
                } else {
                    None
                }
            })
            .take(request.top_k)
            .collect();

        // Sort by score (similarity, descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }

    /// Remove vectors by IDs or filters
    pub async fn delete(&self, ids: &[String]) -> Result<usize, Box<dyn std::error::Error>> {
        let mut vectors_lock = self.vectors.lock();
        let mut index_lock = self.index.write().await;

        let mut deleted = 0;
        for id in ids {
            if vectors_lock.remove(id).is_some() {
                index_lock.remove_vector(id);
                deleted += 1;
            }
        }

        Ok(deleted)
    }

    /// Update vector in-place
    pub async fn update(&self, id: &str, new_vector: Vec<f32>) -> Result<(), Box<dyn std::error::Error>> {
        if new_vector.len() != self.config.dimensionality {
            return Err(format!("Vector dimension mismatch: expected {}, got {}",
                        self.config.dimensionality, new_vector.len()).into());
        }

        let mut vectors_lock = self.vectors.lock();
        let mut index_lock = self.index.write().await;

        if let Some(doc) = vectors_lock.get_mut(id) {
            doc.vector = new_vector.clone();
            index_lock.update_vector(id.to_string(), new_vector);
            Ok(())
        } else {
            Err(format!("Vector with ID {} not found", id).into())
        }
    }

    /// Get database statistics
    pub async fn stats(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let vectors = self.vectors.lock();
        let index = self.index.read().await;

        let stats = serde_json::json!({
            "total_vectors": vectors.len(),
            "dimensionality": self.config.dimensionality,
            "index_type": format!("{:?}", self.config.index_type),
            "memory_mapped": self.mmap_area.is_some(),
            "index_stats": index.stats()?,
            "cache_size_mb": self.cache.size() / (1024 * 1024)
        });

        Ok(stats)
    }

    /// Apply filters to a document
    fn apply_filters(&self, doc: &VectorDocument, filters: &Option<Vec<SearchFilter>>) -> bool {
        if let Some(filters) = filters {
            for filter in filters {
                match filter.operator {
                    FilterOperator::Equal => {
                        if let Some(value) = doc.metadata.get(&filter.field) {
                            if value != &filter.value {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    FilterOperator::GreaterThan => {
                        if let (Some(doc_val), Some(filter_val)) = (
                            doc.metadata.get(&filter.field).and_then(|v| v.as_f64()),
                            filter.value.as_f64()
                        ) {
                            if doc_val <= filter_val {
                                return false;
                            }
                        }
                    }
                    FilterOperator::LessThan => {
                        if let (Some(doc_val), Some(filter_val)) = (
                            doc.metadata.get(&filter.field).and_then(|v| v.as_f64()),
                            filter.value.as_f64()
                        ) {
                            if doc_val >= filter_val {
                                return false;
                            }
                        }
                    }
                    FilterOperator::Contains => {
                        if let (Some(doc_val), Some(filter_str)) = (
                            doc.metadata.get(&filter.field).and_then(|v| v.as_str()),
                            filter.value.as_str()
                        ) {
                            if !doc_val.contains(filter_str) {
                                return false;
                            }
                        }
                    }
                    FilterOperator::NotEqual => {
                        if let Some(value) = doc.metadata.get(&filter.field) {
                            if value == &filter.value {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        true
    }

    /// Serialize vectors for persistent storage
    fn serialize_vectors(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let vectors = self.vectors.lock();
        bincode::serialize(&*vectors).map_err(Into::into)
    }

    /// Deserialize vectors from memory-mapped area
    fn deserialize_from_mmap(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mmap) = &self.mmap_area {
            let vectors: HashMap<String, VectorDocument> = bincode::deserialize(mmap)?;
            *self.vectors.lock() = vectors;
        }
        Ok(())
    }
}

/// Hierarchical Navigable Small World index implementation
struct HNSWIndex {
    config: VectorIndexConfig,
    nodes: HashMap<String, Vec<f32>>,
}

impl HNSWIndex {
    fn new(config: VectorIndexConfig) -> Self {
        Self {
            config,
            nodes: HashMap::new(),
        }
    }

    fn add_vector(&mut self, id: String, vector: Vec<f32>) {
        // Simplified implementation - in practice would build HNSW index
        self.nodes.insert(id, vector);
    }

    fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<(String, f32)>, Box<dyn std::error::Error>> {
        let mut results: Vec<(String, f32)> = self.nodes
            .iter()
            .map(|(id, vector)| (id.clone(), cosine_similarity(query, vector)))
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);

        Ok(results)
    }

    fn remove_vector(&mut self, id: &str) {
        self.nodes.remove(id);
    }

    fn update_vector(&mut self, id: String, vector: Vec<f32>) {
        self.nodes.insert(id, vector);
    }

    fn stats(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        Ok(serde_json::json!({
            "node_count": self.nodes.len(),
            "max_connections": 32, // Example HNSW parameter
            "ef_construction": 200,
            "ef_search": 64
        }))
    }
}

/// Compute cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for i in 0..a.len() {
        dot_product += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vector_database_creation() {
        let config = VectorIndexConfig {
            max_vectors: 1000,
            dimensionality: 128,
            index_type: IndexType::HNSW,
            ..Default::default()
        };

        let db = VectorDatabase::new(config).await.unwrap();
        assert_eq!(db.config.dimensionality, 128);
    }

    #[tokio::test]
    async fn test_cosine_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.0001);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&c, &d) - 0.0).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_vector_operations() {
        let config = VectorIndexConfig {
            dimensionality: 3,
            ..Default::default()
        };

        let db = VectorDatabase::new(config).await.unwrap();

        let doc = VectorDocument {
            id: "test".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            content: Some("test content".to_string()),
            metadata: HashMap::new(),
        };

        // Test addition
        db.add(doc).await.unwrap();

        // Test search
        let request = VectorSearchRequest {
            query_vector: vec![1.0, 2.0, 3.0],
            top_k: 1,
            ..Default::default()
        };

        let results = db.search(request).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "test");
        assert!((results[0].score - 1.0).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_filter_operations() {
        let config = VectorIndexConfig::default();
        let db = VectorDatabase::new(config).await.unwrap();

        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), serde_json::json!("function"));

        let doc = VectorDocument {
            id: "test".to_string(),
            vector: vec![1.0; 768],
            metadata,
            ..Default::default()
        };

        db.add(doc).await.unwrap();

        let filter = SearchFilter {
            field: "type".to_string(),
            operator: FilterOperator::Equal,
            value: serde_json::json!("function"),
        };

        let request = VectorSearchRequest {
            query_vector: vec![1.0; 768],
            top_k: 1,
            filters: Some(vec![filter]),
            ..Default::default()
        };

        let results = db.search(request).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "test");
    }
}