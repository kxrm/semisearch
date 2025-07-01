use crate::core::{EmbeddingConfig, LocalEmbedder};
use crate::search::{
    file_type_strategy::FileTypeStrategy, fuzzy::FuzzySearch, keyword::KeywordSearch,
    regex_search::RegexSearch,
};
use crate::{SearchOptions, SearchResult};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Search mode enum for internal use
#[allow(dead_code)]
enum SearchMode {
    Keyword,
    Fuzzy,
    Regex,
}

/// Decision on which search strategy to use based on semantic score
#[derive(Debug, PartialEq)]
pub enum SearchDecision {
    KeywordOnly,
    AdaptiveSearch,
    SemanticOnly,
}

/// AutoStrategy automatically selects the best search strategy based on query analysis
/// and project context. This implements the "Smart Query Analysis" from the UX plan.
pub struct AutoStrategy {
    keyword_search: KeywordSearch,
    fuzzy_search: FuzzySearch,
    regex_search: RegexSearch,
    #[allow(dead_code)]
    file_type_strategy: FileTypeStrategy,
    semantic_search: Option<crate::search::semantic::SemanticSearch>,
    query_analyzer: crate::query::lightweight_analyzer::LightweightAnalyzer,
    advanced_mode: bool,
}

impl AutoStrategy {
    /// Create a new AutoStrategy with default search engines
    pub fn new() -> Self {
        Self {
            keyword_search: KeywordSearch::new(),
            fuzzy_search: FuzzySearch::new(),
            regex_search: RegexSearch::new(),
            file_type_strategy: FileTypeStrategy::new(),
            semantic_search: None,
            query_analyzer: crate::query::build_analyzer_with_defaults(),
            advanced_mode: false,
        }
    }

    /// Create a new AutoStrategy with advanced mode enabled
    pub fn with_advanced_mode(advanced_mode: bool) -> Self {
        Self {
            keyword_search: KeywordSearch::new(),
            fuzzy_search: FuzzySearch::new(),
            regex_search: RegexSearch::new(),
            file_type_strategy: FileTypeStrategy::new(),
            semantic_search: None,
            query_analyzer: crate::query::build_analyzer_with_defaults(),
            advanced_mode,
        }
    }

    /// Create an AutoStrategy with semantic search capabilities
    pub async fn with_semantic_search() -> Result<Self> {
        let config = EmbeddingConfig::default();
        let mut embedder = LocalEmbedder::new_with_mode(config, false).await?;

        // Build vocabulary with some sample documents to initialize
        // In a real implementation, this would use indexed documents
        let sample_docs = vec![
            "search query analysis".to_string(),
            "semantic understanding".to_string(),
            "keyword matching".to_string(),
            "fuzzy search algorithm".to_string(),
            "authentication system design".to_string(),
            "memory management techniques".to_string(),
            "database optimization strategies".to_string(),
            "caching performance improvements".to_string(),
        ];

        embedder.build_vocabulary(&sample_docs)?;

        let embedder_arc = Arc::new(embedder);

        Ok(Self {
            keyword_search: KeywordSearch::new(),
            fuzzy_search: FuzzySearch::new(),
            regex_search: RegexSearch::new(),
            file_type_strategy: FileTypeStrategy::with_semantic_search(embedder_arc.clone()),
            semantic_search: Some(crate::search::semantic::SemanticSearch::new(embedder_arc)),
            query_analyzer: crate::query::build_analyzer_with_defaults(),
            advanced_mode: false,
        })
    }

    /// Search with automatic strategy selection
    pub async fn search(
        &mut self,
        query: &str,
        path: &str,
        options: Option<&SearchOptions>,
    ) -> Result<Vec<SearchResult>> {
        // Get files to search
        let files = self.get_files_in_path(path, options)?;

        if files.is_empty() {
            return Ok(vec![]);
        }

        // Determine search strategy based on query analysis
        let semantic_score = self.calculate_semantic_score(query);
        let decision = self.should_use_semantic_search(semantic_score);

        // Execute search based on decision
        match decision {
            SearchDecision::KeywordOnly => {
                // Simple keyword search first
                let keyword_results = self
                    .search_in_files(query, &files, SearchMode::Keyword)
                    .await?;

                // If keyword search fails, try fuzzy as fallback
                if keyword_results.is_empty() {
                    self.search_in_files(query, &files, SearchMode::Fuzzy).await
                } else {
                    Ok(keyword_results)
                }
            }

            SearchDecision::AdaptiveSearch => {
                // Try keyword first, then semantic if poor results
                let keyword_results = self
                    .search_in_files(query, &files, SearchMode::Keyword)
                    .await?;

                if self.results_are_poor(&keyword_results) {
                    // Try semantic search if available
                    self.ensure_semantic_search_available().await?;
                    if self.semantic_search.is_some() {
                        let semantic_results = self.semantic_search_in_files(query, &files).await?;
                        if !semantic_results.is_empty() {
                            Ok(semantic_results)
                        } else {
                            // Both keyword and semantic failed, try fuzzy
                            self.search_in_files(query, &files, SearchMode::Fuzzy).await
                        }
                    } else {
                        Ok(keyword_results)
                    }
                } else {
                    Ok(keyword_results)
                }
            }

            SearchDecision::SemanticOnly => {
                // Go straight to semantic (initialize if needed)
                self.ensure_semantic_search_available().await?;
                if self.semantic_search.is_some() {
                    let semantic_results = self.semantic_search_in_files(query, &files).await?;

                    // If semantic search fails or returns nothing, fallback to fuzzy
                    // EXCEPT for regex-like patterns (let them fail to trigger learning)
                    if semantic_results.is_empty() {
                        if self.looks_like_regex(query) {
                            // Don't fallback for regex-like patterns - let them fail
                            // so users learn about regex mode
                            Ok(semantic_results)
                        } else {
                            if self.advanced_mode {
                                eprintln!(
                                    "🔄 Semantic search found no results, trying fuzzy search..."
                                );
                            }
                            self.search_in_files(query, &files, SearchMode::Fuzzy).await
                        }
                    } else {
                        Ok(semantic_results)
                    }
                } else {
                    // No semantic search available, use fuzzy as best alternative
                    eprintln!("🔄 Semantic search unavailable, using fuzzy search...");
                    self.search_in_files(query, &files, SearchMode::Fuzzy).await
                }
            }
        }
    }

    /// Search with a forced mode (bypasses automatic decision logic)
    pub async fn search_with_mode(
        &mut self,
        query: &str,
        path: &str,
        mode: &str,
        options: Option<&SearchOptions>,
    ) -> Result<Vec<SearchResult>> {
        // Get files to search
        let files = self.get_files_in_path(path, options)?;

        if files.is_empty() {
            return Ok(vec![]);
        }

        // Execute search based on forced mode
        match mode {
            "keyword" => {
                self.search_in_files(query, &files, SearchMode::Keyword)
                    .await
            }
            "fuzzy" => self.search_in_files(query, &files, SearchMode::Fuzzy).await,
            "regex" => self.search_in_files(query, &files, SearchMode::Regex).await,
            "semantic" => {
                self.ensure_semantic_search_available().await?;
                if self.semantic_search.is_some() {
                    self.semantic_search_in_files(query, &files).await
                } else {
                    // Fallback to fuzzy if semantic unavailable
                    eprintln!("🔄 Semantic search unavailable, using fuzzy search...");
                    self.search_in_files(query, &files, SearchMode::Fuzzy).await
                }
            }
            "auto" => {
                // Default to automatic mode
                self.search(query, path, options).await
            }
            _ => {
                // Unknown mode, default to automatic mode
                self.search(query, path, options).await
            }
        }
    }

    /// Get all files in a path recursively, applying include/exclude filtering
    fn get_files_in_path(
        &self,
        path: &str,
        options: Option<&SearchOptions>,
    ) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let path = Path::new(path);

        // Check if path exists
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "No such file or directory: '{}' not found",
                path.display()
            ));
        }

        if path.is_file() {
            files.push(path.to_path_buf());
            return Ok(files);
        }

        // Simple recursive directory traversal with default exclusions
        fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
            if dir.is_dir() {
                // Skip common build/cache directories by default
                if let Some(dir_name) = dir.file_name() {
                    if let Some(name_str) = dir_name.to_str() {
                        if [
                            "target",
                            "node_modules",
                            ".git",
                            "build",
                            "dist",
                            "__pycache__",
                            ".cache",
                            ".semisearch",
                        ]
                        .contains(&name_str)
                        {
                            return Ok(()); // Skip this directory
                        }
                    }
                }

                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dirs(&path, files)?;
                    } else {
                        files.push(path);
                    }
                }
            }
            Ok(())
        }

        visit_dirs(path, &mut files)?;

        // Apply include/exclude filtering if options are provided
        if let Some(options) = options {
            files = self.apply_file_filtering(files, options);
        }

        Ok(files)
    }

    /// Apply include/exclude pattern filtering to files
    fn apply_file_filtering(&self, files: Vec<PathBuf>, options: &SearchOptions) -> Vec<PathBuf> {
        files
            .into_iter()
            .filter(|file| {
                let file_path_str = file.to_string_lossy();
                let file_name = file
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");

                // Apply include patterns (if any)
                if !options.include_patterns.is_empty() {
                    let matches_include = options.include_patterns.iter().any(|pattern| {
                        self.glob_match(pattern, &file_path_str)
                            || self.glob_match(pattern, file_name)
                    });
                    if !matches_include {
                        return false; // Skip file - doesn't match any include pattern
                    }
                }

                // Apply exclude patterns (if any)
                if !options.exclude_patterns.is_empty() {
                    let matches_exclude = options.exclude_patterns.iter().any(|pattern| {
                        self.glob_match(pattern, &file_path_str)
                            || self.glob_match(pattern, file_name)
                    });
                    if matches_exclude {
                        return false; // Skip file - matches an exclude pattern
                    }
                }

                true // Include file
            })
            .collect()
    }

    /// Simple glob pattern matching (supports * wildcard)
    fn glob_match(&self, pattern: &str, text: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        // Convert glob pattern to regex-like matching
        if pattern.contains('*') {
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            if pattern_parts.len() == 2 {
                let start = pattern_parts[0];
                let end = pattern_parts[1];

                if start.is_empty() && !end.is_empty() {
                    // Pattern like "*.rs"
                    text.ends_with(end)
                } else if end.is_empty() && !start.is_empty() {
                    // Pattern like "test*"
                    text.starts_with(start)
                } else if !start.is_empty() && !end.is_empty() {
                    // Pattern like "*test*"
                    text.contains(start) && text.contains(end)
                } else {
                    // Pattern is just "*"
                    true
                }
            } else {
                // More complex patterns - simple contains check
                let pattern_without_stars = pattern.replace('*', "");
                text.contains(&pattern_without_stars)
            }
        } else {
            // No wildcards - exact match
            text == pattern
        }
    }

    /// Search in specific files using the specified mode
    async fn search_in_files(
        &self,
        query: &str,
        files: &[PathBuf],
        mode: SearchMode,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        for file in files {
            let file_path = file.to_str().unwrap_or(".");
            let file_results = match mode {
                SearchMode::Keyword => self.keyword_search.search(query, file_path).await?,
                SearchMode::Fuzzy => self.fuzzy_search.search(query, file_path).await?,
                SearchMode::Regex => self.regex_search.search(query, file_path).await?,
            };
            results.extend(file_results);
        }

        Ok(results)
    }

    /// Search using semantic search across files
    async fn semantic_search_in_files(
        &self,
        query: &str,
        files: &[PathBuf],
    ) -> Result<Vec<SearchResult>> {
        let semantic_search = match &self.semantic_search {
            Some(s) => s,
            None => return Ok(vec![]), // No semantic search available
        };

        // Use the database-aware semantic search method
        semantic_search
            .search_with_database_fallback(query, files, 50)
            .await
    }

    /// Converts code patterns to regex patterns
    pub fn code_pattern_to_regex(&self, pattern: &str) -> String {
        match pattern.to_uppercase().as_str() {
            "TODO" => r"TODO.*".to_string(),
            "FIXME" => r"FIXME.*".to_string(),
            "HACK" => r"HACK.*".to_string(),
            "NOTE" => r"NOTE.*".to_string(),
            "WARNING" => r"WARNING.*".to_string(),
            "ERROR" => r"ERROR.*".to_string(),
            "BUG" => r"BUG.*".to_string(),
            "FUNCTION" | "FN" => r"fn\s+\w+".to_string(),
            "CLASS" => r"class\s+\w+".to_string(),
            "STRUCT" => r"struct\s+\w+".to_string(),
            "ENUM" => r"enum\s+\w+".to_string(),
            "TRAIT" => r"trait\s+\w+".to_string(),
            "IMPL" => r"impl\s+\w+".to_string(),
            "IMPORT" => r"import\s+.*".to_string(),
            "EXPORT" => r"export\s+.*".to_string(),
            "ASYNC" => r"async\s+fn\s+\w+".to_string(),
            "AWAIT" => r"await\s+.*".to_string(),
            _ => format!(r"{}.*", regex::escape(pattern)),
        }
    }

    // Removed unused file extension filtering methods that were causing compiler warnings

    /// Ensure semantic search is available, initializing it if needed
    async fn ensure_semantic_search_available(&mut self) -> Result<()> {
        if self.semantic_search.is_none() {
            match Self::create_semantic_search(self.advanced_mode).await {
                Ok((semantic_search, file_type_strategy)) => {
                    self.semantic_search = Some(semantic_search);
                    self.file_type_strategy = file_type_strategy;
                }
                Err(e) => {
                    eprintln!(
                        "Note: Semantic search unavailable ({e}), using keyword search"
                    );
                }
            }
        }
        Ok(())
    }

    /// Create semantic search components
    async fn create_semantic_search(
        advanced_mode: bool,
    ) -> Result<(crate::search::semantic::SemanticSearch, FileTypeStrategy)> {
        let config = EmbeddingConfig::default();
        let mut embedder = LocalEmbedder::new_with_mode(config, advanced_mode).await?;

        // Build vocabulary with some sample documents to initialize
        let sample_docs = vec![
            "search query analysis".to_string(),
            "semantic understanding".to_string(),
            "keyword matching".to_string(),
            "fuzzy search algorithm".to_string(),
            "authentication system design".to_string(),
            "memory management techniques".to_string(),
            "database optimization strategies".to_string(),
            "caching performance improvements".to_string(),
        ];

        embedder.build_vocabulary(&sample_docs)?;
        let embedder_arc = Arc::new(embedder);

        let semantic_search = crate::search::semantic::SemanticSearch::with_advanced_mode(
            embedder_arc.clone(),
            advanced_mode,
        );
        let file_type_strategy = FileTypeStrategy::with_semantic_search(embedder_arc);

        Ok((semantic_search, file_type_strategy))
    }

    /// Calculate semantic score for a query using the lightweight analyzer
    pub fn calculate_semantic_score(&mut self, query: &str) -> f32 {
        let analysis = self.query_analyzer.analyze(query);
        analysis.needs_semantic
    }

    /// Decide which search strategy to use based on semantic score
    pub fn should_use_semantic_search(&self, score: f32) -> SearchDecision {
        if score < 0.45 {
            SearchDecision::KeywordOnly
        } else if score < 0.60 {
            SearchDecision::AdaptiveSearch
        } else {
            SearchDecision::SemanticOnly
        }
    }

    /// Assess if search results are poor quality
    pub fn results_are_poor(&self, results: &[crate::SearchResult]) -> bool {
        if results.is_empty() {
            return true;
        }

        // Check if all results have low scores
        let scores: Vec<f32> = results.iter().filter_map(|r| r.score).collect();

        if scores.is_empty() {
            // No scores available, check result count
            return results.len() < 3;
        }

        let avg_score = scores.iter().sum::<f32>() / scores.len() as f32;

        // Poor if average score is low, regardless of count
        avg_score < 0.3
    }

    /// Check if a query looks like a regex pattern
    fn looks_like_regex(&self, query: &str) -> bool {
        // Check for common regex patterns that users might try
        query.contains(".*")
            || query.contains("\\d")
            || query.contains("\\w")
            || query.contains("\\s")
            || query.contains("[")
            || query.contains("(")
            || query.contains("^")
            || query.contains("$")
            || query.contains("+")
            || query.contains("?")
            || (query.contains("*") && !query.ends_with("*")) // Allow glob-style * at end
    }
}

impl Default for AutoStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Removed test_project_context_detection as it's not core AutoStrategy functionality

    #[test]
    fn test_code_pattern_to_regex() {
        let auto_strategy = AutoStrategy::new();

        assert_eq!(auto_strategy.code_pattern_to_regex("TODO"), r"TODO.*");
        assert_eq!(auto_strategy.code_pattern_to_regex("FIXME"), r"FIXME.*");
        assert_eq!(auto_strategy.code_pattern_to_regex("function"), r"fn\s+\w+");
        assert_eq!(auto_strategy.code_pattern_to_regex("class"), r"class\s+\w+");
    }

    #[tokio::test]
    async fn test_query_analyzer_integration() {
        // Test that we have query analyzer integration
        let mut analyzer = crate::query::build_analyzer_with_defaults();

        // Test semantic queries
        let semantic_score = analyzer.analyze("how does authentication work");
        assert!(
            semantic_score.needs_semantic > 0.6,
            "Expected high semantic score for conceptual query, got {}",
            semantic_score.needs_semantic
        );

        // Test keyword queries - our analyzer is semantic-biased, so even simple queries get moderate scores
        let keyword_score = analyzer.analyze("main.rs");
        assert!(
            keyword_score.needs_semantic < 0.55,
            "Expected moderate semantic score for file name, got {}",
            keyword_score.needs_semantic
        );
    }

    #[tokio::test]
    async fn test_semantic_search_routing() {
        // Create auto strategy with semantic search
        let strategy = AutoStrategy::with_semantic_search().await.unwrap();

        // Test that semantic search is available
        assert!(strategy.semantic_search.is_some());

        // TODO: Add more specific routing tests once implementation is complete
    }

    #[test]
    fn test_semantic_score_calculation() {
        let mut auto_strategy = AutoStrategy::new();

        // Test various queries and their expected semantic scores
        // Note: Our analyzer is semantic-biased, so scores are higher than traditional keyword analyzers
        let test_cases = vec![
            ("TODO", 0.45, 0.60),                // All caps gets moderate score
            ("user authentication", 0.60, 0.75), // Technical concept
            ("how does caching improve performance", 0.70, 1.0), // Question
            ("main.py", 0.40, 0.55),             // File name with extension
            ("difference between TCP and UDP", 0.70, 1.0), // Comparison query
        ];

        for (query, min_score, max_score) in test_cases {
            let score = auto_strategy.calculate_semantic_score(query);
            assert!(
                score >= min_score && score <= max_score,
                "Query '{query}' score {score} not in expected range [{min_score}, {max_score}]"
            );
        }
    }

    #[test]
    fn test_should_use_semantic_search() {
        let auto_strategy = AutoStrategy::new();

        // Test decision making based on scores
        assert_eq!(
            auto_strategy.should_use_semantic_search(0.3),
            SearchDecision::KeywordOnly
        );
        assert_eq!(
            auto_strategy.should_use_semantic_search(0.5),
            SearchDecision::AdaptiveSearch
        );
        assert_eq!(
            auto_strategy.should_use_semantic_search(0.7),
            SearchDecision::SemanticOnly
        );
    }

    #[tokio::test]
    async fn test_adaptive_search_fallback() {
        let _strategy = AutoStrategy::new();

        // Mock a query that returns no results with keyword search
        // This should trigger fallback to semantic search

        // TODO: Implement once we have proper mocking
    }

    #[test]
    fn test_result_quality_assessment() {
        let auto_strategy = AutoStrategy::new();

        // Test empty results
        let empty_results = vec![];
        assert!(auto_strategy.results_are_poor(&empty_results));

        // Test low-score results
        let poor_results = vec![crate::SearchResult {
            file_path: "test.rs".to_string(),
            line_number: 1,
            content: "test content".to_string(),
            score: Some(0.2),
            match_type: Some(crate::MatchType::Exact),
            context_before: None,
            context_after: None,
        }];
        assert!(auto_strategy.results_are_poor(&poor_results));

        // Test good results
        let good_results = vec![crate::SearchResult {
            file_path: "test.rs".to_string(),
            line_number: 1,
            content: "test content".to_string(),
            score: Some(0.8),
            match_type: Some(crate::MatchType::Exact),
            context_before: None,
            context_after: None,
        }];
        assert!(!auto_strategy.results_are_poor(&good_results));
    }
}
