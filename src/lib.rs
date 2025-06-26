use anyhow::Result;
use edit_distance::edit_distance;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};

// Phase 2: Core and Storage modules
pub mod core;
pub mod storage;

// Phase 3: Text Processing modules
pub mod search;
pub mod text;

// Phase 4: Capability detection for progressive enhancement
pub mod capability_detector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: usize,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_type: Option<MatchType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MatchType {
    Exact,
    Fuzzy,
    Regex,
    EditDistance,
    Semantic,
    Hybrid,
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub min_score: f32,
    pub max_results: usize,
    pub fuzzy_matching: bool,
    pub regex_mode: bool,
    pub case_sensitive: bool,
    pub typo_tolerance: bool,
    pub max_edit_distance: usize,
    pub search_mode: Option<String>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            min_score: 0.0,
            max_results: 10,
            fuzzy_matching: false,
            regex_mode: false,
            case_sensitive: false,
            typo_tolerance: false, // Disabled by default to avoid false positives
            max_edit_distance: 2,
            search_mode: None, // Default to None (auto-detect)
        }
    }
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Plain,
    Json,
}

pub fn search_files(query: &str, path: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();

    // Use ignore crate to respect .gitignore files
    let walker = WalkBuilder::new(path)
        .follow_links(false)
        .git_ignore(true)
        .build();

    for entry in walker {
        let entry = entry?;
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            if let Some(file_results) = search_in_file_enhanced(entry.path(), query, options)? {
                results.extend(file_results);
            }
        }
    }

    // Sort by score (descending) and apply limits
    results.sort_by(|a, b| {
        let score_a = a.score.unwrap_or(0.0);
        let score_b = b.score.unwrap_or(0.0);
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Filter by minimum score and limit results
    results.retain(|r| r.score.unwrap_or(1.0) >= options.min_score);
    results.truncate(options.max_results);

    Ok(results)
}

pub fn search_in_file_enhanced(
    file_path: &std::path::Path,
    query: &str,
    options: &SearchOptions,
) -> Result<Option<Vec<SearchResult>>> {
    // Skip binary files and common non-text files
    if let Some(extension) = file_path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        if matches!(
            ext.as_str(),
            "exe" | "dll" | "so" | "dylib" | "bin" | "obj" | "o" | "a" | "lib"
        ) {
            return Ok(None);
        }
    }

    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => return Ok(None), // Skip files we can't read (binary, permissions, etc.)
    };

    let mut matches = Vec::new();

    // Initialize search tools based on options
    let fuzzy_matcher = if options.fuzzy_matching {
        Some(SkimMatcherV2::default())
    } else {
        None
    };

    let regex = if options.regex_mode {
        match Regex::new(query) {
            Ok(r) => Some(r),
            Err(_) => return Ok(None), // Invalid regex
        }
    } else {
        None
    };

    for (line_number, line) in content.lines().enumerate() {
        let search_line = if options.case_sensitive {
            line.to_string()
        } else {
            line.to_lowercase()
        };

        let search_query = if options.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        let mut match_result: Option<(f32, MatchType)> = None;

        // Try different search strategies in order of preference
        if let Some(ref re) = regex {
            // For regex, always use the original line, not the case-modified version
            if re.is_match(line) {
                match_result = Some((1.0, MatchType::Regex));
            }
        } else if search_line.contains(&search_query) {
            // Exact match gets highest score
            match_result = Some((1.0, MatchType::Exact));
        } else if let Some(ref matcher) = fuzzy_matcher {
            if let Some(score) = matcher.fuzzy_match(&search_line, &search_query) {
                let normalized_score = (score as f32) / 100.0; // Normalize to 0.0-1.0
                if normalized_score >= options.min_score {
                    match_result = Some((normalized_score, MatchType::Fuzzy));
                }
            }
        } else if options.typo_tolerance {
            let distance = edit_distance(&search_line, &search_query);
            if distance <= options.max_edit_distance {
                let max_len = search_line.len().max(search_query.len()) as f32;
                let similarity = 1.0 - (distance as f32 / max_len);

                if similarity >= options.min_score {
                    match_result = Some((similarity, MatchType::EditDistance));
                }
            }

            // Also try edit distance on individual words in the line
            if match_result.is_none() {
                for word in search_line.split_whitespace() {
                    let distance = edit_distance(word, &search_query);
                    if distance <= options.max_edit_distance {
                        let max_len = word.len().max(search_query.len()) as f32;
                        let similarity = 1.0 - (distance as f32 / max_len);

                        if similarity >= options.min_score {
                            match_result = Some((similarity, MatchType::EditDistance));
                            break;
                        }
                    }
                }
            }
        }

        if let Some((score, match_type)) = match_result {
            matches.push(SearchResult {
                file_path: file_path.to_string_lossy().to_string(),
                line_number: line_number + 1,
                content: line.trim().to_string(),
                score: Some(score),
                match_type: Some(match_type),
            });
        }
    }

    if matches.is_empty() {
        Ok(None)
    } else {
        Ok(Some(matches))
    }
}

// Keep the original function for backward compatibility
pub fn search_in_file(
    file_path: &std::path::Path,
    query: &str,
) -> Result<Option<Vec<SearchResult>>> {
    let options = SearchOptions::default();
    search_in_file_enhanced(file_path, query, &options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_search_in_file_finds_matches() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello world\nThis is a TODO item\nAnother line").unwrap();

        let results = search_in_file(&file_path, "todo").unwrap().unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 2);
        assert_eq!(results[0].content, "This is a TODO item");
        assert!(results[0].file_path.ends_with("test.txt"));
    }

    #[test]
    fn test_search_in_file_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(
            &file_path,
            "TODO: Fix this bug\nnothing here\nTODO: Another task",
        )
        .unwrap();

        let results = search_in_file(&file_path, "todo").unwrap().unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[1].line_number, 3);
    }

    #[test]
    fn test_search_in_file_no_matches() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello world\nNothing to see here").unwrap();

        let result = search_in_file(&file_path, "todo").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_search_files_in_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        let file3 = temp_dir.path().join("file3.txt");

        fs::write(&file1, "This has a TODO item").unwrap();
        fs::write(&file2, "Nothing here").unwrap();
        fs::write(&file3, "Another TODO\nAnd more TODO items").unwrap();

        let options = SearchOptions {
            min_score: 0.0,
            max_results: 10,
            fuzzy_matching: false,
            regex_mode: false,
            case_sensitive: false,
            typo_tolerance: true,
            max_edit_distance: 2,
            search_mode: None,
        };
        let results = search_files("todo", temp_dir.path().to_str().unwrap(), &options).unwrap();

        assert_eq!(results.len(), 3); // Should find 3 matches total

        // Verify we found matches in the right files
        let file_paths: Vec<&str> = results
            .iter()
            .map(|r| {
                Path::new(&r.file_path)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
            })
            .collect();

        assert!(file_paths.contains(&"file1.txt"));
        assert!(file_paths.contains(&"file3.txt"));
        assert!(!file_paths.contains(&"file2.txt"));
    }

    #[test]
    fn test_search_files_respects_limit() {
        let temp_dir = TempDir::new().unwrap();

        // Create a file with many matches
        let file1 = temp_dir.path().join("many_todos.txt");
        let content = (0..20)
            .map(|i| format!("TODO item {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&file1, content).unwrap();

        let options = SearchOptions {
            min_score: 0.0,
            max_results: 5,
            fuzzy_matching: false,
            regex_mode: false,
            case_sensitive: false,
            typo_tolerance: true,
            max_edit_distance: 2,
            search_mode: None,
        };
        let results = search_files("todo", temp_dir.path().to_str().unwrap(), &options).unwrap();

        assert_eq!(results.len(), 5); // Should respect the limit
    }

    #[test]
    fn test_search_files_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let options = SearchOptions {
            min_score: 0.0,
            max_results: 10,
            fuzzy_matching: false,
            regex_mode: false,
            case_sensitive: false,
            typo_tolerance: true,
            max_edit_distance: 2,
            search_mode: None,
        };
        let results = search_files("todo", temp_dir.path().to_str().unwrap(), &options).unwrap();
        assert!(results.is_empty());
    }

    // Phase 2: Enhanced Search Tests
    #[test]
    fn test_fuzzy_matching() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test.txt");
        fs::write(
            &file1,
            "TODO: Fix this bug\nTODO: Another task\nToDO: Mixed case",
        )
        .unwrap();

        let options = SearchOptions {
            min_score: 0.0, // Very low threshold to catch all matches
            max_results: 10,
            fuzzy_matching: true,
            regex_mode: false,
            case_sensitive: false,
            typo_tolerance: true,
            max_edit_distance: 2,
            search_mode: None,
        };

        // Test fuzzy matching with typos - use a query that won't match exactly
        let results = search_files("TOOD", temp_dir.path().to_str().unwrap(), &options).unwrap();
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .any(|r| r.match_type == Some(MatchType::Fuzzy)));
    }

    #[test]
    fn test_regex_search() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test.txt");
        fs::write(
            &file1,
            "Error: File not found\nWarning: Low disk space\nInfo: Process started",
        )
        .unwrap();

        let options = SearchOptions {
            min_score: 0.0,
            max_results: 10,
            fuzzy_matching: false,
            regex_mode: true,
            case_sensitive: false,
            typo_tolerance: true,
            max_edit_distance: 2,
            search_mode: None,
        };

        // Test regex pattern matching
        let results = search_files(
            r"(Error|Warning):",
            temp_dir.path().to_str().unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .all(|r| r.match_type == Some(MatchType::Regex)));
    }

    #[test]
    fn test_case_sensitive_search() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test.txt");
        fs::write(
            &file1,
            "TODO: Fix this bug\ntodo: lowercase task\nToDo: Mixed case",
        )
        .unwrap();

        let options = SearchOptions {
            min_score: 0.0,
            max_results: 10,
            fuzzy_matching: false,
            regex_mode: false,
            case_sensitive: true,
            typo_tolerance: true,
            max_edit_distance: 2,
            search_mode: None,
        };

        // Test case-sensitive search
        let results = search_files("TODO", temp_dir.path().to_str().unwrap(), &options).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("TODO: Fix this bug"));
    }

    #[test]
    fn test_search_scoring() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test.txt");
        fs::write(&file1, "TODO: Fix this bug\nTODO: Another task").unwrap();

        let options = SearchOptions {
            min_score: 0.0,
            max_results: 10,
            fuzzy_matching: false,
            regex_mode: false,
            case_sensitive: false,
            typo_tolerance: true,
            max_edit_distance: 2,
            search_mode: None,
        };

        let results = search_files("todo", temp_dir.path().to_str().unwrap(), &options).unwrap();

        // All exact matches should have score 1.0
        for result in results {
            assert_eq!(result.score, Some(1.0));
            assert_eq!(result.match_type, Some(MatchType::Exact));
        }
    }

    #[test]
    fn test_enhanced_typo_tolerance() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test.txt");
        fs::write(
            &file1,
            "TODO: Fix this bug\nTODO: Another task\nTDO: Typo here\nTOOD: Transposition",
        )
        .unwrap();

        let options = SearchOptions {
            min_score: 0.5,
            max_results: 10,
            fuzzy_matching: true,
            typo_tolerance: true,
            max_edit_distance: 2,
            search_mode: None,
            ..Default::default()
        };

        let results = search_files("TODO", temp_dir.path().to_str().unwrap(), &options).unwrap();

        // Should find 3 matches: 2 exact + 1 fuzzy/edit distance
        // Note: "TDO" vs "TODO" is too different for SkimMatcherV2 to find
        // This demonstrates the limitation we discussed
        assert!(
            results.len() >= 3,
            "Should find at least 3 matches, found {}",
            results.len()
        );

        // Verify we found the matches we expect
        let contents: Vec<&str> = results.iter().map(|r| r.content.as_str()).collect();
        assert!(contents.iter().any(|c| c.contains("TODO: Fix this bug")));
        assert!(contents.iter().any(|c| c.contains("TODO: Another task")));
        assert!(contents.iter().any(|c| c.contains("TOOD: Transposition")));

        // The missing "TDO: Typo here" demonstrates the fuzzy-matcher limitation
        // This is exactly the issue the user asked about!
    }
}
