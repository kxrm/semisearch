use testlib::{config::Config, processor::FileProcessor, models::SearchResult};
use std::path::PathBuf;
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_search_in_file() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    
    // Write test content to file
    fs::write(&file_path, "Hello, world!\nThis is a test.\nTODO: Fix this.").await.unwrap();
    
    // Create processor with default config
    let config = Config::default();
    let processor = FileProcessor::new(config);
    
    // Search for "test"
    let results = processor.process_directory(&temp_dir, "test").await.unwrap();
    
    // Check results
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].line_number, 2);
    assert_eq!(results[0].content, "This is a test.");
    
    // Search for "TODO"
    let results = processor.process_directory(&temp_dir, "TODO").await.unwrap();
    
    // Check results
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].line_number, 3);
    assert_eq!(results[0].content, "TODO: Fix this.");
}

#[tokio::test]
async fn test_search_with_limit() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    
    // Write test content with multiple matches
    let content = (0..20).map(|i| format!("Line {}: test", i)).collect::<Vec<_>>().join("\n");
    fs::write(&file_path, content).await.unwrap();
    
    // Create processor with limit of 5
    let config = Config::new().with_limit(5);
    let processor = FileProcessor::new(config);
    
    // Search for "test"
    let results = processor.process_directory(&temp_dir, "test").await.unwrap();
    
    // Check results are limited
    assert_eq!(results.len(), 5);
}

#[tokio::test]
async fn test_search_with_extension_filter() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    
    // Create files with different extensions
    fs::write(temp_dir.path().join("test.txt"), "This is a test").await.unwrap();
    fs::write(temp_dir.path().join("test.rs"), "fn test() {}").await.unwrap();
    fs::write(temp_dir.path().join("test.md"), "# Test").await.unwrap();
    
    // Create processor that only includes .rs files
    let config = Config::new().with_include_extensions(vec!["rs".to_string()]);
    let processor = FileProcessor::new(config);
    
    // Search for "test"
    let results = processor.process_directory(&temp_dir, "test").await.unwrap();
    
    // Check only .rs file was included
    assert_eq!(results.len(), 1);
    assert!(results[0].path.to_string_lossy().ends_with(".rs"));
}

#[tokio::test]
async fn test_search_with_score_threshold() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    
    // Create a file with different quality matches
    let content = "This is a high quality test match.\ntest\nSomething else with test buried in it.";
    fs::write(temp_dir.path().join("test.txt"), content).await.unwrap();
    
    // Create processor with high score threshold
    let config = Config::new().with_score_threshold(0.7);
    let processor = FileProcessor::new(config);
    
    // Search for "test"
    let results = processor.process_directory(&temp_dir, "test").await.unwrap();
    
    // Check only high quality matches are included
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].content, "This is a high quality test match.");
} 