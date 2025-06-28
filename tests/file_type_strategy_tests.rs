use search::search::file_type_strategy::{FileType, FileTypeStrategy};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_file_type_detection() {
    let strategy = FileTypeStrategy::new();

    // Test code file detection
    assert_eq!(
        strategy.detect_file_type(&PathBuf::from("test.rs")),
        FileType::Code
    );
    assert_eq!(
        strategy.detect_file_type(&PathBuf::from("test.py")),
        FileType::Code
    );
    assert_eq!(
        strategy.detect_file_type(&PathBuf::from("test.js")),
        FileType::Code
    );

    // Test documentation file detection
    assert_eq!(
        strategy.detect_file_type(&PathBuf::from("README.md")),
        FileType::Documentation
    );
    assert_eq!(
        strategy.detect_file_type(&PathBuf::from("README.txt")),
        FileType::Documentation
    );

    // Test configuration file detection
    assert_eq!(
        strategy.detect_file_type(&PathBuf::from("config.json")),
        FileType::Configuration
    );
    assert_eq!(
        strategy.detect_file_type(&PathBuf::from("settings.yaml")),
        FileType::Configuration
    );
    assert_eq!(
        strategy.detect_file_type(&PathBuf::from(".env")),
        FileType::Configuration
    );

    // Test data file detection
    assert_eq!(
        strategy.detect_file_type(&PathBuf::from("data.csv")),
        FileType::Data
    );
    assert_eq!(
        strategy.detect_file_type(&PathBuf::from("dataset.tsv")),
        FileType::Data
    );
}

#[test]
fn test_group_files_by_type() {
    let strategy = FileTypeStrategy::new();

    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("README.md"),
        PathBuf::from("config.json"),
        PathBuf::from("data.csv"),
    ];

    let grouped = strategy.group_files_by_type(&files);

    assert_eq!(grouped.len(), 4); // Should have 4 file types
    assert!(grouped.contains_key(&FileType::Code));
    assert!(grouped.contains_key(&FileType::Documentation));
    assert!(grouped.contains_key(&FileType::Configuration));
    assert!(grouped.contains_key(&FileType::Data));
}

#[tokio::test]
async fn test_search_with_file_types() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files with different types
    let code_file = temp_dir.path().join("test.rs");
    fs::write(
        &code_file,
        "fn main() { println!(\"TODO: implement this\"); }",
    )
    .unwrap();

    let doc_file = temp_dir.path().join("README.md");
    fs::write(&doc_file, "# Documentation\n\nTODO: Write documentation").unwrap();

    let config_file = temp_dir.path().join("config.json");
    fs::write(&config_file, "{\"todo\": \"Configure this\"}").unwrap();

    // Create strategy and perform search
    let strategy = FileTypeStrategy::new();
    let files = vec![code_file.clone(), doc_file.clone(), config_file.clone()];

    let results = strategy.search("TODO", &files).await.unwrap();

    // Should find matches in all files, but with different strategies applied
    assert!(!results.is_empty());

    // Check that we have results from different file types
    let has_code_result = results.iter().any(|r| r.file_path.ends_with("test.rs"));
    let has_doc_result = results.iter().any(|r| r.file_path.ends_with("README.md"));

    assert!(has_code_result);
    assert!(has_doc_result);
}

#[test]
fn test_strategy_selection_by_file_type() {
    let strategy = FileTypeStrategy::new();

    // Test that each file type has an appropriate strategy
    assert!(strategy.strategies.contains_key(&FileType::Code));
    assert!(strategy.strategies.contains_key(&FileType::Documentation));
    assert!(strategy.strategies.contains_key(&FileType::Configuration));
    assert!(strategy.strategies.contains_key(&FileType::Data));
}
