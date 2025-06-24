use semisearch::core::{FileIndexer, IndexerConfig};
use semisearch::storage::{Database, DatabaseStats};
use std::collections::HashSet;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn test_end_to_end_indexing_and_search() {
    let temp_db = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    fs::write(
        temp_dir.path().join("readme.md"),
        "# My Project\n\nThis is a test project with some content.\n\n## Features\n\n- Feature 1\n- Feature 2"
    ).unwrap();

    fs::write(
        temp_dir.path().join("main.rs"),
        "fn main() {\n    println!(\"Hello, world!\");\n    let x = 42;\n    println!(\"The answer is {}\", x);\n}"
    ).unwrap();

    fs::write(
        temp_dir.path().join("lib.rs"),
        "pub mod utils;\n\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_add() {\n        assert_eq!(add(2, 3), 5);\n    }\n}"
    ).unwrap();

    // Create database and indexer
    let database = Database::new(temp_db.path()).unwrap();
    let indexer = FileIndexer::new(database);

    // Index the directory
    let stats = indexer.index_directory(temp_dir.path()).unwrap();

    // Verify indexing results
    assert!(stats.files_processed > 0 || stats.files_updated > 0);
    assert!(stats.chunks_created > 0);
    assert!(stats.duration_seconds > 0.0);

    // Check database stats
    let db_stats = indexer.get_database_stats().unwrap();
    assert!(db_stats.file_count >= 3);
    assert!(db_stats.chunk_count > 0);
    assert!(db_stats.total_size_bytes > 0);
}

#[test]
fn test_incremental_indexing() {
    let temp_db = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    let database = Database::new(temp_db.path()).unwrap();
    let indexer = FileIndexer::new(database);

    // Create initial file
    fs::write(&test_file, "Initial content").unwrap();

    // First indexing
    let stats1 = indexer.index_directory(temp_dir.path()).unwrap();
    assert_eq!(stats1.files_processed + stats1.files_updated, 1);

    // Index again without changes - should not reprocess
    let stats2 = indexer.index_directory(temp_dir.path()).unwrap();
    assert_eq!(stats2.files_processed, 1); // File exists but no changes
    assert_eq!(stats2.files_updated, 0);

    // Modify file
    fs::write(&test_file, "Modified content").unwrap();

    // Index again - should detect change and reprocess
    let stats3 = indexer.index_directory(temp_dir.path()).unwrap();
    assert_eq!(stats3.files_updated, 1);
}

#[test]
fn test_file_exclusion_patterns() {
    let temp_db = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    // Create various types of files
    fs::write(temp_dir.path().join("source.rs"), "fn main() {}").unwrap();
    fs::write(temp_dir.path().join("document.txt"), "Some text").unwrap();
    fs::write(temp_dir.path().join("binary.exe"), "binary content").unwrap();
    fs::write(temp_dir.path().join("image.jpg"), "image data").unwrap();

    // Create excluded directory
    let node_modules = temp_dir.path().join("node_modules");
    fs::create_dir(&node_modules).unwrap();
    fs::write(node_modules.join("package.js"), "javascript code").unwrap();

    let database = Database::new(temp_db.path()).unwrap();
    let indexer = FileIndexer::new(database);

    let stats = indexer.index_directory(temp_dir.path()).unwrap();

    // Should only process .rs and .txt files, skip binary and excluded directory
    assert_eq!(stats.files_processed + stats.files_updated, 2);
    assert_eq!(stats.files_skipped, 0); // Files are filtered out, not skipped due to errors
}

#[test]
fn test_large_file_handling() {
    let temp_db = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let database = Database::new(temp_db.path()).unwrap();
    
    // Create config with very small file size limit
    let config = IndexerConfig {
        max_file_size_mb: 0, // 0 MB limit
        ..Default::default()
    };
    let indexer = FileIndexer::with_config(database, config);

    // Create a file that exceeds the limit
    fs::write(temp_dir.path().join("large.txt"), "This content exceeds the 0MB limit").unwrap();

    let stats = indexer.index_directory(temp_dir.path()).unwrap();

    // File should be skipped due to size
    assert_eq!(stats.files_processed + stats.files_updated, 0);
    assert_eq!(stats.files_skipped, 1);
    assert!(!stats.errors.is_empty());
}

#[test]
fn test_database_operations() {
    let temp_db = NamedTempFile::new().unwrap();
    let database = Database::new(temp_db.path()).unwrap();

    let now = chrono::Utc::now();

    // Test file insertion
    let file_id = database.insert_file("/test/file.txt", "hash123", now, 1024).unwrap();
    assert!(file_id > 0);

    // Test file retrieval
    let file_record = database.get_file_by_path("/test/file.txt").unwrap();
    assert!(file_record.is_some());
    let record = file_record.unwrap();
    assert_eq!(record.path, "/test/file.txt");
    assert_eq!(record.hash, "hash123");
    assert_eq!(record.size_bytes, 1024);

    // Test chunk insertion
    let chunk_id = database.insert_chunk(file_id, 1, 0, 12, "test content", None).unwrap();
    assert!(chunk_id > 0);

    // Test chunk retrieval
    let chunks = database.get_chunks_for_file(file_id).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].content, "test content");
    assert_eq!(chunks[0].line_number, 1);

    // Test search functionality
    let search_results = database.search_chunks("test", 10).unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].content, "test content");

    // Test reindexing check
    assert!(!database.needs_reindexing("/test/file.txt", "hash123").unwrap()); // Same hash
    assert!(database.needs_reindexing("/test/file.txt", "different_hash").unwrap()); // Different hash
    assert!(database.needs_reindexing("/test/nonexistent.txt", "any_hash").unwrap()); // New file
}

#[test]
fn test_database_stats() {
    let temp_db = NamedTempFile::new().unwrap();
    let database = Database::new(temp_db.path()).unwrap();

    // Initially empty
    let stats = database.get_stats().unwrap();
    assert_eq!(stats.file_count, 0);
    assert_eq!(stats.chunk_count, 0);
    assert_eq!(stats.total_size_bytes, 0);

    // Add some data
    let now = chrono::Utc::now();
    let file_id1 = database.insert_file("/test/file1.txt", "hash1", now, 500).unwrap();
    let file_id2 = database.insert_file("/test/file2.txt", "hash2", now, 750).unwrap();
    
    database.insert_chunk(file_id1, 1, 0, 10, "content 1", None).unwrap();
    database.insert_chunk(file_id1, 2, 11, 20, "content 2", None).unwrap();
    database.insert_chunk(file_id2, 1, 0, 10, "content 3", None).unwrap();

    let stats = database.get_stats().unwrap();
    assert_eq!(stats.file_count, 2);
    assert_eq!(stats.chunk_count, 3);
    assert_eq!(stats.total_size_bytes, 1250);
}

#[test]
fn test_custom_indexer_config() {
    let temp_db = NamedTempFile::new().unwrap();
    let database = Database::new(temp_db.path()).unwrap();

    // Create custom configuration
    let mut excluded_extensions = HashSet::new();
    excluded_extensions.insert("tmp".to_string());
    excluded_extensions.insert("log".to_string());

    let mut excluded_directories = HashSet::new();
    excluded_directories.insert("temp".to_string());

    let config = IndexerConfig {
        max_file_size_mb: 10,
        excluded_extensions,
        excluded_directories,
        chunk_size: 256,
        enable_embeddings: false,
    };

    let indexer = FileIndexer::with_config(database, config);
    
    // Verify configuration is applied
    assert_eq!(indexer.config().max_file_size_mb, 10);
    assert!(indexer.config().excluded_extensions.contains("tmp"));
    assert!(indexer.config().excluded_directories.contains("temp"));
    assert_eq!(indexer.config().chunk_size, 256);
}

#[test]
fn test_file_removal() {
    let temp_db = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    fs::write(&test_file, "Test content for removal").unwrap();

    let database = Database::new(temp_db.path()).unwrap();
    let indexer = FileIndexer::new(database);

    // Index the file
    indexer.index_directory(temp_dir.path()).unwrap();

    // Verify file is indexed
    assert!(indexer.is_file_indexed(&test_file).unwrap());

    let stats_before = indexer.get_database_stats().unwrap();
    assert!(stats_before.file_count > 0);

    // Remove file from index
    indexer.remove_file(&test_file).unwrap();

    // Verify file is no longer indexed
    assert!(!indexer.is_file_indexed(&test_file).unwrap());

    let stats_after = indexer.get_database_stats().unwrap();
    assert_eq!(stats_after.file_count, stats_before.file_count - 1);
}

#[test]
fn test_text_processing_integration() {
    let temp_db = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    // Create a file with various text patterns
    let content = r#"
# Project Documentation

This is a **markdown** file with various content types.

## Code Example

```rust
fn hello_world() {
    println!("Hello, World!");
}
```

## List of Items

1. First item
2. Second item
3. Third item

Some regular text with common words like 'the', 'a', 'and' that should be filtered.
"#;

    fs::write(temp_dir.path().join("doc.md"), content).unwrap();

    let database = Database::new(temp_db.path()).unwrap();
    let indexer = FileIndexer::new(database);

    let stats = indexer.index_directory(temp_dir.path()).unwrap();

    assert!(stats.chunks_created > 0);

    // Check that the database contains processed chunks
    let db_stats = indexer.get_database_stats().unwrap();
    assert!(db_stats.chunk_count > 0);

    // Verify we can search for content by creating a new database connection
    let database2 = Database::new(temp_db.path()).unwrap();
    let search_results = database2.search_chunks("markdown", 10).unwrap();
    assert!(!search_results.is_empty());

    let search_results = database2.search_chunks("hello_world", 10).unwrap();
    assert!(!search_results.is_empty());
} 