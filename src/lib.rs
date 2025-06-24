use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: usize,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub min_score: f32,
    pub max_results: usize,
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Plain,
    Json,
}

pub fn search_files(query: &str, path: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    // Use ignore crate to respect .gitignore files
    let walker = WalkBuilder::new(path)
        .follow_links(false)
        .git_ignore(true)
        .build();

    for entry in walker {
        let entry = entry?;
        if entry.file_type().map_or(false, |ft| ft.is_file()) {
            if let Some(file_results) = search_in_file(entry.path(), &query_lower)? {
                results.extend(file_results);
                if results.len() >= options.max_results {
                    results.truncate(options.max_results);
                    break;
                }
            }
        }
    }

    Ok(results)
}

pub fn search_in_file(
    file_path: &std::path::Path,
    query: &str,
) -> Result<Option<Vec<SearchResult>>> {
    // Skip binary files and common non-text files
    if let Some(extension) = file_path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        if matches!(ext.as_str(), "exe" | "dll" | "so" | "dylib" | "bin" | "obj" | "o" | "a" | "lib") {
            return Ok(None);
        }
    }

    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => return Ok(None), // Skip files we can't read (binary, permissions, etc.)
    };

    let mut matches = Vec::new();
    for (line_number, line) in content.lines().enumerate() {
        let line_lower = line.to_lowercase();
        if line_lower.contains(query) {
            matches.push(SearchResult {
                file_path: file_path.to_string_lossy().to_string(),
                line_number: line_number + 1,
                content: line.trim().to_string(),
            });
        }
    }

    if matches.is_empty() {
        Ok(None)
    } else {
        Ok(Some(matches))
    }
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
        fs::write(&file_path, "TODO: Fix this bug\nnothing here\nTODO: Another task").unwrap();

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
        };
        let results = search_files("todo", temp_dir.path().to_str().unwrap(), &options).unwrap();
        
        assert_eq!(results.len(), 3); // Should find 3 matches total
        
        // Verify we found matches in the right files
        let file_paths: Vec<&str> = results.iter().map(|r| {
            Path::new(&r.file_path).file_name().unwrap().to_str().unwrap()
        }).collect();
        
        assert!(file_paths.contains(&"file1.txt"));
        assert!(file_paths.contains(&"file3.txt"));
        assert!(!file_paths.iter().any(|&f| f == "file2.txt"));
    }

    #[test]
    fn test_search_files_respects_limit() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a file with many matches
        let file1 = temp_dir.path().join("many_todos.txt");
        let content = (0..20).map(|i| format!("TODO item {}", i)).collect::<Vec<_>>().join("\n");
        fs::write(&file1, content).unwrap();

        let options = SearchOptions {
            min_score: 0.0,
            max_results: 5,
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
        };
        let results = search_files("todo", temp_dir.path().to_str().unwrap(), &options).unwrap();
        assert!(results.is_empty());
    }
} 