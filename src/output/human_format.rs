use crate::{MatchType, SearchResult};
use std::time::Duration;

/// Human-friendly output formatter that prioritizes readability over technical details
pub struct HumanFormatter;

impl HumanFormatter {
    /// Format search results in a human-friendly way (simple mode)
    pub fn format_results(results: &[SearchResult], query: &str, _search_time: Duration) -> String {
        if results.is_empty() {
            return Self::format_no_results(query);
        }

        let mut output = String::new();

        // Simple, clear header
        if results.len() == 1 {
            output.push_str("Found 1 match:\n\n");
        } else {
            output.push_str(&format!("Found {} matches:\n\n", results.len()));
        }

        // Group results by file to reduce visual noise
        let mut current_file = "";
        let results_to_show = results.iter().take(10);

        for result in results_to_show {
            // Show file path only when it changes (grouping)
            if result.file_path != current_file {
                output.push_str(&format!("📁 {}\n", result.file_path));
                current_file = &result.file_path;
            }

            // Show context before if available
            if let Some(ref context_before) = result.context_before {
                for (i, line) in context_before.iter().enumerate() {
                    let line_num = result.line_number.saturating_sub(context_before.len() - i);
                    output.push_str(&format!("   Line {}: {}\n", line_num, line.trim()));
                }
            }

            // Show line and content with simple formatting
            let content = result.content.trim();
            output.push_str(&format!("   Line {}: {}\n", result.line_number, content));

            // Show context after if available
            if let Some(ref context_after) = result.context_after {
                for (i, line) in context_after.iter().enumerate() {
                    let line_num = result.line_number + i + 1;
                    output.push_str(&format!("   Line {}: {}\n", line_num, line.trim()));
                }
                output.push_str("   ---\n"); // Separator between matches
            }
        }

        // Show truncation message if there are more results
        if results.len() > 10 {
            output.push('\n');
            output.push_str(&format!("... and {} more matches\n", results.len() - 10));
            output.push_str("💡 Tip: Use more specific terms to narrow results\n");
        }

        output
    }

    /// Format search results with technical details (advanced mode)
    pub fn format_results_advanced(
        results: &[SearchResult],
        query: &str,
        search_time: Duration,
    ) -> String {
        if results.is_empty() {
            return Self::format_no_results(query);
        }

        let mut output = String::new();

        // Advanced header with timing
        output.push_str(&format!(
            "Found {} matches in {:.2}s:\n\n",
            results.len(),
            search_time.as_secs_f64()
        ));

        let mut current_file = "";
        let results_to_show = results.iter().take(10);

        for result in results_to_show {
            // Show file path only when it changes
            if result.file_path != current_file {
                output.push_str(&format!("📁 {}\n", result.file_path));
                current_file = &result.file_path;
            }

            // Show context before if available
            if let Some(ref context_before) = result.context_before {
                for (i, line) in context_before.iter().enumerate() {
                    let line_num = result.line_number.saturating_sub(context_before.len() - i);
                    output.push_str(&format!("   Line {}: {}\n", line_num, line.trim()));
                }
            }

            // Show line and content
            let content = result.content.trim();
            output.push_str(&format!("   Line {}: {}\n", result.line_number, content));

            // Show context after if available
            if let Some(ref context_after) = result.context_after {
                for (i, line) in context_after.iter().enumerate() {
                    let line_num = result.line_number + i + 1;
                    output.push_str(&format!("   Line {}: {}\n", line_num, line.trim()));
                }
                if !context_after.is_empty() {
                    output.push_str("   ---\n"); // Separator between matches
                }
            }

            // Show technical details in advanced mode
            if let Some(score) = result.score {
                if score < 1.0 {
                    output.push_str(&format!("   Relevance: {:.1}%\n", score * 100.0));
                }
            }

            if let Some(match_type) = &result.match_type {
                if *match_type != MatchType::Exact {
                    output.push_str(&format!("   Match type: {match_type:?}\n"));
                }
            }

            output.push('\n');
        }

        // Show truncation message if there are more results
        if results.len() > 10 {
            output.push_str(&format!("... and {} more matches\n", results.len() - 10));
            output.push_str("💡 Tip: Use more specific terms to narrow results\n");
        }

        output
    }

    /// Format no results message with helpful suggestions
    pub fn format_no_results(query: &str) -> String {
        let simplified = Self::suggest_alternative(query);

        format!(
            "No matches found for '{query}'.\n\n\
             Try:\n\
             • Check spelling: semisearch '{query}' --fuzzy\n\
             • Use different words: semisearch '{simplified}'\n\
             • Search everywhere: semisearch '{query}' .\n\
             • Get help: semisearch help-me\n"
        )
    }

    /// Highlight matches in content (simple highlighting for readability)
    pub fn highlight_match(content: &str, _query: &str) -> String {
        // For human-friendly output, we keep highlighting minimal to maintain readability
        // In terminal output, we might use ANSI colors, but for now keep it simple
        content.to_string()
    }

    /// Suggest simpler alternative terms for a query
    fn suggest_alternative(query: &str) -> String {
        // Simple query simplification - remove common noise words and use first significant term
        let words: Vec<&str> = query
            .split_whitespace()
            .filter(|word| {
                ![
                    "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of",
                    "with", "by",
                ]
                .contains(&word.to_lowercase().as_str())
            })
            .collect();

        if words.is_empty() {
            query.to_string()
        } else if words.len() == 1 {
            words[0].to_string()
        } else {
            // Take first two significant words
            words.iter().take(2).cloned().collect::<Vec<_>>().join(" ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_single_result() {
        let results = vec![SearchResult {
            file_path: "src/main.rs".to_string(),
            line_number: 42,
            content: "    // TODO: implement this feature".to_string(),
            score: Some(1.0),
            match_type: Some(MatchType::Exact),
            context_before: None,
            context_after: None,
        }];

        let formatted =
            HumanFormatter::format_results(&results, "TODO", Duration::from_millis(150));

        // Should show clean, simple output
        assert!(formatted.contains("Found 1 match"));
        assert!(formatted.contains("src/main.rs"));
        assert!(formatted.contains("Line 42:"));
        assert!(formatted.contains("TODO: implement this feature"));

        // Should NOT show technical details in simple mode
        assert!(!formatted.contains("Score:"));
        assert!(!formatted.contains("Relevance:"));
        assert!(!formatted.contains("MatchType:"));
        assert!(!formatted.contains("0.15s"));
    }

    #[test]
    fn test_format_multiple_results_same_file() {
        let results = vec![
            SearchResult {
                file_path: "src/lib.rs".to_string(),
                line_number: 10,
                content: "    // TODO: refactor this".to_string(),
                score: Some(0.95),
                match_type: Some(MatchType::Exact),
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "src/lib.rs".to_string(),
                line_number: 25,
                content: "    // TODO: add tests".to_string(),
                score: Some(0.92),
                match_type: Some(MatchType::Exact),
                context_before: None,
                context_after: None,
            },
        ];

        let formatted =
            HumanFormatter::format_results(&results, "TODO", Duration::from_millis(250));

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
    fn test_format_many_results_truncation() {
        let mut results = Vec::new();
        for i in 1..=15 {
            results.push(SearchResult {
                file_path: format!("src/file{i}.rs"),
                line_number: i,
                content: format!("    // TODO: task number {i}"),
                score: Some(1.0 - (i as f32 * 0.01)),
                match_type: Some(MatchType::Exact),
                context_before: None,
                context_after: None,
            });
        }

        let formatted =
            HumanFormatter::format_results(&results, "TODO", Duration::from_millis(500));

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
    fn test_format_no_results() {
        let formatted = HumanFormatter::format_no_results("nonexistent_term");

        // Should provide helpful suggestions
        assert!(formatted.contains("No matches found"));
        assert!(formatted.contains("nonexistent_term"));
        assert!(formatted.contains("Try:"));

        // Should suggest specific actions
        assert!(formatted.contains("--fuzzy"));
        assert!(formatted.contains("different words"));
        assert!(formatted.contains("help-me"));
    }

    #[test]
    fn test_advanced_mode_shows_technical_details() {
        let results = vec![SearchResult {
            file_path: "src/test.rs".to_string(),
            line_number: 42,
            content: "    // TODO: advanced details".to_string(),
            score: Some(0.8),
            match_type: Some(MatchType::Hybrid),
            context_before: None,
            context_after: None,
        }];

        let formatted =
            HumanFormatter::format_results_advanced(&results, "TODO", Duration::from_millis(250));

        // Advanced mode SHOULD show:
        assert!(formatted.contains("0.25s"));
        assert!(formatted.contains("80.0%"));
        assert!(formatted.contains("Hybrid"));
    }

    #[test]
    fn test_suggest_alternative() {
        assert_eq!(
            HumanFormatter::suggest_alternative("function name"),
            "function name"
        );
        assert_eq!(
            HumanFormatter::suggest_alternative("the function"),
            "function"
        );
        assert_eq!(
            HumanFormatter::suggest_alternative("a complex function with parameters"),
            "complex function"
        );
        assert_eq!(HumanFormatter::suggest_alternative("TODO"), "TODO");
    }
}
