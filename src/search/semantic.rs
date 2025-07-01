use crate::core::LocalEmbedder;
use crate::storage::ChunkRecord;
use crate::text::TextChunk;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

/// Semantic search using embeddings
pub struct SemanticSearch {
    embedder: Arc<LocalEmbedder>,
    similarity_threshold: f32,
    advanced_mode: bool,
}

/// Search result with semantic similarity
#[derive(Debug, Clone)]
pub struct SemanticSearchResult {
    pub chunk: ChunkRecord,
    pub similarity_score: f32,
    pub query_embedding: Vec<f32>,
    pub chunk_embedding: Vec<f32>,
}

impl SemanticSearch {
    /// Create a new semantic search instance
    pub fn new(embedder: Arc<LocalEmbedder>) -> Self {
        Self {
            embedder,
            similarity_threshold: 0.7,
            advanced_mode: false,
        }
    }

    /// Create with advanced mode enabled
    pub fn with_advanced_mode(embedder: Arc<LocalEmbedder>, advanced_mode: bool) -> Self {
        Self {
            embedder,
            similarity_threshold: 0.7,
            advanced_mode,
        }
    }

    /// Create with custom similarity threshold
    pub fn with_threshold(embedder: Arc<LocalEmbedder>, threshold: f32) -> Self {
        Self {
            embedder,
            similarity_threshold: threshold,
            advanced_mode: false,
        }
    }

    /// Search semantically similar chunks
    pub fn search(
        &self,
        query: &str,
        chunks: &[ChunkRecord],
        max_results: usize,
    ) -> Result<Vec<SemanticSearchResult>> {
        if !self.embedder.has_vocabulary() {
            return Err(anyhow::anyhow!("Embedder vocabulary not built"));
        }

        // Generate query embedding
        let query_embedding = self.embedder.embed(query)?;

        let mut results = Vec::new();

        for chunk in chunks {
            if let Some(ref embedding_data) = chunk.embedding {
                let similarity = LocalEmbedder::similarity(&query_embedding, embedding_data);

                if similarity >= self.similarity_threshold {
                    results.push(SemanticSearchResult {
                        chunk: chunk.clone(),
                        similarity_score: similarity,
                        query_embedding: query_embedding.clone(),
                        chunk_embedding: embedding_data.clone(),
                    });
                }
            }
        }

        // Sort by similarity score (descending)
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        results.truncate(max_results);

        Ok(results)
    }

    /// Search with custom similarity calculation
    pub fn search_with_reranking(
        &self,
        query: &str,
        chunks: &[ChunkRecord],
        max_results: usize,
        rerank_fn: impl Fn(&SemanticSearchResult, &str) -> f32,
    ) -> Result<Vec<SemanticSearchResult>> {
        let mut initial_results = self.search(query, chunks, max_results * 2)?;

        // Apply reranking function
        for result in &mut initial_results {
            let boosted_score = rerank_fn(result, query);
            result.similarity_score = boosted_score;
        }

        // Re-sort and limit
        initial_results
            .sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        initial_results.truncate(max_results);

        Ok(initial_results)
    }

    /// Get embedding for a text chunk
    pub fn embed_chunk(&self, chunk: &TextChunk) -> Result<Vec<f32>> {
        self.embedder.embed(&chunk.content)
    }

    /// Batch embed multiple chunks
    pub fn embed_chunks(&self, chunks: &[TextChunk]) -> Result<Vec<Vec<f32>>> {
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        self.embedder.embed_batch(&texts)
    }

    /// Update similarity threshold
    pub fn set_similarity_threshold(&mut self, threshold: f32) {
        self.similarity_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Get current similarity threshold
    pub fn similarity_threshold(&self) -> f32 {
        self.similarity_threshold
    }

    /// Check if semantic search is ready
    pub fn is_ready(&self) -> bool {
        self.embedder.has_vocabulary()
    }

    /// Get embedding dimension
    pub fn embedding_dimension(&self) -> usize {
        if self.embedder.has_vocabulary() {
            self.embedder.vocabulary_size()
        } else {
            0
        }
    }

    /// Get a reference to the embedder
    pub fn embedder(&self) -> &Arc<LocalEmbedder> {
        &self.embedder
    }

    /// Database-aware semantic search that tries pre-indexed embeddings first
    pub async fn search_with_database_fallback(
        &self,
        query: &str,
        files: &[PathBuf],
        max_results: usize,
    ) -> Result<Vec<crate::SearchResult>> {
        // First, try to use pre-indexed embeddings from database
        if let Some(home_dir) = dirs::home_dir() {
            let database_path = home_dir.join(".semisearch").join("search.db");
            if let Ok(database) = crate::storage::Database::new(&database_path) {
                if let Ok(indexed_chunks) = database.get_chunks_with_embeddings() {
                    if !indexed_chunks.is_empty() {
                        if self.advanced_mode {
                            println!(
                                "🔍 Using {} pre-indexed chunks from database",
                                indexed_chunks.len()
                            );
                        }

                        // Filter chunks to only include files in our search scope
                        let file_paths: std::collections::HashSet<String> = files
                            .iter()
                            .filter_map(|p| p.to_str())
                            .map(|s| s.to_string())
                            .collect();

                        let relevant_chunks: Vec<_> = indexed_chunks
                            .into_iter()
                            .filter(|chunk| {
                                // Check if chunk's file path matches any of our search paths
                                file_paths.iter().any(|search_path| {
                                    // Handle both exact matches and directory containment
                                    chunk.file_path.starts_with(search_path)
                                        || search_path.starts_with(&chunk.file_path)
                                        || chunk.file_path.contains(search_path)
                                })
                            })
                            .collect();

                        if !relevant_chunks.is_empty() {
                            let semantic_results =
                                self.search(query, &relevant_chunks, max_results)?;

                            let mut results = Vec::new();
                            for result in semantic_results {
                                results.push(crate::SearchResult {
                                    file_path: result.chunk.file_path.clone(),
                                    line_number: result.chunk.line_number,
                                    content: result.chunk.content.clone(),
                                    score: Some(result.similarity_score),
                                    match_type: Some(crate::MatchType::Semantic),
                                    context_before: None,
                                    context_after: None,
                                });
                            }

                            if !results.is_empty() {
                                if self.advanced_mode {
                                    println!(
                                        "✅ Found {} matches using indexed embeddings",
                                        results.len()
                                    );
                                }
                                return Ok(results);
                            }
                        }
                    }
                }
            }
        }

        // Fallback: on-demand embedding generation for unindexed files
        if self.advanced_mode {
            eprintln!("⚠️  No indexed embeddings found, generating on-demand (slower)");
        }
        self.search_on_demand(query, files, max_results).await
    }

    /// Fallback semantic search with on-demand embedding generation
    async fn search_on_demand(
        &self,
        query: &str,
        files: &[PathBuf],
        max_results: usize,
    ) -> Result<Vec<crate::SearchResult>> {
        let mut all_results = Vec::new();

        // Limit the number of files to process to prevent hanging
        let max_files = 10; // Reasonable limit for on-demand semantic search
        let files_to_process = if files.len() > max_files {
            if self.advanced_mode {
                eprintln!(
                    "Note: Limiting on-demand semantic search to first {} files (found {} total)",
                    max_files,
                    files.len()
                );
            }
            &files[..max_files]
        } else {
            files
        };

        // For each file, read content and search semantically
        for file_path in files_to_process {
            let path_str = file_path.to_str().unwrap_or("");

            // Skip binary files
            if let Some(ext) = file_path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if matches!(
                    ext_str.as_str(),
                    "exe" | "dll" | "so" | "dylib" | "bin" | "obj"
                ) {
                    continue;
                }
            }

            // Read file content
            let content = match std::fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue, // Skip unreadable files
            };

            // Split into chunks (simple line-based for now)
            let lines: Vec<&str> = content.lines().collect();

            // Limit lines per file to prevent hanging on large files
            let max_lines_per_file = 50;
            let lines_to_process = if lines.len() > max_lines_per_file {
                &lines[..max_lines_per_file]
            } else {
                &lines[..]
            };

            // Create chunk records for semantic search
            let mut chunks = Vec::new();
            for (idx, line) in lines_to_process.iter().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }

                // Create a simple chunk record
                let chunk = crate::storage::ChunkRecord {
                    id: idx as i64,
                    file_id: 0, // Not used here
                    file_path: path_str.to_string(),
                    line_number: idx + 1,
                    start_char: 0,
                    end_char: line.len(),
                    content: line.to_string(),
                    embedding: None, // Will be generated by semantic search
                };
                chunks.push(chunk);
            }

            // Perform semantic search on chunks
            if !chunks.is_empty() {
                if self.advanced_mode {
                    let total_chunks = chunks.len();
                    eprint!("🔍 Processing {path_str}: ");
                    for (idx, chunk) in chunks.iter_mut().enumerate() {
                        if chunk.embedding.is_none() {
                            // Show progress for every 10 chunks or on the last chunk
                            if idx % 10 == 0 || idx == total_chunks - 1 {
                                eprint!("{}/{} ", idx + 1, total_chunks);
                                std::io::Write::flush(&mut std::io::stderr()).unwrap_or(());
                            }

                            match self.embedder.embed(&chunk.content) {
                                Ok(embedding) => chunk.embedding = Some(embedding),
                                Err(_) => continue,
                            }
                        }
                    }
                    eprintln!("✓");
                } else {
                    // Processing chunks silently
                    for chunk in chunks.iter_mut() {
                        if chunk.embedding.is_none() {
                            match self.embedder.embed(&chunk.content) {
                                Ok(embedding) => chunk.embedding = Some(embedding),
                                Err(_) => continue,
                            }
                        }
                    }
                }

                // Search with lower threshold for better recall
                let semantic_results = self.search(query, &chunks, 10)?;

                // Convert to SearchResult
                for result in semantic_results {
                    all_results.push(crate::SearchResult {
                        file_path: result.chunk.file_path.clone(),
                        line_number: result.chunk.line_number,
                        content: result.chunk.content.clone(),
                        score: Some(result.similarity_score),
                        match_type: Some(crate::MatchType::Semantic),
                        context_before: None,
                        context_after: None,
                    });
                }
            }
        }

        // Sort and limit results
        all_results.sort_by(|a, b| {
            let score_a = a.score.unwrap_or(0.0);
            let score_b = b.score.unwrap_or(0.0);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all_results.truncate(max_results);

        Ok(all_results)
    }
}

/// Semantic search options
#[derive(Debug, Clone)]
pub struct SemanticSearchOptions {
    pub similarity_threshold: f32,
    pub max_results: usize,
    pub enable_reranking: bool,
    pub boost_exact_matches: bool,
    pub boost_recent_files: bool,
}

impl Default for SemanticSearchOptions {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.7,
            max_results: 50,
            enable_reranking: false,
            boost_exact_matches: true,
            boost_recent_files: false,
        }
    }
}

/// Reranking functions for semantic search
pub struct SemanticReranker;

impl SemanticReranker {
    /// Boost results that contain exact query terms
    pub fn boost_exact_matches(result: &SemanticSearchResult, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let content_lower = result.chunk.content.to_lowercase();

        let mut boosted_score = result.similarity_score;

        // Boost for exact phrase match
        if content_lower.contains(&query_lower) {
            boosted_score *= 1.2;
        }

        // Boost for individual word matches
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let matching_words = query_words
            .iter()
            .filter(|&&word| content_lower.contains(word))
            .count();

        if matching_words > 0 {
            let match_ratio = matching_words as f32 / query_words.len() as f32;
            boosted_score *= 1.0 + (match_ratio * 0.1);
        }

        boosted_score.min(1.0)
    }

    /// Boost results from recently modified files
    pub fn boost_recent_files(result: &SemanticSearchResult, _query: &str) -> f32 {
        // This would need file modification time from chunk metadata
        // For now, return unchanged score
        result.similarity_score
    }

    /// Combined reranking with multiple factors
    pub fn combined_reranking(result: &SemanticSearchResult, query: &str) -> f32 {
        let exact_boosted = Self::boost_exact_matches(result, query);
        let recent_boosted = Self::boost_recent_files(result, query);

        // Combine boosts (taking maximum)
        exact_boosted.max(recent_boosted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(target_os = "windows"))] // Skip on Windows due to ONNX Runtime issues
    use crate::text::TextChunk;

    #[cfg(not(target_os = "windows"))] // Skip on Windows due to ONNX Runtime issues
    async fn create_test_embedder() -> Arc<LocalEmbedder> {
        use crate::core::EmbeddingConfig;
        let config = EmbeddingConfig::default();
        let mut embedder = LocalEmbedder::new(config).await.unwrap();

        let documents = vec![
            "machine learning algorithms".to_string(),
            "artificial intelligence research".to_string(),
            "data science projects".to_string(),
            "neural network training".to_string(),
        ];

        embedder.build_vocabulary(&documents).unwrap();
        Arc::new(embedder)
    }

    #[cfg(not(target_os = "windows"))] // Skip on Windows due to ONNX Runtime issues
    fn create_test_chunks() -> Vec<ChunkRecord> {
        vec![
            ChunkRecord {
                id: 1,
                file_id: 1,
                file_path: "ai.md".to_string(),
                line_number: 1,
                start_char: 0,
                end_char: 25,
                content: "machine learning is great".to_string(),
                embedding: Some(vec![0.1, 0.2, 0.3, 0.4]),
            },
            ChunkRecord {
                id: 2,
                file_id: 1,
                file_path: "ai.md".to_string(),
                line_number: 2,
                start_char: 26,
                end_char: 50,
                content: "artificial intelligence rocks".to_string(),
                embedding: Some(vec![0.2, 0.3, 0.4, 0.5]),
            },
            ChunkRecord {
                id: 3,
                file_id: 2,
                file_path: "data.md".to_string(),
                line_number: 1,
                start_char: 0,
                end_char: 20,
                content: "data analysis tools".to_string(),
                embedding: Some(vec![0.0, 0.1, 0.0, 0.1]),
            },
        ]
    }

    #[tokio::test]
    #[cfg(not(target_os = "windows"))] // Skip on Windows due to ONNX Runtime issues
    async fn test_semantic_search_creation() {
        let embedder = create_test_embedder().await;
        let semantic_search = SemanticSearch::new(embedder);

        assert_eq!(semantic_search.similarity_threshold(), 0.7);
        assert!(semantic_search.is_ready());
    }

    #[tokio::test]
    #[cfg(not(target_os = "windows"))] // Skip on Windows due to ONNX Runtime issues
    async fn test_semantic_search_with_threshold() {
        let embedder = create_test_embedder().await;
        let semantic_search = SemanticSearch::with_threshold(embedder, 0.5);

        assert_eq!(semantic_search.similarity_threshold(), 0.5);
    }

    #[tokio::test]
    #[cfg(not(target_os = "windows"))] // Skip on Windows due to ONNX Runtime issues
    async fn test_semantic_search_basic() {
        let embedder = create_test_embedder().await;
        let semantic_search = SemanticSearch::with_threshold(embedder, 0.0); // Low threshold for testing
        let chunks = create_test_chunks();

        let results = semantic_search
            .search("machine learning", &chunks, 10)
            .unwrap();

        assert!(!results.is_empty());
        assert!(results[0].similarity_score >= 0.0);
        assert!(results[0].similarity_score <= 1.0);
    }

    #[tokio::test]
    #[cfg(not(target_os = "windows"))] // Skip on Windows due to ONNX Runtime issues
    async fn test_semantic_search_sorting() {
        let embedder = create_test_embedder().await;
        let semantic_search = SemanticSearch::with_threshold(embedder, 0.0);
        let chunks = create_test_chunks();

        let results = semantic_search
            .search("artificial intelligence", &chunks, 10)
            .unwrap();

        // Results should be sorted by similarity score (descending)
        for i in 1..results.len() {
            assert!(results[i - 1].similarity_score >= results[i].similarity_score);
        }
    }

    #[tokio::test]
    #[cfg(not(target_os = "windows"))] // Skip on Windows due to ONNX Runtime issues
    async fn test_embed_chunk() {
        let embedder = create_test_embedder().await;
        let semantic_search = SemanticSearch::new(embedder);

        let chunk = TextChunk {
            line_number: 1,
            content: "machine learning algorithms".to_string(),
            tokens: vec!["machine".to_string(), "learning".to_string()],
            start_char: 0,
            end_char: 26,
            language_hint: None,
        };

        let embedding = semantic_search.embed_chunk(&chunk).unwrap();
        assert!(!embedding.is_empty());
    }

    #[test]
    fn test_reranking_exact_matches() {
        let chunk = ChunkRecord {
            id: 1,
            file_id: 1,
            file_path: "test.md".to_string(),
            line_number: 1,
            start_char: 0,
            end_char: 25,
            content: "machine learning is great".to_string(),
            embedding: Some(vec![0.1, 0.2, 0.3]),
        };

        let result = SemanticSearchResult {
            chunk,
            similarity_score: 0.8,
            query_embedding: vec![0.1, 0.2, 0.3],
            chunk_embedding: vec![0.1, 0.2, 0.3],
        };

        let boosted_score = SemanticReranker::boost_exact_matches(&result, "machine learning");
        assert!(boosted_score > result.similarity_score);
    }

    #[tokio::test]
    #[cfg(not(target_os = "windows"))] // Skip on Windows due to ONNX Runtime issues
    async fn test_semantic_search_options() {
        let options = SemanticSearchOptions::default();
        assert_eq!(options.similarity_threshold, 0.7);
        assert_eq!(options.max_results, 50);
        assert!(options.boost_exact_matches);
    }

    #[tokio::test]
    #[cfg(not(target_os = "windows"))] // Skip on Windows due to ONNX Runtime issues
    async fn test_embedding_dimension() {
        let embedder = create_test_embedder().await;
        let semantic_search = SemanticSearch::new(embedder);

        let dim = semantic_search.embedding_dimension();
        assert!(dim > 0);
    }

    #[tokio::test]
    #[cfg(not(target_os = "windows"))] // Skip on Windows due to ONNX Runtime issues
    async fn test_threshold_clamping() {
        let embedder = create_test_embedder().await;
        let mut semantic_search = SemanticSearch::new(embedder);

        semantic_search.set_similarity_threshold(-0.5);
        assert_eq!(semantic_search.similarity_threshold(), 0.0);

        semantic_search.set_similarity_threshold(1.5);
        assert_eq!(semantic_search.similarity_threshold(), 1.0);
    }
}
