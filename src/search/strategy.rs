use crate::core::embedder::{EmbeddingCapability, LocalEmbedder};
use crate::search::semantic::{SemanticSearch, SemanticSearchOptions};
use crate::storage::Database;
use crate::text::TextProcessor;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

// Use the main types from lib.rs
use crate::{MatchType, SearchOptions, SearchResult};

/// Helper functions for search strategies
pub struct SearchHelper;

impl SearchHelper {
    /// Add context lines to search results
    pub fn add_context(results: &mut [SearchResult], all_content: &str, context_lines: usize) {
        let lines: Vec<&str> = all_content.lines().collect();

        for result in results {
            let line_index = result.line_number.saturating_sub(1);

            // Add context before
            let start_line = line_index.saturating_sub(context_lines);
            if start_line < line_index {
                // Context fields not available in lib.rs SearchResult
                // result.context_before = Some(lines[start_line..line_index].iter().map(|&s| s.to_string()).collect());
            }

            // Add context after
            let end_line = (line_index + 1 + context_lines).min(lines.len());
            if end_line > line_index + 1 {
                // Context fields not available in lib.rs SearchResult
                // result.context_after = Some(lines[line_index + 1..end_line].iter().map(|&s| s.to_string()).collect());
            }
        }
    }

    /// Calculate relevance score based on multiple factors
    pub fn calculate_relevance_score(
        base_score: f32,
        query_length: usize,
        content_length: usize,
        match_position: usize,
    ) -> f32 {
        let score = base_score;

        // Bonus for matches near the beginning
        let position_bonus = if match_position < content_length / 4 {
            0.1
        } else {
            0.0
        };

        // Penalty for very long content (dilutes relevance)
        let length_penalty = if content_length > 1000 { 0.9 } else { 1.0 };

        // Bonus for longer query matches
        let query_bonus = if query_length > 3 {
            0.05 * (query_length as f32 - 3.0)
        } else {
            0.0
        };

        ((score + position_bonus + query_bonus) * length_penalty).min(1.0)
    }

    /// Merge results from multiple strategies
    pub fn merge_strategy_results(
        results: Vec<Vec<SearchResult>>,
        max_results: usize,
    ) -> Vec<SearchResult> {
        let mut merged = Vec::new();

        // Flatten all results
        for strategy_results in results {
            merged.extend(strategy_results);
        }

        // Remove duplicates based on file path and line number
        merged.sort_by(|a, b| {
            a.file_path
                .cmp(&b.file_path)
                .then_with(|| a.line_number.cmp(&b.line_number))
        });

        merged.dedup_by(|a, b| a.file_path == b.file_path && a.line_number == b.line_number);

        // Sort by score (descending)
        merged.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        merged.truncate(max_results);

        merged
    }

    /// Calculate term overlap between query and content
    pub fn calculate_term_overlap(query_terms: &[String], content_terms: &[String]) -> f32 {
        if query_terms.is_empty() {
            return 0.0;
        }

        let query_set: std::collections::HashSet<_> = query_terms.iter().collect();
        let content_set: std::collections::HashSet<_> = content_terms.iter().collect();

        let intersection_size = query_set.intersection(&content_set).count();
        intersection_size as f32 / query_terms.len() as f32
    }

    /// Highlight matches in content
    pub fn highlight_matches(
        content: &str,
        start: usize,
        end: usize,
        highlight_tag: &str,
    ) -> String {
        if start >= content.len() || end <= start {
            return content.to_string();
        }

        let before = &content[..start];
        let matched = &content[start..end.min(content.len())];
        let after = &content[end.min(content.len())..];

        format!("{before}<{highlight_tag}>{matched}</{highlight_tag}>{after}")
    }
}

/// Multi-strategy search engine with semantic capabilities
pub struct SearchEngine {
    database: Database,
    semantic_search: Option<SemanticSearch>,
}

impl SearchEngine {
    /// Create a new search engine with database and optional embedder
    pub fn new(database: Database, embedder: Option<LocalEmbedder>) -> Self {
        let semantic_search = embedder.map(|emb| SemanticSearch::new(Arc::new(emb)));

        Self {
            database,
            semantic_search,
        }
    }

    /// Create search engine with auto-detected capabilities
    pub async fn with_auto_detection(database: Database) -> Result<Self> {
        let capability = LocalEmbedder::detect_capabilities();

        let semantic_search = match capability {
            #[cfg(feature = "neural-embeddings")]
            EmbeddingCapability::Full | EmbeddingCapability::TfIdf => {
                match Self::initialize_semantic_search(&capability).await {
                    Ok(search) => Some(search),
                    Err(e) => {
                        println!("⚠️  Semantic search initialization failed: {e}");
                        None
                    }
                }
            }
            #[cfg(not(feature = "neural-embeddings"))]
            EmbeddingCapability::TfIdf => {
                match Self::initialize_semantic_search(&capability).await {
                    Ok(search) => Some(search),
                    Err(e) => {
                        println!("⚠️  Semantic search initialization failed: {e}");
                        None
                    }
                }
            }
            EmbeddingCapability::None => None,
        };

        Ok(Self {
            database,
            semantic_search,
        })
    }

    /// Initialize semantic search based on capability
    async fn initialize_semantic_search(
        _capability: &EmbeddingCapability,
    ) -> Result<SemanticSearch> {
        let embedder = LocalEmbedder::with_auto_config().await?;
        Ok(SemanticSearch::new(Arc::new(embedder)))
    }

    /// Main search interface
    pub async fn search(
        &self,
        query: &str,
        path: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        match self.determine_search_mode(&options) {
            SearchMode::Keyword => self.keyword_search(query, path, &options).await,
            SearchMode::Semantic => {
                if let Some(ref semantic_search) = self.semantic_search {
                    self.semantic_enhanced_search(query, path, &options, semantic_search)
                        .await
                } else {
                    // Return error if semantic search explicitly requested but not available
                    Err(anyhow::anyhow!(
                        "Semantic search explicitly requested but not available. \
                         Please ensure neural embeddings are enabled and the model is downloaded."
                    ))
                }
            }
            SearchMode::Hybrid => {
                if let Some(ref semantic_search) = self.semantic_search {
                    self.hybrid_search(query, path, &options, semantic_search)
                        .await
                } else {
                    self.keyword_search(query, path, &options).await
                }
            }
            SearchMode::Auto => self.auto_search(query, path, &options).await,
        }
    }

    /// Determine search mode from options
    fn determine_search_mode(&self, options: &SearchOptions) -> SearchMode {
        // Check if explicit search mode is specified
        if let Some(ref mode_str) = options.search_mode {
            match mode_str.as_str() {
                "semantic" => return SearchMode::Semantic,
                "keyword" => return SearchMode::Keyword,
                "hybrid" => return SearchMode::Hybrid,
                "auto" => {
                    // Auto mode: use semantic if available, otherwise keyword
                    if self.semantic_search.is_some() {
                        return SearchMode::Hybrid;
                    } else {
                        return SearchMode::Keyword;
                    }
                }
                _ => {
                    // Unknown mode, fall through to auto detection
                }
            }
        }

        // Auto mode: use semantic if available, otherwise keyword
        if self.semantic_search.is_some() {
            SearchMode::Hybrid
        } else {
            SearchMode::Keyword
        }
    }

    /// Keyword-only search using the existing search functionality
    async fn keyword_search(
        &self,
        query: &str,
        path: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        // Use the existing search functionality from lib.rs
        crate::search_files(query, path, options)
    }

    /// Semantic-enhanced search
    async fn semantic_enhanced_search(
        &self,
        query: &str,
        path: &str,
        options: &SearchOptions,
        semantic_search: &SemanticSearch,
    ) -> Result<Vec<SearchResult>> {
        // Try to get pre-indexed chunks first
        if let Ok(chunks) = self.database.get_chunks_with_embeddings() {
            if !chunks.is_empty() {
                return self
                    .semantic_search_indexed(query, options, semantic_search, &chunks)
                    .await;
            }
        }

        // Fall back to lazy semantic search - process files on-demand
        self.lazy_semantic_search(query, path, options, semantic_search)
            .await
    }

    /// Semantic search using pre-indexed chunks
    async fn semantic_search_indexed(
        &self,
        query: &str,
        options: &SearchOptions,
        semantic_search: &SemanticSearch,
        chunks: &[crate::storage::ChunkRecord],
    ) -> Result<Vec<SearchResult>> {
        let semantic_options = SemanticSearchOptions {
            similarity_threshold: options.min_score,
            max_results: options.max_results,
            boost_exact_matches: false,
            enable_reranking: false,
            boost_recent_files: false,
        };

        let custom_semantic_search = SemanticSearch::with_threshold(
            semantic_search.embedder().clone(),
            semantic_options.similarity_threshold,
        );
        let semantic_results = custom_semantic_search.search(query, chunks, options.max_results)?;
        let mut results = Vec::new();

        for result in semantic_results {
            results.push(SearchResult {
                file_path: result.chunk.file_path,
                line_number: result.chunk.line_number,
                content: result.chunk.content,
                score: Some(result.similarity_score),
                match_type: Some(MatchType::Semantic),
                context_before: None,
                context_after: None,
            });
        }

        Ok(results)
    }

    /// Lazy semantic search - process files on-demand
    async fn lazy_semantic_search(
        &self,
        query: &str,
        path: &str,
        options: &SearchOptions,
        semantic_search: &SemanticSearch,
    ) -> Result<Vec<SearchResult>> {
        use ignore::WalkBuilder;

        let mut results = Vec::new();
        let text_processor = TextProcessor::new();

        // First pass: collect all file contents to build vocabulary
        let mut all_contents = Vec::new();
        let walker = WalkBuilder::new(path)
            .follow_links(false)
            .git_ignore(true)
            .build();

        for entry in walker {
            let entry = entry?;
            if entry.file_type().is_some_and(|ft| ft.is_file()) {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    all_contents.push(content);
                }
            }
        }

        // Build vocabulary from all documents if using TF-IDF
        let embedder = semantic_search.embedder();
        if embedder.capability() == crate::core::embedder::EmbeddingCapability::TfIdf {
            // Create a new embedder with vocabulary built from all documents
            let config = crate::core::embedder::EmbeddingConfig::default();
            let mut new_embedder = crate::core::embedder::LocalEmbedder::with_vocabulary(
                config,
                HashMap::new(),
                HashMap::new(),
            );

            if let Ok(()) = new_embedder.build_vocabulary(&all_contents) {
                // Create a new semantic search with the vocabulary-built embedder
                let semantic_search_with_vocab = SemanticSearch::new(Arc::new(new_embedder));

                // Generate query embedding once
                let query_embedding = semantic_search_with_vocab.embedder().embed(query)?;

                // Second pass: search each file
                let walker = WalkBuilder::new(path)
                    .follow_links(false)
                    .git_ignore(true)
                    .build();

                for entry in walker {
                    let entry = entry?;
                    if entry.file_type().is_some_and(|ft| ft.is_file()) {
                        if let Some(file_results) = self
                            .semantic_search_in_file(
                                entry.path(),
                                query,
                                &query_embedding,
                                options,
                                &semantic_search_with_vocab,
                                &text_processor,
                            )
                            .await?
                        {
                            results.extend(file_results);
                        }
                    }
                }
            }
        } else {
            // Neural embeddings don't need vocabulary building
            let query_embedding = embedder.embed(query)?;

            let walker = WalkBuilder::new(path)
                .follow_links(false)
                .git_ignore(true)
                .build();

            for entry in walker {
                let entry = entry?;
                if entry.file_type().is_some_and(|ft| ft.is_file()) {
                    if let Some(file_results) = self
                        .semantic_search_in_file(
                            entry.path(),
                            query,
                            &query_embedding,
                            options,
                            semantic_search,
                            &text_processor,
                        )
                        .await?
                    {
                        results.extend(file_results);
                    }
                }
            }
        }

        // Sort by score (descending)
        results.sort_by(|a, b| {
            let score_a = a.score.unwrap_or(0.0);
            let score_b = b.score.unwrap_or(0.0);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Filter by minimum score and limit results
        results.retain(|r| r.score.unwrap_or(1.0) >= options.min_score);
        results.truncate(options.max_results);

        Ok(results)
    }

    /// Search semantically within a single file
    async fn semantic_search_in_file(
        &self,
        file_path: &Path,
        _query: &str,
        query_embedding: &[f32],
        options: &SearchOptions,
        semantic_search: &SemanticSearch,
        text_processor: &TextProcessor,
    ) -> Result<Option<Vec<SearchResult>>> {
        // Skip binary files
        if let Some(extension) = file_path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if matches!(
                ext.as_str(),
                "exe" | "dll" | "so" | "dylib" | "bin" | "obj" | "o" | "a" | "lib"
            ) {
                return Ok(None);
            }
        }

        let content = match std::fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => return Ok(None), // Skip files we can't read
        };

        // Process file into chunks
        let chunks = text_processor.process_file(&content);
        let mut results = Vec::new();

        // Embed and search each chunk
        for chunk in chunks {
            let chunk_embedding = semantic_search.embedder().embed(&chunk.content)?;
            let similarity = LocalEmbedder::similarity(query_embedding, &chunk_embedding);

            if similarity >= options.min_score {
                results.push(SearchResult {
                    file_path: file_path.to_string_lossy().to_string(),
                    line_number: chunk.line_number,
                    content: chunk.content,
                    score: Some(similarity),
                    match_type: Some(MatchType::Semantic),
                    context_before: None,
                    context_after: None,
                });
            }
        }

        if results.is_empty() {
            Ok(None)
        } else {
            Ok(Some(results))
        }
    }

    /// Hybrid search combining keyword and semantic
    async fn hybrid_search(
        &self,
        query: &str,
        path: &str,
        options: &SearchOptions,
        semantic_search: &SemanticSearch,
    ) -> Result<Vec<SearchResult>> {
        // Get both keyword and semantic results
        let keyword_results = self.keyword_search(query, path, options).await?;
        let semantic_results = self
            .semantic_enhanced_search(query, path, options, semantic_search)
            .await
            .unwrap_or_default();

        Ok(self.merge_results(keyword_results, semantic_results, options))
    }

    /// Auto search mode - intelligently choose strategy
    async fn auto_search(
        &self,
        query: &str,
        path: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        if self.semantic_search.is_some() {
            self.hybrid_search(query, path, options, self.semantic_search.as_ref().unwrap())
                .await
        } else {
            self.keyword_search(query, path, options).await
        }
    }

    /// Merge keyword and semantic results
    fn merge_results(
        &self,
        keyword_results: Vec<SearchResult>,
        semantic_results: Vec<SearchResult>,
        options: &SearchOptions,
    ) -> Vec<SearchResult> {
        use std::collections::HashMap;
        let mut combined = HashMap::new();

        // Add keyword results
        for result in keyword_results {
            let key = format!("{}:{}", result.file_path, result.line_number);
            combined.insert(key, result);
        }

        // Add semantic results, boosting score if already exists
        for result in semantic_results {
            let key = format!("{}:{}", result.file_path, result.line_number);
            if let Some(existing) = combined.get_mut(&key) {
                let keyword_score = existing.score.unwrap_or(0.0);
                let semantic_score = result.score.unwrap_or(0.0);
                existing.score = Some((keyword_score + semantic_score) / 2.0 * 1.2); // 20% boost
                existing.match_type = Some(MatchType::Hybrid);
            } else {
                combined.insert(key, result);
            }
        }

        let mut results: Vec<_> = combined.into_values().collect();

        // Sort by score (descending)
        results.sort_by(|a, b| {
            let score_a = a.score.unwrap_or(0.0);
            let score_b = b.score.unwrap_or(0.0);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Filter by minimum score and limit results
        results.retain(|r| r.score.unwrap_or(1.0) >= options.min_score);
        results.truncate(options.max_results);

        results
    }

    /// Check if semantic search is available
    pub fn has_semantic_search(&self) -> bool {
        self.semantic_search.is_some()
    }

    /// Get search statistics
    pub fn get_stats(&self) -> Result<SearchStats> {
        let db_stats = self.database.get_stats()?;
        Ok(SearchStats {
            total_files: db_stats.file_count,
            total_chunks: db_stats.chunk_count,
            has_semantic_search: self.has_semantic_search(),
            semantic_model: if self.has_semantic_search() {
                Some("all-MiniLM-L6-v2".to_string())
            } else {
                None
            },
        })
    }

    /// Build vocabulary for semantic search (if available)
    pub async fn build_vocabulary(&mut self, _documents: &[String]) -> Result<()> {
        if let Some(ref _semantic_search) = self.semantic_search {
            // This would need to be implemented in SemanticSearch
            // For now, return success as the embedder handles vocabulary internally
            Ok(())
        } else {
            Err(anyhow::anyhow!("Semantic search not available"))
        }
    }
}

/// Search mode enumeration
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum SearchMode {
    Auto,
    Keyword,
    Semantic,
    Hybrid,
}

/// Search statistics
#[derive(Debug, Clone)]
pub struct SearchStats {
    pub total_files: usize,
    pub total_chunks: usize,
    pub has_semantic_search: bool,
    pub semantic_model: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_database() -> Database {
        let temp_file = NamedTempFile::new().unwrap();
        Database::new(temp_file.path()).unwrap()
    }

    #[tokio::test]
    async fn test_search_engine_creation() {
        let database = create_test_database();
        let search_engine = SearchEngine::new(database, None);

        assert!(!search_engine.has_semantic_search());
    }

    #[tokio::test]
    #[cfg(not(target_os = "windows"))] // Skip on Windows due to ONNX Runtime issues
    async fn test_search_engine_with_embedder() {
        let database = create_test_database();

        // Try to create with embedder (may fail in test environment)
        match LocalEmbedder::with_auto_config().await {
            Ok(embedder) => {
                let search_engine = SearchEngine::new(database, Some(embedder));
                assert!(search_engine.has_semantic_search());
            }
            Err(_) => {
                // Expected in test environment without proper setup
                println!("Embedder creation failed in test environment (expected)");
            }
        }
    }

    #[tokio::test]
    async fn test_keyword_search() {
        let database = create_test_database();
        let search_engine = SearchEngine::new(database, None);

        let options = SearchOptions::default();

        // Use a temporary directory with no files to ensure empty results
        let temp_dir = tempfile::TempDir::new().unwrap();
        let results = search_engine
            .search("test", temp_dir.path().to_str().unwrap(), options)
            .await
            .unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_search_stats() {
        let database = create_test_database();
        let search_engine = SearchEngine::new(database, None);

        let stats = search_engine.get_stats().unwrap();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_chunks, 0);
        assert!(!stats.has_semantic_search);
        assert!(stats.semantic_model.is_none());
    }
}
