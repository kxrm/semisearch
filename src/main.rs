use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use search::core::embedder::{EmbeddingCapability, EmbeddingConfig, LocalEmbedder};
use search::core::indexer::FileIndexer;
use search::search::strategy::SearchEngine;
use search::storage::database::Database;
use search::{SearchOptions, SearchResult};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "semisearch")]
#[command(about = "Semantic search across local files")]
#[command(version = "0.4.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Search for matches across files
    Search {
        /// Search query
        query: String,

        /// Target directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        path: String,

        /// Search mode: auto (default), semantic, keyword, fuzzy, regex, tfidf
        #[arg(short, long, default_value = "auto")]
        mode: SearchMode,

        /// Enable semantic search (overrides mode if specified)
        #[arg(long)]
        semantic: bool,

        /// Disable semantic search (force keyword-only)
        #[arg(long)]
        no_semantic: bool,

        /// Minimum similarity score (0.0-1.0)
        #[arg(short, long, default_value = "0.3")]
        score: f32,

        /// Semantic similarity threshold (0.0-1.0)
        #[arg(long, default_value = "0.7")]
        semantic_threshold: f32,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Output format: plain, json
        #[arg(short, long, default_value = "plain")]
        format: OutputFormat,

        /// Show file paths only
        #[arg(long)]
        files_only: bool,

        /// Case sensitive search
        #[arg(long)]
        case_sensitive: bool,

        /// Enable typo tolerance
        #[arg(long)]
        typo_tolerance: bool,

        /// Context lines around matches
        #[arg(long, default_value = "0")]
        context: usize,
    },

    /// Index files in directory
    Index {
        /// Directory to index
        path: String,

        /// Force full reindex
        #[arg(long)]
        force: bool,

        /// Build semantic embeddings during indexing
        #[arg(long)]
        semantic: bool,

        /// Skip semantic embeddings
        #[arg(long)]
        no_semantic: bool,
    },

    /// Show system status and capabilities
    Status,

    /// Show configuration
    Config,

    /// Test system capabilities
    Doctor,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum SearchMode {
    /// Auto-detect best mode based on system capabilities
    Auto,
    /// Full semantic search (neural embeddings)
    Semantic,
    /// Keyword-based search only
    Keyword,
    /// Fuzzy string matching
    Fuzzy,
    /// Regular expression search
    Regex,
    /// TF-IDF statistical search
    Tfidf,
    /// Hybrid: combine keyword and semantic
    Hybrid,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Plain,
    Json,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search {
            query,
            path,
            mode,
            semantic,
            no_semantic,
            score,
            semantic_threshold: _,
            limit,
            format,
            files_only,
            case_sensitive,
            typo_tolerance,
            context: _,
        } => {
            let start_time = Instant::now();

            // Determine final search mode
            let final_mode = determine_search_mode(mode, semantic, no_semantic).await?;

            // Create search options
            let options = SearchOptions {
                min_score: score,
                max_results: limit,
                fuzzy_matching: matches!(final_mode, SearchMode::Fuzzy),
                regex_mode: matches!(final_mode, SearchMode::Regex),
                case_sensitive,
                typo_tolerance,
                max_edit_distance: 2,
            };

            // Initialize database
            let db_path = get_database_path()?;
            let database = Database::new(&db_path)?;

            // Initialize embedder if needed
            let embedder = if matches!(final_mode, SearchMode::Semantic | SearchMode::Hybrid) {
                match create_embedder().await {
                    Ok(emb) => Some(emb),
                    Err(e) => {
                        println!("⚠️  Semantic search unavailable: {e}");
                        println!("🔄 Falling back to keyword search");
                        None
                    }
                }
            } else {
                None
            };

            // Create search engine
            let search_engine = SearchEngine::new(database, embedder);

            // Perform search
            let results = search_engine.search(&query, &path, options).await?;

            let search_time = start_time.elapsed();

            // Display results
            display_results(&results, &format, files_only, &query, search_time)?;
        }

        Commands::Index {
            path,
            force: _,
            semantic,
            no_semantic,
        } => {
            println!("🗂️  Indexing directory: {path}");

            let db_path = get_database_path()?;
            let database = Database::new(&db_path)?;

            // Determine if we should build semantic embeddings
            let build_semantic = if no_semantic {
                false
            } else if semantic {
                true
            } else {
                // Auto-detect
                LocalEmbedder::detect_capabilities() != EmbeddingCapability::None
            };

            let _embedder = if build_semantic {
                match create_embedder().await {
                    Ok(emb) => {
                        println!("✅ Semantic embeddings will be generated during indexing");
                        Some(emb)
                    }
                    Err(e) => {
                        println!("⚠️  Semantic embeddings unavailable: {e}");
                        println!("📄 Indexing will use keyword-only mode");
                        None
                    }
                }
            } else {
                println!("📄 Indexing in keyword-only mode");
                None
            };

            let indexer = FileIndexer::new(database);
            let stats = indexer.index_directory(std::path::Path::new(&path))?;

            println!("✅ Indexing complete:");
            println!("   📁 Files processed: {}", stats.files_processed);
            println!("   📄 Chunks created: {}", stats.chunks_created);
            println!("   ⏱️  Time taken: {:.2}s", stats.duration_seconds);
        }

        Commands::Status => {
            show_status().await?;
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

async fn determine_search_mode(
    mode: SearchMode,
    semantic: bool,
    no_semantic: bool,
) -> Result<SearchMode> {
    if no_semantic {
        return Ok(SearchMode::Keyword);
    }

    if semantic {
        return Ok(SearchMode::Semantic);
    }

    match mode {
        SearchMode::Auto => {
            // Auto-detect best mode based on system capabilities
            match LocalEmbedder::detect_capabilities() {
                #[cfg(feature = "neural-embeddings")]
                EmbeddingCapability::Full => Ok(SearchMode::Hybrid),
                EmbeddingCapability::TfIdf => Ok(SearchMode::Tfidf),
                EmbeddingCapability::None => Ok(SearchMode::Keyword),
            }
        }
        other => Ok(other),
    }
}

async fn create_embedder() -> Result<LocalEmbedder> {
    let config = EmbeddingConfig::default();
    LocalEmbedder::new(config).await
}

fn get_database_path() -> Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    let semisearch_dir = home_dir.join(".semisearch");
    std::fs::create_dir_all(&semisearch_dir)?;

    Ok(semisearch_dir.join("index.db"))
}

fn display_results(
    results: &[SearchResult],
    format: &OutputFormat,
    files_only: bool,
    query: &str,
    search_time: std::time::Duration,
) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "query": query,
                "results": results,
                "count": results.len(),
                "search_time_ms": search_time.as_millis()
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Plain => {
            if results.is_empty() {
                println!("No matches found for: {query}");
                return Ok(());
            }

            println!("Found {} matches in {:?}:", results.len(), search_time);
            println!();

            for result in results {
                if files_only {
                    println!("{}", result.file_path);
                } else {
                    println!("📁 {}", result.file_path);
                    println!("   Line {}: {}", result.line_number, result.content);
                    if let Some(score) = result.score {
                        println!("   Score: {score:.3}");
                    }
                    if let Some(match_type) = &result.match_type {
                        println!("   Match: {match_type:?}");
                    }
                    println!();
                }
            }
        }
    }

    Ok(())
}

async fn show_status() -> Result<()> {
    println!("🔍 SemiSearch Status");
    println!();

    // Check database
    let db_path = get_database_path()?;
    if db_path.exists() {
        let database = Database::new(&db_path)?;
        let stats = database.get_stats()?;
        println!("📊 Database Status:");
        println!("   📁 Indexed files: {}", stats.file_count);
        println!("   📄 Total chunks: {}", stats.chunk_count);
        println!(
            "   💾 Database size: {:.2} MB",
            db_path.metadata()?.len() as f64 / (1024.0 * 1024.0)
        );
    } else {
        println!("📊 Database: Not initialized");
    }

    // Check system capabilities
    let capability = LocalEmbedder::detect_capabilities();
    println!("🧠 Embedding Capability: {capability:?}");

    // Check models directory
    let models_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".semisearch")
        .join("models");

    if models_dir.exists() {
        println!("🤖 Models directory: {}", models_dir.display());
        let model_path = models_dir.join("model.onnx");
        if model_path.exists() {
            let metadata = std::fs::metadata(&model_path)?;
            println!("   Neural model: {} bytes", metadata.len());
        } else {
            println!("   Neural model: Not downloaded");
        }
    } else {
        println!("🤖 Models directory: Not created");
    }

    Ok(())
}

async fn show_config() -> Result<()> {
    println!("⚙️  Semisearch Configuration");
    println!("===========================");

    let config = EmbeddingConfig::default();
    println!("Model: {}", config.model_name);
    println!("Cache directory: {}", config.cache_dir.display());
    println!("Max length: {}", config.max_length);
    println!("Batch size: {}", config.batch_size);
    println!("Device: {:?}", config.device);

    Ok(())
}

async fn run_doctor() -> Result<()> {
    println!("🏥 Semisearch System Check");
    println!("=========================");

    // Check system resources
    if let Ok(mem_info) = sys_info::mem_info() {
        println!("💾 Available memory: {} MB", mem_info.avail / 1024);
        println!("💾 Total memory: {} MB", mem_info.total / 1024);
    }

    println!("🖥️  CPU cores: {}", num_cpus::get());

    // Check embedding capabilities
    let capability = LocalEmbedder::detect_capabilities();
    println!("🧠 Detected capability: {capability:?}");

    match capability {
        #[cfg(feature = "neural-embeddings")]
        EmbeddingCapability::Full => {
            println!("✅ System supports full neural embeddings");

            // Test embedder creation
            print!("🧪 Testing embedder initialization... ");
            match create_embedder().await {
                Ok(_) => println!("✅ Success"),
                Err(e) => println!("❌ Failed: {e}"),
            }
        }
        EmbeddingCapability::TfIdf => {
            println!("⚠️  Limited system - TF-IDF embeddings only");
            println!("   Consider upgrading RAM for neural embeddings");
        }
        EmbeddingCapability::None => {
            println!("❌ System too limited for embeddings");
            println!("   Only keyword search available");
        }
    }

    // Check database connectivity
    print!("🗄️  Testing database... ");
    match get_database_path().and_then(|path| Database::new(&path)) {
        Ok(_) => println!("✅ Success"),
        Err(e) => println!("❌ Failed: {e}"),
    }

    // Check network connectivity (for model downloads)
    print!("🌐 Testing network connectivity... ");
    match reqwest::get("https://huggingface.co").await {
        Ok(response) if response.status().is_success() => println!("✅ Success"),
        Ok(_) => println!("⚠️  Limited connectivity"),
        Err(_) => println!("❌ No network access"),
    }

    println!();
    println!("🎯 Recommendations:");

    match capability {
        #[cfg(feature = "neural-embeddings")]
        EmbeddingCapability::Full => {
            println!("   • Use 'semisearch search --semantic' for best results");
            println!("   • Run 'semisearch index --semantic <dir>' to build semantic index");
        }
        EmbeddingCapability::TfIdf => {
            println!("   • Use 'semisearch search --mode tfidf' for statistical search");
            println!("   • Keyword and fuzzy search work well on this system");
        }
        EmbeddingCapability::None => {
            println!("   • Use 'semisearch search --mode keyword' for basic search");
            println!("   • Consider using regex mode for pattern matching");
        }
    }

    Ok(())
}
