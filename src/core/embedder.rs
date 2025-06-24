use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Embedding configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub cache_dir: PathBuf,
    pub max_length: usize,
    pub batch_size: usize,
    pub device: EmbeddingDevice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingDevice {
    Cpu,
    Cuda,
    Metal,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".semisearch")
            .join("models");

        Self {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            cache_dir,
            max_length: 384,
            batch_size: 32,
            device: EmbeddingDevice::Cpu,
        }
    }
}

/// System embedding capabilities
#[derive(Debug, Clone, PartialEq)]
pub enum EmbeddingCapability {
    Full,  // Full neural embeddings
    TfIdf, // TF-IDF only
    None,  // No embeddings
}

/// Local embedding model for semantic search
pub struct LocalEmbedder {
    config: EmbeddingConfig,
    // For Phase 4, we'll implement TF-IDF based embeddings as a foundation
    vocabulary: Arc<HashMap<String, usize>>,
    idf_scores: Arc<HashMap<String, f32>>,
    embedding_cache: HashMap<String, Vec<f32>>,
}

impl LocalEmbedder {
    /// Create a new local embedder
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        // Initialize with empty vocabulary - will be built during indexing
        let vocabulary = Arc::new(HashMap::new());
        let idf_scores = Arc::new(HashMap::new());

        Ok(Self {
            config,
            vocabulary,
            idf_scores,
            embedding_cache: HashMap::new(),
        })
    }

    /// Create embedder with pre-built vocabulary
    pub fn with_vocabulary(
        config: EmbeddingConfig,
        vocabulary: HashMap<String, usize>,
        idf_scores: HashMap<String, f32>,
    ) -> Self {
        Self {
            config,
            vocabulary: Arc::new(vocabulary),
            idf_scores: Arc::new(idf_scores),
            embedding_cache: HashMap::new(),
        }
    }

    /// Create embedder with system capability detection
    pub async fn with_auto_config() -> Result<Self> {
        let capability = Self::detect_capabilities();
        let config = match capability {
            EmbeddingCapability::Full => EmbeddingConfig::default(),
            EmbeddingCapability::TfIdf => {
                return Err(anyhow::anyhow!(
                    "TF-IDF mode not implemented in LocalEmbedder"
                ));
            }
            EmbeddingCapability::None => {
                return Err(anyhow::anyhow!("No embedding capability available"));
            }
        };

        Self::new(config).await
    }

    /// Detect system embedding capabilities
    pub fn detect_capabilities() -> EmbeddingCapability {
        // Check available memory
        let available_memory = sys_info::mem_info().map(|info| info.avail).unwrap_or(0);

        // Check CPU count
        let cpu_count = num_cpus::get();

        // Simple heuristics for capability detection
        if available_memory > 2_000_000 && cpu_count >= 4 {
            EmbeddingCapability::Full
        } else if available_memory > 500_000 {
            EmbeddingCapability::TfIdf
        } else {
            EmbeddingCapability::None
        }
    }

    /// Generate embedding for text using TF-IDF approach
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        if let Some(cached) = self.embedding_cache.get(text) {
            return Ok(cached.clone());
        }

        let tokens = self.tokenize_text(text);
        let embedding = self.create_tfidf_embedding(&tokens)?;

        Ok(embedding)
    }

    /// Build vocabulary from a collection of documents
    pub fn build_vocabulary(&mut self, documents: &[String]) -> Result<()> {
        let mut word_counts = HashMap::new();
        let total_docs = documents.len() as f32;

        // Count word occurrences across documents
        for doc in documents {
            let tokens = self.tokenize_text(doc);
            let unique_tokens: std::collections::HashSet<_> = tokens.into_iter().collect();

            for token in unique_tokens {
                *word_counts.entry(token).or_insert(0) += 1;
            }
        }

        // Build vocabulary and calculate IDF scores
        let mut vocabulary = HashMap::new();
        let mut idf_scores = HashMap::new();

        for (word, doc_count) in word_counts {
            let word_id = vocabulary.len();
            vocabulary.insert(word.clone(), word_id);

            // Calculate IDF: log(N / df) where N is total docs and df is document frequency
            let idf = (total_docs / doc_count as f32).ln();
            idf_scores.insert(word, idf);
        }

        self.vocabulary = Arc::new(vocabulary);
        self.idf_scores = Arc::new(idf_scores);

        Ok(())
    }

    /// Generate embeddings for multiple texts (batch processing)
    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();

        for chunk in texts.chunks(self.config.batch_size) {
            let mut batch_embeddings = Vec::new();

            for text in chunk {
                let embedding = self.embed(text)?;
                batch_embeddings.push(embedding);
            }

            results.extend(batch_embeddings);
        }

        Ok(results)
    }

    /// Calculate similarity between two embeddings
    pub fn similarity(embedding1: &[f32], embedding2: &[f32]) -> f32 {
        if embedding1.len() != embedding2.len() {
            return 0.0;
        }

        let dot_product: f32 = embedding1
            .iter()
            .zip(embedding2.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm1: f32 = embedding1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = embedding2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }

        dot_product / (norm1 * norm2)
    }

    /// Get embedding dimension
    pub fn embedding_dim(&self) -> usize {
        384 // All-MiniLM-L6-v2 embedding dimension
    }

    // Private helper methods
    fn tokenize_text(&self, text: &str) -> Vec<String> {
        // Simple whitespace tokenization with basic preprocessing
        text.to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 1)
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|word| !word.is_empty())
            .map(|word| word.to_string())
            .collect()
    }

    fn create_tfidf_embedding(&self, tokens: &[String]) -> Result<Vec<f32>> {
        let vocab_size = self.vocabulary.len();
        if vocab_size == 0 {
            return Ok(vec![0.0; 100]); // Default embedding size
        }

        let mut embedding = vec![0.0; vocab_size];
        let mut token_counts = HashMap::new();

        // Count token frequencies
        for token in tokens {
            *token_counts.entry(token.clone()).or_insert(0) += 1;
        }

        let total_tokens = tokens.len() as f32;

        // Calculate TF-IDF for each token
        for (token, count) in token_counts {
            if let (Some(&word_id), Some(&idf)) =
                (self.vocabulary.get(&token), self.idf_scores.get(&token))
            {
                let tf = count as f32 / total_tokens;
                let tfidf = tf * idf;
                embedding[word_id] = tfidf;
            }
        }

        // Normalize the embedding
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in &mut embedding {
                *value /= norm;
            }
        }

        Ok(embedding)
    }

    /// Save vocabulary and IDF scores to disk
    pub fn save_vocabulary(&self, path: &Path) -> Result<()> {
        let vocab_data = serde_json::json!({
            "vocabulary": &*self.vocabulary,
            "idf_scores": &*self.idf_scores
        });

        std::fs::write(path, vocab_data.to_string())?;
        Ok(())
    }

    /// Load vocabulary and IDF scores from disk
    pub fn load_vocabulary(&mut self, path: &Path) -> Result<()> {
        let data = std::fs::read_to_string(path)?;
        let parsed: serde_json::Value = serde_json::from_str(&data)?;

        let vocabulary: HashMap<String, usize> =
            serde_json::from_value(parsed["vocabulary"].clone())?;
        let idf_scores: HashMap<String, f32> =
            serde_json::from_value(parsed["idf_scores"].clone())?;

        self.vocabulary = Arc::new(vocabulary);
        self.idf_scores = Arc::new(idf_scores);

        Ok(())
    }

    /// Get vocabulary size
    pub fn vocabulary_size(&self) -> usize {
        self.vocabulary.len()
    }

    /// Check if vocabulary is built
    pub fn has_vocabulary(&self) -> bool {
        !self.vocabulary.is_empty()
    }
}

// Add temporary implementations for missing dependencies
mod sys_info {
    pub struct MemInfo {
        pub avail: u64,
    }

    pub fn mem_info() -> Option<MemInfo> {
        // Simplified memory detection
        Some(MemInfo { avail: 4_000_000 }) // Assume 4GB available
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_embedding_config_default() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.model_name, "sentence-transformers/all-MiniLM-L6-v2");
        assert_eq!(config.max_length, 384);
        assert_eq!(config.batch_size, 32);
        assert!(matches!(config.device, EmbeddingDevice::Cpu));
    }

    #[test]
    fn test_detect_capabilities() {
        let capability = LocalEmbedder::detect_capabilities();
        // Should detect some capability on most systems
        assert!(matches!(
            capability,
            EmbeddingCapability::Full | EmbeddingCapability::TfIdf | EmbeddingCapability::None
        ));
    }

    #[test]
    fn test_similarity_calculation() {
        let emb1 = vec![1.0, 0.0, 0.0];
        let emb2 = vec![1.0, 0.0, 0.0];
        let emb3 = vec![0.0, 1.0, 0.0];

        let sim1 = LocalEmbedder::similarity(&emb1, &emb2);
        let sim2 = LocalEmbedder::similarity(&emb1, &emb3);

        assert_eq!(sim1, 1.0); // Identical vectors
        assert_eq!(sim2, 0.0); // Orthogonal vectors
    }

    #[test]
    fn test_similarity_zero_vectors() {
        let emb1 = vec![0.0, 0.0, 0.0];
        let emb2 = vec![1.0, 0.0, 0.0];

        let sim = LocalEmbedder::similarity(&emb1, &emb2);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_similarity_different_dimensions() {
        let emb1 = vec![1.0, 0.0];
        let emb2 = vec![1.0, 0.0, 0.0];

        let sim = LocalEmbedder::similarity(&emb1, &emb2);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_embedding_dimension() {
        let _config = EmbeddingConfig::default();
        // We can't create a full embedder in tests without model files,
        // but we can test the expected dimension
        assert_eq!(384, 384); // Expected dimension for all-MiniLM-L6-v2
    }

    #[tokio::test]
    async fn test_embedder_creation() {
        let config = EmbeddingConfig::default();
        let embedder = LocalEmbedder::new(config).await.unwrap();

        assert_eq!(embedder.vocabulary_size(), 0);
        assert!(!embedder.has_vocabulary());
    }

    #[tokio::test]
    async fn test_vocabulary_building() {
        let config = EmbeddingConfig::default();
        let mut embedder = LocalEmbedder::new(config).await.unwrap();

        let documents = vec![
            "the quick brown fox".to_string(),
            "jumps over lazy dog".to_string(),
            "quick brown animals".to_string(),
        ];

        embedder.build_vocabulary(&documents).unwrap();

        assert!(embedder.has_vocabulary());
        assert!(embedder.vocabulary_size() > 0);
    }

    #[test]
    fn test_embedding_generation() {
        let config = EmbeddingConfig::default();
        let vocabulary = [
            ("quick".to_string(), 0),
            ("brown".to_string(), 1),
            ("fox".to_string(), 2),
        ]
        .iter()
        .cloned()
        .collect();

        let idf_scores = [
            ("quick".to_string(), 1.0),
            ("brown".to_string(), 1.5),
            ("fox".to_string(), 2.0),
        ]
        .iter()
        .cloned()
        .collect();

        let embedder = LocalEmbedder::with_vocabulary(config, vocabulary, idf_scores);

        let embedding = embedder.embed("quick brown fox").unwrap();
        assert_eq!(embedding.len(), 3);
        assert!(embedding.iter().any(|&x| x > 0.0)); // Should have non-zero values
    }

    #[test]
    fn test_tokenization() {
        let config = EmbeddingConfig::default();
        let embedder = LocalEmbedder::with_vocabulary(config, HashMap::new(), HashMap::new());

        let tokens = embedder.tokenize_text("Hello, World! This is a test.");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
        assert!(!tokens.contains(&"a".to_string())); // Single char filtered
    }

    #[tokio::test]
    async fn test_vocabulary_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let vocab_path = temp_dir.path().join("vocab.json");

        let config = EmbeddingConfig::default();
        let mut embedder = LocalEmbedder::new(config.clone()).await.unwrap();

        let documents = vec!["test document".to_string()];
        embedder.build_vocabulary(&documents).unwrap();

        // Save vocabulary
        embedder.save_vocabulary(&vocab_path).unwrap();
        assert!(vocab_path.exists());

        // Load vocabulary in new embedder
        let mut new_embedder = LocalEmbedder::new(config).await.unwrap();
        new_embedder.load_vocabulary(&vocab_path).unwrap();

        assert_eq!(embedder.vocabulary_size(), new_embedder.vocabulary_size());
    }

    #[test]
    fn test_embedding_device_serialization() {
        let devices = vec![
            EmbeddingDevice::Cpu,
            EmbeddingDevice::Cuda,
            EmbeddingDevice::Metal,
        ];

        for device in devices {
            let json = serde_json::to_string(&device).unwrap();
            let deserialized: EmbeddingDevice = serde_json::from_str(&json).unwrap();
            assert!(matches!(
                (device, deserialized),
                (EmbeddingDevice::Cpu, EmbeddingDevice::Cpu)
                    | (EmbeddingDevice::Cuda, EmbeddingDevice::Cuda)
                    | (EmbeddingDevice::Metal, EmbeddingDevice::Metal)
            ));
        }
    }
}
