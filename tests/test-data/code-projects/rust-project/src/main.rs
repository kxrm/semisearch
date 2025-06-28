use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use testlib::{config::Config, processor::FileProcessor};

/// Test application for searching files
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to search in
    #[clap(short, long, value_parser)]
    path: Option<PathBuf>,

    /// Search query
    #[clap(value_parser)]
    query: String,

    /// Enable verbose output
    #[clap(short, long)]
    verbose: bool,

    /// Maximum results to return
    #[clap(short, long, default_value = "10")]
    limit: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Add proper logging setup
    let args = Args::parse();
    
    // FIXME: Handle paths better
    let path = args.path.unwrap_or_else(|| PathBuf::from("."));
    
    let config = Config::new()
        .with_verbose(args.verbose)
        .with_limit(args.limit);
    
    // Create processor and process files
    let processor = FileProcessor::new(config);
    let results = processor.process_directory(&path, &args.query)
        .await
        .context("Failed to process directory")?;
    
    // Print results
    if results.is_empty() {
        println!("No results found for query: {}", args.query);
        return Ok(());
    }
    
    println!("Found {} results:", results.len());
    for (i, result) in results.iter().enumerate() {
        if i >= args.limit {
            println!("... and {} more results", results.len() - args.limit);
            break;
        }
        
        println!("{}. {} (score: {:.2})", i + 1, result.path.display(), result.score);
        println!("   {}", result.content);
        println!();
    }
    
    // TODO: Implement better error handling
    // TODO: Add support for different output formats
    
    Ok(())
} 