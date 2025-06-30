// User-friendly error handling for UX Remediation Plan

/// User-friendly error messages that replace technical jargon with helpful guidance
/// Implements UX Remediation Plan Task 1.2: Fix Error Messages

#[derive(Debug)]
pub struct UserFriendlyError {
    pub message: String,
    pub suggestions: Vec<String>,
}

impl UserFriendlyError {
    pub fn no_matches(query: &str, _path: &str) -> Self {
        let simplified = simplify_query(query);
        let suggestions = vec![
            format!("Check spelling: semisearch \"{}\" --fuzzy", query),
            format!("Use simpler terms: semisearch \"{}\"", simplified),
            format!("Search in specific folder: semisearch \"{}\" src/", query),
            "Need help? Try: semisearch help-me".to_string(),
        ];

        Self {
            message: format!("No matches found for '{query}'."),
            suggestions,
        }
    }

    pub fn directory_access_error(path: &str) -> Self {
        Self {
            message: format!("Cannot search in {path}."),
            suggestions: vec![
                "Make sure the directory exists and you have permission to read it.".to_string(),
                "Try searching in current directory: semisearch \"your query\" .".to_string(),
                "Check if the path is correct".to_string(),
            ],
        }
    }

    pub fn fallback_mode() -> Self {
        Self {
            message: "🔍 Searching with basic mode (fast but less smart)".to_string(),
            suggestions: vec!["💡 Tip: Install semisearch-models for better results".to_string()],
        }
    }

    pub fn too_many_results(count: usize) -> Self {
        Self {
            message: format!("Found {count} matches! That's a lot."),
            suggestions: vec![
                "💡 Use more specific terms to narrow results".to_string(),
                "💡 Search in a specific folder".to_string(),
                "💡 Use exact phrases in quotes".to_string(),
            ],
        }
    }

    pub fn display(&self) -> String {
        let mut output = self.message.clone();

        if !self.suggestions.is_empty() {
            output.push_str("\n\nTry:\n");
            for suggestion in &self.suggestions {
                if suggestion.starts_with("💡") {
                    output.push_str(&format!("{suggestion}\n"));
                } else {
                    output.push_str(&format!("  • {suggestion}\n"));
                }
            }
        }

        output
    }
}

/// Simplify a query by removing noise words and complex terms
pub fn simplify_query(query: &str) -> String {
    let noise_words = [
        "the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by", "from", "up",
        "about", "into", "through", "during", "before", "after", "above", "below", "between",
        "among", "within", "without", "under", "over",
    ];

    let words: Vec<&str> = query
        .split_whitespace()
        .filter(|word| {
            let clean_word = word.to_lowercase();
            !noise_words.contains(&clean_word.as_str()) && clean_word.len() > 2
        })
        .take(3) // Take only first 3 meaningful words
        .collect();

    if words.is_empty() {
        // If all words were filtered out, return the first word of the original query
        query.split_whitespace().next().unwrap_or(query).to_string()
    } else {
        words.join(" ")
    }
}

/// Extract file path from error messages
pub fn extract_path_from_error(error_msg: &str) -> Option<String> {
    // Simple pattern matching for common path patterns in error messages
    if let Some(start) = error_msg.find("path") {
        if let Some(end) = error_msg[start..].find(' ') {
            return Some(error_msg[start..start + end].to_string());
        }
    }
    None
}

/// Translate technical errors to user-friendly messages
pub fn translate_error(error: &anyhow::Error) -> UserFriendlyError {
    let error_str = error.to_string().to_lowercase();

    if error_str.contains("no such file") || error_str.contains("not found") {
        if let Some(path) = extract_path_from_error(&error_str) {
            UserFriendlyError::directory_access_error(&path)
        } else {
            UserFriendlyError::directory_access_error("that location")
        }
    } else if error_str.contains("permission denied") {
        UserFriendlyError::directory_access_error("that directory (permission denied)")
    } else if error_str.contains("onnx") || error_str.contains("neural") {
        UserFriendlyError::fallback_mode()
    } else {
        // Generic helpful error
        UserFriendlyError {
            message: "Something went wrong with the search.".to_string(),
            suggestions: vec![
                "Try running: semisearch status".to_string(),
                "Check if your query has special characters".to_string(),
                "Need help? Try: semisearch help-me".to_string(),
            ],
        }
    }
}

/// Provide contextual suggestions based on search results
pub fn provide_contextual_suggestions(
    query: &str,
    results_count: usize,
    _project_type: &str,
) -> Option<UserFriendlyError> {
    match results_count {
        0 => Some(UserFriendlyError::no_matches(query, ".")),
        count if count > 50 => Some(UserFriendlyError::too_many_results(count)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_query() {
        assert_eq!(simplify_query("the quick brown fox"), "quick brown fox");
        assert_eq!(
            simplify_query("find all the error handling"),
            "find all error"
        );
        assert_eq!(simplify_query("a"), "a");
        assert_eq!(simplify_query(""), "");
    }

    #[test]
    fn test_no_matches_error() {
        let error = UserFriendlyError::no_matches("databse", ".");
        assert!(error.message.contains("No matches found"));
        assert!(error.suggestions.iter().any(|s| s.contains("--fuzzy")));
    }

    #[test]
    fn test_error_display() {
        let error = UserFriendlyError::no_matches("test", ".");
        let display = error.display();
        assert!(display.contains("No matches found"));
        assert!(display.contains("Try:"));
        assert!(display.contains("--fuzzy"));
    }
}
