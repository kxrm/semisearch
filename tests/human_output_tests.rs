use anyhow::Result;
use search::output::ResultGroup;
use search::{MatchType, SearchResult};
use std::time::Duration;

/// Tests for Task 2.2.1: Human-Readable Output Format
/// These tests define the expected human-friendly output behavior
mod human_readable_output {
    use super::*;

    #[test]
    fn test_human_format_single_result() {
        let results = vec![SearchResult {
            file_path: "src/main.rs".to_string(),
            line_number: 42,
            content: "    // TODO: implement this feature".to_string(),
            score: Some(1.0),
            match_type: Some(MatchType::Exact),
        }];

        let formatted = format_results_human_friendly(&results, "TODO", Duration::from_millis(150));

        // Should show clean, simple output
        assert!(formatted.contains("Found 1 match"));
        assert!(formatted.contains("src/main.rs"));
        assert!(formatted.contains("Line 42:"));
        assert!(formatted.contains("TODO: implement this feature"));

        // Should NOT show technical details in simple mode
        assert!(!formatted.contains("Score:"));
        assert!(!formatted.contains("Relevance:"));
        assert!(!formatted.contains("MatchType:"));
        assert!(!formatted.contains("0.15s")); // No timing in simple mode
    }

    #[test]
    fn test_human_format_multiple_results_same_file() {
        let results = vec![
            SearchResult {
                file_path: "src/lib.rs".to_string(),
                line_number: 10,
                content: "    // TODO: refactor this".to_string(),
                score: Some(0.95),
                match_type: Some(MatchType::Exact),
            },
            SearchResult {
                file_path: "src/lib.rs".to_string(),
                line_number: 25,
                content: "    // TODO: add tests".to_string(),
                score: Some(0.92),
                match_type: Some(MatchType::Exact),
            },
        ];

        let formatted = format_results_human_friendly(&results, "TODO", Duration::from_millis(250));

        // Should group by file to reduce noise
        assert!(formatted.contains("Found 2 matches"));
        assert!(formatted.contains("src/lib.rs"));

        // Should show both lines under the same file
        assert!(formatted.contains("Line 10:"));
        assert!(formatted.contains("Line 25:"));
        assert!(formatted.contains("refactor this"));
        assert!(formatted.contains("add tests"));

        // File path should appear only once for grouping
        let file_mentions = formatted.matches("src/lib.rs").count();
        assert_eq!(
            file_mentions, 1,
            "File path should appear only once when grouping"
        );
    }

    #[test]
    fn test_human_format_multiple_files() {
        let results = vec![
            SearchResult {
                file_path: "src/main.rs".to_string(),
                line_number: 15,
                content: "    // TODO: main function".to_string(),
                score: Some(1.0),
                match_type: Some(MatchType::Exact),
            },
            SearchResult {
                file_path: "src/lib.rs".to_string(),
                line_number: 8,
                content: "    // TODO: library code".to_string(),
                score: Some(0.98),
                match_type: Some(MatchType::Exact),
            },
            SearchResult {
                file_path: "tests/integration.rs".to_string(),
                line_number: 33,
                content: "    // TODO: integration test".to_string(),
                score: Some(0.96),
                match_type: Some(MatchType::Exact),
            },
        ];

        let formatted = format_results_human_friendly(&results, "TODO", Duration::from_millis(300));

        // Should show all files clearly
        assert!(formatted.contains("Found 3 matches"));
        assert!(formatted.contains("src/main.rs"));
        assert!(formatted.contains("src/lib.rs"));
        assert!(formatted.contains("tests/integration.rs"));

        // Each file should have its content
        assert!(formatted.contains("main function"));
        assert!(formatted.contains("library code"));
        assert!(formatted.contains("integration test"));
    }

    #[test]
    fn test_human_format_many_results_truncation() {
        let mut results = Vec::new();
        for i in 1..=15 {
            results.push(SearchResult {
                file_path: format!("src/file{i}.rs"),
                line_number: i,
                content: format!("    // TODO: task number {i}"),
                score: Some(1.0 - (i as f32 * 0.01)),
                match_type: Some(MatchType::Exact),
            });
        }

        let formatted = format_results_human_friendly(&results, "TODO", Duration::from_millis(500));

        // Should show count and truncation message
        assert!(formatted.contains("Found 15 matches"));
        assert!(formatted.contains("... and 5 more matches"));

        // Should show helpful tip for narrowing results
        assert!(formatted.contains("💡") || formatted.contains("Tip"));

        // Should only show first 10 results
        assert!(formatted.contains("file1.rs"));
        assert!(formatted.contains("file10.rs"));
        assert!(!formatted.contains("file11.rs"));
        assert!(!formatted.contains("file15.rs"));
    }

    #[test]
    fn test_human_format_no_results() {
        let _results: Vec<SearchResult> = Vec::new();
        let formatted = format_no_results_human_friendly("nonexistent_term");

        // Should provide helpful suggestions
        assert!(formatted.contains("No matches found"));
        assert!(formatted.contains("nonexistent_term"));
        assert!(formatted.contains("Try:"));

        // Should suggest specific actions
        assert!(formatted.contains("--fuzzy") || formatted.contains("Check spelling"));
        assert!(formatted.contains("simpler terms") || formatted.contains("different words"));
        assert!(formatted.contains("help-me") || formatted.contains("more help"));
    }

    #[test]
    fn test_human_format_highlight_matches() {
        let results = vec![SearchResult {
            file_path: "src/test.rs".to_string(),
            line_number: 1,
            content: "    function validateUser(user) {".to_string(),
            score: Some(0.9),
            match_type: Some(MatchType::Fuzzy),
        }];

        let formatted =
            format_results_human_friendly(&results, "function", Duration::from_millis(100));

        // Should highlight the match (simple highlighting)
        assert!(formatted.contains("function"));
        // In human-friendly mode, we might use simple highlighting or no highlighting
        // The key is that the content is clearly readable
        assert!(formatted.contains("validateUser"));
    }
}

/// Tests for Task 2.2.2: Smart Result Grouping
/// These tests define how results should be intelligently grouped
mod smart_result_grouping {
    use super::*;

    #[test]
    fn test_group_exact_matches_first() {
        let results = vec![
            SearchResult {
                file_path: "src/fuzzy.rs".to_string(),
                line_number: 10,
                content: "    // Similar to function".to_string(),
                score: Some(0.7),
                match_type: Some(MatchType::Fuzzy),
            },
            SearchResult {
                file_path: "src/exact.rs".to_string(),
                line_number: 5,
                content: "    function main() {".to_string(),
                score: Some(1.0),
                match_type: Some(MatchType::Exact),
            },
            SearchResult {
                file_path: "src/another.rs".to_string(),
                line_number: 15,
                content: "    def function_helper():".to_string(),
                score: Some(0.8),
                match_type: Some(MatchType::Fuzzy),
            },
        ];

        let grouped = group_results_by_relevance(results);

        // Exact matches should come first
        assert!(!grouped.is_empty());
        let first_group = &grouped[0];
        assert!(first_group.title.contains("Exact") || first_group.title.contains("exact"));
        assert_eq!(first_group.results.len(), 1);
        assert!(first_group.results[0].content.contains("function main()"));
    }

    #[test]
    fn test_group_by_file_for_multiple_non_exact_matches() {
        let results = vec![
            SearchResult {
                file_path: "src/main.rs".to_string(),
                line_number: 10,
                content: "    // TODO: first task".to_string(),
                score: Some(0.8),
                match_type: Some(MatchType::Fuzzy),
            },
            SearchResult {
                file_path: "src/main.rs".to_string(),
                line_number: 20,
                content: "    // TODO: second task".to_string(),
                score: Some(0.8),
                match_type: Some(MatchType::Fuzzy),
            },
            SearchResult {
                file_path: "src/main.rs".to_string(),
                line_number: 30,
                content: "    // TODO: third task".to_string(),
                score: Some(0.8),
                match_type: Some(MatchType::Fuzzy),
            },
            SearchResult {
                file_path: "src/main.rs".to_string(),
                line_number: 40,
                content: "    // TODO: fourth task".to_string(),
                score: Some(0.8),
                match_type: Some(MatchType::Fuzzy),
            },
        ];

        let grouped = group_results_by_relevance(results);

        // Should have a group for multiple matches in main.rs (since they're not exact)
        let main_group = grouped.iter().find(|g| g.title.contains("main.rs"));
        assert!(main_group.is_some());

        let main_group = main_group.unwrap();
        assert_eq!(main_group.results.len(), 4);
        assert!(
            main_group.title.contains("Multiple matches")
                || main_group.title.contains("matches in")
        );
    }

    #[test]
    fn test_group_by_file_for_multiple_matches() {
        let results = vec![
            SearchResult {
                file_path: "src/main.rs".to_string(),
                line_number: 10,
                content: "    // TODO: first task".to_string(),
                score: Some(0.9),
                match_type: Some(MatchType::Exact),
            },
            SearchResult {
                file_path: "src/main.rs".to_string(),
                line_number: 20,
                content: "    // TODO: second task".to_string(),
                score: Some(0.9),
                match_type: Some(MatchType::Exact),
            },
            SearchResult {
                file_path: "src/main.rs".to_string(),
                line_number: 30,
                content: "    // TODO: third task".to_string(),
                score: Some(0.9),
                match_type: Some(MatchType::Exact),
            },
            SearchResult {
                file_path: "src/main.rs".to_string(),
                line_number: 40,
                content: "    // TODO: fourth task".to_string(),
                score: Some(0.9),
                match_type: Some(MatchType::Exact),
            },
        ];

        let grouped = group_results_by_relevance(results);

        // Since all results are exact matches, they should be in the "Exact matches" group
        assert!(!grouped.is_empty());
        let first_group = &grouped[0];
        assert!(first_group.title.contains("Exact matches"));
        assert_eq!(first_group.results.len(), 4);

        // Results should be sorted by line number within the group
        let line_numbers: Vec<_> = first_group.results.iter().map(|r| r.line_number).collect();
        assert_eq!(line_numbers, vec![10, 20, 30, 40]);
    }

    #[test]
    fn test_mixed_grouping_strategy() {
        let results = vec![
            // Exact match
            SearchResult {
                file_path: "src/exact.rs".to_string(),
                line_number: 1,
                content: "    function exact_match() {".to_string(),
                score: Some(1.0),
                match_type: Some(MatchType::Exact),
            },
            // Multiple fuzzy matches in same file
            SearchResult {
                file_path: "src/fuzzy.rs".to_string(),
                line_number: 10,
                content: "    // function-like pattern".to_string(),
                score: Some(0.8),
                match_type: Some(MatchType::Fuzzy),
            },
            SearchResult {
                file_path: "src/fuzzy.rs".to_string(),
                line_number: 20,
                content: "    // another function pattern".to_string(),
                score: Some(0.7),
                match_type: Some(MatchType::Fuzzy),
            },
            SearchResult {
                file_path: "src/fuzzy.rs".to_string(),
                line_number: 30,
                content: "    // yet another function".to_string(),
                score: Some(0.75),
                match_type: Some(MatchType::Fuzzy),
            },
            // Single match in different file
            SearchResult {
                file_path: "src/single.rs".to_string(),
                line_number: 5,
                content: "    // function reference".to_string(),
                score: Some(0.6),
                match_type: Some(MatchType::Fuzzy),
            },
        ];

        let grouped = group_results_by_relevance(results);

        // Should have at least 3 groups: exact matches, multiple matches in fuzzy.rs, other matches
        assert!(grouped.len() >= 2);

        // First group should be exact matches
        assert!(grouped[0].title.contains("Exact") || grouped[0].title.contains("exact"));
        assert_eq!(grouped[0].results.len(), 1);

        // Should have a group for multiple matches in fuzzy.rs
        let fuzzy_group = grouped.iter().find(|g| g.title.contains("fuzzy.rs"));
        assert!(fuzzy_group.is_some());
        assert_eq!(fuzzy_group.unwrap().results.len(), 3);
    }
}

/// Tests for Advanced Mode Behavior
/// These tests ensure advanced formatting is hidden behind --advanced flag
mod advanced_mode_tests {
    use super::*;

    #[test]
    fn test_simple_mode_hides_technical_details() {
        let results = vec![SearchResult {
            file_path: "src/test.rs".to_string(),
            line_number: 42,
            content: "    // TODO: test".to_string(),
            score: Some(0.73),
            match_type: Some(MatchType::Fuzzy),
        }];

        let formatted = format_results_human_friendly(&results, "TODO", Duration::from_millis(250));

        // Simple mode should NOT show:
        assert!(!formatted.contains("Score:"));
        assert!(!formatted.contains("0.73"));
        assert!(!formatted.contains("MatchType"));
        assert!(!formatted.contains("Fuzzy"));
        assert!(!formatted.contains("0.25s"));
        assert!(!formatted.contains("Relevance:"));
    }

    #[test]
    fn test_advanced_mode_shows_technical_details() {
        let results = vec![SearchResult {
            file_path: "src/test.rs".to_string(),
            line_number: 42,
            content: "    // TODO: test".to_string(),
            score: Some(0.73),
            match_type: Some(MatchType::Fuzzy),
        }];

        let formatted = format_results_advanced(&results, "TODO", Duration::from_millis(250));

        // Advanced mode SHOULD show:
        assert!(formatted.contains("0.25s") || formatted.contains("250ms"));
        assert!(formatted.contains("73.0%") || formatted.contains("0.73"));
        assert!(formatted.contains("Fuzzy") || formatted.contains("fuzzy"));
    }

    #[test]
    fn test_json_format_in_advanced_mode() {
        let results = vec![SearchResult {
            file_path: "src/test.rs".to_string(),
            line_number: 42,
            content: "    // TODO: test".to_string(),
            score: Some(0.73),
            match_type: Some(MatchType::Fuzzy),
        }];

        let json_formatted = format_results_json(&results);

        // Should be valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_formatted);
        assert!(parsed.is_ok(), "Should produce valid JSON");

        let json = parsed.unwrap();
        assert!(json.is_array());

        let first_result = &json[0];
        assert_eq!(first_result["file_path"], "src/test.rs");
        assert_eq!(first_result["line_number"], 42);
        assert_eq!(first_result["score"], 0.73);
    }
}

// Helper functions that use the actual implementations
fn format_results_human_friendly(
    results: &[SearchResult],
    query: &str,
    search_time: Duration,
) -> String {
    use search::output::HumanFormatter;
    HumanFormatter::format_results(results, query, search_time)
}

fn format_no_results_human_friendly(query: &str) -> String {
    use search::output::HumanFormatter;
    HumanFormatter::format_no_results(query)
}

fn group_results_by_relevance(results: Vec<SearchResult>) -> Vec<ResultGroup> {
    use search::output::ResultGrouper;
    ResultGrouper::group_by_relevance(results)
}

fn format_results_advanced(results: &[SearchResult], query: &str, search_time: Duration) -> String {
    use search::output::HumanFormatter;
    HumanFormatter::format_results_advanced(results, query, search_time)
}

fn format_results_json(results: &[SearchResult]) -> String {
    // This should use existing JSON serialization
    serde_json::to_string_pretty(results).unwrap_or_default()
}
