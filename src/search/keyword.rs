use super::{MatchType, ResourceRequirements, SearchOptions, SearchResult, SearchStrategy};
use crate::text::{TextChunk, TextProcessor};
use anyhow::Result;

/// Basic keyword search implementation
pub struct KeywordSearch {
    text_processor: TextProcessor,
}

impl KeywordSearch {
    pub fn new() -> Self {
        Self {
            text_processor: TextProcessor::new(),
        }
    }

    /// Search within a collection of text chunks
    pub fn search_chunks(
        &self,
        query: &str,
        chunks: &[TextChunk],
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let query_tokens = self.text_processor.tokenize(query);
        let mut results = Vec::new();

        for chunk in chunks {
            let score = self.calculate_keyword_score(&query_tokens, &chunk.tokens, options);
            if score > 0.0 && score >= options.min_score {
                // Find the position of the match in the content
                let (start_char, end_char) =
                    self.find_match_position(&chunk.content, &query_tokens);

                results.push(SearchResult {
                    file_path: "memory".to_string(), // Will be set by caller
                    line_number: chunk.line_number,
                    content: chunk.content.clone(),
                    score,
                    match_type: MatchType::Keyword,
                    start_char,
                    end_char,
                    context_before: None,
                    context_after: None,
                });
            }
        }

        // Sort by score (descending)
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(options.max_results);

        Ok(results)
    }

    /// Calculate keyword match score
    fn calculate_keyword_score(
        &self,
        query_tokens: &[String],
        content_tokens: &[String],
        options: &SearchOptions,
    ) -> f32 {
        if query_tokens.is_empty() || content_tokens.is_empty() {
            return 0.0;
        }

        let mut score = 0.0;
        let content_tokens_set: std::collections::HashSet<_> = content_tokens.iter().collect();

        for query_token in query_tokens {
            if options.case_sensitive {
                // For case sensitive, we need to check original content
                score += if content_tokens.contains(query_token) {
                    1.0
                } else {
                    0.0
                };
            } else {
                // Case insensitive matching (tokens are already lowercase)
                if content_tokens_set.contains(query_token) {
                    score += 1.0;
                } else {
                    // Partial match bonus
                    for content_token in content_tokens {
                        if content_token.contains(query_token)
                            || query_token.contains(content_token)
                        {
                            score += 0.5;
                            break;
                        }
                    }
                }
            }
        }

        // Normalize by query length
        let base_score = score / query_tokens.len() as f32;

        // Bonus for exact phrase matches
        let phrase_bonus = self.calculate_phrase_bonus(query_tokens, content_tokens);

        // Calculate final score with bonuses, ensuring it stays within 0-1
        let bonus_total = phrase_bonus;

        // If base score is already 1.0, apply bonuses by slightly reducing base and adding bonus
        if base_score >= 1.0 && bonus_total > 0.0 {
            0.95 + (bonus_total * 0.05) // Keep within 1.0 but allow differentiation
        } else {
            (base_score + bonus_total).min(1.0)
        }
    }

    /// Calculate bonus for phrase matches
    fn calculate_phrase_bonus(&self, query_tokens: &[String], content_tokens: &[String]) -> f32 {
        if query_tokens.len() < 2 {
            return 0.0;
        }

        // Look for consecutive query terms in content
        for i in 0..=content_tokens.len().saturating_sub(query_tokens.len()) {
            let content_slice = &content_tokens[i..i + query_tokens.len()];
            if content_slice == query_tokens {
                return 0.3; // Bonus for exact phrase match
            }
        }

        0.0
    }

    /// Find the position of the match in the content
    fn find_match_position(&self, content: &str, query_tokens: &[String]) -> (usize, usize) {
        if query_tokens.is_empty() {
            return (0, 0);
        }

        let content_lower = content.to_lowercase();
        let first_token = &query_tokens[0];

        if let Some(start) = content_lower.find(first_token) {
            let end = start + first_token.len();
            (start, end)
        } else {
            (0, content.len().min(50)) // Default to first 50 chars if no match found
        }
    }

    /// Search in files at the given path
    pub async fn search(&self, query: &str, path: &str) -> Result<Vec<crate::SearchResult>> {
        use crate::search_files;
        use crate::SearchOptions as LibSearchOptions;

        let options = LibSearchOptions {
            min_score: 0.3,
            max_results: 100,
            search_mode: Some("keyword".to_string()),
            ..Default::default()
        };

        search_files(query, path, &options)
    }
}

impl SearchStrategy for KeywordSearch {
    fn name(&self) -> &str {
        "keyword"
    }

    fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        // This is a simplified implementation for the trait
        // In practice, this would be called with actual file content
        let chunks = self.text_processor.process_file(query);
        self.search_chunks(query, &chunks, options)
    }

    fn required_resources(&self) -> ResourceRequirements {
        ResourceRequirements {
            min_memory_mb: 10,
            requires_ml: false,
            requires_index: false,
            cpu_intensive: false,
        }
    }
}

impl Default for KeywordSearch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::TextChunk;

    fn create_test_chunk(line_number: usize, content: &str) -> TextChunk {
        let processor = TextProcessor::new();
        TextChunk {
            line_number,
            content: content.to_string(),
            tokens: processor.tokenize(content),
            start_char: 0,
            end_char: content.len(),
            language_hint: None,
        }
    }

    #[test]
    fn test_keyword_search_creation() {
        let search = KeywordSearch::new();
        assert_eq!(search.name(), "keyword");

        let requirements = search.required_resources();
        assert_eq!(requirements.min_memory_mb, 10);
        assert!(!requirements.requires_ml);
        assert!(!requirements.requires_index);
    }

    #[test]
    fn test_simple_keyword_match() {
        let search = KeywordSearch::new();
        let options = SearchOptions::default();

        let chunks = vec![
            create_test_chunk(1, "Hello world this is a test"),
            create_test_chunk(2, "Another line without the word"),
            create_test_chunk(3, "This line contains hello again"),
        ];

        let results = search.search_chunks("hello", &chunks, &options).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1); // Should be sorted by score
        assert_eq!(results[1].line_number, 3);
        assert!(results[0].score > 0.0);
        assert_eq!(results[0].match_type, MatchType::Keyword);
    }

    #[test]
    fn test_multi_word_search() {
        let search = KeywordSearch::new();
        let options = SearchOptions::default();

        let chunks = vec![
            create_test_chunk(1, "machine learning algorithm"),
            create_test_chunk(2, "learning new machine techniques"),
            create_test_chunk(3, "unrelated content here"),
        ];

        let results = search
            .search_chunks("machine learning", &chunks, &options)
            .unwrap();

        assert_eq!(results.len(), 2);

        // First result should have higher score due to phrase match
        assert!(
            results[0].score > results[1].score,
            "First result (score: {}) should have higher score than second (score: {})",
            results[0].score,
            results[1].score
        );
    }

    #[test]
    fn test_case_sensitivity() {
        let search = KeywordSearch::new();
        let options = SearchOptions {
            case_sensitive: false,
            ..Default::default()
        };

        let chunks = vec![
            create_test_chunk(1, "Hello World"),
            create_test_chunk(2, "HELLO WORLD"),
            create_test_chunk(3, "hello world"),
        ];

        let results = search.search_chunks("hello", &chunks, &options).unwrap();

        // Should match all three with case insensitive search
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_score_calculation() {
        let search = KeywordSearch::new();
        let options = SearchOptions::default();

        let query_tokens = vec!["test".to_string(), "example".to_string()];
        let content_tokens1 = vec!["test".to_string(), "example".to_string()]; // Perfect match
        let content_tokens2 = vec!["test".to_string(), "other".to_string()]; // Partial match

        let score1 = search.calculate_keyword_score(&query_tokens, &content_tokens1, &options);
        let score2 = search.calculate_keyword_score(&query_tokens, &content_tokens2, &options);

        assert!(score1 > score2);
        assert!(score1 > 0.5); // Should be high for perfect match
        assert!(score2 > 0.0 && score2 < 0.8); // Should be lower for partial match
    }

    #[test]
    fn test_phrase_bonus() {
        let search = KeywordSearch::new();

        let query_tokens = vec!["machine".to_string(), "learning".to_string()];
        let content_tokens1 = vec!["machine".to_string(), "learning".to_string()]; // Exact phrase
        let content_tokens2 = vec!["learning".to_string(), "machine".to_string()]; // Reversed

        let bonus1 = search.calculate_phrase_bonus(&query_tokens, &content_tokens1);
        let bonus2 = search.calculate_phrase_bonus(&query_tokens, &content_tokens2);

        assert!(bonus1 > 0.0);
        assert_eq!(bonus2, 0.0);
    }

    #[test]
    fn test_find_match_position() {
        let search = KeywordSearch::new();

        let content = "Hello world test example";
        let query_tokens = vec!["world".to_string()];

        let (start, end) = search.find_match_position(content, &query_tokens);

        assert_eq!(start, 6); // Position of "world"
        assert_eq!(end, 11); // End of "world"
    }

    #[test]
    fn test_max_results_limit() {
        let search = KeywordSearch::new();
        let options = SearchOptions {
            max_results: 2,
            ..Default::default()
        };

        let chunks = vec![
            create_test_chunk(1, "test content one"),
            create_test_chunk(2, "test content two"),
            create_test_chunk(3, "test content three"),
            create_test_chunk(4, "test content four"),
        ];

        let results = search.search_chunks("test", &chunks, &options).unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_score_threshold() {
        let search = KeywordSearch::new();
        let options = SearchOptions {
            min_score: 0.8, // High threshold
            ..Default::default()
        };

        let chunks = vec![
            create_test_chunk(1, "perfect test match"),
            create_test_chunk(2, "partial testing only"),
        ];

        let results = search.search_chunks("test", &chunks, &options).unwrap();

        // Should filter out low-scoring results
        assert!(results.len() <= 1);
        if !results.is_empty() {
            assert!(results[0].score >= options.min_score);
        }
    }
}
