use crate::{MatchType, SearchResult};
use std::collections::HashMap;

/// A group of search results with a descriptive title
#[derive(Debug, Clone)]
pub struct ResultGroup {
    pub title: String,
    pub results: Vec<SearchResult>,
}

/// Smart result grouper that organizes results in a logical, human-friendly way
pub struct ResultGrouper;

impl ResultGrouper {
    /// Group results by relevance and context for better user experience
    pub fn group_by_relevance(results: Vec<SearchResult>) -> Vec<ResultGroup> {
        if results.is_empty() {
            return Vec::new();
        }

        let mut groups = Vec::new();

        // Step 1: Separate exact matches (highest priority)
        let (exact_matches, other_matches): (Vec<_>, Vec<_>) = results
            .into_iter()
            .partition(|r| matches!(r.match_type, Some(MatchType::Exact)));

        // Add exact matches group if any exist
        if !exact_matches.is_empty() {
            groups.push(ResultGroup {
                title: if exact_matches.len() == 1 {
                    "Exact match".to_string()
                } else {
                    format!("Exact matches ({})", exact_matches.len())
                },
                results: exact_matches,
            });
        }

        // Step 2: Group remaining results by file for files with multiple matches
        let mut by_file: HashMap<String, Vec<SearchResult>> = HashMap::new();
        for result in other_matches {
            by_file
                .entry(result.file_path.clone())
                .or_default()
                .push(result);
        }

        // Step 3: Create groups based on file match count
        let mut multiple_match_files = Vec::new();
        let mut single_match_results = Vec::new();

        for (file_path, mut file_results) in by_file {
            if file_results.len() >= 3 {
                // Files with 3+ matches get their own group
                file_results.sort_by(|a, b| a.line_number.cmp(&b.line_number));
                multiple_match_files.push((file_path, file_results));
            } else {
                // Files with 1-2 matches go into general groups
                single_match_results.extend(file_results);
            }
        }

        // Step 4: Add groups for files with multiple matches
        multiple_match_files.sort_by(|a, b| b.1.len().cmp(&a.1.len())); // Sort by match count descending
        for (file_path, file_results) in multiple_match_files {
            let filename = Self::extract_filename(&file_path);
            groups.push(ResultGroup {
                title: format!("Multiple matches in {} ({})", filename, file_results.len()),
                results: file_results,
            });
        }

        // Step 5: Group remaining single/double matches by match type or score
        if !single_match_results.is_empty() {
            // Sort by score (descending) then by file path for consistency
            single_match_results.sort_by(|a, b| {
                let score_cmp = b
                    .score
                    .unwrap_or(0.0)
                    .partial_cmp(&a.score.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal);
                if score_cmp != std::cmp::Ordering::Equal {
                    score_cmp
                } else {
                    a.file_path.cmp(&b.file_path)
                }
            });

            // Split into high-quality and lower-quality matches
            let (high_quality, lower_quality): (Vec<_>, Vec<_>) = single_match_results
                .into_iter()
                .partition(|r| r.score.unwrap_or(0.0) >= 0.8);

            if !high_quality.is_empty() {
                groups.push(ResultGroup {
                    title: if high_quality.len() == 1 {
                        "Good match".to_string()
                    } else {
                        format!("Good matches ({})", high_quality.len())
                    },
                    results: high_quality,
                });
            }

            if !lower_quality.is_empty() {
                groups.push(ResultGroup {
                    title: if lower_quality.len() == 1 {
                        "Other match".to_string()
                    } else {
                        format!("Other matches ({})", lower_quality.len())
                    },
                    results: lower_quality,
                });
            }
        }

        groups
    }

    /// Group results for simple display (used in human-friendly formatting)
    pub fn group_for_simple_display(results: Vec<SearchResult>) -> Vec<ResultGroup> {
        if results.is_empty() {
            return Vec::new();
        }

        // For simple display, we use a simpler grouping strategy
        let mut groups = Vec::new();

        // Group by file, maintaining order
        let mut by_file: HashMap<String, Vec<SearchResult>> = HashMap::new();
        for result in results {
            by_file
                .entry(result.file_path.clone())
                .or_default()
                .push(result);
        }

        // Convert to groups, sorted by total score of file
        let mut file_groups: Vec<_> = by_file.into_iter().collect();
        file_groups.sort_by(|a, b| {
            let score_a: f32 = a.1.iter().map(|r| r.score.unwrap_or(0.0)).sum();
            let score_b: f32 = b.1.iter().map(|r| r.score.unwrap_or(0.0)).sum();
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (file_path, mut file_results) in file_groups {
            // Sort results within file by line number
            file_results.sort_by(|a, b| a.line_number.cmp(&b.line_number));

            let filename = Self::extract_filename(&file_path);
            let title = if file_results.len() == 1 {
                filename
            } else {
                format!("{} ({} matches)", filename, file_results.len())
            };

            groups.push(ResultGroup {
                title,
                results: file_results,
            });
        }

        groups
    }

    /// Extract just the filename from a full path for cleaner display
    fn extract_filename(path: &str) -> String {
        std::path::Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(path)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
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
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "src/exact.rs".to_string(),
                line_number: 5,
                content: "    function main() {".to_string(),
                score: Some(1.0),
                match_type: Some(MatchType::Exact),
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "src/another.rs".to_string(),
                line_number: 15,
                content: "    def function_helper():".to_string(),
                score: Some(0.8),
                match_type: Some(MatchType::Fuzzy),
                context_before: None,
                context_after: None,
            },
        ];

        let grouped = ResultGrouper::group_by_relevance(results);

        // Exact matches should come first
        assert!(!grouped.is_empty());
        let first_group = &grouped[0];
        assert!(first_group.title.contains("Exact") || first_group.title.contains("exact"));
        assert_eq!(first_group.results.len(), 1);
        assert!(first_group.results[0].content.contains("function main()"));
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
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "src/main.rs".to_string(),
                line_number: 20,
                content: "    // TODO: second task".to_string(),
                score: Some(0.9),
                match_type: Some(MatchType::Exact),
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "src/main.rs".to_string(),
                line_number: 30,
                content: "    // TODO: third task".to_string(),
                score: Some(0.9),
                match_type: Some(MatchType::Exact),
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "src/main.rs".to_string(),
                line_number: 40,
                content: "    // TODO: fourth task".to_string(),
                score: Some(0.9),
                match_type: Some(MatchType::Exact),
                context_before: None,
                context_after: None,
            },
        ];

        let grouped = ResultGrouper::group_by_relevance(results);

        // Should have exact matches group
        assert!(!grouped.is_empty());
        let first_group = &grouped[0];
        assert!(first_group.title.contains("Exact matches"));
        assert_eq!(first_group.results.len(), 4);

        // Results should be sorted by line number
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
                context_before: None,
                context_after: None,
            },
            // Multiple fuzzy matches in same file
            SearchResult {
                file_path: "src/fuzzy.rs".to_string(),
                line_number: 10,
                content: "    // function-like pattern".to_string(),
                score: Some(0.8),
                match_type: Some(MatchType::Fuzzy),
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "src/fuzzy.rs".to_string(),
                line_number: 20,
                content: "    // another function pattern".to_string(),
                score: Some(0.7),
                match_type: Some(MatchType::Fuzzy),
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "src/fuzzy.rs".to_string(),
                line_number: 30,
                content: "    // yet another function".to_string(),
                score: Some(0.75),
                match_type: Some(MatchType::Fuzzy),
                context_before: None,
                context_after: None,
            },
            // Single match in different file
            SearchResult {
                file_path: "src/single.rs".to_string(),
                line_number: 5,
                content: "    // function reference".to_string(),
                score: Some(0.6),
                match_type: Some(MatchType::Fuzzy),
                context_before: None,
                context_after: None,
            },
        ];

        let grouped = ResultGrouper::group_by_relevance(results);

        // Should have at least 2 groups
        assert!(grouped.len() >= 2);

        // First group should be exact matches
        assert!(grouped[0].title.contains("Exact") || grouped[0].title.contains("exact"));
        assert_eq!(grouped[0].results.len(), 1);

        // Should have a group for multiple matches in fuzzy.rs
        let fuzzy_group = grouped.iter().find(|g| g.title.contains("fuzzy.rs"));
        assert!(fuzzy_group.is_some());
        assert_eq!(fuzzy_group.unwrap().results.len(), 3);
    }

    #[test]
    fn test_simple_display_grouping() {
        let results = vec![
            SearchResult {
                file_path: "src/main.rs".to_string(),
                line_number: 15,
                content: "    // TODO: main".to_string(),
                score: Some(1.0),
                match_type: Some(MatchType::Exact),
                context_before: None,
                context_after: None,
            },
            SearchResult {
                file_path: "src/lib.rs".to_string(),
                line_number: 8,
                content: "    // TODO: lib".to_string(),
                score: Some(0.9),
                match_type: Some(MatchType::Exact),
                context_before: None,
                context_after: None,
            },
        ];

        let grouped = ResultGrouper::group_for_simple_display(results);

        // Should group by file with clean titles
        assert_eq!(grouped.len(), 2);

        // Check that filenames are extracted properly
        let titles: Vec<_> = grouped.iter().map(|g| g.title.as_str()).collect();
        assert!(titles.contains(&"main.rs"));
        assert!(titles.contains(&"lib.rs"));
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(ResultGrouper::extract_filename("src/main.rs"), "main.rs");
        assert_eq!(
            ResultGrouper::extract_filename("/full/path/to/file.txt"),
            "file.txt"
        );
        assert_eq!(ResultGrouper::extract_filename("file.rs"), "file.rs");
        assert_eq!(
            ResultGrouper::extract_filename("path/with/no/extension"),
            "extension"
        );
    }
}
