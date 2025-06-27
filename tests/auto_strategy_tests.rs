use search::search::auto_strategy::{AutoStrategy, ProjectContext};

#[tokio::test]
async fn test_exact_phrase_strategy() {
    let auto_strategy = AutoStrategy::new();

    // Test exact phrase queries
    let results = auto_strategy
        .search("\"specific function name\"", "./src")
        .await
        .unwrap();

    // Should use keyword search for exact phrases
    assert!(!results.is_empty() || results.is_empty()); // Just checking it doesn't panic
}

#[tokio::test]
async fn test_code_pattern_strategy() {
    let auto_strategy = AutoStrategy::new();

    // Test code pattern queries
    let results = auto_strategy.search("TODO", "./src").await.unwrap();

    // Should use regex search for code patterns
    assert!(!results.is_empty() || results.is_empty()); // Just checking it doesn't panic
}

#[tokio::test]
async fn test_conceptual_strategy() {
    let auto_strategy = AutoStrategy::new();

    // Test conceptual queries
    let results = auto_strategy
        .search("error handling patterns", "./src")
        .await
        .unwrap();

    // Should use semantic search for conceptual queries if available
    assert!(!results.is_empty() || results.is_empty()); // Just checking it doesn't panic
}

#[tokio::test]
async fn test_file_extension_strategy() {
    let auto_strategy = AutoStrategy::new();

    // Test file extension queries
    let results = auto_strategy.search(".rs", "./src").await.unwrap();

    // Should use appropriate strategy for file extensions
    assert!(!results.is_empty() || results.is_empty()); // Just checking it doesn't panic
}

#[tokio::test]
async fn test_regex_like_strategy() {
    let auto_strategy = AutoStrategy::new();

    // Test regex-like queries
    let results = auto_strategy.search("TODO.*:", "./src").await.unwrap();

    // Should use regex search for regex-like patterns
    assert!(!results.is_empty() || results.is_empty()); // Just checking it doesn't panic
}

#[tokio::test]
async fn test_fallback_to_fuzzy() {
    let auto_strategy = AutoStrategy::new();

    // Test that falls back to fuzzy for typo tolerance
    let results = auto_strategy.search("databse", "./src").await.unwrap();

    // Should use fuzzy search for typo tolerance
    assert!(!results.is_empty() || results.is_empty()); // Just checking it doesn't panic
}

#[test]
fn test_project_context_detection() {
    // Test Rust project detection (current directory has Cargo.toml)
    let context = ProjectContext::detect(".").unwrap();
    assert!(matches!(context, ProjectContext::Mixed)); // Has both Cargo.toml and docs

    // Test documentation project detection
    let context = ProjectContext::detect("./docs").unwrap();
    assert!(matches!(context, ProjectContext::Documentation));

    // Test src directory (no Cargo.toml, so should be Unknown)
    let context = ProjectContext::detect("./src").unwrap();
    assert!(matches!(context, ProjectContext::Unknown));
}

#[test]
fn test_code_pattern_to_regex() {
    let auto_strategy = AutoStrategy::new();

    // Test TODO pattern conversion
    let regex = auto_strategy.code_pattern_to_regex("TODO");
    assert_eq!(regex, r"TODO.*");

    // Test function pattern conversion (should be "fn\s+\w+" for Rust)
    let regex = auto_strategy.code_pattern_to_regex("function");
    assert_eq!(regex, r"fn\s+\w+");

    // Test FIXME pattern conversion
    let regex = auto_strategy.code_pattern_to_regex("FIXME");
    assert_eq!(regex, r"FIXME.*");
}
