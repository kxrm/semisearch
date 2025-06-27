use std::collections::HashSet;

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
        let code_keywords: HashSet<&str> = [
            "function",
            "class",
            "TODO",
            "FIXME",
            "import",
            "export",
            "async",
            "await",
            "Function",
            "Class",
            "todo",
            "fixme",
            "Import",
            "Export",
            "Async",
            "Await",
            "fn",
            "pub",
            "mod",
            "struct",
            "enum",
            "trait",
            "impl",
            "let",
            "const",
            "var",
            "def",
            "method",
            "constructor",
            "abstract",
            "static",
            "final",
            "public",
            "private",
            "protected",
            "virtual",
            "override",
            "extends",
            "implements",
        ]
        .iter()
        .cloned()
        .collect();

        let query_lower = query.to_lowercase();
        let words: Vec<&str> = query_lower.split_whitespace().collect();

        // For multi-word queries, be more conservative
        if words.len() > 2 {
            // Only detect as code pattern if it looks like a code-specific query
            let code_specific_patterns = [
                "function", "class", "TODO", "FIXME", "import", "export", "async", "await", "fn",
                "pub", "mod", "struct", "enum", "trait", "impl", "def", "method",
            ];

            return words
                .iter()
                .any(|word| code_specific_patterns.contains(word));
        }

        // For shorter queries, be more permissive
        words.iter().any(|word| code_keywords.contains(word))
    }

    /// Checks if the query contains file extensions
    fn contains_file_extensions(query: &str) -> bool {
        let file_extensions = [
            ".rs", ".py", ".js", ".ts", ".md", ".txt", ".json", ".toml", ".yaml", ".yml", ".xml",
            ".html", ".css", ".scss", ".sass", ".less", ".sql", ".sh", ".bash", ".zsh", ".fish",
            ".ps1", ".bat", ".cmd", ".exe", ".dll", ".so", ".dylib",
        ];

        file_extensions.iter().any(|ext| query.contains(ext))
    }

    /// Checks if the query looks like a regex pattern
    fn looks_like_regex(query: &str) -> bool {
        let regex_metacharacters = [
            ".*", "\\d+", "\\w+", "\\s+", "\\b", "\\B", "\\A", "\\Z", "\\z", "[", "]", "(", ")",
            "{", "}", "|", "^", "$", "?", "*", "+",
        ];

        // Check for common regex patterns
        let regex_patterns = [
            r"\\d+", r"\\w+", r"\\s+", r"\\b", r"\\B", r"\\A", r"\\Z", r"\\z", r"\[.*\]",
            r"\(.*\)", r"\{.*\}", r".*", r".+", r".?", r".*?", r".+?",
        ];

        // Check for metacharacters
        if regex_metacharacters
            .iter()
            .any(|&meta| query.contains(meta))
        {
            return true;
        }

        // Check for regex patterns
        if regex_patterns.iter().any(|&pattern| {
            // Simple pattern matching - in a real implementation you might use regex crate
            query.contains(pattern)
        }) {
            return true;
        }

        false
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
