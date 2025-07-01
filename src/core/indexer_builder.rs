use crate::core::embedder::LocalEmbedder;
use crate::core::indexer::{FileIndexer, IndexerConfig};
use crate::storage::Database;
use crate::text::TextProcessor;
use anyhow::Result;

/// Builder for FileIndexer following the Builder pattern to avoid multiple constructors
pub struct FileIndexerBuilder {
    database: Option<Database>,
    config: Option<IndexerConfig>,
    embedder: Option<LocalEmbedder>,
    advanced_mode: bool,
    text_processor_config: Option<(usize, usize)>, // (min_chunk_length, max_chunk_length)
}

impl FileIndexerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            database: None,
            config: None,
            embedder: None,
            advanced_mode: false,
            text_processor_config: None,
        }
    }

    /// Set the database (required)
    pub fn with_database(mut self, database: Database) -> Self {
        self.database = Some(database);
        self
    }

    /// Set custom configuration
    pub fn with_config(mut self, config: IndexerConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set embedder for semantic indexing
    pub fn with_embedder(mut self, embedder: LocalEmbedder) -> Self {
        self.embedder = Some(embedder);
        self
    }

    /// Enable advanced mode for detailed progress reporting
    pub fn with_advanced_mode(mut self, advanced_mode: bool) -> Self {
        self.advanced_mode = advanced_mode;
        self
    }

    /// Set text processor configuration
    pub fn with_text_processor_config(
        mut self,
        min_chunk_length: usize,
        max_chunk_length: usize,
    ) -> Self {
        self.text_processor_config = Some((min_chunk_length, max_chunk_length));
        self
    }

    /// Auto-detect and configure embeddings based on system capabilities
    pub async fn with_auto_embeddings(mut self) -> Result<Self> {
        match LocalEmbedder::with_auto_config().await {
            Ok(embedder) => {
                if self.advanced_mode {
                    eprintln!(
                        "📊 Indexer: Embeddings enabled ({:?})",
                        embedder.capability()
                    );
                }
                self.embedder = Some(embedder);

                // Update config to enable embeddings
                if let Some(ref mut config) = self.config {
                    config.enable_embeddings = true;
                } else {
                    let config = IndexerConfig {
                        enable_embeddings: true,
                        ..IndexerConfig::default()
                    };
                    self.config = Some(config);
                }
                Ok(self)
            }
            Err(e) => {
                if self.advanced_mode {
                    eprintln!("📊 Indexer: Embeddings disabled ({e})");
                }
                // Update config to disable embeddings
                if let Some(ref mut config) = self.config {
                    config.enable_embeddings = false;
                } else {
                    let config = IndexerConfig {
                        enable_embeddings: false,
                        ..IndexerConfig::default()
                    };
                    self.config = Some(config);
                }
                Ok(self)
            }
        }
    }

    /// Build the FileIndexer
    pub fn build(self) -> Result<FileIndexer> {
        let database = self
            .database
            .ok_or_else(|| anyhow::anyhow!("Database is required"))?;

        let config = self.config.unwrap_or_default();

        let text_processor = if let Some((min, max)) = self.text_processor_config {
            TextProcessor::with_config(min, max)
        } else {
            TextProcessor::with_config(10, 1000) // Reasonable defaults
        };

        Ok(FileIndexer::from_components(
            database,
            text_processor,
            config,
            self.embedder,
            self.advanced_mode,
        ))
    }
}

impl Default for FileIndexerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Database;
    use tempfile::NamedTempFile;

    fn create_test_database() -> (Database, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let database = Database::new(temp_file.path()).unwrap();
        (database, temp_file)
    }

    #[test]
    fn test_builder_basic_usage() {
        let (database, _temp_file) = create_test_database();

        let indexer = FileIndexerBuilder::new()
            .with_database(database)
            .build()
            .unwrap();

        assert_eq!(indexer.config().max_file_size_mb, 50);
        assert!(!indexer.is_advanced_mode());
    }

    #[test]
    fn test_builder_with_config() {
        let (database, _temp_file) = create_test_database();
        let config = IndexerConfig {
            max_file_size_mb: 100,
            ..IndexerConfig::default()
        };

        let indexer = FileIndexerBuilder::new()
            .with_database(database)
            .with_config(config)
            .build()
            .unwrap();

        assert_eq!(indexer.config().max_file_size_mb, 100);
    }

    #[test]
    fn test_builder_with_advanced_mode() {
        let (database, _temp_file) = create_test_database();

        let indexer = FileIndexerBuilder::new()
            .with_database(database)
            .with_advanced_mode(true)
            .build()
            .unwrap();

        assert!(indexer.is_advanced_mode());
    }

    #[test]
    fn test_builder_missing_database() {
        let result = FileIndexerBuilder::new().build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Database is required"));
    }

    #[tokio::test]
    async fn test_builder_with_auto_embeddings() {
        let (database, _temp_file) = create_test_database();

        // This test may pass or fail depending on system capabilities
        let result = FileIndexerBuilder::new()
            .with_database(database)
            .with_auto_embeddings()
            .await;

        // Should not panic regardless of embedding availability
        assert!(result.is_ok());

        let indexer = result.unwrap().build().unwrap();
        // Can't assert specific embedding state since it depends on system
        assert_eq!(indexer.config().max_file_size_mb, 50);
    }
}
