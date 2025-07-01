use crate::core::patterns::utils;

/// Represents different types of queries that can be analyzed
#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    /// Exact phrase search (quoted strings)
    ExactPhrase,
    /// Conceptual search (multi-word concepts)
    Conceptual,
    /// File extension specific search
    FileExtension,
    /// Code pattern search (function, class, TODO, etc.)
    CodePattern,
    /// Regex-like pattern search
    RegexLike,
}

/// Analyzes queries to determine their type and optimal search strategy
pub struct QueryAnalyzer;

impl QueryAnalyzer {
    /// Analyzes a query string and returns the appropriate QueryType
    pub fn analyze(query: &str) -> QueryType {
        // Check for exact phrase (quoted strings) - highest priority
        if query.contains('"') {
            return QueryType::ExactPhrase;
        }

        // Check for regex-like patterns
        if Self::looks_like_regex(query) {
            return QueryType::RegexLike;
        }

        // Check for file extensions
        if Self::contains_file_extensions(query) {
            return QueryType::FileExtension;
        }

        // Check for code keywords (even in multi-word queries)
        if Self::contains_code_keywords(query) {
            return QueryType::CodePattern;
        }

        // Check for conceptual queries (multi-word)
        if query.split_whitespace().count() > 2 {
            return QueryType::Conceptual;
        }

        // Default to exact phrase for simple queries
        QueryType::ExactPhrase
    }

    /// Checks if the query contains code-related keywords
    fn contains_code_keywords(query: &str) -> bool {
        utils::contains_code_keywords(query)
    }

    /// Checks if the query contains file extensions
    fn contains_file_extensions(query: &str) -> bool {
        utils::contains_file_extensions(query)
    }

    /// Checks if the query looks like a regex pattern
    fn looks_like_regex(query: &str) -> bool {
        utils::looks_like_regex(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_code_keywords() {
        assert!(QueryAnalyzer::contains_code_keywords("function"));
        assert!(QueryAnalyzer::contains_code_keywords("TODO"));
        assert!(QueryAnalyzer::contains_code_keywords("async function"));
        assert!(QueryAnalyzer::contains_code_keywords("export class"));
        assert!(!QueryAnalyzer::contains_code_keywords("hello world"));
    }

    #[test]
    fn test_contains_file_extensions() {
        assert!(QueryAnalyzer::contains_file_extensions(".rs"));
        assert!(QueryAnalyzer::contains_file_extensions("rust files .rs"));
        assert!(QueryAnalyzer::contains_file_extensions("python code .py"));
        assert!(!QueryAnalyzer::contains_file_extensions("hello world"));
    }

    #[test]
    fn test_looks_like_regex() {
        assert!(QueryAnalyzer::looks_like_regex(".*pattern"));
        assert!(QueryAnalyzer::looks_like_regex("\\d+"));
        assert!(QueryAnalyzer::looks_like_regex("[a-z]"));
        assert!(QueryAnalyzer::looks_like_regex("(group)"));
        assert!(!QueryAnalyzer::looks_like_regex("hello world"));
    }
}
