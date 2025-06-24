use super::{MatchType, ResourceRequirements, SearchOptions, SearchResult, SearchStrategy};
use crate::text::{TextChunk, TextProcessor};
use anyhow::Result;
use regex::{Regex, RegexBuilder};
use std::collections::HashMap;

/// Regex-based search implementation
pub struct RegexSearch {
    text_processor: TextProcessor,
    cache: std::sync::Mutex<HashMap<String, Regex>>,
}

impl RegexSearch {
    pub fn new() -> Self {
        Self {
            text_processor: TextProcessor::new(),
            cache: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Search within a collection of text chunks using regex patterns
    pub fn search_chunks(
        &self,
        query: &str,
        chunks: &[TextChunk],
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let regex = self.build_regex(query, options)?;
        let mut results = Vec::new();

        for chunk in chunks {
            let matches = self.find_regex_matches(&regex, &chunk.content);

            for regex_match in matches {
                let score = self.calculate_regex_score(&regex_match, &chunk.content, options);

                if score >= options.min_score {
                    results.push(SearchResult {
                        file_path: "memory".to_string(),
                        line_number: chunk.line_number,
                        content: chunk.content.clone(),
                        score,
                        match_type: MatchType::Regex,
                        start_char: regex_match.start,
                        end_char: regex_match.end,
                        context_before: None,
                        context_after: None,
                    });
                }
            }
        }

        // Sort by score (descending) and then by position
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.start_char.cmp(&b.start_char))
        });

        results.truncate(options.max_results);
        Ok(results)
    }

    /// Build regex from query with caching
    fn build_regex(&self, query: &str, options: &SearchOptions) -> Result<Regex> {
        let cache_key = format!("{}:{}", query, options.case_sensitive);

        // Try to get from cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some(regex) = cache.get(&cache_key) {
                return Ok(regex.clone());
            }
        }

        // Build new regex
        let regex = if self.is_regex_pattern(query) {
            // Query is already a regex pattern
            RegexBuilder::new(query)
                .case_insensitive(!options.case_sensitive)
                .build()
                .map_err(|e| anyhow::anyhow!("Invalid regex pattern: {}", e))?
        } else {
            // Convert plain text to regex pattern
            let escaped_query = regex::escape(query);
            let pattern = if options.whole_words {
                format!(r"\b{}\b", escaped_query)
            } else {
                escaped_query
            };

            RegexBuilder::new(&pattern)
                .case_insensitive(!options.case_sensitive)
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build regex: {}", e))?
        };

        // Cache the regex
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(cache_key, regex.clone());
        }

        Ok(regex)
    }

    /// Check if query looks like a regex pattern
    fn is_regex_pattern(&self, query: &str) -> bool {
        // Simple heuristics to detect regex patterns
        query.contains('[')
            || query.contains('(')
            || query.contains('{')
            || query.contains('*')
            || query.contains('+')
            || query.contains('?')
            || query.contains('^')
            || query.contains('$')
            || query.contains('|')
            || query.contains('\\')
    }

    /// Find all regex matches in content
    fn find_regex_matches(&self, regex: &Regex, content: &str) -> Vec<RegexMatch> {
        regex
            .find_iter(content)
            .map(|m| RegexMatch {
                start: m.start(),
                end: m.end(),
                text: m.as_str().to_string(),
            })
            .collect()
    }

    /// Calculate score for regex match
    fn calculate_regex_score(
        &self,
        regex_match: &RegexMatch,
        content: &str,
        _options: &SearchOptions,
    ) -> f32 {
        let match_length = regex_match.text.len();
        let content_length = content.len();

        if match_length == 0 || content_length == 0 {
            return 0.0;
        }

        // Base score based on match coverage
        let coverage_score = (match_length as f32 / content_length as f32).min(1.0);

        // Bonus for exact word boundaries
        let word_boundary_bonus = if self.is_word_boundary_match(regex_match, content) {
            0.2
        } else {
            0.0
        };

        // Bonus for beginning of line matches
        let line_start_bonus =
            if regex_match.start == 0 || content.chars().nth(regex_match.start - 1) == Some('\n') {
                0.1
            } else {
                0.0
            };

        // Penalty for very short matches (likely noise)
        let length_penalty = if match_length < 3 { -0.1 } else { 0.0 };

        let base_score = 0.7; // Base score for any regex match
        (base_score
            + coverage_score * 0.2
            + word_boundary_bonus
            + line_start_bonus
            + length_penalty)
            .clamp(0.0, 1.0)
    }

    /// Check if match is at word boundaries
    fn is_word_boundary_match(&self, regex_match: &RegexMatch, content: &str) -> bool {
        let chars: Vec<char> = content.chars().collect();

        let start_boundary = regex_match.start == 0
            || !chars
                .get(regex_match.start - 1)
                .unwrap_or(&' ')
                .is_alphanumeric();

        let end_boundary = regex_match.end >= chars.len()
            || !chars.get(regex_match.end).unwrap_or(&' ').is_alphanumeric();

        start_boundary && end_boundary
    }

    /// Convert simple wildcards to regex
    pub fn wildcard_to_regex(&self, pattern: &str) -> String {
        pattern.replace("*", ".*").replace("?", ".")
    }

    /// Search with wildcard patterns
    pub fn search_wildcard(
        &self,
        pattern: &str,
        chunks: &[TextChunk],
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let regex_pattern = self.wildcard_to_regex(pattern);
        self.search_chunks(&regex_pattern, chunks, options)
    }
}

impl SearchStrategy for RegexSearch {
    fn name(&self) -> &str {
        "regex"
    }

    fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        // Simplified implementation for the trait
        let chunks = self.text_processor.process_file(query);
        self.search_chunks(query, &chunks, options)
    }

    fn required_resources(&self) -> ResourceRequirements {
        ResourceRequirements {
            min_memory_mb: 15,
            requires_ml: false,
            requires_index: false,
            cpu_intensive: true,
        }
    }
}

impl Default for RegexSearch {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a regex match with position information
#[derive(Debug, Clone)]
struct RegexMatch {
    start: usize,
    end: usize,
    text: String,
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
    fn test_regex_search_creation() {
        let search = RegexSearch::new();
        assert_eq!(search.name(), "regex");

        let requirements = search.required_resources();
        assert_eq!(requirements.min_memory_mb, 15);
        assert!(!requirements.requires_ml);
        assert!(requirements.cpu_intensive);
    }

    #[test]
    fn test_simple_regex_pattern() {
        let search = RegexSearch::new();
        let options = SearchOptions::default();

        let chunks = vec![
            create_test_chunk(1, "test123 content"),
            create_test_chunk(2, "test456 content"),
            create_test_chunk(3, "other content"),
        ];

        // Search for pattern "test" followed by digits
        let results = search.search_chunks(r"test\d+", &chunks, &options).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[1].line_number, 2);
        assert_eq!(results[0].match_type, MatchType::Regex);
    }

    #[test]
    fn test_plain_text_as_regex() {
        let search = RegexSearch::new();
        let options = SearchOptions::default();

        let chunks = vec![
            create_test_chunk(1, "hello world"),
            create_test_chunk(2, "hello universe"),
        ];

        // Plain text should be treated as literal match
        let results = search.search_chunks("hello", &chunks, &options).unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_case_sensitivity() {
        let search = RegexSearch::new();
        let options = SearchOptions {
            case_sensitive: true,
            ..Default::default()
        };

        let chunks = vec![
            create_test_chunk(1, "Hello World"),
            create_test_chunk(2, "hello world"),
        ];

        let results = search.search_chunks("Hello", &chunks, &options).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 1);
    }

    #[test]
    fn test_whole_words_option() {
        let search = RegexSearch::new();
        let options = SearchOptions {
            whole_words: true,
            ..Default::default()
        };

        let chunks = vec![
            create_test_chunk(1, "testing tested content"), // No standalone "test"
            create_test_chunk(2, "just test here"),         // Has standalone "test"
        ];

        let results = search.search_chunks("test", &chunks, &options).unwrap();

        // Should only match whole word "test", not "testing" or "tested"
        assert_eq!(
            results.len(),
            1,
            "Expected 1 result but got {}",
            results.len()
        );
        assert_eq!(results[0].line_number, 2);

        // Verify it doesn't match partial words
        let results_no_whole_words = search
            .search_chunks("test", &chunks, &SearchOptions::default())
            .unwrap();
        assert!(
            results_no_whole_words.len() >= results.len(),
            "Without whole_words should find same or more matches"
        );
    }

    #[test]
    fn test_regex_pattern_detection() {
        let search = RegexSearch::new();

        assert!(search.is_regex_pattern(r"\d+"));
        assert!(search.is_regex_pattern("test*"));
        assert!(search.is_regex_pattern("(hello|world)"));
        assert!(search.is_regex_pattern("[a-z]+"));
        assert!(search.is_regex_pattern("^start"));
        assert!(search.is_regex_pattern("end$"));

        assert!(!search.is_regex_pattern("simple text"));
        assert!(!search.is_regex_pattern("hello world"));
    }

    #[test]
    fn test_wildcard_conversion() {
        let search = RegexSearch::new();

        assert_eq!(search.wildcard_to_regex("test*"), "test.*");
        assert_eq!(search.wildcard_to_regex("test?"), "test.");
        assert_eq!(search.wildcard_to_regex("*.txt"), ".*.txt");
    }

    #[test]
    fn test_wildcard_search() {
        let search = RegexSearch::new();
        let options = SearchOptions::default();

        let chunks = vec![
            create_test_chunk(1, "file.txt content"),
            create_test_chunk(2, "file.rs content"),
            create_test_chunk(3, "document.pdf content"),
        ];

        let results = search.search_wildcard("*.txt", &chunks, &options).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 1);
    }

    #[test]
    fn test_complex_regex_pattern() {
        let search = RegexSearch::new();
        let options = SearchOptions::default();

        let chunks = vec![
            create_test_chunk(1, "email: user@example.com"),
            create_test_chunk(2, "email: admin@test.org"),
            create_test_chunk(3, "not an email address"),
        ];

        // Email regex pattern
        let email_pattern = r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b";
        let results = search
            .search_chunks(email_pattern, &chunks, &options)
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[1].line_number, 2);
    }

    #[test]
    fn test_regex_match_positions() {
        let search = RegexSearch::new();
        let options = SearchOptions::default();

        let chunks = vec![create_test_chunk(1, "The word test appears here")];

        let results = search.search_chunks("test", &chunks, &options).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].start_char, 9); // Position of "test"
        assert_eq!(results[0].end_char, 13); // End of "test"
    }

    #[test]
    fn test_invalid_regex_handling() {
        let search = RegexSearch::new();
        let options = SearchOptions::default();

        let chunks = vec![create_test_chunk(1, "test content")];

        // Invalid regex pattern
        let result = search.search_chunks("[invalid", &chunks, &options);

        assert!(result.is_err());
    }

    #[test]
    fn test_regex_caching() {
        let search = RegexSearch::new();
        let options = SearchOptions::default();

        let chunks = vec![create_test_chunk(1, "test content")];

        // First search should cache the regex
        let _result1 = search.search_chunks("test", &chunks, &options).unwrap();

        // Second search should use cached regex
        let _result2 = search.search_chunks("test", &chunks, &options).unwrap();

        // Cache should contain the compiled regex
        let cache = search.cache.lock().unwrap();
        assert!(!cache.is_empty());
    }

    #[test]
    fn test_multiple_matches_per_line() {
        let search = RegexSearch::new();
        let options = SearchOptions::default();

        let chunks = vec![create_test_chunk(1, "test and test and test")];

        let results = search.search_chunks("test", &chunks, &options).unwrap();

        // Should find multiple matches in the same line
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.line_number == 1));

        // Matches should have different positions
        let positions: Vec<usize> = results.iter().map(|r| r.start_char).collect();
        assert_eq!(positions, vec![0, 9, 18]);
    }

    #[test]
    fn test_word_boundary_detection() {
        let search = RegexSearch::new();

        let regex_match = RegexMatch {
            start: 5,
            end: 9,
            text: "test".to_string(),
        };

        let content = "This test works";
        assert!(search.is_word_boundary_match(&regex_match, content));

        let content2 = "Thistest works";
        assert!(!search.is_word_boundary_match(&regex_match, content2));
    }

    #[test]
    fn test_score_calculation() {
        let search = RegexSearch::new();
        let options = SearchOptions::default();

        let regex_match = RegexMatch {
            start: 0,
            end: 4,
            text: "test".to_string(),
        };

        let score = search.calculate_regex_score(&regex_match, "test content", &options);

        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_line_start_bonus() {
        let search = RegexSearch::new();
        let options = SearchOptions::default();

        let chunks = vec![
            create_test_chunk(1, "test at start"),
            create_test_chunk(2, "not test here"),
        ];

        let results = search.search_chunks("test", &chunks, &options).unwrap();

        assert_eq!(results.len(), 2);
        // First result should have higher score due to line start bonus
        assert!(results[0].score >= results[1].score);
        assert_eq!(results[0].line_number, 1);
    }
}
