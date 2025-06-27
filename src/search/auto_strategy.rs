use crate::core::{EmbeddingConfig, LocalEmbedder};
use crate::query::analyzer::{QueryAnalyzer, QueryType};
use crate::search::{fuzzy::FuzzySearch, keyword::KeywordSearch, regex_search::RegexSearch};
use crate::SearchResult;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

/// Represents the context of a project to help determine search strategy
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectContext {
    /// Code project (Rust, JavaScript, Python, etc.)
    Code,
    /// Documentation project (mostly markdown, text files)
    Documentation,
    /// Mixed project (both code and documentation)
    Mixed,
    /// Unknown project type
    Unknown,
}

impl ProjectContext {
    /// Detects the project context based on the directory structure
    pub fn detect(path: &str) -> Result<Self> {
        let path = Path::new(path);

        // Check for code project indicators
        let has_cargo_toml = path.join("Cargo.toml").exists();
        let has_package_json = path.join("package.json").exists();
        let has_requirements_txt = path.join("requirements.txt").exists();
        let has_pyproject_toml = path.join("pyproject.toml").exists();
        let has_go_mod = path.join("go.mod").exists();
        let has_makefile = path.join("Makefile").exists() || path.join("makefile").exists();

        // Check for documentation indicators
        let has_readme = path.join("README.md").exists() || path.join("README.txt").exists();
        let has_docs_dir = path.join("docs").exists();
        let is_docs_dir = path.file_name().map(|name| name == "docs").unwrap_or(false);
        let has_documentation = has_readme || has_docs_dir || is_docs_dir;

        // Determine project type
        let is_code_project = has_cargo_toml
            || has_package_json
            || has_requirements_txt
            || has_pyproject_toml
            || has_go_mod
            || has_makefile;

        match (is_code_project, has_documentation) {
            (true, true) => Ok(ProjectContext::Mixed),
            (true, false) => Ok(ProjectContext::Code),
            (false, true) => Ok(ProjectContext::Documentation),
            (false, false) => Ok(ProjectContext::Unknown),
        }
    }
}

/// Automatically selects the best search strategy based on query analysis and project context
pub struct AutoStrategy {
    keyword_search: KeywordSearch,
    fuzzy_search: FuzzySearch,
    regex_search: RegexSearch,
    semantic_search: Option<crate::search::semantic::SemanticSearch>,
}

impl AutoStrategy {
    /// Creates a new AutoStrategy instance
    pub fn new() -> Self {
        // For now, we'll create without semantic search to avoid async issues
        // In a real implementation, this would be async or use a different approach
        Self {
            keyword_search: KeywordSearch::new(),
            fuzzy_search: FuzzySearch::new(),
            regex_search: RegexSearch::new(),
            semantic_search: None,
        }
    }

    /// Creates a new AutoStrategy instance with semantic search (async)
    pub async fn with_semantic_search() -> Result<Self> {
        let config = EmbeddingConfig::default();
        let embedder = LocalEmbedder::new(config).await?;

        Ok(Self {
            keyword_search: KeywordSearch::new(),
            fuzzy_search: FuzzySearch::new(),
            regex_search: RegexSearch::new(),
            semantic_search: Some(crate::search::semantic::SemanticSearch::new(Arc::new(
                embedder,
            ))),
        })
    }

    /// Performs a search using the automatically selected strategy
    pub async fn search(&self, query: &str, path: &str) -> Result<Vec<SearchResult>> {
        let query_type = QueryAnalyzer::analyze(query);
        let context = ProjectContext::detect(path)?;

        match (query_type, context, &self.semantic_search) {
            // Code patterns in code projects use regex
            (QueryType::CodePattern, ProjectContext::Code, _) => {
                let regex_query = self.code_pattern_to_regex(query);
                self.regex_search.search(&regex_query, path).await
            }

            // Conceptual queries use semantic search if available
            (QueryType::Conceptual, _, Some(_semantic)) => {
                // For now, fallback to fuzzy since semantic search doesn't have path-based search
                // In a real implementation, this would use the semantic search with file chunks
                self.fuzzy_search.search(query, path).await
            }

            // Exact phrases use keyword search
            (QueryType::ExactPhrase, _, _) => self.keyword_search.search(query, path).await,

            // Regex-like patterns use regex search
            (QueryType::RegexLike, _, _) => self.regex_search.search(query, path).await,

            // File extensions use keyword search with file filtering
            (QueryType::FileExtension, _, _) => self.keyword_search.search(query, path).await,

            // Default to fuzzy for typo tolerance
            _ => self.fuzzy_search.search(query, path).await,
        }
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
        // Test with current directory (should be mixed since we're in a Rust project)
        let context = ProjectContext::detect(".").unwrap();
        assert!(matches!(context, ProjectContext::Mixed));
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
