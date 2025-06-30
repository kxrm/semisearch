use clap::{Arg, ArgMatches, Args, Command, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "semisearch")]
#[command(about = "Semantic search across local files")]
#[command(version = "0.6.0")]
pub struct Cli {
    /// Enable advanced options (for power users)
    #[arg(long, global = true)]
    pub advanced: bool,

    /// Allow typos and similar words
    #[arg(long, global = true)]
    pub fuzzy: bool,

    /// Find exact matches only
    #[arg(long, global = true)]
    pub exact: bool,

    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    /// Parse CLI with dynamic help based on advanced mode
    pub fn parse_advanced_aware() -> Self {
        let mut args: Vec<String> = std::env::args().collect();

        // Implement default command behavior (from UX Remediation Plan)
        // If no subcommand provided, assume "search"
        if args.len() > 1 {
            // First, check if --advanced is present and handle it properly
            let is_advanced_flag_present = args.contains(&"--advanced".to_string());

            // Find the first non-flag argument (potential command or query)
            let mut first_non_flag_index = None;
            for (i, arg) in args.iter().enumerate().skip(1) {
                if !arg.starts_with('-') {
                    first_non_flag_index = Some(i);
                    break;
                }
            }

            if let Some(index) = first_non_flag_index {
                let potential_command = &args[index];

                // Check if first non-flag argument is a known command
                let known_commands = [
                    "search", "s", "help-me", "status", "index", "config", "doctor", "help",
                ];
                let is_known_command = known_commands.contains(&potential_command.as_str());

                // If it's not a known command, treat it as a search query
                if !is_known_command {
                    // Insert "search" as the subcommand before the query
                    args.insert(index, "search".to_string());

                    // Now we need to check if there's a path argument after the query
                    // Look for the next non-flag argument that could be a path
                    let query_index = index + 1; // The query is now at this index
                    let mut _path_index = None;

                    // Look for a potential path argument after the query
                    for (i, arg) in args.iter().enumerate().skip(query_index + 1) {
                        if !arg.starts_with('-') {
                            // This could be a path - check if it looks like a path
                            if arg.contains('/') || arg.contains('\\') || arg == "." || arg == ".."
                            {
                                _path_index = Some(i);
                                break;
                            }
                        }
                    }

                    // If we found a potential path, we need to ensure it's properly positioned
                    // The CLI structure expects: search <query> <path> [flags...]
                    // clap will automatically parse it as the path argument
                }
            } else if !is_advanced_flag_present {
                // No non-flag arguments found and no --advanced flag - this is likely an error
                // Let clap handle this case normally
            }
        }

        let is_advanced = args.contains(&"--advanced".to_string());

        if is_advanced {
            // Use dynamic CLI with all options visible
            let app = Self::build_advanced_cli();
            let matches = app.get_matches_from(args);
            Self::from_matches(&matches)
        } else {
            // Use regular CLI with hidden options
            Self::parse_from(args)
        }
    }

    /// Build CLI structure for advanced mode with all options visible
    fn build_advanced_cli() -> Command {
        Command::new("semisearch")
            .about("Semantic search across local files")
            .version("0.6.0")
            .arg(
                Arg::new("advanced")
                    .long("advanced")
                    .help("Enable advanced options (for power users)")
                    .action(clap::ArgAction::SetTrue)
                    .global(true),
            )
            // Add common search options at the top level (flattened CLI)
            .arg(
                Arg::new("fuzzy")
                    .long("fuzzy")
                    .help("Allow typos and similar words")
                    .action(clap::ArgAction::SetTrue)
                    .global(true),
            )
            .arg(
                Arg::new("exact")
                    .long("exact")
                    .help("Find exact matches only")
                    .action(clap::ArgAction::SetTrue)
                    .global(true),
            )
            // Additional advanced options at top level in advanced mode
            .arg(
                Arg::new("mode")
                    .long("mode")
                    .help("Search mode: auto, semantic, keyword, fuzzy, regex, tfidf, hybrid")
                    .default_value("auto")
                    .value_name("MODE")
                    .global(true),
            )
            .arg(
                Arg::new("semantic-threshold")
                    .long("semantic-threshold")
                    .help("Semantic similarity threshold (0.0-1.0)")
                    .default_value("0.7")
                    .value_name("THRESHOLD")
                    .global(true),
            )
            .arg(
                Arg::new("format")
                    .short('f')
                    .long("format")
                    .help("Output format: plain, json")
                    .default_value("plain")
                    .value_name("FORMAT")
                    .global(true),
            )
            .arg(
                Arg::new("files-only")
                    .long("files-only")
                    .help("Show file paths only")
                    .action(clap::ArgAction::SetTrue)
                    .global(true),
            )
            .arg(
                Arg::new("context")
                    .long("context")
                    .help("Context lines around matches")
                    .default_value("0")
                    .value_name("LINES")
                    .global(true),
            )
            .arg(
                Arg::new("semantic")
                    .long("semantic")
                    .help("Enable semantic search")
                    .action(clap::ArgAction::SetTrue)
                    .global(true),
            )
            .arg(
                Arg::new("no-semantic")
                    .long("no-semantic")
                    .help("Disable semantic search")
                    .action(clap::ArgAction::SetTrue)
                    .global(true),
            )
            .arg(
                Arg::new("regex")
                    .long("regex")
                    .help("Use regex pattern matching")
                    .action(clap::ArgAction::SetTrue)
                    .global(true),
            )
            .arg(
                Arg::new("include-binary")
                    .long("include-binary")
                    .help("Include binary files")
                    .action(clap::ArgAction::SetTrue)
                    .global(true),
            )
            .arg(
                Arg::new("follow-links")
                    .long("follow-links")
                    .help("Follow symbolic links")
                    .action(clap::ArgAction::SetTrue)
                    .global(true),
            )
            .arg(
                Arg::new("include")
                    .long("include")
                    .help("Include files matching patterns (e.g., *.rs, *.md)")
                    .action(clap::ArgAction::Append)
                    .value_name("PATTERN")
                    .global(true),
            )
            .arg(
                Arg::new("exclude")
                    .long("exclude")
                    .help("Exclude files matching patterns (e.g., *test*, *.tmp)")
                    .action(clap::ArgAction::Append)
                    .value_name("PATTERN")
                    .global(true),
            )
            .subcommand(
                Command::new("search")
                    .about("Search for text in files (default behavior)")
                    .visible_alias("s")
                    .arg(
                        Arg::new("query")
                            .help("What to search for")
                            .required(true)
                            .value_name("QUERY"),
                    )
                    .arg(
                        Arg::new("path")
                            .help("Where to search (default: current directory)")
                            .default_value(".")
                            .value_name("PATH"),
                    )
                    // Basic options (always visible)
                    .arg(
                        Arg::new("fuzzy")
                            .long("fuzzy")
                            .help("Allow typos and similar words")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("exact")
                            .long("exact")
                            .help("Find exact matches only")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("score")
                            .short('s')
                            .long("score")
                            .help("Minimum similarity score (0.0-1.0)")
                            .default_value("0.3")
                            .value_name("SCORE"),
                    )
                    .arg(
                        Arg::new("limit")
                            .short('l')
                            .long("limit")
                            .help("Maximum number of results")
                            .default_value("10")
                            .value_name("LIMIT"),
                    )
                    .arg(
                        Arg::new("case-sensitive")
                            .long("case-sensitive")
                            .help("Case sensitive search")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("typo-tolerance")
                            .long("typo-tolerance")
                            .help("Enable typo tolerance")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("path-flag")
                            .long("path")
                            .help(
                                "Target directory (legacy --path flag for backward compatibility)",
                            )
                            .value_name("PATH_FLAG"),
                    )
                    // Advanced options (visible in advanced mode)
                    .arg(
                        Arg::new("mode")
                            .long("mode")
                            .help(
                                "Search mode: auto, semantic, keyword, fuzzy, regex, tfidf, hybrid",
                            )
                            .default_value("auto")
                            .value_name("MODE"),
                    )
                    .arg(
                        Arg::new("semantic-threshold")
                            .long("semantic-threshold")
                            .help("Semantic similarity threshold (0.0-1.0)")
                            .default_value("0.7")
                            .value_name("THRESHOLD"),
                    )
                    .arg(
                        Arg::new("format")
                            .short('f')
                            .long("format")
                            .help("Output format: plain, json")
                            .default_value("plain")
                            .value_name("FORMAT"),
                    )
                    .arg(
                        Arg::new("files-only")
                            .long("files-only")
                            .help("Show file paths only")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("context")
                            .long("context")
                            .help("Context lines around matches")
                            .default_value("0")
                            .value_name("LINES"),
                    )
                    .arg(
                        Arg::new("semantic")
                            .long("semantic")
                            .help("Enable semantic search")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("no-semantic")
                            .long("no-semantic")
                            .help("Disable semantic search")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("regex")
                            .long("regex")
                            .help("Use regex pattern matching")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("include-binary")
                            .long("include-binary")
                            .help("Include binary files")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("follow-links")
                            .long("follow-links")
                            .help("Follow symbolic links")
                            .action(clap::ArgAction::SetTrue),
                    ),
            )
            .subcommand(Command::new("help-me").about("Get help for beginners"))
            .subcommand(Command::new("status").about("Check if tool is working properly"))
            .subcommand(
                Command::new("index")
                    .about("Index files for faster searching")
                    .arg(
                        Arg::new("path")
                            .help("Directory to index")
                            .required(true)
                            .value_name("PATH"),
                    )
                    .arg(
                        Arg::new("force")
                            .long("force")
                            .help("Force full reindex")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("semantic")
                            .long("semantic")
                            .help("Build semantic embeddings during indexing")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("no-semantic")
                            .long("no-semantic")
                            .help("Skip semantic embeddings")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("batch-size")
                            .long("batch-size")
                            .help("Batch size for processing")
                            .default_value("100")
                            .value_name("SIZE"),
                    )
                    .arg(
                        Arg::new("workers")
                            .long("workers")
                            .help("Number of worker threads")
                            .default_value("4")
                            .value_name("COUNT"),
                    ),
            )
            .subcommand(Command::new("config").about("Show configuration"))
            .subcommand(Command::new("doctor").about("Test system capabilities"))
    }

    /// Convert ArgMatches to Cli struct for advanced mode
    fn from_matches(matches: &ArgMatches) -> Self {
        let advanced = matches.get_flag("advanced");

        let command = match matches.subcommand() {
            Some(("search", sub_matches)) => Commands::Search(SearchArgs {
                query: sub_matches.get_one::<String>("query").unwrap().clone(),
                path: sub_matches.get_one::<String>("path").unwrap().clone(),
                fuzzy: sub_matches.get_flag("fuzzy"),
                exact: sub_matches.get_flag("exact"),
                score: sub_matches
                    .get_one::<String>("score")
                    .unwrap()
                    .parse()
                    .unwrap_or(0.3),
                limit: sub_matches
                    .get_one::<String>("limit")
                    .unwrap()
                    .parse()
                    .unwrap_or(10),
                case_sensitive: sub_matches.get_flag("case-sensitive"),
                typo_tolerance: sub_matches.get_flag("typo-tolerance"),
                mode: sub_matches
                    .get_one::<String>("mode")
                    .unwrap_or(&"auto".to_string())
                    .clone(),
                semantic_threshold: sub_matches
                    .get_one::<String>("semantic-threshold")
                    .unwrap()
                    .parse()
                    .unwrap_or(0.7),
                format: sub_matches
                    .get_one::<String>("format")
                    .unwrap_or(&"plain".to_string())
                    .clone(),
                files_only: sub_matches.get_flag("files-only"),
                context: sub_matches
                    .get_one::<String>("context")
                    .unwrap()
                    .parse()
                    .unwrap_or(0),
                semantic: sub_matches.get_flag("semantic"),
                no_semantic: sub_matches.get_flag("no-semantic"),
                regex: sub_matches.get_flag("regex"),
                include_binary: sub_matches.get_flag("include-binary"),
                follow_links: sub_matches.get_flag("follow-links"),
                include: sub_matches
                    .get_many::<String>("include")
                    .unwrap_or_default()
                    .cloned()
                    .collect(),
                exclude: sub_matches
                    .get_many::<String>("exclude")
                    .unwrap_or_default()
                    .cloned()
                    .collect(),
                path_flag: sub_matches.get_one::<String>("path-flag").cloned(),
            }),
            Some(("help-me", _)) => Commands::HelpMe,
            Some(("status", _)) => Commands::Status,
            Some(("index", sub_matches)) => Commands::Index(IndexArgs {
                path: sub_matches.get_one::<String>("path").unwrap().clone(),
                force: sub_matches.get_flag("force"),
                semantic: sub_matches.get_flag("semantic"),
                no_semantic: sub_matches.get_flag("no-semantic"),
                batch_size: sub_matches
                    .get_one::<String>("batch-size")
                    .unwrap()
                    .parse()
                    .unwrap_or(100),
                workers: sub_matches
                    .get_one::<String>("workers")
                    .unwrap()
                    .parse()
                    .unwrap_or(4),
            }),
            Some(("config", _)) => Commands::Config,
            Some(("doctor", _)) => Commands::Doctor,
            _ => Commands::Search(SearchArgs {
                query: "".to_string(),
                path: ".".to_string(),
                fuzzy: false,
                exact: false,
                score: 0.3,
                limit: 10,
                case_sensitive: false,
                typo_tolerance: false,
                mode: "auto".to_string(),
                semantic_threshold: 0.7,
                format: "plain".to_string(),
                files_only: false,
                context: 0,
                semantic: false,
                no_semantic: false,
                regex: false,
                include_binary: false,
                follow_links: false,
                include: vec![],
                exclude: vec![],
                path_flag: None,
            }),
        };

        Self {
            advanced,
            fuzzy: matches.get_flag("fuzzy"),
            exact: matches.get_flag("exact"),
            command,
        }
    }
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
    #[arg(short, long, default_value = "plain", hide = true)]
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

    /// Include files matching patterns (e.g., *.rs, *.md)
    #[arg(long, hide = true)]
    pub include: Vec<String>,

    /// Exclude files matching patterns (e.g., *test*, *.tmp)
    #[arg(long, hide = true)]
    pub exclude: Vec<String>,

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
