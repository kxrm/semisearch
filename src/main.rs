use anyhow::Result;

use search::core::embedder::{EmbeddingCapability, EmbeddingConfig, LocalEmbedder};
use search::core::{FileIndexerBuilder, IndexerConfig};
use search::errors::ErrorTranslator;
// Removed unused import
use search::storage::database::Database;
use search::user::feature_discovery::FeatureDiscovery;
use search::user::usage_tracker::UsageTracker;
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
                context_lines: args.context,
                // Only respect --mode in advanced mode, otherwise use auto
                search_mode: if cli.advanced {
                    Some(args.mode.clone())
                } else {
                    Some("auto".to_string())
                },
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
            let results =
                match execute_search(&args.query, &search_path, &options, cli.advanced).await {
                    Ok(results) => results,
                    Err(e) => {
                        handle_error_with_context(e, Some(&args.query), Some(&search_path)).await;
                        return Ok(()); // This line won't be reached due to process::exit in handle_error_with_context
                    }
                };

            let search_time = start_time.elapsed();

            // Track usage for progressive feature discovery (ignore errors)
            if track_search_usage(&args.query, args.fuzzy, cli.advanced, results.len())
                .await
                .is_err()
            {
                // Silently ignore tracking errors - don't break user experience
            }

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
                // Use unified display with mode-specific formatting
                display_unified_results(&results, &args.query, search_time, cli.advanced)?;
            }
        }
        Commands::HelpMe => {
            handle_help_me().await?;
        }
        Commands::Status => {
            handle_simple_status().await?;
        }
        Commands::Index(args) => {
            handle_index(
                &args.path,
                args.force,
                args.semantic,
                args.no_semantic,
                cli.advanced,
            )
            .await?;
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

/// Unified display function for all search result types
fn display_unified_results(
    results: &[SearchResult],
    query: &str,
    search_time: std::time::Duration,
    advanced_mode: bool,
) -> Result<()> {
    if advanced_mode {
        display_advanced_results(results, query, search_time)
    } else {
        display_simple_results(results, query, search_time)
    }
}

/// Display search results with advanced technical details
fn display_advanced_results(
    results: &[SearchResult],
    query: &str,
    search_time: std::time::Duration,
) -> Result<()> {
    use search::errors::provide_contextual_suggestions;
    use search::output::HumanFormatter;

    if results.is_empty() {
        // Handle no results case
        let mut progressive_tip_shown = false;
        if let Ok(usage_file) = UsageTracker::default_usage_file() {
            if let Ok(tracker) = UsageTracker::load(usage_file) {
                let stats = tracker.get_stats();

                if FeatureDiscovery::should_show_tip_for_query(stats, query) {
                    if let Some(tip) = FeatureDiscovery::suggest_next_step(stats, query, 0) {
                        println!("{tip}");
                        println!();
                        progressive_tip_shown = true;
                    }
                }
            }
        }

        if !progressive_tip_shown {
            // Create no matches error for advanced mode
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
        return Ok(());
    }

    // Check for contextual suggestions for large result sets
    if let Some(suggestion) = provide_contextual_suggestions(query, results.len(), "general") {
        if results.len() > 50 {
            // Show results but also provide suggestions for narrowing
            let formatted_output = HumanFormatter::format_results(results, query, search_time);
            print!("{formatted_output}");

            // Show progressive feature discovery tips even for many results
            let mut progressive_tip_shown = false;
            if let Ok(usage_file) = UsageTracker::default_usage_file() {
                if let Ok(tracker) = UsageTracker::load(usage_file) {
                    let stats = tracker.get_stats();

                    if FeatureDiscovery::should_show_tip_for_query(stats, query) {
                        if let Some(tip) =
                            FeatureDiscovery::suggest_next_step(stats, query, results.len())
                        {
                            println!();
                            println!("{tip}");
                            progressive_tip_shown = true;
                        }
                    }
                }
            }

            // Show contextual suggestions only if no progressive tip was shown
            if !progressive_tip_shown {
                println!("\n{}", suggestion.display());
            }

            return Ok(());
        }
    }

    // Use unified formatting with mode-specific details
    let formatted_output = HumanFormatter::format_results_advanced(results, query, search_time);
    print!("{formatted_output}");

    // Show progressive feature discovery tips (prioritized over contextual help)
    let mut progressive_tip_shown = false;
    if let Ok(usage_file) = UsageTracker::default_usage_file() {
        if let Ok(tracker) = UsageTracker::load(usage_file) {
            let stats = tracker.get_stats();

            // Only show tips if appropriate for user's experience level
            if FeatureDiscovery::should_show_tip_for_query(stats, query) {
                if let Some(tip) = FeatureDiscovery::suggest_next_step(stats, query, results.len())
                {
                    println!();
                    println!("{tip}");
                    progressive_tip_shown = true;
                }
            }
        }
    }

    // Show contextual help based on results (only if no progressive tip was shown)
    if !progressive_tip_shown {
        use search::help::contextual::ContextualHelp;
        let tips = ContextualHelp::generate_tips(query, results);
        if !tips.is_empty() {
            println!();
            let tip_count = 2;
            for tip in tips.iter().take(tip_count) {
                println!("{tip}");
            }
        }
    }

    Ok(())
}

/// Track search usage for progressive feature discovery
async fn track_search_usage(
    query: &str,
    fuzzy_used: bool,
    advanced_used: bool,
    result_count: usize,
) -> Result<()> {
    // Load or create usage tracker
    let usage_file = UsageTracker::default_usage_file()?;
    let mut tracker = UsageTracker::load(usage_file)?;

    // Record this search
    tracker.record_search(query, fuzzy_used, advanced_used, result_count);

    // Save usage data
    tracker.save()?;

    Ok(())
}

/// Execute search with the given parameters
async fn execute_search(
    query: &str,
    path: &str,
    options: &SearchOptions,
    advanced_mode: bool,
) -> Result<Vec<SearchResult>> {
    use search::search::auto_strategy::AutoStrategy;

    // Create AutoStrategy with advanced mode setting
    // It will initialize semantic search on-demand if needed
    let mut auto_strategy = AutoStrategy::with_advanced_mode(advanced_mode);

    // Only pass options if in advanced mode AND they contain filtering patterns
    let options_to_pass = if advanced_mode
        && (!options.include_patterns.is_empty() || !options.exclude_patterns.is_empty())
    {
        Some(options) // Advanced mode with filtering patterns
    } else {
        None // Basic mode or no filtering patterns
    };

    // Check if a specific mode was requested
    if let Some(mode) = &options.search_mode {
        if mode != "auto" {
            // Use forced mode
            return auto_strategy
                .search_with_mode(query, path, mode, options_to_pass)
                .await;
        }
    }

    // Use automatic strategy selection
    auto_strategy.search(query, path, options_to_pass).await
}

/// Display search results in a user-friendly format
fn display_simple_results(
    results: &[SearchResult],
    query: &str,
    search_time: std::time::Duration,
) -> Result<()> {
    use search::errors::{provide_contextual_suggestions, UserFriendlyError};
    use search::output::HumanFormatter;

    // Check for contextual suggestions based on results
    if let Some(suggestion) = provide_contextual_suggestions(query, results.len(), "general") {
        // Handle no results or too many results with user-friendly messages
        if results.is_empty() {
            // Show progressive feature discovery tips for no results before exiting
            if let Ok(usage_file) = UsageTracker::default_usage_file() {
                if let Ok(tracker) = UsageTracker::load(usage_file) {
                    let stats = tracker.get_stats();

                    if FeatureDiscovery::should_show_tip_for_query(stats, query) {
                        if let Some(tip) = FeatureDiscovery::suggest_next_step(stats, query, 0) {
                            println!("{tip}");
                            println!();
                        }
                    }
                }
            }

            // Show contextual suggestions regardless of progressive tip status
            eprintln!("{}", suggestion.display());
            std::process::exit(1);
        } else if results.len() > 50 {
            // Show results but also provide suggestions for narrowing
            let formatted_output = HumanFormatter::format_results(results, query, search_time);
            print!("{formatted_output}");

            // Show progressive feature discovery tips even for many results
            let mut progressive_tip_shown = false;
            if let Ok(usage_file) = UsageTracker::default_usage_file() {
                if let Ok(tracker) = UsageTracker::load(usage_file) {
                    let stats = tracker.get_stats();

                    if FeatureDiscovery::should_show_tip_for_query(stats, query) {
                        if let Some(tip) =
                            FeatureDiscovery::suggest_next_step(stats, query, results.len())
                        {
                            println!();
                            println!("{tip}");
                            progressive_tip_shown = true;
                        }
                    }
                }
            }

            // Show contextual suggestions only if no progressive tip was shown
            if !progressive_tip_shown {
                println!("\n{}", suggestion.display());
            }

            return Ok(());
        }
    }

    if results.is_empty() {
        // Show progressive feature discovery tips for no results before exiting
        let mut progressive_tip_shown = false;
        if let Ok(usage_file) = UsageTracker::default_usage_file() {
            if let Ok(tracker) = UsageTracker::load(usage_file) {
                let stats = tracker.get_stats();

                if FeatureDiscovery::should_show_tip_for_query(stats, query) {
                    if let Some(tip) = FeatureDiscovery::suggest_next_step(stats, query, 0) {
                        println!("{tip}");
                        println!();
                        progressive_tip_shown = true;
                    }
                }
            }
        }

        // Fallback if no contextual suggestions were provided
        if !progressive_tip_shown {
            let error = UserFriendlyError::no_matches(query, ".");
            eprintln!("{}", error.display());
        }
        std::process::exit(1);
    }

    // Use human-friendly formatting
    let formatted_output = HumanFormatter::format_results(results, query, search_time);
    print!("{formatted_output}");

    // Show progressive feature discovery tips (prioritized over contextual help)
    let mut progressive_tip_shown = false;
    if let Ok(usage_file) = UsageTracker::default_usage_file() {
        if let Ok(tracker) = UsageTracker::load(usage_file) {
            let stats = tracker.get_stats();

            // Only show tips if appropriate for user's experience level
            if FeatureDiscovery::should_show_tip_for_query(stats, query) {
                if let Some(tip) = FeatureDiscovery::suggest_next_step(stats, query, results.len())
                {
                    println!();
                    println!("{tip}");
                    progressive_tip_shown = true;
                }
            }
        }
    }

    // Show contextual help based on results (only if no progressive tip was shown)
    if !progressive_tip_shown {
        use search::help::contextual::ContextualHelp;
        let tips = ContextualHelp::generate_tips(query, results);
        if !tips.is_empty() {
            println!();
            for tip in tips.iter().take(1) {
                println!("{tip}");
            }
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
    use search::context::{ContextAwareConfig, ProjectDetector};

    println!("🏥 SemiSearch Health Check");
    println!();

    // Check basic functionality
    println!("✅ Basic search: Ready");

    // Show project context (UX Remediation Plan Task 2.1)
    let current_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let project_type = ProjectDetector::detect(&current_path);
    let _config = ContextAwareConfig::from_project_type(project_type.clone());

    match project_type {
        search::context::ProjectType::RustProject => {
            println!("📦 Project type: Rust project");
            println!("  • Focused on: src/, tests/ directories");
            println!("  • File types: *.rs files");
        }
        search::context::ProjectType::JavaScriptProject => {
            println!("📦 Project type: JavaScript/TypeScript project");
            println!("  • Focused on: src/, lib/ directories");
            println!("  • File types: *.js, *.ts files");
        }
        search::context::ProjectType::PythonProject => {
            println!("📦 Project type: Python project");
            println!("  • Focused on: src/, lib/, tests/ directories");
            println!("  • File types: *.py files");
        }
        search::context::ProjectType::Documentation => {
            println!("📦 Project type: Documentation project");
            println!("  • Focused on: all directories");
            println!("  • File types: *.md, *.txt files");
        }
        search::context::ProjectType::Mixed => {
            println!("📦 Project type: Mixed project");
            println!("  • Focused on: all directories");
            println!("  • File types: all files");
        }
        search::context::ProjectType::Unknown => {
            println!("📦 Project type: General");
            println!("  • Focused on: all directories");
            println!("  • File types: all files");
        }
    }

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

    // Provide contextual tips based on project type
    match project_type {
        search::context::ProjectType::RustProject => {
            println!("  • Find TODO comments: semisearch \"TODO\"");
            println!("  • Find functions: semisearch \"fn main\"");
            println!("  • Search tests: semisearch \"#[test]\"");
        }
        search::context::ProjectType::JavaScriptProject => {
            println!("  • Find TODO comments: semisearch \"TODO\"");
            println!("  • Find functions: semisearch \"function\"");
            println!("  • Find imports: semisearch \"import\"");
        }
        search::context::ProjectType::PythonProject => {
            println!("  • Find TODO comments: semisearch \"TODO\"");
            println!("  • Find functions: semisearch \"def \"");
            println!("  • Find classes: semisearch \"class \"");
        }
        search::context::ProjectType::Documentation => {
            println!("  • Find sections: semisearch \"# Introduction\"");
            println!("  • Find todos: semisearch \"TODO\"");
            println!("  • Find examples: semisearch \"example\"");
        }
        _ => {
            println!("  • Everything looks good? Try: semisearch \"TODO\"");
            println!("  • Find files: semisearch \"config\"");
            println!("  • Search content: semisearch \"error\"");
        }
    }

    println!("  • Need help? Try: semisearch help-me");
    println!("  • Advanced diagnostics: semisearch doctor");

    Ok(())
}

/// Handle indexing with simple interface
async fn handle_index(
    path: &str,
    force: bool,
    semantic: bool,
    no_semantic: bool,
    advanced_mode: bool,
) -> Result<()> {
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
            Ok(embedder) => FileIndexerBuilder::new()
                .with_database(database)
                .with_config(config)
                .with_embedder(embedder)
                .with_advanced_mode(advanced_mode)
                .build()?,
            Err(e) => {
                println!("⚠️  Semantic indexing failed: {e}");
                println!("🔄 Falling back to keyword-only indexing");
                FileIndexerBuilder::new()
                    .with_database(database)
                    .with_config(config)
                    .with_advanced_mode(advanced_mode)
                    .build()?
            }
        }
    } else {
        println!("📝 Keyword-only indexing");
        FileIndexerBuilder::new()
            .with_database(database)
            .with_config(config)
            .with_advanced_mode(advanced_mode)
            .build()?
    };

    // Index the directory
    let path_buf = PathBuf::from(path);

    // Handle force reindex by clearing existing data if needed
    if force {
        println!("🗑️  Clearing existing index data...");
        // TODO: Add database method to clear files in path
    }

    match indexer.index_directory_with_force(&path_buf, force) {
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

    match execute_search(test_query, test_path, &test_options, false).await {
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
async fn create_embedder_with_mode(
    semantic_requested: bool,
    advanced_mode: bool,
) -> Result<LocalEmbedder> {
    let config = EmbeddingConfig::default();

    if semantic_requested {
        LocalEmbedder::new_with_mode(config, advanced_mode).await
    } else {
        LocalEmbedder::new_tfidf_only(config).await
    }
}

async fn create_embedder(semantic_requested: bool) -> Result<LocalEmbedder> {
    create_embedder_with_mode(semantic_requested, false).await
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
async fn handle_error_with_context(error: anyhow::Error, query: Option<&str>, _path: Option<&str>) {
    use search::errors::translate_error;

    let user_friendly_error = translate_error(&error);

    // Display the user-friendly error message
    eprintln!("{}", user_friendly_error.display());

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

    std::process::exit(1);
}
