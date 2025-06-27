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
    fn simplify_query(query: &str) -> String {
        // Remove common programming terms and simplify
        let words: Vec<&str> = query
            .split_whitespace()
            .filter(|word| !word.contains("()") && !word.contains("{}"))
            .take(2) // Take first 2 words
            .collect();

        if words.is_empty() {
            "search term".to_string()
        } else {
            words.join(" ")
        }
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserError::NoMatches { query, suggestions } => {
                writeln!(f, "No matches found for '{}'.", query)?;
                writeln!(f)?;
                writeln!(f, "Try:")?;
                for suggestion in suggestions {
                    writeln!(f, "  • {}", suggestion)?;
                }
            }
            UserError::DirectoryAccess {
                path,
                reason,
                suggestions,
            } => {
                writeln!(f, "Cannot search in {}.", path)?;
                writeln!(f, "{}", reason)?;
                writeln!(f)?;
                writeln!(f, "Try:")?;
                for suggestion in suggestions {
                    writeln!(f, "  • {}", suggestion)?;
                }
            }
            UserError::FallbackMode { reason: _, tip } => {
                write!(f, "{}", tip)?;
            }
            UserError::Database { issue, suggestions } => {
                writeln!(f, "Database issue: {}", issue)?;
                writeln!(f)?;
                writeln!(f, "Try:")?;
                for suggestion in suggestions {
                    writeln!(f, "  • {}", suggestion)?;
                }
            }
            UserError::InvalidQuery {
                query,
                issue,
                suggestions,
            } => {
                writeln!(f, "Invalid query '{}': {}", query, issue)?;
                writeln!(f)?;
                writeln!(f, "Try:")?;
                for suggestion in suggestions {
                    writeln!(f, "  • {}", suggestion)?;
                }
            }
            UserError::Permission {
                path,
                operation,
                suggestions,
            } => {
                writeln!(f, "Permission denied: cannot {} in {}", operation, path)?;
                writeln!(f)?;
                writeln!(f, "Try:")?;
                for suggestion in suggestions {
                    writeln!(f, "  • {}", suggestion)?;
                }
            }
            UserError::Generic {
                message,
                suggestions,
            } => {
                writeln!(f, "{}", message)?;
                if !suggestions.is_empty() {
                    writeln!(f)?;
                    writeln!(f, "Try:")?;
                    for suggestion in suggestions {
                        writeln!(f, "  • {}", suggestion)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl std::error::Error for UserError {}
