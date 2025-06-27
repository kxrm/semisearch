use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "semisearch")]
#[command(about = "Semantic search across local files")]
#[command(version = "0.6.0")]
pub struct Cli {
    /// Enable advanced options (for power users)
    #[arg(long, global = true)]
    pub advanced: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Search for text in files (default behavior)
    #[command(name = "search", visible_alias = "s")]
    Search(SearchArgs),

    /// Get help for beginners
    #[command(name = "help-me")]
    HelpMe,

    /// Check if tool is working properly
    #[command(name = "status")]
    Status,

    /// Index files for faster searching
    #[command(name = "index")]
    Index(IndexArgs),

    /// Show configuration
    #[command(name = "config")]
    Config,

    /// Test system capabilities
    #[command(name = "doctor")]
    Doctor,
}

#[derive(Args)]
pub struct SearchArgs {
    /// What to search for
    pub query: String,

    /// Where to search (default: current directory)
    #[arg(default_value = ".")]
    pub path: String,

    /// Allow typos and similar words
    #[arg(long)]
    pub fuzzy: bool,

    /// Find exact matches only
    #[arg(long)]
    pub exact: bool,

    /// Minimum similarity score (0.0-1.0)
    #[arg(short, long, default_value = "0.3")]
    pub score: f32,

    /// Maximum number of results
    #[arg(short, long, default_value = "10")]
    pub limit: usize,

    /// Case sensitive search
    #[arg(long)]
    pub case_sensitive: bool,

    /// Enable typo tolerance
    #[arg(long)]
    pub typo_tolerance: bool,

    // Advanced options (hidden by default)
    /// Search mode: auto, semantic, keyword, fuzzy, regex, tfidf, hybrid
    #[arg(long, default_value = "auto", hide = true)]
    pub mode: String,

    /// Semantic similarity threshold (0.0-1.0)
    #[arg(long, default_value = "0.7", hide = true)]
    pub semantic_threshold: f32,

    /// Output format: plain, json
    #[arg(short, long, default_value = "plain")]
    pub format: String,

    /// Show file paths only
    #[arg(long, hide = true)]
    pub files_only: bool,

    /// Context lines around matches
    #[arg(long, default_value = "0", hide = true)]
    pub context: usize,

    /// Enable semantic search
    #[arg(long, hide = true)]
    pub semantic: bool,

    /// Disable semantic search
    #[arg(long, hide = true)]
    pub no_semantic: bool,

    /// Use regex pattern matching
    #[arg(long, hide = true)]
    pub regex: bool,

    /// Include binary files
    #[arg(long, hide = true)]
    pub include_binary: bool,

    /// Follow symbolic links
    #[arg(long, hide = true)]
    pub follow_links: bool,

    /// Target directory (legacy --path flag for backward compatibility)
    #[arg(long = "path")]
    pub path_flag: Option<String>,
}

#[derive(Args)]
pub struct IndexArgs {
    /// Directory to index
    pub path: String,

    /// Force full reindex
    #[arg(long)]
    pub force: bool,

    /// Build semantic embeddings during indexing
    #[arg(long)]
    pub semantic: bool,

    /// Skip semantic embeddings
    #[arg(long)]
    pub no_semantic: bool,

    /// Batch size for processing
    #[arg(long, default_value = "100")]
    pub batch_size: usize,

    /// Number of worker threads
    #[arg(long, default_value = "4")]
    pub workers: usize,
}
