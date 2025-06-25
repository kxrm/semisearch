use search::core::{EmbeddingCapability, EmbeddingConfig, LocalEmbedder};
use search::search::semantic::{SemanticReranker, SemanticSearch, SemanticSearchOptions};
use search::storage::{ChunkRecord, Database};
use search::text::TextProcessor;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_phase4_end_to_end_embeddings() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create database
    let _database = Database::new(&db_path).unwrap();

    // Create embedder
    let config = EmbeddingConfig::default();
    let mut embedder = LocalEmbedder::new(config).await.unwrap();

    // Sample documents for building vocabulary
    let documents = vec![
        "machine learning algorithms for data analysis".to_string(),
        "artificial intelligence research and development".to_string(),
        "deep neural networks and backpropagation".to_string(),
        "natural language processing with transformers".to_string(),
        "computer vision and image recognition".to_string(),
        "reinforcement learning in robotics".to_string(),
    ];

    // Build vocabulary
    embedder.build_vocabulary(&documents).unwrap();
    assert!(embedder.has_vocabulary());
    assert!(embedder.vocabulary_size() > 0);

    // Create semantic search with lower threshold for testing
    let semantic_search = SemanticSearch::with_threshold(Arc::new(embedder), 0.0);
    assert!(semantic_search.is_ready());

    // Create test chunks with embeddings
    let text_processor = TextProcessor::new();
    let mut chunks = Vec::new();

    for (i, doc) in documents.iter().enumerate() {
        let text_chunks = text_processor.process_file(doc);

        for text_chunk in text_chunks {
            let embedding = semantic_search.embed_chunk(&text_chunk).unwrap();

            let chunk_record = ChunkRecord {
                id: (i * 10 + text_chunk.line_number) as i64,
                file_id: i as i64,
                file_path: format!("doc_{i}.txt"),
                line_number: text_chunk.line_number,
                start_char: text_chunk.start_char,
                end_char: text_chunk.end_char,
                content: text_chunk.content,
                embedding: Some(embedding),
            };

            chunks.push(chunk_record);
        }
    }

    // Test semantic search
    let results = semantic_search
        .search("machine learning", &chunks, 5)
        .unwrap();
    assert!(!results.is_empty());

    // Results should be sorted by similarity
    for i in 1..results.len() {
        assert!(results[i - 1].similarity_score >= results[i].similarity_score);
    }

    // Test with reranking
    let reranked_results = semantic_search
        .search_with_reranking(
            "machine learning algorithms",
            &chunks,
            3,
            SemanticReranker::boost_exact_matches,
        )
        .unwrap();

    assert!(!reranked_results.is_empty());
    assert!(reranked_results.len() <= 3);

    // Test similarity calculations
    if results.len() >= 2 {
        let sim =
            LocalEmbedder::similarity(&results[0].query_embedding, &results[0].chunk_embedding);
        assert!((0.0..=1.0).contains(&sim));
    }
}

#[tokio::test]
async fn test_embedding_vocabulary_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let vocab_path = temp_dir.path().join("vocabulary.json");

    // Create and train embedder
    let config = EmbeddingConfig::default();
    let mut embedder1 = LocalEmbedder::new(config.clone()).await.unwrap();

    let documents = vec![
        "artificial intelligence".to_string(),
        "machine learning".to_string(),
        "deep learning".to_string(),
    ];

    embedder1.build_vocabulary(&documents).unwrap();
    let original_vocab_size = embedder1.vocabulary_size();

    // Save vocabulary
    embedder1.save_vocabulary(&vocab_path).unwrap();
    assert!(vocab_path.exists());

    // Create new embedder and load vocabulary
    let mut embedder2 = LocalEmbedder::new(config).await.unwrap();
    assert!(!embedder2.has_vocabulary());

    embedder2.load_vocabulary(&vocab_path).unwrap();
    assert!(embedder2.has_vocabulary());
    assert_eq!(embedder2.vocabulary_size(), original_vocab_size);

    // Test that both embedders produce same embeddings
    let text = "artificial intelligence";
    let emb1 = embedder1.embed(text).unwrap();
    let emb2 = embedder2.embed(text).unwrap();

    assert_eq!(emb1.len(), emb2.len());

    // Embeddings should be very similar (allowing for floating point precision)
    let similarity = LocalEmbedder::similarity(&emb1, &emb2);
    assert!(similarity > 0.99);
}

#[test]
fn test_capability_detection() {
    let capability = LocalEmbedder::detect_capabilities();

    // Should detect some capability on any system
    assert!(matches!(
        capability,
        EmbeddingCapability::Full | EmbeddingCapability::TfIdf | EmbeddingCapability::None
    ));
}

#[tokio::test]
async fn test_batch_embedding() {
    let config = EmbeddingConfig::default();
    let mut embedder = LocalEmbedder::new(config).await.unwrap();

    let documents = vec![
        "first document".to_string(),
        "second document".to_string(),
        "third document".to_string(),
    ];

    embedder.build_vocabulary(&documents).unwrap();

    let batch_embeddings = embedder.embed_batch(&documents).unwrap();
    assert_eq!(batch_embeddings.len(), documents.len());

    for embedding in &batch_embeddings {
        assert!(!embedding.is_empty());
        assert_eq!(embedding.len(), embedder.vocabulary_size());
    }
}

#[tokio::test]
async fn test_semantic_search_options() {
    let options = SemanticSearchOptions::default();

    assert_eq!(options.similarity_threshold, 0.7);
    assert_eq!(options.max_results, 50);
    assert!(options.boost_exact_matches);
    assert!(!options.enable_reranking);
    assert!(!options.boost_recent_files);

    // Test custom options
    let custom_options = SemanticSearchOptions {
        similarity_threshold: 0.5,
        max_results: 10,
        enable_reranking: true,
        boost_exact_matches: false,
        boost_recent_files: true,
    };

    assert_eq!(custom_options.similarity_threshold, 0.5);
    assert_eq!(custom_options.max_results, 10);
    assert!(custom_options.enable_reranking);
}

#[tokio::test]
async fn test_empty_vocabulary_handling() {
    let config = EmbeddingConfig::default();
    let embedder = LocalEmbedder::new(config).await.unwrap();
    let semantic_search = SemanticSearch::new(Arc::new(embedder));

    let chunks = vec![ChunkRecord {
        id: 1,
        file_id: 1,
        file_path: "test.txt".to_string(),
        line_number: 1,
        start_char: 0,
        end_char: 4,
        content: "test".to_string(),
        embedding: Some(vec![0.1, 0.2, 0.3]),
    }];

    // Should fail with empty vocabulary
    let result = semantic_search.search("test query", &chunks, 10);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("vocabulary not built"));
}

#[tokio::test]
async fn test_embedding_normalization() {
    let config = EmbeddingConfig::default();
    let mut embedder = LocalEmbedder::new(config).await.unwrap();

    // Use multiple documents to build a proper vocabulary
    let documents = vec![
        "test document for normalization".to_string(),
        "another test document".to_string(),
        "machine learning example".to_string(),
    ];
    embedder.build_vocabulary(&documents).unwrap();

    let embedding = embedder.embed("test document").unwrap();

    // Check that embedding vector has expected structure
    assert!(!embedding.is_empty());
    assert_eq!(embedding.len(), embedder.vocabulary_size());

    // Check that at least some values are non-zero (for a meaningful embedding)
    let non_zero_count = embedding.iter().filter(|&&x| x != 0.0).count();
    assert!(
        non_zero_count > 0,
        "Embedding should have at least some non-zero values"
    );

    // Check normalization constraint (L2 norm should be <= 1.0 after normalization)
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(
        norm <= 1.0,
        "Normalized embedding should have L2 norm <= 1.0"
    );
}

#[test]
fn test_similarity_edge_cases() {
    // Test identical embeddings
    let emb1 = vec![1.0, 0.0, 0.0];
    let emb2 = vec![1.0, 0.0, 0.0];
    assert_eq!(LocalEmbedder::similarity(&emb1, &emb2), 1.0);

    // Test orthogonal embeddings
    let emb3 = vec![0.0, 1.0, 0.0];
    assert_eq!(LocalEmbedder::similarity(&emb1, &emb3), 0.0);

    // Test different dimensions
    let emb4 = vec![1.0, 0.0];
    assert_eq!(LocalEmbedder::similarity(&emb1, &emb4), 0.0);

    // Test zero vectors
    let emb5 = vec![0.0, 0.0, 0.0];
    assert_eq!(LocalEmbedder::similarity(&emb1, &emb5), 0.0);
    assert_eq!(LocalEmbedder::similarity(&emb5, &emb5), 0.0);
}
