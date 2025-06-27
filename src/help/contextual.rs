use crate::cli::Commands;
use crate::errors::user_errors::UserError;
use crate::SearchResult;

/// Contextual help system that provides specific guidance based on user actions and results
pub struct ContextualHelp;

impl ContextualHelp {
    /// Generate contextual help based on the last command and its results
    pub fn generate_help(last_command: &Commands, results: &[SearchResult]) -> String {
        match (last_command, results.is_empty(), results.len()) {
            // Search command with no results
            (Commands::Search(search_args), true, _) => {
                let query = &search_args.query;
                let simplified = UserError::simplify_query(query);

                format!(
                    "No results for '{query}'. Here's what you can try:\n\
                     \n\
                     🔤 Check spelling:\n\
                     semisearch \"{query}\" --fuzzy\n\
                     \n\
                     🎯 Try simpler terms:\n\
                     semisearch \"{simplified}\"\n\
                     \n\
                     📂 Search specific files:\n\
                     semisearch \"{query}\" src/\n\
                     \n\
                     ❓ Need more help? Run: semisearch help-me"
                )
            }

            // Search command with too many results (more than 50)
            (Commands::Search(_), false, count) if count > 50 => "Found lots of matches! Try:\n\
                 • More specific terms\n\
                 • Search in a specific folder\n\
                 • Use exact phrases in quotes"
                .to_string(),

            // Other commands or good results - no contextual help needed
            _ => String::new(),
        }
    }

    /// Generate help suggestions for search errors
    pub fn generate_error_help(query: &str, error_type: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        match error_type {
            "no_matches" => {
                let simplified = UserError::simplify_query(query);
                suggestions.push(format!("Check spelling: semisearch \"{query}\" --fuzzy"));
                suggestions.push(format!("Use simpler terms: semisearch \"{simplified}\""));
                suggestions.push(format!("Search everywhere: semisearch \"{query}\" ."));
                suggestions.push("Try different keywords or phrases".to_string());
            }
            "directory_access" => {
                suggestions.push(
                    "Make sure the directory exists and you have permission to read it".to_string(),
                );
                suggestions.push("Check the path spelling".to_string());
                suggestions.push("Try using an absolute path".to_string());
                suggestions.push(
                    "Try: semisearch \"<query>\" .  # to search current directory".to_string(),
                );
            }
            "permission" => {
                suggestions.push("Check file/directory permissions".to_string());
                suggestions.push("Make sure you have read access to the directory".to_string());
                suggestions.push("Try running with appropriate permissions".to_string());
                suggestions.push("Try a different directory".to_string());
            }
            _ => {
                suggestions.push("Try running: semisearch status".to_string());
                suggestions.push("Need help? Try: semisearch help-me".to_string());
            }
        }

        suggestions
    }

    /// Generate usage examples based on query patterns
    pub fn generate_usage_examples(query: &str) -> Vec<String> {
        let mut examples = Vec::new();
        let query_lower = query.to_lowercase();

        if query_lower.contains("todo") || query_lower.contains("fixme") {
            examples.push("Find all TODO comments: semisearch \"TODO\"".to_string());
            examples.push("Find FIXME comments: semisearch \"FIXME\"".to_string());
        } else if query_lower.contains("function") || query_lower.contains("fn") {
            examples.push("Find function definitions: semisearch \"function login\"".to_string());
            examples.push("Find async functions: semisearch \"async function\"".to_string());
        } else if query_lower.contains("error") || query_lower.contains("exception") {
            examples.push("Find error handling: semisearch \"try catch\"".to_string());
            examples.push("Find error messages: semisearch \"error\"".to_string());
        } else if query_lower.contains("config") || query_lower.contains("setting") {
            examples.push("Find configuration: semisearch \"config\"".to_string());
            examples.push("Find settings files: semisearch \"settings.json\"".to_string());
        } else {
            // Generic examples
            examples.push("Basic search: semisearch \"your search term\"".to_string());
            examples.push("Fuzzy search: semisearch \"your search\" --fuzzy".to_string());
            examples.push("Exact search: semisearch \"exact phrase\" --exact".to_string());
        }

        examples
    }

    /// Generate tips based on search patterns and results
    pub fn generate_tips(query: &str, results: &[SearchResult]) -> Vec<String> {
        let mut tips = Vec::new();

        // Tips based on query complexity
        if query.len() > 50 {
            tips.push("💡 Long queries can be simplified. Try using key terms only.".to_string());
        }

        if query.contains("(") || query.contains("{") || query.contains("[") {
            tips.push("💡 Remove special characters for broader matches.".to_string());
        }

        // Tips based on results
        if results.is_empty() {
            tips.push("💡 No matches? Try --fuzzy for typo tolerance.".to_string());
            tips.push("💡 Use simpler, more common terms.".to_string());
        } else if results.len() == 1 {
            tips.push(
                "💡 Found exactly one match! Try broader terms for more results.".to_string(),
            );
        } else if results.len() > 20 {
            tips.push(
                "💡 Many matches found. Use more specific terms or search in a specific folder."
                    .to_string(),
            );
        }

        // File type specific tips
        let has_code_files = results.iter().any(|r| {
            r.file_path.ends_with(".rs")
                || r.file_path.ends_with(".py")
                || r.file_path.ends_with(".js")
                || r.file_path.ends_with(".ts")
        });

        if has_code_files {
            tips.push(
                "💡 Searching code? Try function names, variable names, or comments.".to_string(),
            );
        }

        tips
    }
}
