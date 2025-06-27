use search::cli::{Commands, SearchArgs};
use search::help::contextual::ContextualHelp;
use search::help::interactive::InteractiveHelp;
use search::{MatchType, SearchResult};
use std::io::Cursor;

#[tokio::test]
async fn test_contextual_help_no_results() {
    let query = "nonexistent_function_xyz";
    let empty_results = Vec::<SearchResult>::new();
    let last_command = Commands::Search(SearchArgs {
        query: query.to_string(),
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
        path_flag: None,
    });

    let help_text = ContextualHelp::generate_help(&last_command, &empty_results);

    assert!(help_text.contains(&format!("No results for '{query}'")));
    assert!(help_text.contains("Check spelling"));
    assert!(help_text.contains("--fuzzy"));
    assert!(help_text.contains("Try simpler terms"));
    assert!(help_text.contains("Search specific files"));
    assert!(help_text.contains("semisearch help-me"));
}

#[tokio::test]
async fn test_contextual_help_too_many_results() {
    let query = "function";
    // Create 51 dummy results to trigger "too many results" help
    let many_results: Vec<SearchResult> = (0..51)
        .map(|i| SearchResult {
            file_path: format!("file{i}.rs"),
            line_number: i as usize,
            content: format!("function test{i}"),
            score: Some(0.8),
            match_type: Some(MatchType::Exact),
        })
        .collect();

    let last_command = Commands::Search(SearchArgs {
        query: query.to_string(),
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
        path_flag: None,
    });

    let help_text = ContextualHelp::generate_help(&last_command, &many_results);

    assert!(help_text.contains("Found lots of matches"));
    assert!(help_text.contains("More specific terms"));
    assert!(help_text.contains("Search in a specific folder"));
    assert!(help_text.contains("Use exact phrases in quotes"));
}

#[tokio::test]
async fn test_contextual_help_good_results() {
    let query = "login";
    let good_results: Vec<SearchResult> = vec![SearchResult {
        file_path: "auth.rs".to_string(),
        line_number: 42,
        content: "fn login(user: &str) -> Result<()>".to_string(),
        score: Some(0.9),
        match_type: Some(MatchType::Exact),
    }];

    let last_command = Commands::Search(SearchArgs {
        query: query.to_string(),
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
        path_flag: None,
    });

    let help_text = ContextualHelp::generate_help(&last_command, &good_results);

    // Should return empty string for good results
    assert!(help_text.is_empty());
}

#[tokio::test]
async fn test_interactive_help_basic() {
    let mut input = Cursor::new(b"TODO\nquit\n");
    let mut output = Vec::new();

    let result = InteractiveHelp::run_with_io(&mut input, &mut output).await;
    assert!(result.is_ok());

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Welcome to SemiSearch"));
    assert!(output_str.contains("What do you want to search for?"));
    assert!(output_str.contains("Examples:"));
    assert!(output_str.contains("TODO comments"));
    assert!(output_str.contains("Error handling"));
    assert!(output_str.contains("Function definitions"));
}

#[tokio::test]
async fn test_interactive_help_with_search() {
    let mut input = Cursor::new(b"test query\nquit\n");
    let mut output = Vec::new();

    let result = InteractiveHelp::run_with_io(&mut input, &mut output).await;
    assert!(result.is_ok());

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Searching for: test query"));
}

#[tokio::test]
async fn test_contextual_help_simplifies_complex_queries() {
    let complex_query =
        "async function getUserData() { return database.query('SELECT * FROM users'); }";
    let empty_results = Vec::<SearchResult>::new();
    let last_command = Commands::Search(SearchArgs {
        query: complex_query.to_string(),
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
        path_flag: None,
    });

    let help_text = ContextualHelp::generate_help(&last_command, &empty_results);

    // Should suggest a simplified version
    assert!(help_text.contains("Try simpler terms"));

    // Look for the line that contains the simplified semisearch command
    let simplified_part = help_text
        .lines()
        .find(|line| line.contains("semisearch \"getUserData"))
        .expect("Should find simplified suggestion");

    // Extract the simplified query from the suggestion
    let start = simplified_part.find("semisearch \"").unwrap() + 12;
    let end = simplified_part[start..].find("\"").unwrap() + start;
    let simplified = &simplified_part[start..end];

    // The simplified query should be much shorter than the original
    assert!(simplified.len() < complex_query.len() / 2);
    assert!(!simplified.contains("async"));
    assert!(!simplified.contains("function"));
    // Note: 'SELECT might remain as part of a quoted string fragment
    assert!(!simplified.contains("async") && !simplified.contains("function"));
}

// Note: Integration test for help-me command would require
// refactoring main.rs to expose testable functions

#[tokio::test]
async fn test_contextual_help_handles_different_command_types() {
    // Test with Status command
    let results = Vec::<SearchResult>::new();
    let status_command = Commands::Status;

    let help_text = ContextualHelp::generate_help(&status_command, &results);
    // Status command shouldn't generate contextual help
    assert!(help_text.is_empty());

    // Test with HelpMe command
    let help_me_command = Commands::HelpMe;
    let help_text = ContextualHelp::generate_help(&help_me_command, &results);
    // HelpMe command shouldn't generate contextual help
    assert!(help_text.is_empty());
}
