use super::UserError;
use anyhow::Error as AnyhowError;

/// Translates technical errors into user-friendly error messages
pub struct ErrorTranslator;

impl ErrorTranslator {
    /// Translate a technical error into a user-friendly error
    pub fn translate_technical_error(error: &AnyhowError) -> UserError {
        Self::translate_technical_error_with_context(error, None, None)
    }

    /// Translate a technical error with additional context (query, path)
    pub fn translate_technical_error_with_context(
        error: &AnyhowError,
        query: Option<&str>,
        path: Option<&str>,
    ) -> UserError {
        let error_str = error.to_string().to_lowercase();
        let error_chain = Self::get_error_chain(error);

        // Check for specific error patterns
        if Self::is_no_results_error(&error_str, &error_chain) {
            Self::handle_no_results_with_context(query, path)
        } else if Self::is_directory_access_error(&error_str, &error_chain) {
            Self::handle_directory_access_error_with_context(error, &error_str, path)
        } else if Self::is_permission_error(&error_str, &error_chain) {
            Self::handle_permission_error_with_context(error, &error_str, path)
        } else if Self::is_database_error(&error_str, &error_chain) {
            Self::handle_database_error(error, &error_str)
        } else if Self::is_neural_embedding_error(&error_str, &error_chain) {
            Self::handle_neural_embedding_error(&error_str)
        } else if Self::is_invalid_query_error(&error_str, &error_chain) {
            Self::handle_invalid_query_error(query, &error_str)
        } else {
            Self::handle_generic_error_with_context(error, &error_str, query, path)
        }
    }

    /// Extract error chain for better analysis
    fn get_error_chain(error: &AnyhowError) -> Vec<String> {
        let mut chain = Vec::new();
        let mut current = error.source();

        while let Some(err) = current {
            chain.push(err.to_string().to_lowercase());
            current = err.source();
        }

        chain
    }

    /// Check if this is a directory/file access error
    fn is_directory_access_error(error_str: &str, chain: &[String]) -> bool {
        let patterns = [
            "no such file or directory",
            "not found",
            "does not exist",
            "io error",
            "os error 2",
        ];

        patterns.iter().any(|&pattern| {
            error_str.contains(pattern) || chain.iter().any(|e| e.contains(pattern))
        })
    }

    /// Check if this is a permission error
    fn is_permission_error(error_str: &str, chain: &[String]) -> bool {
        let patterns = [
            "permission denied",
            "access denied",
            "os error 13",
            "insufficient permissions",
        ];

        patterns.iter().any(|&pattern| {
            error_str.contains(pattern) || chain.iter().any(|e| e.contains(pattern))
        })
    }

    /// Check if this is a database error
    fn is_database_error(error_str: &str, chain: &[String]) -> bool {
        let patterns = ["database", "sqlite", "sql", "index", "rusqlite"];

        patterns.iter().any(|&pattern| {
            error_str.contains(pattern) || chain.iter().any(|e| e.contains(pattern))
        })
    }

    /// Check if this is a neural embedding error
    fn is_neural_embedding_error(error_str: &str, chain: &[String]) -> bool {
        let patterns = [
            "onnx",
            "neural",
            "embedding",
            "model",
            "tokenizer",
            "huggingface",
        ];

        patterns.iter().any(|&pattern| {
            error_str.contains(pattern) || chain.iter().any(|e| e.contains(pattern))
        })
    }

    /// Check if this is a no results error
    fn is_no_results_error(error_str: &str, chain: &[String]) -> bool {
        let patterns = [
            "no results",
            "no matches",
            "not found",
            "empty result",
            "0 matches",
        ];

        patterns.iter().any(|&pattern| {
            error_str.contains(pattern) || chain.iter().any(|e| e.contains(pattern))
        })
    }

    /// Check if this is an invalid query error
    fn is_invalid_query_error(error_str: &str, chain: &[String]) -> bool {
        let patterns = [
            "invalid query",
            "malformed query",
            "regex error",
            "parse error",
            "syntax error",
        ];

        patterns.iter().any(|&pattern| {
            error_str.contains(pattern) || chain.iter().any(|e| e.contains(pattern))
        })
    }

    /// Handle no results with context
    fn handle_no_results_with_context(query: Option<&str>, _path: Option<&str>) -> UserError {
        let query = query.unwrap_or("search term");
        UserError::no_matches(query)
    }

    /// Handle invalid query errors
    fn handle_invalid_query_error(query: Option<&str>, error_str: &str) -> UserError {
        let query = query.unwrap_or("query");
        let issue = if error_str.contains("regex") {
            "Invalid regular expression syntax"
        } else if error_str.contains("parse") {
            "Unable to parse query"
        } else {
            "Query format is not valid"
        };

        UserError::InvalidQuery {
            query: query.to_string(),
            issue: issue.to_string(),
            suggestions: vec![
                "Try a simpler search without special characters".to_string(),
                "Check your query syntax".to_string(),
                "For help with search syntax, run: semisearch help-me".to_string(),
            ],
        }
    }

    /// Handle directory access errors with context
    fn handle_directory_access_error_with_context(
        _error: &AnyhowError,
        error_str: &str,
        path: Option<&str>,
    ) -> UserError {
        let path = path
            .map(|s| s.to_string())
            .or_else(|| Self::extract_path_from_error(error_str))
            .unwrap_or_else(|| "specified path".to_string());

        let reason =
            if error_str.contains("no such file or directory") || error_str.contains("not found") {
                "The directory or file does not exist"
            } else if error_str.contains("os error 2") {
                "The directory or file was not found"
            } else {
                "Cannot access the directory or file"
            };

        UserError::directory_access(&path, reason)
    }

    /// Handle permission errors with context
    fn handle_permission_error_with_context(
        _error: &AnyhowError,
        error_str: &str,
        path: Option<&str>,
    ) -> UserError {
        let path = path
            .map(|s| s.to_string())
            .or_else(|| Self::extract_path_from_error(error_str))
            .unwrap_or_else(|| "specified path".to_string());

        let operation = if error_str.contains("index") {
            "index"
        } else {
            "search"
        };

        UserError::permission(&path, operation)
    }

    /// Handle database errors
    fn handle_database_error(_error: &AnyhowError, error_str: &str) -> UserError {
        let issue = if error_str.contains("sqlite") {
            "SQLite database error"
        } else if error_str.contains("index") {
            "Search index error"
        } else {
            "Database connection or operation failed"
        };

        UserError::database(issue)
    }

    /// Handle neural embedding errors
    fn handle_neural_embedding_error(error_str: &str) -> UserError {
        let reason = if error_str.contains("onnx") {
            "ONNX Runtime not available"
        } else if error_str.contains("model") {
            "Neural model not found"
        } else {
            "Neural embeddings unavailable"
        };

        UserError::fallback_mode(reason)
    }

    /// Handle generic errors with context
    fn handle_generic_error_with_context(
        error: &AnyhowError,
        _error_str: &str,
        query: Option<&str>,
        path: Option<&str>,
    ) -> UserError {
        // Try to extract a meaningful message without technical jargon
        let message = Self::clean_error_message(&error.to_string());

        let mut suggestions = vec![
            "Try running 'semisearch status' to check if everything is working".to_string(),
            "Check the command syntax and try again".to_string(),
        ];

        // Add context-specific suggestions
        if let Some(q) = query {
            suggestions.push(format!(
                "Try a simpler search: semisearch \"{q}\" --fuzzy"
            ));
        }
        if path.is_some() {
            suggestions.push("Check that the search path exists and is accessible".to_string());
        }

        suggestions.push("For help, run: semisearch help-me".to_string());

        UserError::Generic {
            message,
            suggestions,
        }
    }

    /// Extract file path from error message
    fn extract_path_from_error(error_str: &str) -> Option<String> {
        // Look for common path patterns in error messages
        let lines: Vec<&str> = error_str.lines().collect();

        for line in lines {
            // Look for patterns like "/path/to/file:" or "path: error"
            if let Some(colon_pos) = line.find(':') {
                let potential_path = line[..colon_pos].trim();

                // Check if it looks like a path
                if potential_path.starts_with('/')
                    || potential_path.starts_with("./")
                    || potential_path.starts_with("../")
                    || potential_path.contains('\\')
                {
                    return Some(potential_path.to_string());
                }
            }

            // Look for "No such file or directory" patterns
            if line.contains("No such file or directory") {
                // Extract path before the error message
                if let Some(start) = line.find(' ') {
                    let potential_path = line[..start].trim();
                    if !potential_path.is_empty()
                        && (potential_path.starts_with('/') || potential_path.contains('/'))
                    {
                        return Some(potential_path.to_string());
                    }
                }
            }
        }

        None
    }

    /// Clean error message by removing technical jargon
    fn clean_error_message(message: &str) -> String {
        let message = message
            .replace("anyhow::Error", "Error")
            .replace("std::io::Error", "File system error")
            .replace("Error:", "")
            .trim()
            .to_string();

        // Take only the first line if it's a multi-line error
        if let Some(first_line) = message.lines().next() {
            first_line.trim().to_string()
        } else {
            "An unexpected error occurred".to_string()
        }
    }

    /// Check if a query returned no results (not an error, but needs special handling)
    pub fn handle_no_results(query: &str) -> UserError {
        UserError::no_matches(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_path_from_error() {
        let error_msg = "/nonexistent/path: No such file or directory";
        let path = ErrorTranslator::extract_path_from_error(error_msg);
        assert_eq!(path, Some("/nonexistent/path".to_string()));
    }

    #[test]
    fn test_clean_error_message() {
        let message = "anyhow::Error: Something went wrong";
        let cleaned = ErrorTranslator::clean_error_message(message);
        assert_eq!(cleaned, "Something went wrong");
    }

    #[test]
    fn test_directory_access_error_detection() {
        let error_str = "no such file or directory";
        let chain = vec![];
        assert!(ErrorTranslator::is_directory_access_error(
            error_str, &chain
        ));
    }
}
