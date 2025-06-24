use clap::{Parser, Subcommand};
use semisearch::{search_files, OutputFormat, SearchOptions, SearchResult};
use semisearch::core::{FileIndexer, IndexStats};
use semisearch::storage::Database;
use std::path::Path;
use std::process;

#[derive(Parser)]
#[command(name = "semisearch")]
#[command(about = "Semantic search across local files")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Search for matches in files
    Search {
        /// Search query
        query: String,

        /// Target directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        path: String,

        /// Minimum similarity score (0.0-1.0)
        #[arg(short, long, default_value = "0.0")]
        score: f32,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Output format (plain, json)
        #[arg(short, long, default_value = "plain")]
        format: String,

        /// Enable fuzzy matching
        #[arg(long)]
        fuzzy: bool,

        /// Use regex pattern matching
        #[arg(long)]
        regex: bool,

        /// Case-sensitive search
        #[arg(long)]
        case_sensitive: bool,

        /// Enable typo tolerance using edit distance
        #[arg(long)]
        typo_tolerance: bool,

        /// Maximum edit distance for typo tolerance (default: 2)
        #[arg(long, default_value = "2")]
        max_edit_distance: usize,
    },

    /// Index files in directory (placeholder for future implementation)
    Index {
        /// Directory to index
        path: String,
    },

    /// Show configuration (placeholder for future implementation)
    Config,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search {
            query,
            path,
            score,
            limit,
            format,
            fuzzy,
            regex,
            case_sensitive,
            typo_tolerance,
            max_edit_distance,
        } => {
            let options = SearchOptions {
                min_score: score,
                max_results: limit,
                fuzzy_matching: fuzzy,
                regex_mode: regex,
                case_sensitive,
                typo_tolerance,
                max_edit_distance,
            };

            let output_format = match format.as_str() {
                "json" => OutputFormat::Json,
                _ => OutputFormat::Plain,
            };

            match search_files(&query, &path, &options) {
                Ok(results) => {
                    if results.is_empty() {
                        eprintln!("No matches found for '{query}'");
                        process::exit(1);
                    }

                    let output = format_results(&results, output_format);
                    println!("{output}");
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }
            }
        }

        Commands::Index { path } => {
            // Create database path in user's home directory
            let db_path = get_database_path();
            
            match Database::new(&db_path) {
                Ok(database) => {
                    let indexer = FileIndexer::new(database);
                    
                    match indexer.index_directory(Path::new(&path)) {
                        Ok(stats) => {
                            println!("Indexing completed successfully!");
                            println!("Database location: {}", db_path.display());
                            
                            if !stats.errors.is_empty() {
                                eprintln!("\nErrors encountered:");
                                for error in &stats.errors {
                                    eprintln!("  {}", error);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error during indexing: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error creating database: {}", e);
                    process::exit(1);
                }
            }
        }

        Commands::Config => {
            let db_path = get_database_path();
            
            println!("semisearch Configuration:");
            println!("  Database path: {}", db_path.display());
            
            // Try to get database stats if it exists
            if db_path.exists() {
                match Database::new(&db_path) {
                    Ok(database) => {
                        match database.get_stats() {
                            Ok(stats) => {
                                println!("  Files indexed: {}", stats.file_count);
                                println!("  Chunks stored: {}", stats.chunk_count);
                                println!("  Total size: {} MB", stats.total_size_bytes / (1024 * 1024));
                            }
                            Err(e) => eprintln!("  Error reading stats: {}", e),
                        }
                    }
                    Err(e) => eprintln!("  Error opening database: {}", e),
                }
            } else {
                println!("  Status: No index found (run 'semisearch index <path>' to create one)");
            }
        }
    }
}

fn format_results(results: &[SearchResult], format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(results).unwrap_or_else(|_| "[]".to_string())
        }
        OutputFormat::Plain => results
            .iter()
            .map(|r| format!("{}:{}:{}", r.file_path, r.line_number, r.content))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

/// Get the default database path for the user
fn get_database_path() -> std::path::PathBuf {
    // Create database in user's home directory under .semisearch/
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let semisearch_dir = home_dir.join(".semisearch");
    
    // Create directory if it doesn't exist
    if !semisearch_dir.exists() {
        std::fs::create_dir_all(&semisearch_dir).unwrap_or_else(|e| {
            eprintln!("Warning: Could not create directory {}: {}", semisearch_dir.display(), e);
        });
    }
    
    semisearch_dir.join("index.db")
}
