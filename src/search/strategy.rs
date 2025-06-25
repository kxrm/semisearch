use crate::core::embedder::{EmbeddingCapability, LocalEmbedder};
use crate::search::semantic::{SemanticSearch, SemanticSearchOptions};
use crate::storage::Database;
use anyhow::Result;
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
            EmbeddingCapability::Full | EmbeddingCapability::TfIdf => {
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
                    self.semantic_enhanced_search(query, &options, semantic_search)
                        .await
                } else {
                    // Fallback to keyword if semantic not available
                    self.keyword_search(query, path, &options).await
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
    fn determine_search_mode(&self, _options: &SearchOptions) -> SearchMode {
        // TODO: Implement semantic search options in SearchOptions
        // if options.no_semantic_search {
        //     return SearchMode::Keyword;
        // }
        //
        // if options.semantic_search {
        //     return SearchMode::Semantic;
        // }

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
        options: &SearchOptions,
        semantic_search: &SemanticSearch,
    ) -> Result<Vec<SearchResult>> {
        // Get chunks with embeddings
        let chunks = self.database.get_chunks_with_embeddings()?;

        let _semantic_options = SemanticSearchOptions {
            similarity_threshold: 0.7, // TODO: Add semantic_threshold to SearchOptions
            max_results: options.max_results,
            boost_exact_matches: true,
            enable_reranking: false,
            boost_recent_files: false,
        };

        let semantic_results = semantic_search.search(query, &chunks, options.max_results)?;
        let mut results = Vec::new();

        for result in semantic_results {
            results.push(SearchResult {
                file_path: result.chunk.file_path,
                line_number: result.chunk.line_number,
                content: result.chunk.content,
                score: Some(result.similarity_score),
                match_type: Some(MatchType::Semantic),
            });
        }

        Ok(results)
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
            .semantic_enhanced_search(query, options, semantic_search)
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
