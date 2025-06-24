use clap::{Parser, Subcommand};
use semisearch::{search_files, OutputFormat, SearchOptions, SearchResult};
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
                        eprintln!("No matches found for '{}'", query);
                        process::exit(1);
                    }

                    let output = format_results(&results, output_format);
                    println!("{}", output);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }

        Commands::Index { path } => {
            println!(
                "Indexing functionality not yet implemented for path: {}",
                path
            );
            println!("This will be added in Phase 2 (Persistent Index)");
        }

        Commands::Config => {
            println!("Configuration management not yet implemented");
            println!("This will be added in Phase 2 (Enhanced Search)");
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
