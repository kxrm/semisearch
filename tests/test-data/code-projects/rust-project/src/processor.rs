use crate::{config::Config, error::Error, models::SearchResult, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Processes files and directories to find matching content
pub struct FileProcessor {
    config: Config,
}

impl FileProcessor {
    /// Create a new file processor with the given configuration
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// Process a directory recursively to find matching content
    pub async fn process_directory(&self, path: impl AsRef<Path>, query: &str) -> Result<Vec<SearchResult>> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(Error::PathNotFound(path.to_path_buf()));
        }
        
        if !path.is_dir() {
            return self.process_file(path, query).await;
        }
        
        let mut results = Vec::new();
        let mut entries = fs::read_dir(path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                // TODO: Handle symlinks and cycles
                let mut dir_results = self.process_directory(&path, query).await?;
                results.append(&mut dir_results);
            } else {
                if self.should_process_file(&path) {
                    if let Ok(mut file_results) = self.process_file(&path, query).await {
                        results.append(&mut file_results);
                    }
                }
            }
            
            // Limit results if we've found enough
            if results.len() >= self.config.limit {
                results.truncate(self.config.limit);
                break;
            }
        }
        
        // Sort results by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(results)
    }
    
    /// Process a single file to find matching content
    async fn process_file(&self, path: impl AsRef<Path>, query: &str) -> Result<Vec<SearchResult>> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(Error::PathNotFound(path.to_path_buf()));
        }
        
        if !path.is_file() {
            return Err(Error::NotAFile(path.to_path_buf()));
        }
        
        // Read file content
        let content = fs::read_to_string(path).await?;
        let mut results = Vec::new();
        
        // Split content into lines and search for matches
        for (i, line) in content.lines().enumerate() {
            if line.contains(query) {
                let score = self.calculate_score(line, query);
                
                if score >= self.config.score_threshold {
                    results.push(SearchResult {
                        path: path.to_path_buf(),
                        line_number: i + 1,
                        content: line.to_string(),
                        score,
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    /// Calculate a relevance score for a match
    fn calculate_score(&self, line: &str, query: &str) -> f64 {
        // FIXME: Implement a better scoring algorithm
        let base_score = 0.5;
        
        // Bonus for exact match
        let exact_match_bonus = if line.contains(query) { 0.3 } else { 0.0 };
        
        // Bonus for match at start of line
        let start_match_bonus = if line.trim_start().starts_with(query) { 0.2 } else { 0.0 };
        
        // Penalty for very long lines
        let length_penalty = if line.len() > 200 { 0.1 } else { 0.0 };
        
        base_score + exact_match_bonus + start_match_bonus - length_penalty
    }
    
    /// Check if a file should be processed based on its extension
    fn should_process_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            
            // Skip excluded extensions
            if self.config.exclude_extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                return false;
            }
            
            // If include extensions is empty, include all non-excluded
            // Otherwise, only include specified extensions
            if self.config.include_extensions.is_empty() {
                return true;
            } else {
                return self.config.include_extensions.iter().any(|e| e.to_lowercase() == ext_str);
            }
        }
        
        // Files without extensions are included unless include_extensions is specified
        self.config.include_extensions.is_empty()
    }
} 