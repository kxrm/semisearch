use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// User-friendly error types that provide helpful guidance instead of technical details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "error_type", content = "details")]
pub enum UserError {
    /// No matches found for the search query
    NoMatches {
        query: String,
        suggestions: Vec<String>,
    },

    /// Cannot access the specified directory or file
    DirectoryAccess {
        path: String,
        reason: String,
        suggestions: Vec<String>,
    },

    /// Search capabilities are limited (fallback mode)
    FallbackMode { reason: String, tip: String },

    /// Database-related issues
    Database {
        issue: String,
        suggestions: Vec<String>,
    },

    /// Invalid search query or parameters
    InvalidQuery {
        query: String,
        issue: String,
        suggestions: Vec<String>,
    },

    /// Permission-related errors
    Permission {
        path: String,
        operation: String,
        suggestions: Vec<String>,
    },

    /// Generic error with helpful suggestions
    Generic {
        message: String,
        suggestions: Vec<String>,
    },
}

impl UserError {
    /// Create a NoMatches error with helpful suggestions
    pub fn no_matches(query: &str) -> Self {
        let simplified = Self::simplify_query(query);
        Self::NoMatches {
            query: query.to_string(),
            suggestions: vec![
                format!("Check spelling: semisearch \"{}\" --fuzzy", query),
                format!("Use simpler terms: semisearch \"{}\"", simplified),
                format!("Search everywhere: semisearch \"{}\" .", query),
                "Try different keywords or phrases".to_string(),
            ],
        }
    }

    /// Create a DirectoryAccess error
    pub fn directory_access(path: &str, reason: &str) -> Self {
        Self::DirectoryAccess {
            path: path.to_string(),
            reason: reason.to_string(),
            suggestions: vec![
                "Make sure the directory exists and you have permission to read it".to_string(),
                "Check the path spelling".to_string(),
                "Try using an absolute path".to_string(),
                format!("Try: semisearch \"<query>\" .  # to search current directory"),
            ],
        }
    }

    /// Create a FallbackMode error
    pub fn fallback_mode(reason: &str) -> Self {
        Self::FallbackMode {
            reason: reason.to_string(),
            tip: "🔍 Searching with basic mode (fast but less smart)\n💡 Tip: Install semisearch-models for better results".to_string(),
        }
    }

    /// Create a Database error
    pub fn database(issue: &str) -> Self {
        Self::Database {
            issue: issue.to_string(),
            suggestions: vec![
                "Try running: semisearch status".to_string(),
                "Try running: semisearch index .".to_string(),
                "Check if you have write permissions to ~/.semisearch/".to_string(),
            ],
        }
    }

    /// Create a Permission error
    pub fn permission(path: &str, operation: &str) -> Self {
        Self::Permission {
            path: path.to_string(),
            operation: operation.to_string(),
            suggestions: vec![
                "Check file/directory permissions".to_string(),
                "Make sure you have read access to the directory".to_string(),
                "Try running with appropriate permissions".to_string(),
                format!("Try a different directory: semisearch \"<query>\" ."),
            ],
        }
    }

    /// Get the exit code for this error type
    pub fn exit_code(&self) -> i32 {
        match self {
            UserError::NoMatches { .. } => 1, // No matches found - exit code 1 (Unix convention)
            UserError::DirectoryAccess { .. } => 1,
            UserError::Permission { .. } => 1,
            UserError::Database { .. } => 1,
            UserError::InvalidQuery { .. } => 2,
            UserError::FallbackMode { .. } => 0, // Fallback is not an error
            UserError::Generic { .. } => 1,
        }
    }

    /// Format as JSON for structured output
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Simplify a query for suggestions
    pub fn simplify_query(query: &str) -> String {
        // Common programming terms that can be removed for simplification
        let programming_terms: std::collections::HashSet<&str> = [
            "function",
            "class",
            "method",
            "async",
            "await",
            "const",
            "let",
            "var",
            "public",
            "private",
            "protected",
            "static",
            "final",
            "abstract",
            "interface",
            "type",
            "enum",
            "struct",
            "trait",
            "impl",
            "mod",
            "import",
            "export",
            "require",
            "include",
            "using",
            "namespace",
            "try",
            "catch",
            "throw",
            "throws",
            "error",
            "exception",
            "handler",
            "validate",
            "validation",
            "check",
            "verify",
            "test",
            "testing",
            "config",
            "configuration",
            "setup",
            "initialize",
            "init",
            "db",
            "query",
            "sql",
            "api",
            "endpoint",
            "route",
            "controller",
            "service",
            "repository",
            "model",
            "view",
            "component",
            "utils",
            "utility",
            "helper",
            "fn",
            "pub",
            "def",
            "return",
        ]
        .iter()
        .cloned()
        .collect();

        // Common file extensions that can be removed
        let file_extensions: std::collections::HashSet<&str> = [
            ".rs", ".py", ".js", ".ts", ".md", ".txt", ".json", ".toml", ".yaml", ".yml", ".xml",
            ".html", ".css", ".scss", ".sql", ".sh", ".bash",
        ]
        .iter()
        .cloned()
        .collect();

        // Common noise words
        let noise_words: std::collections::HashSet<&str> = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do",
            "does", "did", "will", "would", "could", "should", "may", "might", "can", "this",
            "that", "these", "those",
        ]
        .iter()
        .cloned()
        .collect();

        // Split on whitespace and punctuation
        let re = Regex::new(r"[ \t\n\r\.\:\(\)\{\}\[\],;]+")
            .expect("Invalid regex for query simplification");
        let tokens: Vec<&str> = re
            .split(query)
            .filter(|token| {
                let token_lower = token.to_lowercase();
                token.len() > 2
                    && !programming_terms.contains(token_lower.as_str())
                    && !file_extensions.contains(token)
                    && !noise_words.contains(token_lower.as_str())
            })
            .collect();

        if tokens.is_empty() {
            "search term".to_string()
        } else {
            tokens.into_iter().take(3).collect::<Vec<_>>().join(" ")
        }
    }

    #[cfg(test)]
    pub fn test_simplify_query(query: &str) -> String {
        Self::simplify_query(query)
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserError::NoMatches { query, suggestions } => {
                writeln!(f, "No matches found for '{query}'.")?;
                writeln!(f)?;
                writeln!(f, "Try:")?;
                for suggestion in suggestions {
                    writeln!(f, "  • {suggestion}")?;
                }
            }
            UserError::DirectoryAccess {
                path,
                reason,
                suggestions,
            } => {
                writeln!(f, "Cannot search in {path}.")?;
                writeln!(f, "{reason}")?;
                writeln!(f)?;
                writeln!(f, "Try:")?;
                for suggestion in suggestions {
                    writeln!(f, "  • {suggestion}")?;
                }
            }
            UserError::FallbackMode { reason: _, tip } => {
                write!(f, "{tip}")?;
            }
            UserError::Database { issue, suggestions } => {
                writeln!(f, "Database issue: {issue}")?;
                writeln!(f)?;
                writeln!(f, "Try:")?;
                for suggestion in suggestions {
                    writeln!(f, "  • {suggestion}")?;
                }
            }
            UserError::InvalidQuery {
                query,
                issue,
                suggestions,
            } => {
                writeln!(f, "Invalid query '{query}': {issue}")?;
                writeln!(f)?;
                writeln!(f, "Try:")?;
                for suggestion in suggestions {
                    writeln!(f, "  • {suggestion}")?;
                }
            }
            UserError::Permission {
                path,
                operation,
                suggestions,
            } => {
                writeln!(f, "Permission denied: cannot {operation} in {path}")?;
                writeln!(f)?;
                writeln!(f, "Try:")?;
                for suggestion in suggestions {
                    writeln!(f, "  • {suggestion}")?;
                }
            }
            UserError::Generic {
                message,
                suggestions,
            } => {
                writeln!(f, "{message}")?;
                if !suggestions.is_empty() {
                    writeln!(f)?;
                    writeln!(f, "Try:")?;
                    for suggestion in suggestions {
                        writeln!(f, "  • {suggestion}")?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl std::error::Error for UserError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_query_programming_terms() {
        // Test that programming terms are removed
        assert_eq!(
            UserError::test_simplify_query("function validateUserInput"),
            "validateUserInput"
        );
        assert_eq!(
            UserError::test_simplify_query("async await authentication"),
            "authentication"
        );
        assert_eq!(
            UserError::test_simplify_query("const let var database"),
            "database"
        );
    }

    #[test]
    fn test_simplify_query_file_extensions() {
        // Test that file extensions are removed or split
        assert_eq!(UserError::test_simplify_query("config.json setup"), "json");
        assert_eq!(
            UserError::test_simplify_query("file.rs extension"),
            "file extension"
        );
        assert_eq!(
            UserError::test_simplify_query("test.py validation"),
            "search term"
        );
    }

    #[test]
    fn test_simplify_query_noise_words() {
        // Test that noise words are removed
        assert_eq!(
            UserError::test_simplify_query("the quick brown fox"),
            "quick brown fox"
        );
        assert_eq!(
            UserError::test_simplify_query("a user login system"),
            "user login system"
        );
        assert_eq!(
            UserError::test_simplify_query("in the database with"),
            "database"
        );
    }

    #[test]
    fn test_simplify_query_special_chars() {
        // Test that words with special characters are split and filtered
        assert_eq!(
            UserError::test_simplify_query("complex.function.name()"),
            "complex name"
        );
        assert_eq!(
            UserError::test_simplify_query("array[index] access"),
            "array index access"
        );
        assert_eq!(
            UserError::test_simplify_query("object{property} get"),
            "object property get"
        );
    }

    #[test]
    fn test_simplify_query_short_words() {
        // Test that very short words are filtered appropriately
        assert_eq!(UserError::test_simplify_query("a b c long"), "long");
        assert_eq!(
            UserError::test_simplify_query("x y z important"),
            "important"
        );
        assert_eq!(UserError::test_simplify_query("1 2 3 data"), "data");
    }

    #[test]
    fn test_simplify_query_fallback() {
        // Test fallback behavior when everything is filtered out
        assert_eq!(
            UserError::test_simplify_query("function async"),
            "search term"
        );
        assert_eq!(UserError::test_simplify_query(""), "search term");
        assert_eq!(UserError::test_simplify_query("a"), "search term");
    }

    #[test]
    fn test_simplify_query_real_world_examples() {
        // Test real-world query examples
        assert_eq!(
            UserError::test_simplify_query("TODO: implement error handling"),
            "TODO implement handling"
        );
        assert_eq!(
            UserError::test_simplify_query("user authentication login system"),
            "user authentication login"
        );
        assert_eq!(
            UserError::test_simplify_query("database query optimization"),
            "database optimization"
        );
        assert_eq!(
            UserError::test_simplify_query("API endpoint configuration"),
            "search term"
        );
    }

    #[test]
    fn test_no_matches_error_with_simplified_suggestions() {
        let error = UserError::no_matches("function validateUserInput database query");
        // Only check the simplified suggestion, not the whole error string
        if let UserError::NoMatches { suggestions, .. } = error {
            let suggestion = &suggestions[1];
            assert!(!suggestion.contains("function"));
            assert!(!suggestion.contains("query"));
        } else {
            panic!("Expected NoMatches error");
        }
    }
}
