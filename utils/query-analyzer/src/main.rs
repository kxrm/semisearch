use clap::{Arg, Command};

mod lightweight_analysis;
use lightweight_analysis::build_analyzer_with_defaults;

mod adaptive_search;
use adaptive_search::demonstrate_adaptive_search;

fn main() {
    let matches = Command::new("Query Analyzer")
        .version("3.0")
        .about("Lightweight semantic search query analyzer")
        .arg(
            Arg::new("QUERY")
                .help("The query to analyze")
                .required_unless_present("demo")
                .index(1),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Show detailed analysis")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("demo")
                .short('d')
                .long("demo")
                .help("Run adaptive search strategy demo")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("demo") {
        demonstrate_adaptive_search();
        return;
    }

    let query = matches.get_one::<String>("QUERY").unwrap();
    let verbose = matches.get_flag("verbose");

    // Build analyzer with pre-computed statistics
    let mut analyzer = build_analyzer_with_defaults();
    let result = analyzer.analyze(query);
    
    println!("\nQuery Analysis Results");
    println!("Query: {}", query);
    println!();
    
    // Show semantic score as a bar
    let bar_width = 40;
    let filled = (result.needs_semantic * bar_width as f32) as usize;
    let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);
    
    println!("Semantic Score: [{bar}] {:.2}", result.needs_semantic);
    println!("Confidence:     {:.0}%", result.confidence * 100.0);
    
    let recommendation = if result.needs_semantic > 0.5 {
        "✅ Semantic search recommended"
    } else {
        "❌ Better suited for keyword search"
    };
    
    println!("Recommendation: {}", recommendation);
    println!("Explanation:    {}", result.explanation);
    
    if verbose {
        println!("\nDetailed Analysis:");
        analyzer.explain_analysis(query);
    }
} 