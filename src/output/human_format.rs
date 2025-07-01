use crate::{MatchType, SearchResult};
use std::time::Duration;

/// Human-friendly output formatter that prioritizes readability over technical details
pub struct HumanFormatter;

impl HumanFormatter {
    /// Format search results in a human-friendly way (simple mode)
    pub fn format_results(results: &[SearchResult], query: &str, _search_time: Duration) -> String {
        Self::format_results_unified(results, query, false, _search_time)
    }

    /// Format search results with technical details (advanced mode)
    pub fn format_results_advanced(
        results: &[SearchResult],
        query: &str,
        search_time: Duration,
    ) -> String {
        Self::format_results_unified(results, query, true, search_time)
    }

    /// Unified formatting function for all search result types
    fn format_results_unified(
        results: &[SearchResult],
        query: &str,
        advanced_mode: bool,
        search_time: Duration,
    ) -> String {
        if results.is_empty() {
            return Self::format_no_results(query);
        }

        let mut output = String::new();

        // Header with optional timing
        if advanced_mode {
            output.push_str(&format!(
                "Found {} matches in {:.2}s:\n\n",
                results.len(),
                search_time.as_secs_f64()
            ));
        } else if results.len() == 1 {
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

            // Show line and content with visual relevance bar and highlighting
            let content = result.content.trim();
            let relevance_bar = Self::get_relevance_bar(result.score, results.len(), advanced_mode);
            let highlighted_content = Self::highlight_match(content, query);

            output.push_str(&format!(
                "   Line {}: {}{}\n",
                result.line_number, relevance_bar, highlighted_content
            ));

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
            if advanced_mode {
                if let Some(score) = result.score {
                    output.push_str(&format!("   Score: {score:.2}\n"));
                    if score < 1.0 {
                        let relevance = score * 100.0;
                        output.push_str(&format!("   Relevance: {relevance:.1}%\n"));
                    }
                }

                if let Some(match_type) = &result.match_type {
                    if *match_type != MatchType::Exact {
                        output.push_str(&format!("   Match type: {match_type:?}\n"));
                    }
                }

                output.push('\n');
            }
        }

        // Show truncation message if there are more results
        if results.len() > 10 {
            output.push('\n');
            output.push_str(&format!("... and {} more matches\n", results.len() - 10));

            // Only show basic tips for small-to-moderate result sets
            // Let FeatureDiscovery handle guidance for larger result sets and user progression
            if results.len() >= 11 && results.len() <= 25 {
                output.push_str("💡 Tip: Use more specific terms to narrow results\n");
            }
        }

        output
    }

    /// Generate visual relevance bar based on semantic score
    fn get_relevance_bar(score: Option<f32>, _total_results: usize, advanced_mode: bool) -> String {
        // Show relevance bars for ANY results with scores
        // Following UX principles: visual feedback for scored results
        let should_show_bar = score.is_some();

        if !should_show_bar {
            return String::new();
        }

        let score_value = score.unwrap_or(0.0);

        // Debug: Check for problematic scores
        if score_value.is_nan() || score_value.is_infinite() || !(0.0..=10.0).contains(&score_value)
        {
            // Return empty string for invalid scores to avoid crashes
            return String::new();
        }

        // Create visual bar with consistent width for alignment
        let bar_width = 10;

        // Clamp score to valid range [0.0, 1.0] to prevent overflow
        let clamped_score = score_value.clamp(0.0, 1.0);
        let filled_width = (clamped_score * bar_width as f32) as usize;
        let filled_width = filled_width.min(bar_width); // Extra safety check
        let empty_width = bar_width.saturating_sub(filled_width); // Use saturating_sub to prevent underflow

        // Use different characters for different score ranges
        let (fill_char, empty_char, color_start, color_end) = if std::env::var("NO_COLOR").is_ok() {
            // No color mode - use ASCII characters
            ('█', '░', "", "")
        } else {
            // Color mode with ANSI escape codes
            match score_value {
                s if s >= 0.9 => ('█', '░', "\x1b[32m", "\x1b[0m"), // Green for excellent
                s if s >= 0.7 => ('█', '░', "\x1b[33m", "\x1b[0m"), // Yellow for good
                s if s >= 0.5 => ('█', '░', "\x1b[36m", "\x1b[0m"), // Cyan for okay
                _ => ('█', '░', "\x1b[37m", "\x1b[0m"),             // Gray for weak
            }
        };

        // Safety check: ensure we don't try to repeat with invalid values
        let filled_width = filled_width.min(bar_width);
        let empty_width = empty_width.min(bar_width);

        let bar = format!(
            "{}{}{}{}",
            color_start,
            fill_char.to_string().repeat(filled_width),
            empty_char.to_string().repeat(empty_width),
            color_end
        );

        if advanced_mode {
            format!("[{bar}] {score_value:.2} ")
        } else {
            format!("[{bar}] ")
        }
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

    /// Highlight matches in content with ANSI colors (respects NO_COLOR env var)
    pub fn highlight_match(content: &str, query: &str) -> String {
        // Check if colors should be disabled
        if std::env::var("NO_COLOR").is_ok() {
            return content.to_string();
        }

        // Try exact case-insensitive match first
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();

        // Find exact match using char indices to handle Unicode properly
        let content_chars: Vec<char> = content.chars().collect();
        let content_lower_chars: Vec<char> = content_lower.chars().collect();
        let query_lower_chars: Vec<char> = query_lower.chars().collect();

        // Look for exact match
        for i in 0..content_lower_chars.len() {
            if i + query_lower_chars.len() > content_lower_chars.len() {
                break;
            }

            let slice = &content_lower_chars[i..i + query_lower_chars.len()];
            if slice == query_lower_chars.as_slice() {
                // Found exact match, highlight it
                let before: String = content_chars[..i].iter().collect();
                let matched: String = content_chars[i..i + query_lower_chars.len()]
                    .iter()
                    .collect();
                let after: String = content_chars[i + query_lower_chars.len()..]
                    .iter()
                    .collect();

                return format!("{before}\x1b[1;33m{matched}\x1b[0m{after}");
            }
        }

        // Try fuzzy highlighting for partial matches
        // Look for the longest common substring or prefix match
        let mut best_match_start = None;
        let mut best_match_len = 0;

        // Check if query is a prefix of any word in content
        let mut word_start = 0;
        let mut in_word = false;

        for (i, ch) in content_lower_chars.iter().enumerate() {
            if ch.is_whitespace() {
                if in_word {
                    // End of word, check if it starts with query
                    let word_chars = &content_lower_chars[word_start..i];
                    if word_chars.len() >= query_lower_chars.len() {
                        let prefix = &word_chars[..query_lower_chars.len()];
                        if prefix == query_lower_chars.as_slice() {
                            best_match_start = Some(word_start);
                            best_match_len = query_lower_chars.len();
                            break;
                        }
                    }
                }
                in_word = false;
            } else if !in_word {
                word_start = i;
                in_word = true;
            }
        }

        // Check last word if we're still in one
        if in_word && best_match_start.is_none() {
            let word_chars = &content_lower_chars[word_start..];
            if word_chars.len() >= query_lower_chars.len() {
                let prefix = &word_chars[..query_lower_chars.len()];
                if prefix == query_lower_chars.as_slice() {
                    best_match_start = Some(word_start);
                    best_match_len = query_lower_chars.len();
                }
            }
        }

        // If no prefix match, look for any substring that shares significant characters
        if best_match_start.is_none() && query_lower_chars.len() >= 3 {
            // Simple fuzzy match: find first character and check consecutive matches
            for i in 0..content_lower_chars.len() {
                if content_lower_chars[i] == query_lower_chars[0] {
                    let mut matched = 1;

                    for j in 1..query_lower_chars.len() {
                        if i + j < content_lower_chars.len()
                            && content_lower_chars[i + j] == query_lower_chars[j]
                        {
                            matched += 1;
                        } else {
                            break;
                        }
                    }

                    // If we matched at least half the query, consider it a match
                    if matched >= query_lower_chars.len() / 2 && matched > best_match_len {
                        best_match_start = Some(i);
                        best_match_len = matched;
                    }
                }
            }
        }

        // Apply highlighting if we found a match
        if let Some(start) = best_match_start {
            let before: String = content_chars[..start].iter().collect();
            let matched: String = content_chars[start..start + best_match_len]
                .iter()
                .collect();
            let after: String = content_chars[start + best_match_len..].iter().collect();

            format!("{before}\x1b[1;33m{matched}\x1b[0m{after}")
        } else {
            // No match found, return content as-is
            content.to_string()
        }
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
        // Set NO_COLOR to disable ANSI color codes during testing
        std::env::set_var("NO_COLOR", "1");

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

        // Clean up environment variable
        std::env::remove_var("NO_COLOR");

        // Should show clean, simple output
        assert!(formatted.contains("Found 1 match"));
        assert!(formatted.contains("src/main.rs"));
        assert!(formatted.contains("Line 42:"));
        // Check for the main parts of the content (highlighting may affect exact match)
        assert!(formatted.contains("TODO"));
        assert!(formatted.contains("implement this feature"));

        // Should show relevance bars for ANY scored results (user requirement)
        assert!(formatted.contains("[")); // Visual bars shown for all scored results

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

    #[test]
    fn test_visual_relevance_bars() {
        // Create multiple results with varying scores to test visual bars
        let results = vec![
            SearchResult {
                file_path: "src/main.rs".to_string(),
                line_number: 10,
                content: "perfect match content".to_string(),
                score: Some(0.95),
                match_type: Some(MatchType::Semantic),
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "src/lib.rs".to_string(),
                line_number: 20,
                content: "good match content".to_string(),
                score: Some(0.75),
                match_type: Some(MatchType::Semantic),
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "src/utils.rs".to_string(),
                line_number: 30,
                content: "related content".to_string(),
                score: Some(0.55),
                match_type: Some(MatchType::Semantic),
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "src/test.rs".to_string(),
                line_number: 40,
                content: "possibly related content".to_string(),
                score: Some(0.35),
                match_type: Some(MatchType::Semantic),
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "src/other.rs".to_string(),
                line_number: 50,
                content: "weak match content".to_string(),
                score: Some(0.15),
                match_type: Some(MatchType::Semantic),
                context_before: None,
                context_after: None,
            },
        ];

        // Test simple mode - should show visual bars for moderate result count
        let formatted =
            HumanFormatter::format_results(&results, "content", Duration::from_millis(100));

        // Should show visual bars with different fill levels
        assert!(formatted.contains("[")); // Visual bar brackets
        assert!(formatted.contains("█")); // Filled bar characters
        assert!(formatted.contains("░")); // Empty bar characters

        // Should show all content
        assert!(formatted.contains("perfect match"));
        assert!(formatted.contains("good match"));
        assert!(formatted.contains("related"));
        assert!(formatted.contains("possibly related"));
        assert!(formatted.contains("weak match"));

        // Test advanced mode - should show bars with numeric scores
        let formatted_advanced = HumanFormatter::format_results_advanced(
            &results,
            "content",
            Duration::from_millis(100),
        );

        // Should show visual bars with scores in advanced mode
        assert!(formatted_advanced.contains("["));
        assert!(formatted_advanced.contains("0.95")); // High score
        assert!(formatted_advanced.contains("0.75")); // Good score
        assert!(formatted_advanced.contains("0.55")); // Medium score
        assert!(formatted_advanced.contains("Score:")); // Technical details
    }
}
