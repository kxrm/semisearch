use crate::help::contextual::ContextualHelp;
use crate::SearchResult;
use anyhow::Result;
use std::io::{BufRead, Write};

/// Interactive help system that guides users through search scenarios
pub struct InteractiveHelp;

impl InteractiveHelp {
    /// Run interactive help with standard input/output
    pub async fn run() -> Result<()> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut input = stdin.lock();
        let mut output = stdout.lock();

        Self::run_with_io(&mut input, &mut output).await
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

                    // Process search query
                    writeln!(output, "Searching for: {user_input}")?;

                    // Generate contextual examples based on the query
                    let examples = ContextualHelp::generate_usage_examples(user_input);
                    if !examples.is_empty() {
                        writeln!(output, "\n💡 Related examples:")?;
                        for example in examples.iter().take(3) {
                            writeln!(output, "  {example}")?;
                        }
                    }

                    // Show what the actual command would be
                    writeln!(output, "\n🔍 To run this search:")?;
                    writeln!(output, "  semisearch \"{user_input}\"")?;

                    // Show variations
                    writeln!(output, "\n🎯 Variations you can try:")?;
                    writeln!(
                        output,
                        "  semisearch \"{user_input}\" --fuzzy    # Handle typos"
                    )?;
                    writeln!(
                        output,
                        "  semisearch \"{user_input}\" --exact    # Exact matches only"
                    )?;
                    writeln!(
                        output,
                        "  semisearch \"{user_input}\" src/       # Search only in src/"
                    )?;
                    writeln!(output)?;
                }
                Err(_) => break,
            }
        }

        Ok(())
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
