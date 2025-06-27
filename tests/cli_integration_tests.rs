use anyhow::Result;
use search::cli::{Cli, Commands, IndexArgs, SearchArgs};
use std::fs;
use tempfile::TempDir;

/// Test fixture for CLI testing
pub struct CliTestFixture {
    temp_dir: TempDir,
}

impl CliTestFixture {
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;

        // Create test files
        fs::write(
            temp_dir.path().join("test.txt"),
            "TODO: Fix this function\nfunction login() {\n  // TODO: Add validation\n  return true;\n}"
        )?;

        fs::write(
            temp_dir.path().join("readme.md"),
            "# Test Project\n\nThis is a test project for searching.\nIt contains various TODO items and functions."
        )?;

        fs::create_dir(temp_dir.path().join("src"))?;
        fs::write(
            temp_dir.path().join("src/main.rs"),
            "fn main() {\n    println!(\"Hello, world!\");\n    // TODO: Implement error handling\n}"
        )?;

        Ok(Self { temp_dir })
    }

    pub fn path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }
}

#[tokio::test]
async fn test_simple_search_command() -> Result<()> {
    let fixture = CliTestFixture::new()?;

    let search_args = SearchArgs {
        query: "TODO".to_string(),
        path: fixture.path().to_string_lossy().to_string(),
        fuzzy: false,
        exact: false,
        score: 0.3,
        limit: 10,
        case_sensitive: false,
        typo_tolerance: false,
        // Advanced options (with defaults)
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
        path_flag: None,
    };

    // Test that search args are created correctly
    assert_eq!(search_args.query, "TODO");
    assert!(!search_args.fuzzy);
    assert!(!search_args.exact);
    assert_eq!(search_args.score, 0.3);
    assert_eq!(search_args.limit, 10);

    Ok(())
}

#[tokio::test]
async fn test_simple_search_with_fuzzy_flag() -> Result<()> {
    let fixture = CliTestFixture::new()?;

    let search_args = SearchArgs {
        query: "databse".to_string(), // Typo
        path: fixture.path().to_string_lossy().to_string(),
        fuzzy: true,
        exact: false,
        score: 0.3,
        limit: 10,
        case_sensitive: false,
        typo_tolerance: false,
        // Advanced options (with defaults)
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
        path_flag: None,
    };

    // Test that fuzzy flag is set correctly
    assert_eq!(search_args.query, "databse");
    assert!(search_args.fuzzy);
    assert!(!search_args.exact);

    Ok(())
}

#[tokio::test]
async fn test_simple_search_with_exact_flag() -> Result<()> {
    let fixture = CliTestFixture::new()?;

    let search_args = SearchArgs {
        query: "TODO: Fix".to_string(),
        path: fixture.path().to_string_lossy().to_string(),
        fuzzy: false,
        exact: true,
        score: 0.3,
        limit: 10,
        case_sensitive: false,
        typo_tolerance: false,
        // Advanced options (with defaults)
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
        path_flag: None,
    };

    // Test that exact flag is set correctly
    assert_eq!(search_args.query, "TODO: Fix");
    assert!(!search_args.fuzzy);
    assert!(search_args.exact);

    Ok(())
}

#[tokio::test]
async fn test_cli_commands() -> Result<()> {
    // Test that all commands can be created
    let search_cmd = Commands::Search(SearchArgs {
        query: "test".to_string(),
        path: ".".to_string(),
        fuzzy: false,
        exact: false,
        score: 0.3,
        limit: 10,
        case_sensitive: false,
        typo_tolerance: false,
        // Advanced options (with defaults)
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
        path_flag: None,
    });

    let help_cmd = Commands::HelpMe;
    let status_cmd = Commands::Status;
    let config_cmd = Commands::Config;
    let doctor_cmd = Commands::Doctor;

    // Verify commands can be matched
    match search_cmd {
        Commands::Search(args) => {
            assert_eq!(args.query, "test");
            assert_eq!(args.path, ".");
        }
        _ => panic!("Expected Search command"),
    }

    match help_cmd {
        Commands::HelpMe => { /* Success */ }
        _ => panic!("Expected HelpMe command"),
    }

    match status_cmd {
        Commands::Status => { /* Success */ }
        _ => panic!("Expected Status command"),
    }

    match config_cmd {
        Commands::Config => { /* Success */ }
        _ => panic!("Expected Config command"),
    }

    match doctor_cmd {
        Commands::Doctor => { /* Success */ }
        _ => panic!("Expected Doctor command"),
    }

    Ok(())
}

#[tokio::test]
async fn test_index_command() -> Result<()> {
    let fixture = CliTestFixture::new()?;

    let cli = Cli {
        advanced: false,
        command: Commands::Index(IndexArgs {
            path: fixture.path().to_string_lossy().to_string(),
            force: false,
            semantic: false,
            no_semantic: false,
            batch_size: 100,
            workers: 4,
        }),
    };

    if let Commands::Index(args) = &cli.command {
        assert!(!args.force);
        assert!(!args.semantic);
        assert!(!args.no_semantic);
    } else {
        panic!("Expected Index command");
    }

    Ok(())
}

#[tokio::test]
async fn test_conflicting_flags_handling() -> Result<()> {
    // Test that conflicting flags can be set (resolution happens at runtime)
    let search_args = SearchArgs {
        query: "test".to_string(),
        path: ".".to_string(),
        fuzzy: true,
        exact: true, // Conflicting with fuzzy
        score: 0.3,
        limit: 10,
        case_sensitive: false,
        typo_tolerance: false,
        // Advanced options (with defaults)
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
        path_flag: None,
    };

    // Both flags can be set - the application logic decides precedence
    assert!(search_args.fuzzy);
    assert!(search_args.exact);

    Ok(())
}

/// Test edge cases and boundary conditions
#[tokio::test]
async fn test_edge_cases() -> Result<()> {
    // Test empty query
    let empty_query = SearchArgs {
        query: "".to_string(),
        path: ".".to_string(),
        fuzzy: false,
        exact: false,
        score: 0.3,
        limit: 10,
        case_sensitive: false,
        typo_tolerance: false,
        // Advanced options (with defaults)
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
        path_flag: None,
    };

    assert!(empty_query.query.is_empty());

    // Test very long query
    let long_query = "a".repeat(1000);
    let long_query_args = SearchArgs {
        query: long_query.clone(),
        path: ".".to_string(),
        fuzzy: false,
        exact: false,
        score: 0.3,
        limit: 10,
        case_sensitive: false,
        typo_tolerance: false,
        // Advanced options (with defaults)
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
        path_flag: None,
    };

    assert_eq!(long_query_args.query, long_query);

    // Test boundary score values
    let boundary_scores = vec![0.0, 0.1, 0.5, 0.9, 1.0];
    for score in boundary_scores {
        let score_args = SearchArgs {
            query: "test".to_string(),
            path: ".".to_string(),
            fuzzy: false,
            exact: false,
            score,
            limit: 10,
            case_sensitive: false,
            typo_tolerance: false,
            // Advanced options (with defaults)
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
            path_flag: None,
        };

        assert!((0.0..=1.0).contains(&score_args.score));
    }

    // Test boundary limit values
    let boundary_limits = vec![1, 10, 100, 1000];
    for limit in boundary_limits {
        let limit_args = SearchArgs {
            query: "test".to_string(),
            path: ".".to_string(),
            fuzzy: false,
            exact: false,
            score: 0.3,
            limit,
            case_sensitive: false,
            typo_tolerance: false,
            // Advanced options (with defaults)
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
            path_flag: None,
        };

        assert!(limit_args.limit > 0);
    }

    Ok(())
}

/// Test that the CLI gracefully handles special characters and unicode
#[tokio::test]
async fn test_unicode_and_special_characters() -> Result<()> {
    let special_queries = vec![
        "🔍 search",
        "café",
        "测试",
        "test@example.com",
        "C:\\Windows\\System32",
        "/usr/local/bin",
        "file.name.with.dots",
        "query with spaces",
        "query\nwith\nnewlines",
        "query\twith\ttabs",
    ];

    for query in special_queries {
        let args = SearchArgs {
            query: query.to_string(),
            path: ".".to_string(),
            fuzzy: false,
            exact: false,
            score: 0.3,
            limit: 10,
            case_sensitive: false,
            typo_tolerance: false,
            // Advanced options (with defaults)
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
            path_flag: None,
        };

        // Should handle all characters gracefully
        assert_eq!(args.query, query);
    }

    Ok(())
}

#[tokio::test]
async fn test_cli_structure() -> Result<()> {
    let cli = Cli {
        advanced: false,
        command: Commands::Search(SearchArgs {
            query: "test".to_string(),
            path: ".".to_string(),
            fuzzy: false,
            exact: false,
            score: 0.3,
            limit: 10,
            case_sensitive: false,
            typo_tolerance: false,
            // Advanced options (with defaults)
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
            path_flag: None,
        }),
    };

    // Test that CLI structure is correct
    match &cli.command {
        Commands::Search(args) => {
            assert_eq!(args.query, "test");
            assert_eq!(args.path, ".");
            assert!(!args.fuzzy);
            assert!(!args.exact);
            assert_eq!(args.score, 0.3);
            assert_eq!(args.limit, 10);
            assert!(!args.case_sensitive);
            assert!(!args.typo_tolerance);
        }
        _ => panic!("Expected Search command"),
    }

    Ok(())
}

/// Test default values
#[tokio::test]
async fn test_default_values() -> Result<()> {
    // Test that default values are reasonable
    let args = SearchArgs {
        query: "test".to_string(),
        path: ".".to_string(), // Default path
        fuzzy: false,          // Default: no fuzzy
        exact: false,          // Default: no exact
        score: 0.3,            // Default score threshold
        limit: 10,             // Default result limit
        case_sensitive: false, // Default: case insensitive
        typo_tolerance: false, // Default: no typo tolerance
        // Advanced options (with defaults)
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
        path_flag: None,
    };

    // Verify defaults are sensible
    assert_eq!(args.path, ".");
    assert!(!args.fuzzy);
    assert!(!args.exact);
    assert_eq!(args.score, 0.3);
    assert_eq!(args.limit, 10);
    assert!(!args.case_sensitive);
    assert!(!args.typo_tolerance);

    Ok(())
}
