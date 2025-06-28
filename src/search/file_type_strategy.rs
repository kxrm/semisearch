use crate::search::{
    fuzzy::FuzzySearch, keyword::KeywordSearch, regex_search::RegexSearch, semantic::SemanticSearch,
};
use crate::SearchResult;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Represents different file types for specialized search strategies
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum FileType {
    /// Code files (e.g., .rs, .js, .py)
    Code,
    /// Documentation files (e.g., .md, .txt)
    Documentation,
    /// Configuration files (e.g., .json, .yaml, .toml)
    Configuration,
    /// Data files (e.g., .csv, .tsv)
    Data,
}

/// A trait for file type specific search strategies
#[async_trait]
pub trait SearchStrategy: Send + Sync {
    /// Perform a search using this strategy
    async fn search(&self, query: &str, files: &[PathBuf]) -> Result<Vec<SearchResult>>;
}

/// Code search strategy optimized for code files
pub struct CodeSearchStrategy {
    regex_search: RegexSearch,
    keyword_search: KeywordSearch,
}

impl CodeSearchStrategy {
    pub fn new() -> Self {
        Self {
            regex_search: RegexSearch::new(),
            keyword_search: KeywordSearch::new(),
        }
    }
}

impl Default for CodeSearchStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SearchStrategy for CodeSearchStrategy {
    async fn search(&self, query: &str, files: &[PathBuf]) -> Result<Vec<SearchResult>> {
        // For code, prefer regex for patterns like TODO, function names, etc.
        if query.to_uppercase() == "TODO"
            || query.to_uppercase() == "FIXME"
            || query.contains("fn ")
            || query.contains("function")
            || query.contains("class")
        {
            let regex_query = if query.to_uppercase() == "TODO" {
                "TODO.*".to_string()
            } else if query.to_uppercase() == "FIXME" {
                "FIXME.*".to_string()
            } else {
                query.to_string()
            };

            let mut results = Vec::new();
            for file in files {
                if let Ok(file_results) = self
                    .regex_search
                    .search(&regex_query, file.to_str().unwrap_or("."))
                    .await
                {
                    results.extend(file_results);
                }
            }
            Ok(results)
        } else {
            // For other code searches, use keyword search
            let mut results = Vec::new();
            for file in files {
                if let Ok(file_results) = self
                    .keyword_search
                    .search(query, file.to_str().unwrap_or("."))
                    .await
                {
                    results.extend(file_results);
                }
            }
            Ok(results)
        }
    }
}

/// Documentation search strategy optimized for documentation files
pub struct DocumentationSearchStrategy {
    semantic_search: Option<SemanticSearch>,
    fuzzy_search: FuzzySearch,
}

impl DocumentationSearchStrategy {
    pub fn new() -> Self {
        Self {
            semantic_search: None,
            fuzzy_search: FuzzySearch::new(),
        }
    }

    pub fn with_semantic_search(semantic_search: Arc<crate::core::LocalEmbedder>) -> Self {
        Self {
            semantic_search: Some(SemanticSearch::new(semantic_search)),
            fuzzy_search: FuzzySearch::new(),
        }
    }
}

impl Default for DocumentationSearchStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SearchStrategy for DocumentationSearchStrategy {
    async fn search(&self, query: &str, files: &[PathBuf]) -> Result<Vec<SearchResult>> {
        // For documentation, prefer semantic search for concepts if available
        if query.split_whitespace().count() > 2 && self.semantic_search.is_some() {
            // In a real implementation, we'd use semantic search here
            // For now, fall back to fuzzy search
            let mut results = Vec::new();
            for file in files {
                if let Ok(file_results) = self
                    .fuzzy_search
                    .search(query, file.to_str().unwrap_or("."))
                    .await
                {
                    results.extend(file_results);
                }
            }
            Ok(results)
        } else {
            // For simpler queries, use fuzzy search
            let mut results = Vec::new();
            for file in files {
                if let Ok(file_results) = self
                    .fuzzy_search
                    .search(query, file.to_str().unwrap_or("."))
                    .await
                {
                    results.extend(file_results);
                }
            }
            Ok(results)
        }
    }
}

/// Configuration search strategy optimized for configuration files
pub struct ConfigurationSearchStrategy {
    keyword_search: KeywordSearch,
}

impl ConfigurationSearchStrategy {
    pub fn new() -> Self {
        Self {
            keyword_search: KeywordSearch::new(),
        }
    }
}

impl Default for ConfigurationSearchStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SearchStrategy for ConfigurationSearchStrategy {
    async fn search(&self, query: &str, files: &[PathBuf]) -> Result<Vec<SearchResult>> {
        // For configuration files, use exact keyword matching
        let mut results = Vec::new();
        for file in files {
            if let Ok(file_results) = self
                .keyword_search
                .search(query, file.to_str().unwrap_or("."))
                .await
            {
                results.extend(file_results);
            }
        }
        Ok(results)
    }
}

/// Data search strategy optimized for data files
pub struct DataSearchStrategy {
    keyword_search: KeywordSearch,
}

impl DataSearchStrategy {
    pub fn new() -> Self {
        Self {
            keyword_search: KeywordSearch::new(),
        }
    }
}

impl Default for DataSearchStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SearchStrategy for DataSearchStrategy {
    async fn search(&self, query: &str, files: &[PathBuf]) -> Result<Vec<SearchResult>> {
        // For data files, use keyword search
        let mut results = Vec::new();
        for file in files {
            if let Ok(file_results) = self
                .keyword_search
                .search(query, file.to_str().unwrap_or("."))
                .await
            {
                results.extend(file_results);
            }
        }
        Ok(results)
    }
}

/// Main strategy that selects appropriate search methods based on file types
pub struct FileTypeStrategy {
    pub strategies: HashMap<FileType, Box<dyn SearchStrategy>>,
}

impl FileTypeStrategy {
    /// Create a new FileTypeStrategy with default strategies
    pub fn new() -> Self {
        let mut strategies: HashMap<FileType, Box<dyn SearchStrategy>> = HashMap::new();

        strategies.insert(FileType::Code, Box::new(CodeSearchStrategy::new()));

        strategies.insert(
            FileType::Documentation,
            Box::new(DocumentationSearchStrategy::new()),
        );

        strategies.insert(
            FileType::Configuration,
            Box::new(ConfigurationSearchStrategy::new()),
        );

        strategies.insert(FileType::Data, Box::new(DataSearchStrategy::new()));

        Self { strategies }
    }

    /// Create a new FileTypeStrategy with semantic search capabilities
    pub fn with_semantic_search(semantic_search: Arc<crate::core::LocalEmbedder>) -> Self {
        let mut strategies: HashMap<FileType, Box<dyn SearchStrategy>> = HashMap::new();

        strategies.insert(FileType::Code, Box::new(CodeSearchStrategy::new()));

        strategies.insert(
            FileType::Documentation,
            Box::new(DocumentationSearchStrategy::with_semantic_search(
                semantic_search,
            )),
        );

        strategies.insert(
            FileType::Configuration,
            Box::new(ConfigurationSearchStrategy::new()),
        );

        strategies.insert(FileType::Data, Box::new(DataSearchStrategy::new()));

        Self { strategies }
    }

    /// Detect the file type based on file extension
    pub fn detect_file_type(&self, path: &Path) -> FileType {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext = ext_str.to_lowercase();

                // Code files
                if [
                    "rs", "py", "js", "ts", "c", "cpp", "h", "hpp", "java", "go", "rb", "php",
                    "cs", "swift",
                ]
                .contains(&ext.as_str())
                {
                    return FileType::Code;
                }

                // Documentation files
                if ["md", "txt", "rst", "adoc", "org", "wiki", "html", "htm"]
                    .contains(&ext.as_str())
                {
                    return FileType::Documentation;
                }

                // Configuration files
                if [
                    "json", "yaml", "yml", "toml", "ini", "conf", "config", "xml", "env",
                ]
                .contains(&ext.as_str())
                {
                    return FileType::Configuration;
                }

                // Data files
                if ["csv", "tsv", "xlsx", "xls", "ods", "db", "sqlite", "sql"]
                    .contains(&ext.as_str())
                {
                    return FileType::Data;
                }
            }
        }

        // Special filename checks
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                let name = name_str.to_lowercase();

                // Common documentation files
                if ["readme", "license", "contributing", "changelog"]
                    .iter()
                    .any(|prefix| name.starts_with(prefix))
                {
                    return FileType::Documentation;
                }

                // Common configuration files
                if [".env", "dockerfile", "makefile", "config"]
                    .iter()
                    .any(|prefix| name.starts_with(prefix))
                {
                    return FileType::Configuration;
                }
            }
        }

        // Default to code for unknown types
        FileType::Code
    }

    /// Group files by their detected type
    pub fn group_files_by_type(&self, files: &[PathBuf]) -> HashMap<FileType, Vec<PathBuf>> {
        let mut grouped = HashMap::new();

        for file in files {
            let file_type = self.detect_file_type(file);
            grouped
                .entry(file_type)
                .or_insert_with(Vec::new)
                .push(file.clone());
        }

        grouped
    }

    /// Perform a search using type-specific strategies
    pub async fn search(&self, query: &str, files: &[PathBuf]) -> Result<Vec<SearchResult>> {
        let mut all_results = Vec::new();

        // Group files by type
        let files_by_type = self.group_files_by_type(files);

        // Search each group with appropriate strategy
        for (file_type, type_files) in files_by_type {
            if let Some(strategy) = self.strategies.get(&file_type) {
                let results = strategy.search(query, &type_files).await?;
                all_results.extend(results);
            }
        }

        // Sort by relevance
        all_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(all_results)
    }
}

impl Default for FileTypeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_detection() {
        let strategy = FileTypeStrategy::new();

        // Test code file detection
        assert_eq!(
            strategy.detect_file_type(&PathBuf::from("test.rs")),
            FileType::Code
        );
        assert_eq!(
            strategy.detect_file_type(&PathBuf::from("main.py")),
            FileType::Code
        );
        assert_eq!(
            strategy.detect_file_type(&PathBuf::from("app.js")),
            FileType::Code
        );

        // Test documentation file detection
        assert_eq!(
            strategy.detect_file_type(&PathBuf::from("README.md")),
            FileType::Documentation
        );
        assert_eq!(
            strategy.detect_file_type(&PathBuf::from("guide.txt")),
            FileType::Documentation
        );

        // Test configuration file detection
        assert_eq!(
            strategy.detect_file_type(&PathBuf::from("config.json")),
            FileType::Configuration
        );
        assert_eq!(
            strategy.detect_file_type(&PathBuf::from("settings.yaml")),
            FileType::Configuration
        );

        // Test data file detection
        assert_eq!(
            strategy.detect_file_type(&PathBuf::from("data.csv")),
            FileType::Data
        );
        assert_eq!(
            strategy.detect_file_type(&PathBuf::from("results.tsv")),
            FileType::Data
        );
    }

    #[test]
    fn test_group_files_by_type() {
        let strategy = FileTypeStrategy::new();

        let files = vec![
            PathBuf::from("main.rs"),
            PathBuf::from("README.md"),
            PathBuf::from("config.json"),
            PathBuf::from("data.csv"),
        ];

        let grouped = strategy.group_files_by_type(&files);

        assert_eq!(grouped.len(), 4); // Should have 4 different file types
        assert_eq!(grouped.get(&FileType::Code).unwrap().len(), 1);
        assert_eq!(grouped.get(&FileType::Documentation).unwrap().len(), 1);
        assert_eq!(grouped.get(&FileType::Configuration).unwrap().len(), 1);
        assert_eq!(grouped.get(&FileType::Data).unwrap().len(), 1);
    }
}
