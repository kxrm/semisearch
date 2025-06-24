use clap::{Arg, Command};
use std::process;
use walkdir::WalkDir;

fn main() {
    let matches = Command::new("semisearch")
        .about("Semantic search across local files")
        .version("0.1.0")
        .arg(
            Arg::new("query")
                .help("Search query")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("path")
                .help("Target directory (default: current directory)")
                .short('p')
                .long("path")
                .default_value("."),
        )
        .arg(
            Arg::new("limit")
                .help("Maximum number of results")
                .short('l')
                .long("limit")
                .default_value("10"),
        )
        .get_matches();

    let query = matches.get_one::<String>("query").unwrap();
    let path = matches.get_one::<String>("path").unwrap();
    let limit: usize = matches
        .get_one::<String>("limit")
        .unwrap()
        .parse()
        .unwrap_or(10);

    match search_files(query, path, limit) {
        Ok(results) => {
            if results.is_empty() {
                eprintln!("No matches found for '{}'", query);
                process::exit(1);
            }
            for result in results {
                println!("{}:{}:{}", result.file_path, result.line_number, result.content);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: usize,
    pub content: String,
}

pub fn search_files(query: &str, path: &str, limit: usize) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(file_results) = search_in_file(entry.path(), &query_lower)? {
                results.extend(file_results);
                if results.len() >= limit {
                    results.truncate(limit);
                    break;
                }
            }
        }
    }

    Ok(results)
}

fn search_in_file(
    file_path: &std::path::Path,
    query: &str,
) -> Result<Option<Vec<SearchResult>>, Box<dyn std::error::Error>> {
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

        let results = search_files("todo", temp_dir.path().to_str().unwrap(), 10).unwrap();
        
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

        let results = search_files("todo", temp_dir.path().to_str().unwrap(), 5).unwrap();
        
        assert_eq!(results.len(), 5); // Should respect the limit
    }

    #[test]
    fn test_search_files_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let results = search_files("todo", temp_dir.path().to_str().unwrap(), 10).unwrap();
        assert!(results.is_empty());
    }
} 