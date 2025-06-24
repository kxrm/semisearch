pub mod fuzzy;
pub mod keyword;
pub mod regex_search;
pub mod semantic;
pub mod strategy;
pub mod tfidf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core trait that all search strategies must implement
pub trait SearchStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>>;
    fn required_resources(&self) -> ResourceRequirements;
}

/// Search configuration options
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub min_score: f32,
    pub max_results: usize,
    pub case_sensitive: bool,
    pub whole_words: bool,
    pub include_context: bool,
    pub context_lines: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            min_score: 0.3,
            max_results: 100,
            case_sensitive: false,
            whole_words: false,
            include_context: false,
            context_lines: 2,
        }
    }
}

/// Resource requirements for search strategies
#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    pub min_memory_mb: u64,
    pub requires_ml: bool,
    pub requires_index: bool,
    pub cpu_intensive: bool,
}

/// Search result with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: usize,
    pub content: String,
    pub score: f32,
    pub match_type: MatchType,
    pub start_char: usize,
    pub end_char: usize,
    pub context_before: Option<Vec<String>>,
    pub context_after: Option<Vec<String>>,
}

/// Type of match found
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MatchType {
    Exact,
    Keyword,
    Fuzzy,
    Regex,
    Semantic,
    TfIdf,
    Hybrid,
}

/// Multi-strategy search engine
pub struct SearchEngine {
    strategies: HashMap<String, Box<dyn SearchStrategy>>,
    default_strategy: String,
}

impl SearchEngine {
    pub fn new() -> Self {
        let mut strategies: HashMap<String, Box<dyn SearchStrategy>> = HashMap::new();

        // Register available strategies
        strategies.insert(
            "keyword".to_string(),
            Box::new(keyword::KeywordSearch::new()),
        );
        strategies.insert("fuzzy".to_string(), Box::new(fuzzy::FuzzySearch::new()));
        strategies.insert(
            "regex".to_string(),
            Box::new(regex_search::RegexSearch::new()),
        );
        strategies.insert("tfidf".to_string(), Box::new(tfidf::TfIdfSearch::new()));

        Self {
            strategies,
            default_strategy: "keyword".to_string(),
        }
    }

    pub fn search(
        &self,
        query: &str,
        strategy_name: Option<&str>,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let strategy_name = strategy_name.unwrap_or(&self.default_strategy);

        let strategy = self
            .strategies
            .get(strategy_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown search strategy: {}", strategy_name))?;

        strategy.search(query, options)
    }

    pub fn search_multi_strategy(
        &self,
        query: &str,
        strategies: &[&str],
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let mut all_results = Vec::new();

        for strategy_name in strategies {
            if let Some(strategy) = self.strategies.get(*strategy_name) {
                let results = strategy.search(query, options)?;
                all_results.extend(results);
            }
        }

        // Merge and deduplicate results
        self.merge_results(all_results)
    }

    pub fn available_strategies(&self) -> Vec<&str> {
        self.strategies.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_strategy_requirements(&self, strategy_name: &str) -> Option<ResourceRequirements> {
        self.strategies
            .get(strategy_name)
            .map(|s| s.required_resources())
    }

    fn merge_results(&self, mut results: Vec<SearchResult>) -> Result<Vec<SearchResult>> {
        // Sort by score (descending) then by file path and line number
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.file_path.cmp(&b.file_path))
                .then_with(|| a.line_number.cmp(&b.line_number))
        });

        // Remove duplicates based on file path and line number
        results.dedup_by(|a, b| a.file_path == b.file_path && a.line_number == b.line_number);

        Ok(results)
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_engine_creation() {
        let engine = SearchEngine::new();
        let strategies = engine.available_strategies();

        assert!(strategies.contains(&"keyword"));
        assert!(strategies.contains(&"fuzzy"));
        assert!(strategies.contains(&"regex"));
        assert!(strategies.contains(&"tfidf"));
    }

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert_eq!(options.min_score, 0.3);
        assert_eq!(options.max_results, 100);
        assert!(!options.case_sensitive);
    }

    #[test]
    fn test_merge_results() {
        let engine = SearchEngine::new();
        let results = vec![
            SearchResult {
                file_path: "test.txt".to_string(),
                line_number: 1,
                content: "test content".to_string(),
                score: 0.8,
                match_type: MatchType::Keyword,
                start_char: 0,
                end_char: 4,
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "test.txt".to_string(),
                line_number: 1,
                content: "test content".to_string(),
                score: 0.7,
                match_type: MatchType::Fuzzy,
                start_char: 0,
                end_char: 4,
                context_before: None,
                context_after: None,
            },
        ];

        let merged = engine.merge_results(results).unwrap();
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].score, 0.8);
    }
}
