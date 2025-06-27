use crate::core::embedder::{EmbeddingConfig, LocalEmbedder};
use crate::help::contextual::ContextualHelp;
use crate::search::strategy::SearchEngine;
use crate::storage::database::Database;
use crate::SearchOptions;
use crate::SearchResult;
use anyhow::Result;
use std::io::{BufRead, Write};
use std::path::PathBuf;

/// Interactive help system that guides users through search scenarios
pub struct InteractiveHelp;

impl InteractiveHelp {
    /// Run interactive help with stdin/stdout
    pub async fn run() -> Result<()> {
        let stdin = std::io::stdin();
        let mut stdin_lock = stdin.lock();
        let mut stdout = std::io::stdout();

        Self::run_with_io(&mut stdin_lock, &mut stdout).await
    }

    /// Run interactive help with custom input/output (for testing)
    pub async fn run_with_io<R: BufRead, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
        writeln!(output, "👋 Welcome to SemiSearch!")?;
        writeln!(output, "Let's find what you're looking for.\n")?;

        writeln!(output, "What do you want to search for?")?;
        writeln!(output, "Examples:")?;
        writeln!(output, "  • TODO comments: semisearch \"TODO\"")?;
        writeln!(output, "  • Error handling: semisearch \"try catch\"")?;
        writeln!(
            output,
            "  • Function definitions: semisearch \"function login\""
        )?;
        writeln!(output)?;

        writeln!(output, "Type your search below, or 'quit' to exit:")?;

        loop {
            write!(output, "> ")?;
            output.flush()?;

            let mut user_input = String::new();
            match input.read_line(&mut user_input) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let user_input = user_input.trim();

                    if user_input.is_empty() {
                        continue;
                    }

                    if user_input == "quit" || user_input == "exit" {
                        writeln!(output, "👋 Goodbye!")?;
                        break;
                    }

                    // Handle special commands
                    if user_input == "help" || user_input == "?" {
                        Self::show_interactive_help(output)?;
                        continue;
                    }

                    if user_input == "examples" {
                        Self::show_examples(output)?;
                        continue;
                    }

                    if user_input == "tips" {
                        Self::show_tips(output)?;
                        continue;
                    }

                    // Process search query - ACTUALLY EXECUTE THE SEARCH
                    writeln!(output, "Searching for: {user_input}")?;
                    writeln!(output)?;

                    // Execute real search
                    let search_results = Self::execute_search(user_input).await;
                    match search_results {
                        Ok(results) => {
                            // Show search results
                            Self::display_search_results(output, user_input, &results)?;

                            // Show contextual help based on results
                            if results.is_empty() {
                                writeln!(output, "\n💡 Try:")?;
                                writeln!(output, "  • Check spelling with --fuzzy flag")?;
                                writeln!(output, "  • Use simpler terms")?;
                                writeln!(output, "  • Search in specific folders")?;
                            }
                        }
                        Err(e) => {
                            writeln!(output, "❌ Search failed: {e}")?;

                            // Show error-specific help
                            let error_help =
                                ContextualHelp::generate_error_help(user_input, "no_matches");
                            if !error_help.is_empty() {
                                writeln!(output, "\n{}", error_help.join("\n"))?;
                            }
                        }
                    }
                    writeln!(output)?;
                }
                Err(_) => break,
            }
        }

        Ok(())
    }

    /// Execute a real search with the given query
    async fn execute_search(query: &str) -> Result<Vec<SearchResult>> {
        // Get database path - same as main.rs
        let db_path = Self::get_database_path()?;
        let database = Database::new(&db_path)?;

        // Determine if we should use semantic search - same logic as main.rs
        let use_semantic = Self::should_use_semantic_search(query);

        // Initialize embedder if needed - same as main.rs
        let embedder = if use_semantic {
            Self::create_embedder(true).await.ok()
        } else {
            None
        };

        // Create search engine - same as main.rs
        let search_engine = SearchEngine::new(database, embedder);

        // Create default search options
        let options = SearchOptions {
            min_score: 0.3,
            max_results: 10,
            case_sensitive: false,
            typo_tolerance: false,
            fuzzy_matching: false,
            regex_mode: false,
            ..Default::default()
        };

        // Perform search - same as main.rs
        search_engine.search(query, ".", options).await
    }

    /// Get database path - copied from main.rs
    fn get_database_path() -> Result<PathBuf> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let db_dir = home_dir.join(".semisearch");
        std::fs::create_dir_all(&db_dir)?;
        Ok(db_dir.join("search.db"))
    }

    /// Determine if we should use semantic search - copied from main.rs
    fn should_use_semantic_search(query: &str) -> bool {
        let conceptual_indicators = [
            "error handling",
            "authentication",
            "database",
            "security",
            "performance",
            "optimization",
            "algorithm",
            "pattern",
            "architecture",
            "design",
            "implementation",
            "solution",
        ];

        let query_lower = query.to_lowercase();
        conceptual_indicators
            .iter()
            .any(|&indicator| query_lower.contains(indicator))
            || query.split_whitespace().count() > 2
    }

    /// Create embedder - copied from main.rs
    async fn create_embedder(semantic_requested: bool) -> Result<LocalEmbedder> {
        let config = EmbeddingConfig::default();

        if semantic_requested {
            LocalEmbedder::new(config).await
        } else {
            LocalEmbedder::new_tfidf_only(config).await
        }
    }

    /// Display search results in a user-friendly format
    fn display_search_results<W: Write>(
        output: &mut W,
        query: &str,
        results: &[SearchResult],
    ) -> Result<()> {
        if results.is_empty() {
            writeln!(output, "No matches found for '{query}'.")?;
            writeln!(output)?;
            writeln!(output, "Try:")?;
            writeln!(output, "  • Check spelling: semisearch \"{query}\" --fuzzy")?;
            writeln!(
                output,
                "  • Use simpler terms: semisearch \"{}\"",
                Self::simplify_query(query)
            )?;
            writeln!(output, "  • Search everywhere: semisearch \"{query}\" .")?;
            writeln!(output, "  • Try different keywords or phrases")?;
        } else if results.len() == 1 {
            writeln!(output, "✅ Found 1 match:")?;
            writeln!(output)?;
            Self::display_single_result(output, &results[0])?;
        } else if results.len() <= 5 {
            writeln!(output, "✅ Found {} matches:", results.len())?;
            writeln!(output)?;
            for result in results {
                Self::display_single_result(output, result)?;
                writeln!(output)?;
            }
        } else if results.len() > 20 {
            writeln!(output, "📊 Found {} results - that's a lot!", results.len())?;
            writeln!(output, "\n💡 To narrow down results:")?;
            writeln!(output, "  • Use more specific terms")?;
            writeln!(
                output,
                "  • Search in a specific folder: semisearch \"{query}\" src/"
            )?;
            writeln!(
                output,
                "  • Use exact phrases: semisearch \"{query}\" --exact"
            )?;
        } else {
            writeln!(output, "✅ Found {} good results!", results.len())?;
            writeln!(output)?;
            for result in results.iter().take(5) {
                Self::display_single_result(output, result)?;
                writeln!(output)?;
            }
            if results.len() > 5 {
                writeln!(output, "... and {} more matches", results.len() - 5)?;
            }
        }

        Ok(())
    }

    /// Display a single search result
    fn display_single_result<W: Write>(output: &mut W, result: &SearchResult) -> Result<()> {
        writeln!(output, "📁 {}", result.file_path)?;
        writeln!(
            output,
            "   Line {}: {}",
            result.line_number,
            result.content.trim()
        )?;

        if let Some(score) = result.score {
            if score < 1.0 {
                writeln!(output, "   Relevance: {:.1}%", score * 100.0)?;
            }
        }

        Ok(())
    }

    /// Simplify query for suggestions - copied from user_errors.rs
    fn simplify_query(query: &str) -> String {
        use crate::errors::user_errors::UserError;
        UserError::simplify_query(query)
    }

    /// Show interactive help commands
    fn show_interactive_help<W: Write>(output: &mut W) -> Result<()> {
        writeln!(output, "\n📚 Interactive Help Commands:")?;
        writeln!(output, "  help or ?    - Show this help")?;
        writeln!(output, "  examples     - Show common search examples")?;
        writeln!(output, "  tips         - Show search tips")?;
        writeln!(output, "  quit or exit - Exit interactive help")?;
        writeln!(
            output,
            "\n💡 Just type what you want to search for, and I'll show you how!"
        )?;
        writeln!(output)?;
        Ok(())
    }

    /// Show common search examples
    fn show_examples<W: Write>(output: &mut W) -> Result<()> {
        writeln!(output, "\n🎯 Common Search Examples:")?;
        writeln!(output)?;

        writeln!(output, "📝 For Developers:")?;
        writeln!(
            output,
            "  semisearch \"TODO\"              # Find TODO comments"
        )?;
        writeln!(
            output,
            "  semisearch \"function login\"     # Find login functions"
        )?;
        writeln!(
            output,
            "  semisearch \"async function\"     # Find async functions"
        )?;
        writeln!(
            output,
            "  semisearch \"import React\"       # Find React imports"
        )?;
        writeln!(
            output,
            "  semisearch \"try catch\"          # Find error handling"
        )?;
        writeln!(output)?;

        writeln!(output, "✍️  For Writers:")?;
        writeln!(
            output,
            "  semisearch \"needs revision\"     # Find draft sections"
        )?;
        writeln!(
            output,
            "  semisearch \"citation needed\"    # Find unsourced claims"
        )?;
        writeln!(
            output,
            "  semisearch \"methodology\"        # Find research methods"
        )?;
        writeln!(output)?;

        writeln!(output, "🔧 For Configuration:")?;
        writeln!(
            output,
            "  semisearch \"password\"           # Find password-related items"
        )?;
        writeln!(
            output,
            "  semisearch \"config\"             # Find configuration files"
        )?;
        writeln!(
            output,
            "  semisearch \"port 8080\"          # Find port configurations"
        )?;
        writeln!(output)?;

        Ok(())
    }

    /// Show search tips
    fn show_tips<W: Write>(output: &mut W) -> Result<()> {
        writeln!(output, "\n💡 Search Tips:")?;
        writeln!(output)?;

        writeln!(output, "🎯 Getting Better Results:")?;
        writeln!(
            output,
            "  • Use specific terms: 'login function' vs 'function'"
        )?;
        writeln!(
            output,
            "  • Try different words: 'error', 'exception', 'fail'"
        )?;
        writeln!(
            output,
            "  • Use quotes for exact phrases: '\"exact phrase\"'"
        )?;
        writeln!(output)?;

        writeln!(output, "🔍 Search Flags:")?;
        writeln!(output, "  • --fuzzy     Handle typos and similar words")?;
        writeln!(output, "  • --exact     Find exact matches only")?;
        writeln!(output, "  • --limit 20  Show more results (default: 10)")?;
        writeln!(output)?;

        writeln!(output, "📂 Search Locations:")?;
        writeln!(
            output,
            "  • semisearch \"query\" .          # Current directory"
        )?;
        writeln!(
            output,
            "  • semisearch \"query\" src/       # Specific folder"
        )?;
        writeln!(
            output,
            "  • semisearch \"query\" file.txt   # Specific file"
        )?;
        writeln!(output)?;

        writeln!(output, "🚀 Pro Tips:")?;
        writeln!(output, "  • Start simple, then get specific")?;
        writeln!(
            output,
            "  • Use 'semisearch status' to check if everything works"
        )?;
        writeln!(
            output,
            "  • Use 'semisearch --advanced --help' for power user options"
        )?;
        writeln!(output)?;

        Ok(())
    }

    /// Generate a guided search tutorial
    pub async fn run_guided_tutorial() -> Result<()> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut input = stdin.lock();
        let mut output = stdout.lock();

        writeln!(output, "🎓 SemiSearch Guided Tutorial")?;
        writeln!(output, "Let's learn by doing! Press Enter to continue...")?;

        let mut buffer = String::new();
        input.read_line(&mut buffer)?;

        // Step 1: Basic search
        writeln!(output, "\n📚 Step 1: Basic Search")?;
        writeln!(
            output,
            "Let's start with a simple search. Try searching for 'TODO':"
        )?;
        writeln!(output, "Command: semisearch \"TODO\"")?;
        writeln!(output, "\nThis will find all TODO comments in your code.")?;
        writeln!(output, "Press Enter to continue...")?;
        buffer.clear();
        input.read_line(&mut buffer)?;

        // Step 2: Fuzzy search
        writeln!(output, "\n🔍 Step 2: Fuzzy Search")?;
        writeln!(output, "What if you make a typo? Try this:")?;
        writeln!(output, "Command: semisearch \"databse\" --fuzzy")?;
        writeln!(
            output,
            "\nThe --fuzzy flag will find 'database' even with the typo!"
        )?;
        writeln!(output, "Press Enter to continue...")?;
        buffer.clear();
        input.read_line(&mut buffer)?;

        // Step 3: Specific locations
        writeln!(output, "\n📂 Step 3: Search Specific Locations")?;
        writeln!(output, "You can search in specific folders:")?;
        writeln!(output, "Command: semisearch \"function\" src/")?;
        writeln!(
            output,
            "\nThis searches for 'function' only in the src/ directory."
        )?;
        writeln!(output, "Press Enter to continue...")?;
        buffer.clear();
        input.read_line(&mut buffer)?;

        // Step 4: Getting help
        writeln!(output, "\n❓ Step 4: Getting Help")?;
        writeln!(output, "When you need help:")?;
        writeln!(output, "• semisearch help-me     # Interactive help")?;
        writeln!(
            output,
            "• semisearch status      # Check if everything works"
        )?;
        writeln!(output, "• semisearch --help      # Command reference")?;
        writeln!(output)?;

        writeln!(output, "🎉 Tutorial Complete!")?;
        writeln!(
            output,
            "You're ready to search! Try: semisearch \"your search term\""
        )?;

        Ok(())
    }

    /// Show contextual help based on recent search results
    pub fn show_contextual_help<W: Write>(
        output: &mut W,
        query: &str,
        results: &[SearchResult],
    ) -> Result<()> {
        if results.is_empty() {
            writeln!(output, "\n🤔 No results found for '{query}'")?;
            writeln!(output, "\n💡 Try these alternatives:")?;

            let suggestions = ContextualHelp::generate_error_help(query, "no_matches");
            for suggestion in suggestions {
                writeln!(output, "  • {suggestion}")?;
            }

            let examples = ContextualHelp::generate_usage_examples(query);
            if !examples.is_empty() {
                writeln!(output, "\n🎯 Related examples:")?;
                for example in examples.iter().take(3) {
                    writeln!(output, "  • {example}")?;
                }
            }
        } else if results.len() > 20 {
            writeln!(
                output,
                "\n📊 Found {} results - that's a lot!",
                results.len()
            )?;
            writeln!(output, "\n💡 To narrow down results:")?;
            writeln!(output, "  • Use more specific terms")?;
            writeln!(
                output,
                "  • Search in a specific folder: semisearch \"{query}\" src/"
            )?;
            writeln!(
                output,
                "  • Use exact phrases: semisearch \"{query}\" --exact"
            )?;
        } else {
            writeln!(output, "\n✅ Found {} good results!", results.len())?;

            let tips = ContextualHelp::generate_tips(query, results);
            if !tips.is_empty() {
                writeln!(output, "\n💡 Tips:")?;
                for tip in tips.iter().take(2) {
                    writeln!(output, "  {tip}")?;
                }
            }
        }

        writeln!(output)?;
        Ok(())
    }
}
