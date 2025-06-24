#[allow(unused_imports)]
use super::{MatchType, SearchResult};

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
                result.context_before = Some(
                    lines[start_line..line_index]
                        .iter()
                        .map(|&s| s.to_string())
                        .collect(),
                );
            }

            // Add context after
            let end_line = (line_index + 1 + context_lines).min(lines.len());
            if end_line > line_index + 1 {
                result.context_after = Some(
                    lines[line_index + 1..end_line]
                        .iter()
                        .map(|&s| s.to_string())
                        .collect(),
                );
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

        format!(
            "{}<{}>{}</{}>{}",
            before, highlight_tag, matched, highlight_tag, after
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_relevance_score() {
        let score1 = SearchHelper::calculate_relevance_score(0.8, 5, 100, 10); // Near beginning
        let score2 = SearchHelper::calculate_relevance_score(0.8, 5, 100, 80); // Near end
        let score3 = SearchHelper::calculate_relevance_score(0.8, 2, 100, 10); // Short query
        let score4 = SearchHelper::calculate_relevance_score(0.8, 5, 2000, 10); // Long content

        assert!(score1 > score2); // Position bonus
        assert!(score1 > score3); // Query length bonus
        assert!(score1 > score4); // Length penalty
    }

    #[test]
    fn test_calculate_term_overlap() {
        let query_terms = vec!["machine".to_string(), "learning".to_string()];
        let content_terms1 = vec![
            "machine".to_string(),
            "learning".to_string(),
            "algorithm".to_string(),
        ];
        let content_terms2 = vec!["machine".to_string(), "vision".to_string()];
        let content_terms3 = vec!["deep".to_string(), "neural".to_string()];

        let overlap1 = SearchHelper::calculate_term_overlap(&query_terms, &content_terms1);
        let overlap2 = SearchHelper::calculate_term_overlap(&query_terms, &content_terms2);
        let overlap3 = SearchHelper::calculate_term_overlap(&query_terms, &content_terms3);

        assert_eq!(overlap1, 1.0); // Complete overlap
        assert_eq!(overlap2, 0.5); // Partial overlap
        assert_eq!(overlap3, 0.0); // No overlap
    }

    #[test]
    fn test_highlight_matches() {
        let content = "This is a test example";
        let highlighted = SearchHelper::highlight_matches(content, 10, 14, "mark");

        assert_eq!(highlighted, "This is a <mark>test</mark> example");
    }

    #[test]
    fn test_merge_strategy_results() {
        let results1 = vec![SearchResult {
            file_path: "file1.txt".to_string(),
            line_number: 1,
            content: "content1".to_string(),
            score: 0.9,
            match_type: MatchType::Keyword,
            start_char: 0,
            end_char: 4,
            context_before: None,
            context_after: None,
        }];

        let results2 = vec![
            SearchResult {
                file_path: "file1.txt".to_string(),
                line_number: 1,
                content: "content1".to_string(),
                score: 0.8,
                match_type: MatchType::Fuzzy,
                start_char: 0,
                end_char: 4,
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "file2.txt".to_string(),
                line_number: 1,
                content: "content2".to_string(),
                score: 0.7,
                match_type: MatchType::Regex,
                start_char: 0,
                end_char: 4,
                context_before: None,
                context_after: None,
            },
        ];

        let merged = SearchHelper::merge_strategy_results(vec![results1, results2], 10);

        assert_eq!(merged.len(), 2); // Duplicates removed
        assert_eq!(merged[0].score, 0.9); // Sorted by score
        assert_eq!(merged[1].score, 0.7);
    }

    #[test]
    fn test_add_context() {
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
        let mut results = vec![SearchResult {
            file_path: "test.txt".to_string(),
            line_number: 3,
            content: "Line 3".to_string(),
            score: 0.8,
            match_type: MatchType::Keyword,
            start_char: 0,
            end_char: 6,
            context_before: None,
            context_after: None,
        }];

        SearchHelper::add_context(&mut results, content, 1);

        assert!(results[0].context_before.is_some());
        assert!(results[0].context_after.is_some());
        assert_eq!(
            results[0].context_before.as_ref().unwrap(),
            &vec!["Line 2".to_string()]
        );
        assert_eq!(
            results[0].context_after.as_ref().unwrap(),
            &vec!["Line 4".to_string()]
        );
    }

    #[test]
    fn test_empty_query_overlap() {
        let query_terms: Vec<String> = vec![];
        let content_terms = vec!["test".to_string()];

        let overlap = SearchHelper::calculate_term_overlap(&query_terms, &content_terms);
        assert_eq!(overlap, 0.0);
    }

    #[test]
    fn test_highlight_edge_cases() {
        let content = "test";

        // Out of bounds
        let highlighted1 = SearchHelper::highlight_matches(content, 10, 15, "mark");
        assert_eq!(highlighted1, "test");

        // Start >= end
        let highlighted2 = SearchHelper::highlight_matches(content, 2, 1, "mark");
        assert_eq!(highlighted2, "test");

        // End beyond content
        let highlighted3 = SearchHelper::highlight_matches(content, 0, 10, "mark");
        assert_eq!(highlighted3, "<mark>test</mark>");
    }
}
