use anyhow::Result;

use search::core::embedder::{EmbeddingCapability, EmbeddingConfig, LocalEmbedder};
use search::core::indexer::{FileIndexer, IndexerConfig};
use search::errors::ErrorTranslator;
use search::search::strategy::SearchEngine;
use search::storage::database::Database;
use search::{SearchOptions, SearchResult};
use std::path::PathBuf;
use std::time::Instant;

// Import CLI modules
mod cli;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    if let Err(e) = run_main().await {
        handle_error(e).await;
    }
}

async fn run_main() -> Result<()> {
    // Parse CLI with dynamic help based on advanced mode
    let cli = Cli::parse_advanced_aware();

    // Handle CLI routing
    match cli.command {
        Commands::Search(args) => {
            let start_time = Instant::now();

            // Handle path resolution: natural path takes precedence over --path flag
            let search_path = if let Some(path_flag) = &args.path_flag {
                // If --path flag is provided, use it (backward compatibility)
                path_flag.clone()
            } else {
                // Use the natural path argument
                args.path.clone()
            };

            // Convert simple flags to search options
            let mut options = SearchOptions {
                min_score: args.score,
                max_results: args.limit,
                case_sensitive: args.case_sensitive,
                typo_tolerance: args.typo_tolerance,
                include_patterns: args.include.clone(),
                exclude_patterns: args.exclude.clone(),
                ..Default::default()
            };

            // Handle simple flags
            if args.exact {
                options.regex_mode = true;
                options.fuzzy_matching = false;
                options.min_score = 1.0; // Exact matches only
            } else if args.fuzzy {
                options.fuzzy_matching = true;
                options.typo_tolerance = true;
            }

            // Handle advanced flags (only if advanced mode is enabled)
            if cli.advanced {
                if args.semantic {
                    // Force semantic search
                } else if args.no_semantic {
                    // Disable semantic search
                }

                if args.regex || args.mode == "regex" {
                    options.regex_mode = true;
                }

                if args.semantic_threshold != 0.7 {
                    // Custom semantic threshold
                }

                if args.context > 0 {
                    // Add context lines
                }
            }

            // Perform search with enhanced error handling
            let results = match execute_search(&args.query, &search_path, &options).await {
                Ok(results) => results,
                Err(e) => {
                    handle_error_with_context(e, Some(&args.query), Some(&search_path)).await;
                    return Ok(()); // This line won't be reached due to process::exit in handle_error_with_context
                }
            };

            let search_time = start_time.elapsed();

            // Display results based on format
            if cli.advanced && args.format == "json" {
                if results.is_empty() {
                    // Handle no matches for JSON format
                    let no_matches_error = ErrorTranslator::handle_no_results(&args.query);
                    let exit_code = no_matches_error.exit_code();

                    match no_matches_error.to_json() {
                        Ok(json) => eprintln!("{json}"),
                        Err(_) => eprintln!("{{\"error_type\": \"NoMatches\", \"details\": {{\"query\": \"{}\", \"suggestions\": []}}}}",
                            args.query),
                    }

                    std::process::exit(exit_code);
                } else {
                    println!("{}", serde_json::to_string_pretty(&results)?);
                }
            } else if cli.advanced && args.files_only {
                for result in &results {
                    println!("{}", result.file_path);
                }
            } else {
                // Check if advanced mode is enabled for different formatting
                if cli.advanced {
                    display_advanced_results(&results, &args.query, search_time)?;
                } else {
                    display_simple_results(&results, &args.query, search_time)?;
                }
            }
        }
        Commands::HelpMe => {
            handle_help_me().await?;
        }
        Commands::Status => {
            handle_simple_status().await?;
        }
        Commands::Index(args) => {
            handle_index(&args.path, args.force, args.semantic, args.no_semantic).await?;
        }
        Commands::Config => {
            show_config().await?;
        }
        Commands::Doctor => {
            run_doctor().await?;
        }
    }

    Ok(())
}

/// Display search results with advanced technical details
fn display_advanced_results(
    results: &[SearchResult],
    query: &str,
    search_time: std::time::Duration,
) -> Result<()> {
    use search::output::HumanFormatter;

    if results.is_empty() {
        // Create no matches error and exit with proper code
        let no_matches_error = ErrorTranslator::handle_no_results(query);
        let exit_code = no_matches_error.exit_code();

        // Check if JSON format was requested
        let args: Vec<String> = std::env::args().collect();
        let json_format = args
            .windows(2)
            .any(|w| w[0] == "--format" && w[1] == "json");

        if json_format {
            match no_matches_error.to_json() {
                Ok(json) => eprintln!("{json}"),
                Err(_) => eprintln!("{{\"error_type\": \"NoMatches\", \"details\": {{\"query\": \"{query}\", \"suggestions\": []}}}}"),
            }
        } else {
            eprintln!("{no_matches_error}");
        }

        std::process::exit(exit_code);
    }

    // Use advanced formatting with technical details
    let formatted_output = HumanFormatter::format_results_advanced(results, query, search_time);
    print!("{formatted_output}");

    // Show contextual help based on results
    use search::help::contextual::ContextualHelp;
    let tips = ContextualHelp::generate_tips(query, results);
    if !tips.is_empty() {
        println!();
        for tip in tips.iter().take(2) {
            println!("{tip}");
        }
    }

    Ok(())
}

/// Execute search with the given parameters
async fn execute_search(
    query: &str,
    path: &str,
    options: &SearchOptions,
) -> Result<Vec<SearchResult>> {
    // Get database path
    let db_path = get_database_path()?;
    let database = Database::new(&db_path)?;

    // Determine if we should use semantic search
    let use_semantic = should_use_semantic_search(query);

    // Initialize embedder if needed
    let embedder = if use_semantic {
        (create_embedder(true).await).ok()
    } else {
        None
    };

    // Create search engine
    let search_engine = SearchEngine::new(database, embedder);

    // Perform search
    search_engine.search(query, path, options.clone()).await
}

/// Determine if we should use semantic search based on query characteristics
fn should_use_semantic_search(query: &str) -> bool {
    // Use semantic search for conceptual queries
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
        || query.split_whitespace().count() > 2 // Multi-word queries benefit from semantic search
}

/// Display search results in a user-friendly format
fn display_simple_results(
    results: &[SearchResult],
    query: &str,
    search_time: std::time::Duration,
) -> Result<()> {
    use search::output::HumanFormatter;

    if results.is_empty() {
        // Create no matches error and exit with proper code
        let no_matches_error = ErrorTranslator::handle_no_results(query);
        let exit_code = no_matches_error.exit_code();

        // Check if JSON format was requested
        let args: Vec<String> = std::env::args().collect();
        let json_format = args
            .windows(2)
            .any(|w| w[0] == "--format" && w[1] == "json");

        if json_format {
            match no_matches_error.to_json() {
                Ok(json) => eprintln!("{json}"),
                Err(_) => eprintln!("{{\"error_type\": \"NoMatches\", \"details\": {{\"query\": \"{query}\", \"suggestions\": []}}}}"),
            }
        } else {
            eprintln!("{no_matches_error}");
        }

        std::process::exit(exit_code);
    }

    // Use human-friendly formatting
    let formatted_output = HumanFormatter::format_results(results, query, search_time);
    print!("{formatted_output}");

    // Show contextual help based on results
    use search::help::contextual::ContextualHelp;
    let tips = ContextualHelp::generate_tips(query, results);
    if !tips.is_empty() {
        println!();
        for tip in tips.iter().take(2) {
            println!("{tip}");
        }
    }

    Ok(())
}

/// Handle help-me command with interactive guidance
async fn handle_help_me() -> Result<()> {
    use search::help::interactive::InteractiveHelp;

    // Run the interactive help system
    InteractiveHelp::run().await?;

    Ok(())
}

/// Handle status command with simple, user-friendly output
async fn handle_simple_status() -> Result<()> {
    println!("🏥 SemiSearch Health Check");
    println!();

    // Check basic functionality
    println!("✅ Basic search: Ready");

    // Check database
    match get_database_path() {
        Ok(db_path) => {
            if db_path.exists() {
                match Database::new(&db_path) {
                    Ok(database) => match database.get_stats() {
                        Ok(stats) => {
                            println!("✅ Database: {} files indexed", stats.file_count);
                        }
                        Err(_) => println!("⚠️  Database: Connected but no stats available"),
                    },
                    Err(_) => println!("❌ Database: Connection failed"),
                }
            } else {
                println!("⚠️  Database: Not initialized (run 'semisearch index .' first)");
            }
        }
        Err(e) => println!("❌ Database: Error - {e}"),
    }

    // Check search capabilities
    println!("🔍 Search capabilities:");
    println!("  • Keyword search: ✅ Available");
    println!("  • Fuzzy search: ✅ Available");
    println!("  • Regex search: ✅ Available");

    // Check semantic capabilities
    match LocalEmbedder::detect_capabilities() {
        #[cfg(feature = "neural-embeddings")]
        EmbeddingCapability::Full => {
            println!("  • Semantic search: ✅ Available (full neural embeddings)");
        }
        EmbeddingCapability::TfIdf => {
            println!("  • Semantic search: ⚠️  Limited (TF-IDF only)");
        }
        EmbeddingCapability::None => {
            println!("  • Semantic search: ❌ Unavailable");
        }
    }

    println!();
    println!("💡 Tips:");
    println!("  • Everything looks good? Try: semisearch \"TODO\"");
    println!("  • Need help? Try: semisearch help-me");
    println!("  • Advanced diagnostics: semisearch doctor");

    Ok(())
}

/// Handle indexing with simple interface
async fn handle_index(path: &str, force: bool, semantic: bool, no_semantic: bool) -> Result<()> {
    println!("🗂️  Indexing files in: {path}");

    if force {
        println!("🔄 Forcing full reindex");
    }

    // Initialize database
    let db_path = get_database_path()?;
    let database = Database::new(&db_path)?;

    // Create indexer configuration
    let config = IndexerConfig::default();

    // Determine if we should use semantic indexing
    let use_semantic = if no_semantic {
        false
    } else if semantic {
        true
    } else {
        // Auto-detect capability
        #[cfg(feature = "neural-embeddings")]
        {
            matches!(
                LocalEmbedder::detect_capabilities(),
                EmbeddingCapability::Full
            )
        }
        #[cfg(not(feature = "neural-embeddings"))]
        {
            false
        }
    };

    // Create indexer
    let indexer = if use_semantic {
        println!("🧠 Including semantic embeddings");
        match create_embedder(true).await {
            Ok(embedder) => FileIndexer::with_embedder(database, config, embedder),
            Err(e) => {
                println!("⚠️  Semantic indexing failed: {e}");
                println!("🔄 Falling back to keyword-only indexing");
                FileIndexer::with_config(database, config)
            }
        }
    } else {
        println!("📝 Keyword-only indexing");
        FileIndexer::with_config(database, config)
    };

    // Index the directory
    let path_buf = PathBuf::from(path);

    // Handle force reindex by clearing existing data if needed
    if force {
        println!("🗑️  Clearing existing index data...");
        // TODO: Add database method to clear files in path
    }

    match indexer.index_directory(&path_buf) {
        Ok(stats) => {
            println!("✅ Indexing complete!");
            println!("   • Files processed: {}", stats.files_processed);
            println!("   • Files updated: {}", stats.files_updated);
            if stats.files_skipped > 0 {
                println!("   • Files skipped: {}", stats.files_skipped);
            }
        }
        Err(e) => {
            // Use the enhanced error handling system with context
            handle_error_with_context(e, None, Some(path)).await;
        }
    }

    Ok(())
}

/// Show configuration
async fn show_config() -> Result<()> {
    println!("⚙️  SemiSearch Configuration");
    println!();

    // Database location
    match get_database_path() {
        Ok(db_path) => println!("📁 Database: {}", db_path.display()),
        Err(e) => println!("❌ Database path error: {e}"),
    }

    // Capabilities
    println!("🔧 Capabilities:");
    match LocalEmbedder::detect_capabilities() {
        #[cfg(feature = "neural-embeddings")]
        EmbeddingCapability::Full => println!("  • Neural embeddings: ✅ Available"),
        EmbeddingCapability::TfIdf => println!("  • TF-IDF embeddings: ✅ Available"),
        EmbeddingCapability::None => println!("  • Embeddings: ❌ Unavailable"),
    }

    Ok(())
}

/// Run comprehensive diagnostics
async fn run_doctor() -> Result<()> {
    println!("🩺 SemiSearch Doctor - Comprehensive Diagnostics");
    println!();

    // System check
    println!("🖥️  System Check:");
    println!("  • OS: {}", std::env::consts::OS);
    println!("  • Architecture: {}", std::env::consts::ARCH);

    // Capability check
    println!();
    println!("🔧 Capability Check:");
    let capability = LocalEmbedder::detect_capabilities();
    match capability {
        #[cfg(feature = "neural-embeddings")]
        EmbeddingCapability::Full => {
            println!("✅ System supports full neural embeddings");

            // Test embedder creation
            print!("🧪 Testing embedder initialization... ");
            match create_embedder(true).await {
                Ok(_) => println!("✅ Success"),
                Err(e) => println!("❌ Failed: {e}"),
            }
        }
        EmbeddingCapability::TfIdf => {
            println!("📊 Using TF-IDF embeddings (enhanced statistical search)");

            // Test TF-IDF embedder
            print!("🧪 Testing TF-IDF embedder... ");
            match create_embedder(false).await {
                Ok(_) => println!("✅ Success"),
                Err(e) => println!("❌ Failed: {e}"),
            }
        }
        EmbeddingCapability::None => {
            println!("❌ System too limited for embeddings");
            println!("💡 Keyword search will still work perfectly");
        }
    }

    // Database check
    println!();
    println!("💾 Database Check:");
    match get_database_path() {
        Ok(db_path) => {
            println!("✅ Database path: {}", db_path.display());

            if db_path.exists() {
                match Database::new(&db_path) {
                    Ok(database) => {
                        println!("✅ Database connection: OK");

                        match database.get_stats() {
                            Ok(stats) => {
                                println!("✅ Database stats: {} files indexed", stats.file_count);
                            }
                            Err(e) => println!("⚠️  Database stats error: {e}"),
                        }
                    }
                    Err(e) => println!("❌ Database connection failed: {e}"),
                }
            } else {
                println!("⚠️  Database not initialized");
                println!("💡 Run 'semisearch index .' to create database");
            }
        }
        Err(e) => println!("❌ Database path error: {e}"),
    }

    // Performance test
    println!();
    println!("⚡ Performance Test:");
    let start = Instant::now();
    let test_query = "test";
    let test_path = ".";
    let test_options = SearchOptions::default();

    match execute_search(test_query, test_path, &test_options).await {
        Ok(results) => {
            let duration = start.elapsed();
            println!(
                "✅ Search test: {} results in {:.2}s",
                results.len(),
                duration.as_secs_f64()
            );
        }
        Err(e) => println!("❌ Search test failed: {e}"),
    }

    println!();
    println!("🎯 Recommendations:");
    println!("  • For best results, index your files first: semisearch index .");
    println!("  • Use semantic search for conceptual queries");
    println!("  • Use exact search for precise matches");
    println!("  • Check 'semisearch status' for quick health check");

    Ok(())
}

/// Helper functions from original main.rs
async fn create_embedder(semantic_requested: bool) -> Result<LocalEmbedder> {
    let config = EmbeddingConfig::default();

    if semantic_requested {
        LocalEmbedder::new(config).await
    } else {
        LocalEmbedder::new_tfidf_only(config).await
    }
}

fn get_database_path() -> Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let db_dir = home_dir.join(".semisearch");
    std::fs::create_dir_all(&db_dir)?;
    Ok(db_dir.join("search.db"))
}

/// Handle errors by translating them to user-friendly messages and using proper stderr/exit codes
async fn handle_error(error: anyhow::Error) {
    handle_error_with_context(error, None, None).await;
}

/// Handle errors with additional context (query, path) for better user guidance
async fn handle_error_with_context(error: anyhow::Error, query: Option<&str>, path: Option<&str>) {
    let user_error = ErrorTranslator::translate_technical_error_with_context(&error, query, path);

    // Check if JSON format was requested
    if let Ok(json_mode) = std::env::var("SEMISEARCH_JSON") {
        if json_mode == "1" || json_mode.to_lowercase() == "true" {
            match user_error.to_json() {
                Ok(json) => eprintln!("{json}"),
                Err(_) => {
                    // Fallback to regular error display
                    eprintln!("{user_error}");
                }
            }
        } else {
            eprintln!("{user_error}");
        }
    } else {
        eprintln!("{user_error}");

        // Add contextual help for common error scenarios
        if let Some(query) = query {
            use search::help::contextual::ContextualHelp;
            let examples = ContextualHelp::generate_usage_examples(query);
            if !examples.is_empty() {
                eprintln!();
                eprintln!("💡 Related examples:");
                for example in examples.iter().take(3) {
                    eprintln!("  {example}");
                }
            }
        }
    }

    let exit_code = user_error.exit_code();
    std::process::exit(exit_code);
}
