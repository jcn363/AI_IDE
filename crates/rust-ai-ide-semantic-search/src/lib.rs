use std::{collections::HashMap, path::{Path, PathBuf}, sync::Arc};
use tokio::{sync::{mpsc, RwLock}, task::{spawn_blocking, JoinHandle}};
use parking_lot::{Mutex, RwLock as ParkingRwLock};
use regex::Regex;
use tree_sitter::{Parser, Query};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use rust_ai_ide_shared_types::{
    CodeSearchRequest, CodeSearchResult, SearchRanking,
    CodeResultType, ContextLine, HighlightSpan, MatchType
};
use rust_ai_ide_vector_database::{VectorDatabase, VectorDocument, VectorSearchRequest};
use rust_ai_ide_onnx_runtime::{ONNXInferenceService, InferenceRequest};
use rust_ai_ide_common::validation::validate_secure_path;
use rust_ai_ide_cache::strategies::AdaptiveCache;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchConfig {
    pub max_code_chunk_length: usize,
    pub overlap_size: usize,
    pub batch_size: usize,
    pub include_comments: bool,
    pub include_docstrings: bool,
    pub supported_languages: Vec<String>,
    pub vector_dimension: usize,
}

impl Default for SemanticSearchConfig {
    fn default() -> Self {
        Self {
            max_code_chunk_length: 512,
            overlap_size: 50,
            batch_size: 32,
            include_comments: true,
            include_docstrings: true,
            supported_languages: vec![
                "rust".to_string(),
                "go".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "python".to_string(),
                "java".to_string(),
                "cpp".to_string(),
            ],
            vector_dimension: 768,
        }
    }
}

/// Code chunk extracted from source files for semantic indexing
#[derive(Clone, Debug)]
pub struct CodeChunk {
    pub file_path: PathBuf,
    pub content: String,
    pub line_start: usize,
    pub line_end: usize,
    pub language: String,
    pub chunk_type: CodeResultType,
    pub context: Option<String>,
    pub hash: String,
}

/// Multi-threaded semantic code search engine
pub struct SemanticSearchEngine {
    config: SemanticSearchConfig,
    vector_db: Arc<VectorDatabase>,
    onnx_service: Arc<ONNXInferenceService>,
    index_cache: Arc<AdaptiveCache<CodeChunk>>,
    parser_cache: Arc<Mutex<HashMap<String, Parser>>>,
    indexing_state: Arc<RwLock<IndexingState>>,
    indexer_handle: Mutex<Option<JoinHandle<()>>>,
}

#[derive(Clone, Debug, Default)]
struct IndexingState {
    pub total_files: usize,
    pub indexed_files: usize,
    pub last_indexed_hash: String,
    pub is_indexing: bool,
    pub error_count: usize,
}

impl SemanticSearchEngine {
    pub async fn new(
        config: SemanticSearchConfig,
        vector_db: Arc<VectorDatabase>,
        onnx_service: Arc<ONNXInferenceService>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize tree-sitter parsers for supported languages
        let mut parser_cache = HashMap::new();
        for language in &config.supported_languages {
            let mut parser = Parser::new();
            Self::setup_parser(&mut parser, language)?;
            parser_cache.insert(language.clone(), parser);
        }

        Ok(Self {
            config,
            vector_db,
            onnx_service,
            index_cache: Arc::new(AdaptiveCache::new(100 * 1024 * 1024)), // 100MB cache
            parser_cache: Arc::new(Mutex::new(parser_cache)),
            indexing_state: Arc::new(RwLock::new(IndexingState::default())),
            indexer_handle: Mutex::new(None),
        })
    }

    /// Index a codebase directory tree
    pub async fn index_codebase(&self, root_path: &Path, force_reindex: bool) -> Result<(), Box<dyn std::error::Error>> {
        validate_secure_path(root_path.to_str().unwrap())?;

        if !root_path.exists() {
            return Err("Codebase path does not exist".into());
        }

        let mut state = self.indexing_state.write().await;
        if state.is_indexing && !force_reindex {
            return Err("Indexing already in progress".into());
        }

        state.is_indexing = true;
        drop(state);

        let self_clone = self as *const Self as *mut Self;
        let indexer_handle = spawn_blocking(move || unsafe {
            (*self_clone).perform_indexing_blocking(root_path)
        });

        let mut handle_lock = self.indexer_handle.lock();
        *handle_lock = Some(indexer_handle);

        Ok(())
    }

    fn perform_indexing_blocking(&self, root_path: &Path) {
        // Use walkdir for efficient directory traversal
        let walker = ignore::WalkBuilder::new(root_path)
            .git_ignore(true)
            .hidden(false)
            .follow_links(false)
            .build();

        let mut chunks = Vec::new();

        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        // Process file in blocking context
                        let path = entry.path();
                        if let Err(e) = self.process_file_blocking(path, &mut chunks) {
                            // Log error but continue processing
                            eprintln!("Failed to process file {}: {}", path.display(), e);
                        }

                        // Batch index periodically
                        if chunks.len() >= self.config.batch_size {
                            if let Err(e) = self.create_and_index_chunks_blocking(chunks.drain(..).collect()) {
                                eprintln!("Failed to index batch: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error walking directory: {}", e);
                }
            }
        }

        // Index remaining chunks
        if let Err(e) = self.create_and_index_chunks_blocking(chunks) {
            eprintln!("Failed to index final batch: {}", e);
        }

        // Update indexing state
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let mut state = self.indexing_state.write().await;
            state.is_indexing = false;
        });
    }

    fn process_file_blocking(&self, file_path: &Path, chunks: &mut Vec<CodeChunk>) -> Result<(), Box<dyn std::error::Error>> {
        let extension = file_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let language = match extension {
            "rs" => "rust",
            "go" => "go",
            "js" => "javascript",
            "ts" => "typescript",
            "py" => "python",
            "java" => "java",
            "cpp" | "cc" | "cxx" => "cpp",
            _ => return Ok(()), // Skip unsupported files
        };

        if !self.config.supported_languages.contains(&language.to_string()) {
            return Ok(());
        }

        let content = std::fs::read_to_string(file_path)?;
        let hash = Self::hash_content(&content);

        // Check if file has changed
        let cache_key = file_path.to_string_lossy().to_string();
        if let Some(cached) = self.index_cache.get(&cache_key) {
            if cached.hash == hash {
                return Ok(()); // Already indexed and unchanged
            }
        }

        let code_chunks = self.extract_code_chunks(&content, file_path, language)?;
        chunks.extend(code_chunks);

        // Update cache
        let cached_chunk = CodeChunk {
            file_path: file_path.to_path_buf(),
            content: content.clone(),
            line_start: 0,
            line_end: content.lines().count(),
            language: language.to_string(),
            chunk_type: CodeResultType::Other,
            context: None,
            hash,
        };

        self.index_cache.insert(cache_key, cached_chunk);

        Ok(())
    }

    fn extract_code_chunks(&self, content: &str, file_path: &Path, language: &str) -> Result<Vec<CodeChunk>, Box<dyn std::error::Error>> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Use tree-sitter for structural parsing
        let mut parser_lock = self.parser_cache.lock();
        if let Some(parser) = parser_lock.get_mut(language) {
            if let Some(tree) = parser.parse(content, None) {
                chunks.extend(self.extract_structural_chunks(&tree, content, file_path, language)?);
            }
        }

        // Fallback to line-based chunking if parser not available
        if chunks.is_empty() {
            chunks.extend(self.extract_line_based_chunks(&lines, file_path, language));
        }

        Ok(chunks)
    }

    fn extract_structural_chunks(&self, tree: &tree_sitter::Tree, content: &str, file_path: &Path, language: &str)
                                  -> Result<Vec<CodeChunk>, Box<dyn std::error::Error>> {
        let mut chunks = Vec::new();
        let mut cursor = tree.root_node().walk();

        loop {
            let node = cursor.node();

            match node.kind() {
                // Function definitions
                "function_declaration" | "fn_item" if self.config.supported_languages.contains(&language.to_string()) => {
                    if let Ok(chunk) = self.extract_function_chunk(node, content, file_path, language) {
                        chunks.push(chunk);
                    }
                }
                // Class/struct definitions
                "class_declaration" | "struct_item" | "impl_item" => {
                    if let Ok(chunk) = self.extract_structural_chunk(node, content, file_path, language, CodeResultType::Class) {
                        chunks.push(chunk);
                    }
                }
                // Import statements
                "import_statement" | "use_declaration" => {
                    if let Ok(chunk) = self.extract_structural_chunk(node, content, file_path, language, CodeResultType::Import) {
                        chunks.push(chunk);
                    }
                }
                _ => {}
            }

            if !cursor.goto_first_child() {
                if !cursor.goto_next_sibling() {
                    while !cursor.goto_parent() || !cursor.goto_next_sibling() {
                        if cursor.node().parent().is_none() {
                            break;
                        }
                    }
                }
            } else {
                continue;
            }

            if cursor.node().parent().is_none() {
                break;
            }
        }

        Ok(chunks)
    }

    fn extract_function_chunk(&self, node: tree_sitter::Node, content: &str, file_path: &Path, language: &str)
                              -> Result<CodeChunk, Box<dyn std::error::Error>> {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        let function_content = &content[start_byte..end_byte];

        let line_start = content[..start_byte].chars().filter(|&c| c == '\n').count();
        let line_end = line_start + function_content.chars().filter(|&c| c == '\n').count();

        Ok(CodeChunk {
            file_path: file_path.to_path_buf(),
            content: function_content.to_string(),
            line_start,
            line_end,
            language: language.to_string(),
            chunk_type: CodeResultType::Function,
            context: Some(Self::extract_context(content, line_start, 3)),
            hash: Self::hash_content(function_content),
        })
    }

    fn extract_structural_chunk(&self, node: tree_sitter::Node, content: &str, file_path: &Path, language: &str, chunk_type: CodeResultType)
                                 -> Result<CodeChunk, Box<dyn std::error::Error>> {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        let chunk_content = &content[start_byte..end_byte];

        let line_start = content[..start_byte].chars().filter(|&c| c == '\n').count();

        Ok(CodeChunk {
            file_path: file_path.to_path_buf(),
            content: chunk_content.to_string(),
            line_start,
            line_end: line_start + 1, // Simplified
            language: language.to_string(),
            chunk_type,
            context: None,
            hash: Self::hash_content(chunk_content),
        })
    }

    fn extract_line_based_chunks(&self, lines: &[&str], file_path: &Path, language: &str) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut start_line = 0;

        for (i, line) in lines.iter().enumerate() {
            current_chunk.push_str(line);
            current_chunk.push('\n');

            if current_chunk.len() >= self.config.max_code_chunk_length {
                chunks.push(CodeChunk {
                    file_path: file_path.to_path_buf(),
                    content: current_chunk.clone(),
                    line_start,
                    line_end: i,
                    language: language.to_string(),
                    chunk_type: CodeResultType::Other,
                    context: None,
                    hash: Self::hash_content(&current_chunk),
                });

                current_chunk = String::new();
                start_line = i + 1;
            }
        }

        // Add remaining content
        if !current_chunk.is_empty() {
            chunks.push(CodeChunk {
                file_path: file_path.to_path_buf(),
                content: current_chunk,
                line_start,
                line_end: lines.len() - 1,
                language: language.to_string(),
                chunk_type: CodeResultType::Other,
                context: None,
                hash: Self::hash_content(""),
            });
        }

        chunks
    }

    fn create_and_index_chunks_blocking(&self, chunks: Vec<CodeChunk>) -> Result<(), Box<dyn std::error::Error>> {
        if chunks.is_empty() {
            return Ok(());
        }

        // Generate embeddings for chunks using ONNX runtime
        let embeddings = self.generate_embeddings_blocking(chunks.iter().map(|c| c.content.as_str()).collect())?;

        // Convert to vector documents
        let vector_docs = chunks.into_iter().zip(embeddings.into_iter()).map(|(chunk, embedding)| {
            VectorDocument {
                id: format!("{}_{}", chunk.file_path.display(), chunk.line_start),
                vector: embedding,
                content: Some(chunk.content),
                metadata: HashMap::from([
                    ("file_path".to_string(), serde_json::Value::String(chunk.file_path.to_string_lossy().to_string())),
                    ("line_start".to_string(), serde_json::Value::Number(chunk.line_start.into())),
                    ("line_end".to_string(), serde_json::Value::Number(chunk.line_end.into())),
                    ("language".to_string(), serde_json::Value::String(chunk.language.clone())),
                    ("chunk_type".to_string(), serde_json::Value::String(format!("{:?}", chunk.chunk_type))),
                ]),
            }
        }).collect::<Vec<_>>();

        // Add to vector database in blocking context
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            self.vector_db.add_batch(vector_docs).await
        })?;

        Ok(())
    }

    fn generate_embeddings_blocking(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // In a real implementation, this would call a pre-trained embedding model
        // For now, return mock embeddings
        let embedding_size = self.config.vector_dimension;
        let mut embeddings = Vec::new();

        for _ in texts {
            // Generate mock embedding vector
            let embedding = (0..embedding_size)
                .map(|i| (i as f32 * 0.01).sin())
                .collect::<Vec<f32>>();
            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    /// Perform semantic code search
    pub async fn search(&self, request: CodeSearchRequest) -> Result<Vec<CodeSearchResult>, Box<dyn std::error::Error>> {
        // Generate embedding for the query using ONNX
        let query_embedding = self.generate_query_embedding(&request.query).await?;
        let normalized_embedding = Self::normalize_vector(&query_embedding)?;

        // Prepare vector search
        let vector_request = VectorSearchRequest {
            query_vector: normalized_embedding,
            top_k: request.max_code_chunk_length,
            filters: self.build_langauge_filters(&request.languages),
            config: Default::default(),
        };

        // Search vector database
        let vector_results = self.vector_db.search(vector_request).await?;

        // Convert to semantic search results
        let mut results = Vec::new();

        for result in vector_results {
            let file_path = result.metadata.get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let language = result.metadata.get("language")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let line_number = result.metadata.get("line_start")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;

            results.push(CodeSearchResult {
                id: result.id,
                code_snippet: result.content.unwrap_or_default(),
                file_path,
                line_number,
                language,
                score: result.score,
                result_type: CodeResultType::Other, // Would be determined from metadata
                context: vec![], // Would be populated from surrounding context
                highlights: vec![], // Would be populated with highlight spans
            });
        }

        // Apply ranking
        self.rank_results(&mut results, &request.ranking);

        Ok(results)
    }

    async fn generate_query_embedding(&self, query: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // In a real implementation, this would use the ONNX service to encode the query
        let mock_embedding = (0..self.config.vector_dimension)
            .map(|i| query.chars().nth(i % query.len()).unwrap_or(' ') as u32 as f32 / 255.0)
            .collect::<Vec<f32>>();

        Ok(mock_embedding)
    }

    fn build_langauge_filters(&self, languages: &[String]) -> Option<Vec<rust_ai_ide_vector_database::SearchFilter>> {
        if languages.is_empty() {
            return None;
        }

        Some(vec![rust_ai_ide_vector_database::SearchFilter {
            field: "language".to_string(),
            operator: rust_ai_ide_vector_database::FilterOperator::Equal,
            value: serde_json::json!(languages.first().unwrap()),
        }])
    }

    fn rank_results(&self, results: &mut [CodeSearchResult], ranking: &SearchRanking) {
        for result in results.iter_mut() {
            // Apply ranking weights
            let semantic_score = result.score * ranking.semantic_weight;
            let exact_match_score = if result.code_snippet.contains(&result.code_snippet.chars().take(5).collect::<String>()) {
                ranking.exact_match_weight
            } else {
                0.0
            };
            let distance_score = (1.0 / (result.line_number as f32 + 1.0)) * ranking.proximity_weight;
            let recency_score = ranking.recency_factor; // Simplified

            result.score = semantic_score + exact_match_score + distance_score + recency_score;
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Get indexing status
    pub async fn get_indexing_status(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let state = self.indexing_state.read().await;
        Ok(serde_json::json!({
            "is_indexing": state.is_indexing,
            "total_files": state.total_files,
            "indexed_files": state.indexed_files,
            "progress": if state.total_files > 0 {
                state.indexed_files as f64 / state.total_files as f64
            } else {
                0.0
            },
            "error_count": state.error_count
        }))
    }

    fn setup_parser(parser: &mut Parser, language: &str) -> Result<(), Box<dyn std::error::Error>> {
        match language {
            "rust" => {
                parser.set_language(&tree_sitter_rust::language())?;
            }
            "javascript" => {
                parser.set_language(&tree_sitter_javascript::language())?;
            }
            "typescript" => {
                parser.set_language(&tree_sitter_typescript::language_typescript())?;
            }
            "python" => {
                parser.set_language(&tree_sitter_python::language())?;
            }
            "go" => {
                parser.set_language(&tree_sitter_go::language())?;
            }
            "java" => {
                parser.set_language(&tree_sitter_java::language())?;
            }
            "cpp" => {
                parser.set_language(&tree_sitter_cpp::language())?;
            }
            _ => {
                return Err(format!("Unsupported language: {}", language).into());
            }
        }

        Ok(())
    }

    fn extract_context(content: &str, line_number: usize, context_lines: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let start = line_number.saturating_sub(context_lines);
        let end = (line_number + context_lines + 1).min(lines.len());

        lines[start..end].join("\n")
    }

    fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    fn normalize_vector(vector: &[f32]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm == 0.0 {
            return Err("Cannot normalize zero vector".into());
        }

        let normalized = vector.iter().map(|x| x / norm).collect();
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use rust_ai_ide_vector_database::VectorIndexConfig;
    use rust_ai_ide_onnx_runtime::ONNXConfig;

    #[tokio::test]
    async fn test_semantic_search_initialization() {
        let config = SemanticSearchConfig::default();
        let vector_config = VectorIndexConfig::default();
        let onnx_config = ONNXConfig::default();

        let vector_db = Arc::new(VectorDatabase::new(vector_config).await.unwrap());
        let onnx_service = Arc::new(ONNXInferenceService::new(onnx_config));

        let search_engine = SemanticSearchEngine::new(config, vector_db, onnx_service).await.unwrap();
        assert!(!search_engine.config.supported_languages.is_empty());
    }

    #[tokio::test]
    async fn test_code_chunking() {
        let config = SemanticSearchConfig::default();
        let vector_config = VectorIndexConfig::default();
        let onnx_config = ONNXConfig::default();

        let vector_db = Arc::new(VectorDatabase::new(vector_config).await.unwrap());
        let onnx_service = Arc::new(ONNXInferenceService::new(onnx_config));

        let search_engine = SemanticSearchEngine::new(config, vector_db, onnx_service).await.unwrap();

        let rust_code = r#"
        fn main() {
            println!("Hello, World!");
        }
        "#;

        let file_path = PathBuf::from("test.rs");
        let chunks = search_engine.extract_line_based_chunks(&[rust_code.lines().next().unwrap()], &file_path, "rust");
        assert!(!chunks.is_empty());
    }

    #[tokio::test]
    async fn test_search_request() {
        let config = SemanticSearchConfig::default();
        let vector_config = VectorIndexConfig::default();
        let onnx_config = ONNXConfig::default();

        let vector_db = Arc::new(VectorDatabase::new(vector_config).await.unwrap());
        let onnx_service = Arc::new(ONNXInferenceService::new(onnx_config));

        let search_engine = SemanticSearchEngine::new(config, vector_db, onnx_service).await.unwrap();

        let request = CodeSearchRequest {
            query: "function".to_string(),
            ..Default::default()
        };

        // Should not panic even with empty database
        let _results = search_engine.search(request).await;
    }
}