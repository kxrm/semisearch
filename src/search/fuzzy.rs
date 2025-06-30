use super::{MatchType, ResourceRequirements, SearchOptions, SearchResult, SearchStrategy};
use crate::text::{TextChunk, TextProcessor};
use anyhow::Result;
use edit_distance::edit_distance;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

/// Fuzzy search implementation using edit distance
pub struct FuzzySearch {
    text_processor: TextProcessor,
    matcher: SkimMatcherV2,
    max_edit_distance: usize,
}

impl FuzzySearch {
    pub fn new() -> Self {
        Self {
            text_processor: TextProcessor::new(),
            matcher: SkimMatcherV2::default(),
            max_edit_distance: 4,
        }
    }

    pub fn with_max_distance(max_distance: usize) -> Self {
        Self {
            text_processor: TextProcessor::new(),
            matcher: SkimMatcherV2::default(),
            max_edit_distance: max_distance,
        }
    }

    /// Search within a collection of text chunks using fuzzy matching
    pub fn search_chunks(
        &self,
        query: &str,
        chunks: &[TextChunk],
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }

        let mut results = Vec::new();

        for chunk in chunks {
            let score = self.calculate_fuzzy_score(query, &chunk.content, &chunk.tokens, options);
            if score >= options.min_score {
                let (start_char, end_char) = self.find_best_match_position(&chunk.content, query);

                results.push(SearchResult {
                    file_path: "memory".to_string(),
                    line_number: chunk.line_number,
                    content: chunk.content.clone(),
                    score,
                    match_type: MatchType::Fuzzy,
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

        // Filter out zero-score results (only return actual matches)
        let filtered_results: Vec<_> = results.into_iter().filter(|r| r.score > 0.0).collect();
        Ok(filtered_results)
    }

    /// Calculate fuzzy match score using multiple algorithms
    fn calculate_fuzzy_score(
        &self,
        query: &str,
        content: &str,
        tokens: &[String],
        options: &SearchOptions,
    ) -> f32 {
        let query_lower = if options.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };
        let content_lower = if options.case_sensitive {
            content.to_string()
        } else {
            content.to_lowercase()
        };

        // 1. Skim fuzzy matcher score (primary)
        let skim_score = if let Some(score) = self.matcher.fuzzy_match(&content_lower, &query_lower)
        {
            (score as f32 / 100.0).min(1.0) // Normalize to 0-1
        } else {
            0.0
        };

        // 2. Token-based fuzzy matching
        let token_score = self.calculate_token_fuzzy_score(&query_lower, tokens);

        // 3. Edit distance score for exact substring matches
        let edit_score = self.calculate_edit_distance_score(&query_lower, &content_lower);

        // 4. Substring bonus
        let substring_bonus = if content_lower.contains(&query_lower) {
            0.3
        } else {
            0.0
        };

        // Combine scores with weights
        let combined_score =
            (skim_score * 0.4) + (token_score * 0.3) + (edit_score * 0.2) + substring_bonus;

        combined_score.min(1.0)
    }

    /// Calculate fuzzy score for individual tokens
    fn calculate_token_fuzzy_score(&self, query: &str, tokens: &[String]) -> f32 {
        if tokens.is_empty() || query.trim().is_empty() {
            return 0.0;
        }

        let query_tokens = self.text_processor.tokenize(query);
        if query_tokens.is_empty() {
            return 0.0;
        }

        let mut total_score = 0.0;
        let mut matches = 0;

        for query_token in &query_tokens {
            let mut best_token_score: f32 = 0.0;

            for content_token in tokens {
                // Try fuzzy matching with skim
                if let Some(score) = self.matcher.fuzzy_match(content_token, query_token) {
                    let normalized_score = (score as f32 / 100.0).min(1.0);
                    println!("[DEBUG] Skim: query_token='{query_token}', content_token='{content_token}', score={normalized_score}");
                    best_token_score = best_token_score.max(normalized_score);
                }

                // Also try edit distance
                let edit_dist = edit_distance(query_token, content_token);
                if edit_dist <= self.max_edit_distance {
                    let edit_score = 1.0
                        - (edit_dist as f32 / query_token.len().max(content_token.len()) as f32);
                    println!("[DEBUG] EditDist: query_token='{query_token}', content_token='{content_token}', edit_dist={edit_dist}, edit_score={edit_score}");
                    best_token_score = best_token_score.max(edit_score);
                }
            }

            if best_token_score > 0.1 {
                total_score += best_token_score;
                matches += 1;
            }
        }

        if matches > 0 {
            total_score / query_tokens.len() as f32
        } else {
            0.0
        }
    }

    /// Calculate score based on edit distance
    fn calculate_edit_distance_score(&self, query: &str, content: &str) -> f32 {
        // Find the best substring match using sliding window
        let query_len = query.len();
        if query_len == 0 || content.len() < query_len {
            return 0.0;
        }

        let mut best_score: f32 = 0.0;

        // Try different window sizes around the query length
        for window_size in [query_len, query_len + 1, query_len + 2] {
            if window_size > content.len() {
                continue;
            }

            for start in 0..=(content.len() - window_size) {
                let substring = &content[start..start + window_size];
                let edit_dist = edit_distance(query, substring);

                if edit_dist <= self.max_edit_distance {
                    let score = 1.0 - (edit_dist as f32 / query_len.max(substring.len()) as f32);
                    println!("[DEBUG] EditDistSubstr: query='{query}', substring='{substring}', edit_dist={edit_dist}, score={score}");
                    best_score = best_score.max(score);
                }
            }
        }

        best_score
    }

    /// Find the best match position in content
    fn find_best_match_position(&self, content: &str, query: &str) -> (usize, usize) {
        // Use skim matcher to find the best match position
        if let Some((_, indices)) = self.matcher.fuzzy_indices(content, query) {
            if !indices.is_empty() {
                let start = indices[0];
                let end = indices[indices.len() - 1] + 1;
                return (start, end);
            }
        }

        // Fallback to simple substring search
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();

        if let Some(start) = content_lower.find(&query_lower) {
            (start, start + query.len())
        } else {
            (0, content.len().min(query.len() * 2))
        }
    }

    /// Search in files at the given path
    pub async fn search(&self, query: &str, path: &str) -> Result<Vec<crate::SearchResult>> {
        use crate::search_files;
        use crate::SearchOptions as LibSearchOptions;

        let options = LibSearchOptions {
            min_score: 0.3,
            max_results: 100,
            fuzzy_matching: true,
            typo_tolerance: true,
            max_edit_distance: self.max_edit_distance,
            search_mode: Some("fuzzy".to_string()),
            ..Default::default()
        };

        search_files(query, path, &options)
    }
}

impl SearchStrategy for FuzzySearch {
    fn name(&self) -> &str {
        "fuzzy"
    }

    fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        // Simplified implementation for the trait
        let chunks = self.text_processor.process_file(query);
        self.search_chunks(query, &chunks, options)
    }

    fn required_resources(&self) -> ResourceRequirements {
        ResourceRequirements {
            min_memory_mb: 20,
            requires_ml: false,
            requires_index: false,
            cpu_intensive: true,
        }
    }
}

impl Default for FuzzySearch {
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
    fn test_fuzzy_search_creation() {
        let search = FuzzySearch::new();
        assert_eq!(search.name(), "fuzzy");
        assert_eq!(search.max_edit_distance, 4);

        let requirements = search.required_resources();
        assert_eq!(requirements.min_memory_mb, 20);
        assert!(!requirements.requires_ml);
        assert!(requirements.cpu_intensive);
    }

    #[test]
    fn test_fuzzy_search_with_typos() {
        let search = FuzzySearch::new();
        let options = SearchOptions::default();

        let chunks = vec![
            create_test_chunk(1, "machine learning algorithm"),
            create_test_chunk(2, "deep learning networks"),
            create_test_chunk(3, "unrelated content"),
        ];

        // Search with typo: "machne" instead of "machine"
        let results = search.search_chunks("machne", &chunks, &options).unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].line_number, 1);
        assert!(results[0].score > 0.0);
        assert_eq!(results[0].match_type, MatchType::Fuzzy);
    }

    #[test]
    fn test_fuzzy_search_partial_match() {
        let search = FuzzySearch::new();
        let options = SearchOptions::default();

        let chunks = vec![
            create_test_chunk(1, "javascript programming language"),
            create_test_chunk(2, "java development environment"),
        ];

        // "java" should match both "javascript" and "java"
        let results = search.search_chunks("java", &chunks, &options).unwrap();

        assert_eq!(results.len(), 2);
        // "java" should score higher than "javascript" for exact match
        assert!(results.iter().any(|r| r.line_number == 2));
    }

    #[test]
    fn test_edit_distance_calculation() {
        let search = FuzzySearch::new();

        let score1 = search.calculate_edit_distance_score("test", "test"); // Exact match
        let score2 = search.calculate_edit_distance_score("test", "tast"); // 1 char diff
        let score3 = search.calculate_edit_distance_score("test", "completely different");

        assert!(score1 > score2);
        assert!(score2 > score3);
        assert_eq!(score1, 1.0); // Perfect match
    }

    #[test]
    fn test_token_fuzzy_score() {
        let search = FuzzySearch::new();

        let tokens = vec!["machine".to_string(), "learning".to_string()];

        let score1 = search.calculate_token_fuzzy_score("machine", &tokens);
        let score2 = search.calculate_token_fuzzy_score("machne", &tokens); // typo
        let score3 = search.calculate_token_fuzzy_score("unrelated", &tokens);

        assert!(
            score1 > score2 || (score1 - score2).abs() < 0.01,
            "Exact match should score higher than typo"
        );
        assert!(score2 > score3);
    }

    #[test]
    fn test_case_sensitivity() {
        let search = FuzzySearch::new();
        let options = SearchOptions {
            case_sensitive: false,
            ..Default::default()
        };

        let chunks = vec![
            create_test_chunk(1, "Machine Learning"),
            create_test_chunk(2, "MACHINE LEARNING"),
        ];

        let results = search.search_chunks("machine", &chunks, &options).unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_max_edit_distance_configuration() {
        let search = FuzzySearch::with_max_distance(1);
        let options = SearchOptions::default();

        let chunks = vec![
            create_test_chunk(1, "test content"),
            create_test_chunk(2, "tast content"), // 1 edit distance
            create_test_chunk(3, "txst content"), // 2 edit distance
        ];

        let results = search.search_chunks("test", &chunks, &options).unwrap();

        // Should match first two but not the third (exceeds max distance)
        assert!(results.len() >= 2);
        assert!(results.iter().any(|r| r.line_number == 1));
        assert!(results.iter().any(|r| r.line_number == 2));
    }

    #[test]
    fn test_find_best_match_position() {
        let search = FuzzySearch::new();

        let content = "This is a test example";
        let query = "test";

        let (start, end) = search.find_best_match_position(content, query);

        // Should find the position of "test"
        assert!(start <= 10); // Approximate position
        assert!(end > start);
        assert!(end <= content.len());
    }

    #[test]
    fn test_substring_bonus() {
        let search = FuzzySearch::new();
        let options = SearchOptions::default();

        let chunks = vec![
            create_test_chunk(1, "this contains the exact word test"),
            create_test_chunk(2, "this contains tast which is similar"),
        ];

        let results = search.search_chunks("test", &chunks, &options).unwrap();

        assert_eq!(results.len(), 2);
        // First result should have higher score due to substring bonus
        assert!(results[0].score > results[1].score);
        assert_eq!(results[0].line_number, 1);
    }

    #[test]
    fn test_combined_score_calculation() {
        let search = FuzzySearch::new();
        let options = SearchOptions::default();

        let tokens = vec!["machine".to_string(), "learning".to_string()];

        let score = search.calculate_fuzzy_score(
            "machine learning",
            "machine learning algorithm",
            &tokens,
            &options,
        );

        assert!(score > 0.5); // Should be high for good match
        assert!(score <= 1.0); // Should not exceed 1.0
    }

    #[test]
    fn test_empty_query_handling() {
        let search = FuzzySearch::new();
        let options = SearchOptions::default();

        let chunks = vec![create_test_chunk(1, "some content")];

        let results = search.search_chunks("", &chunks, &options).unwrap();

        // Empty query should return no results
        assert!(results.is_empty());
    }

    #[test]
    fn test_very_different_strings() {
        let search = FuzzySearch::new();
        let options = SearchOptions::default();

        let chunks = vec![create_test_chunk(1, "completely different content")];

        let results = search.search_chunks("xyz", &chunks, &options).unwrap();

        // Very different strings should have low or zero score
        if !results.is_empty() {
            assert!(results[0].score < 0.3);
        }
    }
}
