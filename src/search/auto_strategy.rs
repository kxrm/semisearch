use crate::context::{ContextAwareConfig, ProjectDetector, ProjectType};
use crate::core::{EmbeddingConfig, LocalEmbedder};
use crate::query::analyzer::{QueryAnalyzer, QueryType};
use crate::search::{
    file_type_strategy::FileTypeStrategy, fuzzy::FuzzySearch, keyword::KeywordSearch,
    regex_search::RegexSearch,
};
use crate::SearchOptions;
use crate::SearchResult;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Search mode enum for internal use
enum SearchMode {
    Keyword,
    Fuzzy,
    Regex,
}

/// AutoStrategy automatically selects the best search strategy based on query analysis
/// and project context. This implements the "Smart Query Analysis" from the UX plan.
pub struct AutoStrategy {
    keyword_search: KeywordSearch,
    fuzzy_search: FuzzySearch,
    regex_search: RegexSearch,
    file_type_strategy: FileTypeStrategy,
    semantic_search: Option<crate::search::semantic::SemanticSearch>,
}

impl AutoStrategy {
    /// Create a new AutoStrategy with default search engines
    pub fn new() -> Self {
        Self {
            keyword_search: KeywordSearch::new(),
            fuzzy_search: FuzzySearch::new(),
            regex_search: RegexSearch::new(),
            file_type_strategy: FileTypeStrategy::new(),
            semantic_search: None,
        }
    }

    /// Create an AutoStrategy with semantic search capabilities
    pub async fn with_semantic_search() -> Result<Self> {
        let config = EmbeddingConfig::default();
        let embedder = LocalEmbedder::new(config).await?;
        let embedder_arc = Arc::new(embedder);

        Ok(Self {
            keyword_search: KeywordSearch::new(),
            fuzzy_search: FuzzySearch::new(),
            regex_search: RegexSearch::new(),
            file_type_strategy: FileTypeStrategy::with_semantic_search(embedder_arc.clone()),
            semantic_search: Some(crate::search::semantic::SemanticSearch::new(embedder_arc)),
        })
    }

    /// Performs a search using the automatically selected strategy
    /// Integrates context detection silently (UX Remediation Plan Task 2.1)
    /// Now accepts SearchOptions for advanced filtering (include/exclude patterns)
    pub async fn search(
        &self,
        query: &str,
        path: &str,
        options: Option<&SearchOptions>,
    ) -> Result<Vec<SearchResult>> {
        let query_type = QueryAnalyzer::analyze(query);

        // Silent context detection - no output to user
        let path_buf = Path::new(path).to_path_buf();
        let project_type = ProjectDetector::detect(&path_buf);
        // Context config available for future use (file patterns, ignore patterns, etc.)
        let _context_config = ContextAwareConfig::from_project_type(project_type.clone());

        // Get all files in the path, applying include/exclude filtering if provided
        let files = self.get_files_in_path(path, options)?;

        // For file extension queries, extract file extension and filter files
        if query_type == QueryType::FileExtension {
            return self.search_with_file_extension_filter(query, &files).await;
        }

        // Try primary search strategy based on project type and query type
        let primary_results = match (query_type.clone(), project_type, &self.semantic_search) {
            // Code patterns in code projects use regex
            (QueryType::CodePattern, ProjectType::RustProject, _)
            | (QueryType::CodePattern, ProjectType::JavaScriptProject, _)
            | (QueryType::CodePattern, ProjectType::PythonProject, _) => {
                let regex_query = self.code_pattern_to_regex(query);
                self.search_in_files(&regex_query, &files, SearchMode::Regex)
                    .await?
            }

            // Conceptual queries use semantic search if available, otherwise fuzzy
            (QueryType::Conceptual, _, Some(_semantic)) => {
                // For now, fallback to fuzzy since semantic search doesn't have path-based search
                // In a real implementation, this would use the semantic search with file chunks
                self.search_in_files(query, &files, SearchMode::Fuzzy)
                    .await?
            }

            // Exact phrases use keyword search
            (QueryType::ExactPhrase, _, _) => {
                self.search_in_files(query, &files, SearchMode::Keyword)
                    .await?
            }

            // Regex-like patterns use regex search
            (QueryType::RegexLike, _, _) => {
                self.search_in_files(query, &files, SearchMode::Regex)
                    .await?
            }

            // Documentation projects use file type strategy
            (_, ProjectType::Documentation, _) | (_, ProjectType::Mixed, _) => {
                self.file_type_strategy.search(query, &files).await?
            }

            // Default to keyword search first
            _ => {
                self.search_in_files(query, &files, SearchMode::Keyword)
                    .await?
            }
        };

        // If no results found, automatically try fuzzy search for typo tolerance
        // This implements the automatic typo correction from smart query analysis
        if primary_results.is_empty() && !matches!(query_type, QueryType::RegexLike) {
            self.search_in_files(query, &files, SearchMode::Fuzzy).await
        } else {
            Ok(primary_results)
        }
    }

    /// Get all files in a path recursively, applying include/exclude filtering
    fn get_files_in_path(
        &self,
        path: &str,
        options: Option<&SearchOptions>,
    ) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let path = Path::new(path);

        // Check if path exists
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "No such file or directory: '{}' not found",
                path.display()
            ));
        }

        if path.is_file() {
            files.push(path.to_path_buf());
            return Ok(files);
        }

        // Simple recursive directory traversal with default exclusions
        fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
            if dir.is_dir() {
                // Skip common build/cache directories by default
                if let Some(dir_name) = dir.file_name() {
                    if let Some(name_str) = dir_name.to_str() {
                        if [
                            "target",
                            "node_modules",
                            ".git",
                            "build",
                            "dist",
                            "__pycache__",
                            ".cache",
                            ".semisearch",
                        ]
                        .contains(&name_str)
                        {
                            return Ok(()); // Skip this directory
                        }
                    }
                }

                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dirs(&path, files)?;
                    } else {
                        files.push(path);
                    }
                }
            }
            Ok(())
        }

        visit_dirs(path, &mut files)?;

        // Apply include/exclude filtering if options are provided
        if let Some(options) = options {
            files = self.apply_file_filtering(files, options);
        }

        Ok(files)
    }

    /// Apply include/exclude pattern filtering to files
    fn apply_file_filtering(&self, files: Vec<PathBuf>, options: &SearchOptions) -> Vec<PathBuf> {
        files
            .into_iter()
            .filter(|file| {
                let file_path_str = file.to_string_lossy();
                let file_name = file
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");

                // Apply include patterns (if any)
                if !options.include_patterns.is_empty() {
                    let matches_include = options.include_patterns.iter().any(|pattern| {
                        self.glob_match(pattern, &file_path_str)
                            || self.glob_match(pattern, file_name)
                    });
                    if !matches_include {
                        return false; // Skip file - doesn't match any include pattern
                    }
                }

                // Apply exclude patterns (if any)
                if !options.exclude_patterns.is_empty() {
                    let matches_exclude = options.exclude_patterns.iter().any(|pattern| {
                        self.glob_match(pattern, &file_path_str)
                            || self.glob_match(pattern, file_name)
                    });
                    if matches_exclude {
                        return false; // Skip file - matches an exclude pattern
                    }
                }

                true // Include file
            })
            .collect()
    }

    /// Simple glob pattern matching (supports * wildcard)
    fn glob_match(&self, pattern: &str, text: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        // Convert glob pattern to regex-like matching
        if pattern.contains('*') {
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            if pattern_parts.len() == 2 {
                let start = pattern_parts[0];
                let end = pattern_parts[1];

                if start.is_empty() && !end.is_empty() {
                    // Pattern like "*.rs"
                    text.ends_with(end)
                } else if end.is_empty() && !start.is_empty() {
                    // Pattern like "test*"
                    text.starts_with(start)
                } else if !start.is_empty() && !end.is_empty() {
                    // Pattern like "*test*"
                    text.contains(start) && text.contains(end)
                } else {
                    // Pattern is just "*"
                    true
                }
            } else {
                // More complex patterns - simple contains check
                let pattern_without_stars = pattern.replace('*', "");
                text.contains(&pattern_without_stars)
            }
        } else {
            // No wildcards - exact match
            text == pattern
        }
    }

    /// Search in specific files using the specified mode
    async fn search_in_files(
        &self,
        query: &str,
        files: &[PathBuf],
        mode: SearchMode,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        for file in files {
            let file_path = file.to_str().unwrap_or(".");
            let file_results = match mode {
                SearchMode::Keyword => self.keyword_search.search(query, file_path).await?,
                SearchMode::Fuzzy => self.fuzzy_search.search(query, file_path).await?,
                SearchMode::Regex => self.regex_search.search(query, file_path).await?,
            };
            results.extend(file_results);
        }

        Ok(results)
    }

    /// Converts code patterns to regex patterns
    pub fn code_pattern_to_regex(&self, pattern: &str) -> String {
        match pattern.to_uppercase().as_str() {
            "TODO" => r"TODO.*".to_string(),
            "FIXME" => r"FIXME.*".to_string(),
            "HACK" => r"HACK.*".to_string(),
            "NOTE" => r"NOTE.*".to_string(),
            "WARNING" => r"WARNING.*".to_string(),
            "ERROR" => r"ERROR.*".to_string(),
            "BUG" => r"BUG.*".to_string(),
            "FUNCTION" | "FN" => r"fn\s+\w+".to_string(),
            "CLASS" => r"class\s+\w+".to_string(),
            "STRUCT" => r"struct\s+\w+".to_string(),
            "ENUM" => r"enum\s+\w+".to_string(),
            "TRAIT" => r"trait\s+\w+".to_string(),
            "IMPL" => r"impl\s+\w+".to_string(),
            "IMPORT" => r"import\s+.*".to_string(),
            "EXPORT" => r"export\s+.*".to_string(),
            "ASYNC" => r"async\s+fn\s+\w+".to_string(),
            "AWAIT" => r"await\s+.*".to_string(),
            _ => format!(r"{}.*", regex::escape(pattern)),
        }
    }

    /// Search with file extension filtering
    async fn search_with_file_extension_filter(
        &self,
        query: &str,
        files: &[PathBuf],
    ) -> Result<Vec<SearchResult>> {
        // Extract file extensions from query
        let extensions = self.extract_file_extensions(query);

        // Filter files by extensions if any were found
        let filtered_files: Vec<PathBuf> = if !extensions.is_empty() {
            files
                .iter()
                .filter(|file| {
                    if let Some(ext) = file.extension() {
                        let ext_str = format!(".{}", ext.to_string_lossy().to_lowercase());
                        extensions.contains(&ext_str)
                    } else {
                        false
                    }
                })
                .cloned()
                .collect()
        } else {
            files.to_vec()
        };

        // Extract the actual search term (remove file extension references)
        let clean_query = self.clean_query_from_extensions(query);

        // Search in filtered files using the appropriate strategy
        let mut results = self
            .search_in_files(&clean_query, &filtered_files, SearchMode::Keyword)
            .await?;

        // If no results with filtered files, fall back to fuzzy search in all files
        if results.is_empty() && !filtered_files.is_empty() {
            results = self
                .search_in_files(&clean_query, &filtered_files, SearchMode::Fuzzy)
                .await?;
        }

        Ok(results)
    }

    /// Extract file extensions from query
    fn extract_file_extensions(&self, query: &str) -> Vec<String> {
        let file_extensions = [
            ".rs", ".py", ".js", ".ts", ".md", ".txt", ".json", ".toml", ".yaml", ".yml", ".xml",
            ".html", ".css", ".scss", ".sass", ".less", ".sql", ".sh", ".bash", ".zsh", ".fish",
            ".ps1", ".bat", ".cmd", ".exe", ".dll", ".so", ".dylib",
        ];

        file_extensions
            .iter()
            .filter(|ext| query.contains(*ext))
            .map(|ext| ext.to_string())
            .collect()
    }

    /// Clean query by removing file extension references
    fn clean_query_from_extensions(&self, query: &str) -> String {
        let mut clean = query.to_string();

        // Remove common file extension patterns
        let patterns_to_remove = [
            ".rs files",
            ".py files",
            ".js files",
            ".ts files",
            ".md files",
            ".rs",
            ".py",
            ".js",
            ".ts",
            ".md",
            ".txt",
            ".json",
            ".toml",
            "files",
            "file",
        ];

        for &pattern in patterns_to_remove.iter() {
            clean = clean.replace(pattern, "");
        }

        // Clean up extra whitespace
        clean
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }
}

impl Default for AutoStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_context_detection() {
        // Test with current directory (should be Rust project since we have Cargo.toml)
        let path = Path::new(".");
        let project_type = ProjectDetector::detect(path);
        assert!(matches!(project_type, ProjectType::RustProject));
    }

    #[test]
    fn test_code_pattern_to_regex() {
        let auto_strategy = AutoStrategy::new();

        assert_eq!(auto_strategy.code_pattern_to_regex("TODO"), r"TODO.*");
        assert_eq!(auto_strategy.code_pattern_to_regex("FIXME"), r"FIXME.*");
        assert_eq!(auto_strategy.code_pattern_to_regex("function"), r"fn\s+\w+");
        assert_eq!(auto_strategy.code_pattern_to_regex("class"), r"class\s+\w+");
    }
}
