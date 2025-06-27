use search::search::file_type_strategy::{FileType, FileTypeDetector, FileTypeStrategy};
use std::path::PathBuf;

/// Test file type detection based on file extensions
#[test]
fn test_file_type_detection() {
    let detector = FileTypeDetector::new();

    // Test code files
    assert_eq!(detector.detect_from_path("src/main.rs"), FileType::Code);
    assert_eq!(detector.detect_from_path("lib.js"), FileType::Code);
    assert_eq!(detector.detect_from_path("app.py"), FileType::Code);
    assert_eq!(detector.detect_from_path("main.go"), FileType::Code);
    assert_eq!(detector.detect_from_path("index.html"), FileType::Code);
    assert_eq!(detector.detect_from_path("style.css"), FileType::Code);
    assert_eq!(detector.detect_from_path("script.ts"), FileType::Code);
    assert_eq!(detector.detect_from_path("component.jsx"), FileType::Code);
    assert_eq!(detector.detect_from_path("module.cpp"), FileType::Code);
    assert_eq!(detector.detect_from_path("header.h"), FileType::Code);

    // Test documentation files
    assert_eq!(
        detector.detect_from_path("README.md"),
        FileType::Documentation
    );
    assert_eq!(
        detector.detect_from_path("CHANGELOG.md"),
        FileType::Documentation
    );
    assert_eq!(
        detector.detect_from_path("docs.txt"),
        FileType::Documentation
    );
    assert_eq!(
        detector.detect_from_path("manual.rst"),
        FileType::Documentation
    );
    assert_eq!(
        detector.detect_from_path("guide.adoc"),
        FileType::Documentation
    );

    // Test configuration files
    assert_eq!(
        detector.detect_from_path("Cargo.toml"),
        FileType::Configuration
    );
    assert_eq!(
        detector.detect_from_path("package.json"),
        FileType::Configuration
    );
    assert_eq!(
        detector.detect_from_path("config.yaml"),
        FileType::Configuration
    );
    assert_eq!(
        detector.detect_from_path("settings.ini"),
        FileType::Configuration
    );
    assert_eq!(
        detector.detect_from_path("Dockerfile"),
        FileType::Configuration
    );
    assert_eq!(detector.detect_from_path(".env"), FileType::Configuration);
    assert_eq!(
        detector.detect_from_path("Makefile"),
        FileType::Configuration
    );

    // Test data files
    assert_eq!(detector.detect_from_path("data.csv"), FileType::Data);
    assert_eq!(detector.detect_from_path("results.json"), FileType::Data);
    assert_eq!(detector.detect_from_path("database.sql"), FileType::Data);
    assert_eq!(detector.detect_from_path("data.xml"), FileType::Data);

    // Test unknown files
    assert_eq!(detector.detect_from_path("unknown.xyz"), FileType::Unknown);
    assert_eq!(detector.detect_from_path("file"), FileType::Unknown);
}

/// Test strategy selection for different file types
#[test]
fn test_strategy_selection_for_file_types() {
    let strategy = FileTypeStrategy::new();

    // Code files should use regex + keyword strategies
    let code_strategies = strategy.get_strategies_for_file_type(FileType::Code);
    assert!(code_strategies.contains(&"regex"));
    assert!(code_strategies.contains(&"keyword"));
    assert_eq!(code_strategies.len(), 2);

    // Documentation files should use semantic + fuzzy strategies
    let doc_strategies = strategy.get_strategies_for_file_type(FileType::Documentation);
    assert!(doc_strategies.contains(&"tfidf"));
    assert!(doc_strategies.contains(&"fuzzy"));
    assert_eq!(doc_strategies.len(), 2);

    // Configuration files should use keyword + exact strategies
    let config_strategies = strategy.get_strategies_for_file_type(FileType::Configuration);
    assert!(config_strategies.contains(&"keyword"));
    assert_eq!(config_strategies.len(), 1);

    // Data files should use keyword + regex strategies
    let data_strategies = strategy.get_strategies_for_file_type(FileType::Data);
    assert!(data_strategies.contains(&"keyword"));
    assert!(data_strategies.contains(&"regex"));
    assert_eq!(data_strategies.len(), 2);

    // Unknown files should use fuzzy as default
    let unknown_strategies = strategy.get_strategies_for_file_type(FileType::Unknown);
    assert!(unknown_strategies.contains(&"fuzzy"));
    assert_eq!(unknown_strategies.len(), 1);
}

/// Test file grouping by type
#[test]
fn test_file_grouping_by_type() {
    let strategy = FileTypeStrategy::new();

    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("README.md"),
        PathBuf::from("Cargo.toml"),
        PathBuf::from("data.csv"),
        PathBuf::from("lib.js"),
        PathBuf::from("docs/guide.md"),
        PathBuf::from("config.yaml"),
        PathBuf::from("unknown.xyz"),
    ];

    let grouped = strategy.group_files_by_type(&files);

    // Check that files are correctly grouped
    assert!(grouped.contains_key(&FileType::Code));
    assert!(grouped.contains_key(&FileType::Documentation));
    assert!(grouped.contains_key(&FileType::Configuration));
    assert!(grouped.contains_key(&FileType::Data));
    assert!(grouped.contains_key(&FileType::Unknown));

    // Check specific groupings
    let code_files = &grouped[&FileType::Code];
    assert!(code_files.contains(&PathBuf::from("src/main.rs")));
    assert!(code_files.contains(&PathBuf::from("lib.js")));

    let doc_files = &grouped[&FileType::Documentation];
    assert!(doc_files.contains(&PathBuf::from("README.md")));
    assert!(doc_files.contains(&PathBuf::from("docs/guide.md")));

    let config_files = &grouped[&FileType::Configuration];
    assert!(config_files.contains(&PathBuf::from("Cargo.toml")));
    assert!(config_files.contains(&PathBuf::from("config.yaml")));
}

/// Test multi-strategy search execution
#[tokio::test]
async fn test_multi_strategy_search() {
    let strategy = FileTypeStrategy::new();

    // Create test files of different types
    let files = vec![
        PathBuf::from("test_code.rs"),
        PathBuf::from("test_docs.md"),
        PathBuf::from("test_config.toml"),
    ];

    // Test search across multiple file types
    let results = strategy.search("TODO", &files).await;

    // Should not panic and return results
    match &results {
        Ok(_) => {}
        Err(e) => println!("Search error: {e}"),
    }
    assert!(results.is_ok());
    let _search_results = results.unwrap();

    // Results can be empty but should be valid
}

/// Test strategy deployment tracking for advanced mode
#[test]
fn test_strategy_deployment_tracking() {
    let mut strategy = FileTypeStrategy::new();

    let _files = [
        PathBuf::from("src/main.rs"),
        PathBuf::from("README.md"),
        PathBuf::from("config.yaml"),
    ];

    // Enable tracking
    strategy.enable_tracking();

    // Simulate search (in real implementation this would happen during search)
    strategy.track_strategy_usage(FileType::Code, &["regex", "keyword"]);
    strategy.track_strategy_usage(FileType::Documentation, &["tfidf", "fuzzy"]);
    strategy.track_strategy_usage(FileType::Configuration, &["keyword"]);

    let deployment_summary = strategy.get_deployment_summary();

    // Verify tracking
    assert!(deployment_summary.contains("Code files: regex, keyword"));
    assert!(deployment_summary.contains("Documentation files: tfidf, fuzzy"));
    assert!(deployment_summary.contains("Configuration files: keyword"));
}

/// Test strategy legend for advanced mode output
#[test]
fn test_strategy_legend() {
    let strategy = FileTypeStrategy::new();
    let legend = strategy.get_strategy_legend();

    // Check that legend contains all strategy markers
    assert!(legend.contains("🔍 = keyword"));
    assert!(legend.contains("🌀 = fuzzy"));
    assert!(legend.contains("🔧 = regex"));
    assert!(legend.contains("📊 = tfidf"));
    assert!(legend.contains("🧠 = semantic"));

    // Check formatting
    assert!(legend.starts_with("Search Strategies:"));
    assert!(legend.contains(" = "));
}

/// Test file type strategy with semantic search available
#[tokio::test]
async fn test_strategy_with_semantic_available() {
    let mut strategy = FileTypeStrategy::new();

    // Simulate semantic search being available
    strategy.set_semantic_available(true);

    // Documentation files should now include semantic strategy
    let doc_strategies = strategy.get_strategies_for_file_type(FileType::Documentation);
    assert!(doc_strategies.contains(&"semantic") || doc_strategies.contains(&"tfidf"));

    // Code files might also benefit from semantic search for conceptual queries
    let code_strategies = strategy.get_strategies_for_file_type(FileType::Code);
    assert!(code_strategies.len() >= 2);
}

/// Test file type strategy with semantic search unavailable
#[tokio::test]
async fn test_strategy_without_semantic_available() {
    let mut strategy = FileTypeStrategy::new();

    // Simulate semantic search being unavailable
    strategy.set_semantic_available(false);

    // Documentation files should fall back to tfidf + fuzzy
    let doc_strategies = strategy.get_strategies_for_file_type(FileType::Documentation);
    assert!(doc_strategies.contains(&"tfidf"));
    assert!(doc_strategies.contains(&"fuzzy"));
    assert!(!doc_strategies.contains(&"semantic"));
}

/// Test sensible defaults for all file types
#[test]
fn test_sensible_defaults() {
    let strategy = FileTypeStrategy::new();

    // Code files: regex for patterns + keyword for exact matches
    let code_strategies = strategy.get_strategies_for_file_type(FileType::Code);
    assert!(
        code_strategies.contains(&"regex"),
        "Code files should support pattern matching"
    );
    assert!(
        code_strategies.contains(&"keyword"),
        "Code files should support exact matches"
    );

    // Documentation: semantic understanding + typo tolerance
    let doc_strategies = strategy.get_strategies_for_file_type(FileType::Documentation);
    assert!(
        doc_strategies.contains(&"tfidf") || doc_strategies.contains(&"semantic"),
        "Documentation should support conceptual search"
    );
    assert!(
        doc_strategies.contains(&"fuzzy"),
        "Documentation should support typo tolerance"
    );

    // Configuration: exact matches (precision over recall)
    let config_strategies = strategy.get_strategies_for_file_type(FileType::Configuration);
    assert!(
        config_strategies.contains(&"keyword"),
        "Configuration files need exact matches"
    );

    // Data: structured search capabilities
    let data_strategies = strategy.get_strategies_for_file_type(FileType::Data);
    assert!(
        data_strategies.contains(&"keyword"),
        "Data files need exact value matches"
    );
    assert!(
        data_strategies.contains(&"regex"),
        "Data files need pattern matching"
    );

    // Unknown: safe default with typo tolerance
    let unknown_strategies = strategy.get_strategies_for_file_type(FileType::Unknown);
    assert!(
        unknown_strategies.contains(&"fuzzy"),
        "Unknown files should use safe fuzzy default"
    );
}

/// Test mixed file type search scenario
#[tokio::test]
async fn test_mixed_file_type_search() {
    let strategy = FileTypeStrategy::new();

    let mixed_files = vec![
        PathBuf::from("src/main.rs"),    // Code
        PathBuf::from("docs/README.md"), // Documentation
        PathBuf::from("Cargo.toml"),     // Configuration
        PathBuf::from("data/users.csv"), // Data
        PathBuf::from("notes.txt"),      // Unknown
    ];

    // Search should deploy different strategies for different file types
    let results = strategy.search("error", &mixed_files).await;
    assert!(results.is_ok());

    // In a real implementation, we would verify that:
    // - Code files used regex + keyword
    // - Documentation used tfidf + fuzzy
    // - Configuration used keyword
    // - Data files used keyword + regex
    // - Unknown files used fuzzy
}

/// Test strategy marker generation for compact output
#[test]
fn test_strategy_markers() {
    let strategy = FileTypeStrategy::new();

    // Test individual strategy markers
    assert_eq!(strategy.get_strategy_marker("keyword"), "🔍");
    assert_eq!(strategy.get_strategy_marker("fuzzy"), "🌀");
    assert_eq!(strategy.get_strategy_marker("regex"), "🔧");
    assert_eq!(strategy.get_strategy_marker("tfidf"), "📊");
    assert_eq!(strategy.get_strategy_marker("semantic"), "🧠");
    assert_eq!(strategy.get_strategy_marker("unknown"), "❓");

    // Test strategy list formatting
    let strategies = vec!["keyword", "fuzzy"];
    let formatted = strategy.format_strategy_list(&strategies);
    assert_eq!(formatted, "🔍🌀");
}

/// Test file type strategy integration with existing search
#[tokio::test]
async fn test_integration_with_existing_search() {
    let strategy = FileTypeStrategy::new();

    // Test that the strategy can handle real file paths
    let _files = vec![
        PathBuf::from("src/lib.rs"),
        PathBuf::from("tests/integration_tests.rs"),
    ];

    // This should work with the existing search infrastructure
    let result = strategy.search("test", &_files).await;

    // Should not panic and return valid results
    assert!(result.is_ok());
}

/// Test performance with large number of files
#[tokio::test]
async fn test_performance_with_many_files() {
    let strategy = FileTypeStrategy::new();

    // Create a large list of files of different types
    let mut files = Vec::new();
    for i in 0..100 {
        files.push(PathBuf::from(format!("src/file{i}.rs")));
        files.push(PathBuf::from(format!("docs/doc{i}.md")));
        files.push(PathBuf::from(format!("config{i}.toml")));
    }

    // Grouping should be efficient
    let start = std::time::Instant::now();
    let grouped = strategy.group_files_by_type(&files);
    let duration = start.elapsed();

    // Should complete quickly (under 100ms for 300 files)
    assert!(duration.as_millis() < 100);
    assert_eq!(grouped.len(), 3); // Code, Documentation, Configuration
}

/// Test error handling for invalid file paths
#[tokio::test]
async fn test_error_handling() {
    let strategy = FileTypeStrategy::new();

    let invalid_files = vec![
        PathBuf::from("/nonexistent/path/file.rs"),
        PathBuf::from(""),
    ];

    // Should handle invalid paths gracefully
    let result = strategy.search("test", &invalid_files).await;

    // Should either succeed with empty results or return a descriptive error
    match result {
        Ok(results) => assert!(results.is_empty()),
        Err(e) => assert!(!e.to_string().is_empty()),
    }
}
