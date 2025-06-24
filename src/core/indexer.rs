use crate::core::LocalEmbedder;
use crate::storage::{Database, DatabaseStats};
use crate::text::TextProcessor;
use anyhow::Result;
use chrono::DateTime;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// File indexer for processing and storing file content
pub struct FileIndexer {
    database: Database,
    text_processor: TextProcessor,
    config: IndexerConfig,
    embedder: Option<LocalEmbedder>,
}

/// Configuration for the indexer
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    pub max_file_size_mb: u64,
    pub excluded_extensions: HashSet<String>,
    pub excluded_directories: HashSet<String>,
    pub chunk_size: usize,
    pub enable_embeddings: bool,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        let excluded_extensions: HashSet<String> = [
            "exe", "dll", "so", "dylib", "bin", "obj", "o", "a", "lib", "zip", "tar", "gz", "bz2",
            "7z", "rar", "jpg", "jpeg", "png", "gif", "bmp", "tiff", "svg", "mp3", "mp4", "avi",
            "mov", "wav", "flac", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        ]
        .iter()
        .map(|&s| s.to_string())
        .collect();

        let excluded_directories: HashSet<String> = [
            ".git",
            ".svn",
            ".hg",
            "node_modules",
            ".venv",
            "venv",
            "__pycache__",
            "target",
            "build",
            "dist",
            ".aws",
            ".ssh",
            ".gnupg",
            ".cargo",
            ".rustup",
        ]
        .iter()
        .map(|&s| s.to_string())
        .collect();

        Self {
            max_file_size_mb: 50,
            excluded_extensions,
            excluded_directories,
            chunk_size: 512,
            enable_embeddings: false, // Phase 2 doesn't include ML embeddings yet
        }
    }
}

/// Statistics from indexing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub files_processed: usize,
    pub files_skipped: usize,
    pub files_updated: usize,
    pub chunks_created: usize,
    pub total_size_bytes: u64,
    pub duration_seconds: f64,
    pub errors: Vec<String>,
}

impl Default for IndexStats {
    fn default() -> Self {
        Self {
            files_processed: 0,
            files_skipped: 0,
            files_updated: 0,
            chunks_created: 0,
            total_size_bytes: 0,
            duration_seconds: 0.0,
            errors: Vec::new(),
        }
    }
}

impl FileIndexer {
    /// Create a new file indexer
    pub fn new(database: Database) -> Self {
        Self {
            database,
            text_processor: TextProcessor::new(),
            config: IndexerConfig::default(),
            embedder: None,
        }
    }

    /// Create indexer with custom configuration
    pub fn with_config(database: Database, config: IndexerConfig) -> Self {
        Self {
            database,
            text_processor: TextProcessor::with_config(config.chunk_size, config.chunk_size * 2),
            config,
            embedder: None,
        }
    }

    /// Create indexer with embeddings support
    pub fn with_embedder(
        database: Database,
        config: IndexerConfig,
        embedder: LocalEmbedder,
    ) -> Self {
        Self {
            database,
            text_processor: TextProcessor::with_config(config.chunk_size, config.chunk_size * 2),
            config,
            embedder: Some(embedder),
        }
    }

    /// Index a directory recursively
    pub fn index_directory(&self, path: &Path) -> Result<IndexStats> {
        let start_time = std::time::Instant::now();
        let mut stats = IndexStats::default();

        println!("Indexing directory: {}", path.display());

        // Create thread-safe filter criteria
        let excluded_dirs = self.config.excluded_directories.clone();
        let excluded_exts = self.config.excluded_extensions.clone();

        let walker = WalkBuilder::new(path)
            .follow_links(false)
            .git_ignore(true)
            .filter_entry(move |entry| {
                Self::should_include_entry_static(entry, &excluded_dirs, &excluded_exts)
            })
            .build();

        for entry in walker {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_some_and(|ft| ft.is_file()) {
                        match self.process_file(entry.path()) {
                            Ok(file_stats) => {
                                if file_stats.was_updated {
                                    stats.files_updated += 1;
                                } else {
                                    stats.files_processed += 1;
                                }
                                stats.chunks_created += file_stats.chunks_created;
                                stats.total_size_bytes += file_stats.size_bytes;
                            }
                            Err(e) => {
                                stats.files_skipped += 1;
                                stats
                                    .errors
                                    .push(format!("{}: {e}", entry.path().display()));
                                eprintln!("Error processing {}: {e}", entry.path().display());
                            }
                        }
                    }
                }
                Err(e) => {
                    stats.errors.push(format!("Walk error: {e}"));
                    eprintln!("Walk error: {e}");
                }
            }
        }

        stats.duration_seconds = start_time.elapsed().as_secs_f64();

        println!("Indexing complete:");
        println!("  Files processed: {}", stats.files_processed);
        println!("  Files updated: {}", stats.files_updated);
        println!("  Files skipped: {}", stats.files_skipped);
        println!("  Chunks created: {}", stats.chunks_created);
        println!(
            "  Total size: {} MB",
            stats.total_size_bytes / (1024 * 1024)
        );
        println!("  Duration: {:.2}s", stats.duration_seconds);

        if !stats.errors.is_empty() {
            println!("  Errors: {}", stats.errors.len());
        }

        Ok(stats)
    }

    /// Process a single file
    fn process_file(&self, path: &Path) -> Result<FileProcessingStats> {
        // Check file size
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();

        if file_size > self.config.max_file_size_mb * 1024 * 1024 {
            return Err(anyhow::anyhow!(
                "File too large: {} MB",
                file_size / (1024 * 1024)
            ));
        }

        // Read file content
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Could not read as UTF-8: {}", e))?;

        // Calculate file hash
        let file_hash = self.calculate_file_hash(&content);
        let path_str = path.to_string_lossy().to_string();

        // Check if file needs reindexing
        if !self.database.needs_reindexing(&path_str, &file_hash)? {
            return Ok(FileProcessingStats {
                chunks_created: 0,
                size_bytes: file_size,
                was_updated: false,
            });
        }

        // Get file modification time
        let modified_at = DateTime::from(metadata.modified()?);

        // Insert or update file record
        let file_id =
            self.database
                .insert_file(&path_str, &file_hash, modified_at, file_size as i64)?;

        // Process text into chunks
        let chunks = self.text_processor.process_file(&content);

        // Store chunks in database
        for chunk in &chunks {
            self.database.insert_chunk(
                file_id,
                chunk.line_number,
                chunk.start_char,
                chunk.end_char,
                &chunk.content,
                None, // No embeddings in Phase 2
            )?;
        }

        Ok(FileProcessingStats {
            chunks_created: chunks.len(),
            size_bytes: file_size,
            was_updated: true,
        })
    }

    /// Check if a directory entry should be included (static version for thread safety)
    fn should_include_entry_static(
        entry: &ignore::DirEntry,
        excluded_dirs: &HashSet<String>,
        excluded_exts: &HashSet<String>,
    ) -> bool {
        // Skip excluded directories
        if let Some(file_name) = entry.file_name().to_str() {
            if excluded_dirs.contains(file_name) {
                return false;
            }
        }

        // For files, check extension
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            if let Some(extension) = entry.path().extension() {
                if let Some(ext_str) = extension.to_str() {
                    if excluded_exts.contains(&ext_str.to_lowercase()) {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Check if a directory entry should be included
    #[allow(dead_code)]
    fn should_include_entry(&self, entry: &ignore::DirEntry) -> bool {
        Self::should_include_entry_static(
            entry,
            &self.config.excluded_directories,
            &self.config.excluded_extensions,
        )
    }

    /// Calculate SHA-256 hash of file content
    fn calculate_file_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get indexer configuration
    pub fn config(&self) -> &IndexerConfig {
        &self.config
    }

    /// Get database statistics
    pub fn get_database_stats(&self) -> Result<DatabaseStats> {
        self.database.get_stats()
    }

    /// Remove a file from the index
    pub fn remove_file(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        self.database.remove_file(&path_str)
    }

    /// Check if a file is indexed
    pub fn is_file_indexed(&self, path: &Path) -> Result<bool> {
        let path_str = path.to_string_lossy().to_string();
        Ok(self.database.get_file_by_path(&path_str)?.is_some())
    }
}

/// Statistics from processing a single file
#[derive(Debug)]
struct FileProcessingStats {
    chunks_created: usize,
    size_bytes: u64,
    was_updated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Database;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    fn create_test_indexer() -> (FileIndexer, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let database = Database::new(temp_file.path()).unwrap();
        let indexer = FileIndexer::new(database);
        (indexer, temp_file)
    }

    #[test]
    fn test_indexer_creation() {
        let (indexer, _temp_file) = create_test_indexer();
        assert_eq!(indexer.config.max_file_size_mb, 50);
        assert!(indexer.config.excluded_extensions.contains("exe"));
        assert!(indexer.config.excluded_directories.contains(".git"));
    }

    #[test]
    fn test_file_hash_calculation() {
        let (indexer, _temp_file) = create_test_indexer();

        let hash1 = indexer.calculate_file_hash("test content");
        let hash2 = indexer.calculate_file_hash("test content");
        let hash3 = indexer.calculate_file_hash("different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex characters
    }

    #[test]
    fn test_process_single_file() {
        let (indexer, _temp_file) = create_test_indexer();
        let temp_dir = TempDir::new().unwrap();

        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(
            &test_file,
            "Hello world\nThis is a test\nWith multiple lines",
        )
        .unwrap();

        // Process the file
        let stats = indexer.process_file(&test_file).unwrap();
        assert!(stats.was_updated);
        assert!(stats.chunks_created > 0);
        assert!(stats.size_bytes > 0);

        // Process again - should not update
        let stats2 = indexer.process_file(&test_file).unwrap();
        assert!(!stats2.was_updated);
        assert_eq!(stats2.chunks_created, 0);
    }

    #[test]
    fn test_index_directory() {
        let (indexer, _temp_file) = create_test_indexer();
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        fs::write(temp_dir.path().join("file1.txt"), "Content of file 1").unwrap();
        fs::write(
            temp_dir.path().join("file2.rs"),
            "fn main() { println!(\"Hello\"); }",
        )
        .unwrap();

        // Create excluded directory
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        fs::write(git_dir.join("config"), "git config").unwrap();

        // Index directory
        let stats = indexer.index_directory(temp_dir.path()).unwrap();

        assert_eq!(stats.files_processed + stats.files_updated, 2); // Should skip .git/config
        assert_eq!(stats.files_skipped, 0);
        assert!(stats.chunks_created > 0);
        assert!(stats.duration_seconds > 0.0);
    }

    #[test]
    fn test_excluded_extensions() {
        let (indexer, _temp_file) = create_test_indexer();
        let temp_dir = TempDir::new().unwrap();

        // Create files with different extensions
        fs::write(temp_dir.path().join("text.txt"), "Text file").unwrap();
        fs::write(temp_dir.path().join("binary.exe"), "Binary file").unwrap();
        fs::write(temp_dir.path().join("image.jpg"), "Image file").unwrap();

        let stats = indexer.index_directory(temp_dir.path()).unwrap();

        // Should only process the .txt file
        assert_eq!(stats.files_processed + stats.files_updated, 1);
    }

    #[test]
    fn test_large_file_skipping() {
        let temp_file = NamedTempFile::new().unwrap();
        let database = Database::new(temp_file.path()).unwrap();

        // Create config with very small max file size
        let config = IndexerConfig {
            max_file_size_mb: 0, // 0 MB limit
            ..Default::default()
        };
        let indexer = FileIndexer::with_config(database, config);

        let temp_dir = TempDir::new().unwrap();
        let large_file = temp_dir.path().join("large.txt");
        fs::write(&large_file, "This file is too large").unwrap();

        // Should fail to process due to size limit
        let result = indexer.process_file(&large_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_database_integration() {
        let (indexer, _temp_file) = create_test_indexer();
        let temp_dir = TempDir::new().unwrap();

        // Initially empty
        let stats = indexer.get_database_stats().unwrap();
        assert_eq!(stats.file_count, 0);

        // Create and index a file
        fs::write(temp_dir.path().join("test.txt"), "Test content").unwrap();
        indexer.index_directory(temp_dir.path()).unwrap();

        // Should have data
        let stats = indexer.get_database_stats().unwrap();
        assert!(stats.file_count > 0);
        assert!(stats.chunk_count > 0);
    }

    #[test]
    fn test_file_removal() {
        let (indexer, _temp_file) = create_test_indexer();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // Create and index file
        fs::write(&test_file, "Test content").unwrap();
        indexer.index_directory(temp_dir.path()).unwrap();

        assert!(indexer.is_file_indexed(&test_file).unwrap());

        // Remove from index
        indexer.remove_file(&test_file).unwrap();
        assert!(!indexer.is_file_indexed(&test_file).unwrap());
    }
}
