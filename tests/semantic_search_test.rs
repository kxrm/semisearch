// tests/semantic_search_test.rs
//
// Integration test for semantic search functionality

use search::core::embedder::{EmbeddingConfig, LocalEmbedder};
use search::core::indexer::{FileIndexer, IndexerConfig};
use search::search::strategy::SearchEngine;
use search::storage::database::Database;
use search::SearchOptions;
use std::path::Path;
use tempfile::tempdir;

fn find_onnx_runtime() -> Option<String> {
    // 1. Check ORT_DYLIB_PATH
    if let Ok(path) = std::env::var("ORT_DYLIB_PATH") {
        if Path::new(&path).exists() {
            return Some(path);
        }
    }
    // 2. Check LD_LIBRARY_PATH
    if let Ok(ld_path) = std::env::var("LD_LIBRARY_PATH") {
        for dir in ld_path.split(':') {
            let candidate = Path::new(dir).join("libonnxruntime.so");
            if candidate.exists() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }
    None
}

#[tokio::test]
async fn test_tfidf_search_integration() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_db.sqlite");

    // Create database
    let database = Database::new(&db_path).expect("Failed to create database");

    // Create embedder for indexing (will fall back to TF-IDF)
    let config = EmbeddingConfig::default();
    let indexer_embedder = LocalEmbedder::new_tfidf_only(config)
        .await
        .expect("Failed to create indexer embedder");

    let indexer_config = IndexerConfig {
        chunk_size: 16, // Smaller chunk size for short test content
        ..Default::default()
    };

    let indexer = FileIndexer::with_embedder(database, indexer_config, indexer_embedder);

    // Create test files with distinct content
    let ai_content = "Machine learning and artificial intelligence are changing the world. Neural networks provide sophisticated models for data analysis.";
    let cooking_content = "The chef prepares a delicious meal using fresh ingredients. Culinary arts require both skill and creativity. The best recipes are passed down through generations.";
    let space_content = "Astronomers discovered a new exoplanet. Space exploration missions to Mars are underway. Rocket technology is advancing rapidly.";

    std::fs::write(temp_dir.path().join("ai.md"), ai_content).expect("Failed to write AI file");
    std::fs::write(temp_dir.path().join("cooking.md"), cooking_content)
        .expect("Failed to write cooking file");
    std::fs::write(temp_dir.path().join("space.md"), space_content)
        .expect("Failed to write space file");

    // Index the directory
    let stats = indexer
        .index_directory(temp_dir.path())
        .expect("Failed to index directory");
    assert!(stats.chunks_created > 0, "Chunks should have been created");
    assert_eq!(stats.files_updated, 3);

    // Create a new database connection and embedder for search engine
    let search_database = Database::new(&db_path).expect("Failed to create search database");
    let search_config = EmbeddingConfig::default();
    let search_embedder = LocalEmbedder::new_tfidf_only(search_config)
        .await
        .expect("Failed to create search embedder");
    let search_engine = SearchEngine::new(search_database, Some(search_embedder));

    // Test TF-IDF search for content
    let options = SearchOptions {
        search_mode: Some("tfidf".to_string()),
        min_score: 0.1,
        max_results: 3,
        ..Default::default()
    };

    let results = search_engine
        .search(
            "machine learning",
            temp_dir.path().to_str().unwrap(),
            options,
        )
        .await
        .expect("Search failed");

    // Should find at least one match
    assert!(!results.is_empty(), "Should find at least one match");

    // The AI document should be among the results
    let ai_file_path = temp_dir.path().join("ai.md").to_string_lossy().to_string();
    let found_ai = results.iter().any(|r| r.file_path == ai_file_path);
    assert!(
        found_ai,
        "AI document should be found for 'machine learning' query"
    );

    println!("✅ TF-IDF search integration test passed");
    println!("   Found {} results for 'machine learning'", results.len());
    for (i, result) in results.iter().enumerate() {
        println!(
            "   {}. {} (score: {:.3})",
            i + 1,
            result.file_path,
            result.score.unwrap_or(0.0)
        );
    }
}

#[tokio::test]
async fn test_tfidf_vs_keyword_search() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_db.sqlite");

    let database = Database::new(&db_path).expect("Failed to create database");

    let config = EmbeddingConfig::default();
    let indexer_embedder = LocalEmbedder::new_tfidf_only(config)
        .await
        .expect("Failed to create indexer embedder");

    let indexer_config = IndexerConfig {
        chunk_size: 16, // Smaller chunk size for short test content
        ..Default::default()
    };

    let indexer = FileIndexer::with_embedder(database, indexer_config, indexer_embedder);

    // Create test content in a subdirectory to avoid indexing the database file
    let test_dir = temp_dir.path().join("test_files");
    std::fs::create_dir(&test_dir).expect("Failed to create test directory");

    let content = "Machine learning algorithms are used in artificial intelligence research.";
    std::fs::write(test_dir.join("test.md"), content).expect("Failed to write test file");

    // Index the test directory
    let stats = indexer
        .index_directory(&test_dir)
        .expect("Failed to index directory");
    assert!(stats.chunks_created > 0, "Chunks should have been created");

    // Create a new database connection and embedder for search engine
    let search_database = Database::new(&db_path).expect("Failed to create search database");
    let search_config = EmbeddingConfig::default();
    let search_embedder = LocalEmbedder::new_tfidf_only(search_config)
        .await
        .expect("Failed to create search embedder");
    let search_engine = SearchEngine::new(search_database, Some(search_embedder));

    // Test TF-IDF search for a term in the content
    let tfidf_options = SearchOptions {
        search_mode: Some("tfidf".to_string()),
        min_score: 0.1,
        ..Default::default()
    };

    let tfidf_results = search_engine
        .search("algorithms", test_dir.to_str().unwrap(), tfidf_options)
        .await
        .expect("TF-IDF search failed");

    // Test keyword search for the same term
    let keyword_options = SearchOptions {
        search_mode: Some("keyword".to_string()),
        min_score: 0.1,
        ..Default::default()
    };

    let keyword_results = search_engine
        .search("algorithms", test_dir.to_str().unwrap(), keyword_options)
        .await
        .expect("Keyword search failed");

    // Both should find results since the term is in the content
    assert!(
        !tfidf_results.is_empty(),
        "TF-IDF search should find content"
    );
    assert!(
        !keyword_results.is_empty(),
        "Keyword search should find content"
    );

    println!("✅ TF-IDF vs keyword search test passed");
    println!("   TF-IDF search found {} results", tfidf_results.len());
    println!("   Keyword search found {} results", keyword_results.len());
}

#[tokio::test]
#[cfg(feature = "neural-embeddings")]
async fn test_semantic_search_integration() {
    let onnx_path = find_onnx_runtime();
    if let Some(path) = onnx_path {
        std::env::set_var("ORT_DYLIB_PATH", &path);
    } else {
        eprintln!("⚠️  Skipping test_semantic_search_integration: ONNX Runtime not found in ORT_DYLIB_PATH or LD_LIBRARY_PATH");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_db.sqlite");

    // Create database
    let database = Database::new(&db_path).expect("Failed to create database");

    // Create embedder for indexing
    let config = EmbeddingConfig::default();
    let indexer_embedder = LocalEmbedder::new(config)
        .await
        .expect("Failed to create indexer embedder");

    let indexer_config = IndexerConfig {
        chunk_size: 16, // Smaller chunk size for test content
        ..Default::default()
    };

    let indexer = FileIndexer::with_embedder(database, indexer_config, indexer_embedder);

    // Create test files with distinct semantic content
    let ai_content = "Machine learning and artificial intelligence are changing the world. Neural networks provide sophisticated models for data analysis.";
    let cooking_content = "The chef prepares a delicious meal using fresh ingredients. Culinary arts require both skill and creativity. The best recipes are passed down through generations.";
    let space_content = "Astronomers discovered a new exoplanet. Space exploration missions to Mars are underway. Rocket technology is advancing rapidly.";

    std::fs::write(temp_dir.path().join("ai.md"), ai_content).expect("Failed to write AI file");
    std::fs::write(temp_dir.path().join("cooking.md"), cooking_content)
        .expect("Failed to write cooking file");
    std::fs::write(temp_dir.path().join("space.md"), space_content)
        .expect("Failed to write space file");

    // Index the directory
    let stats = indexer
        .index_directory(temp_dir.path())
        .expect("Failed to index directory");
    assert!(stats.chunks_created > 0, "Chunks should have been created");
    assert_eq!(stats.files_updated, 3);

    // Create a new database connection and embedder for search engine
    let search_database = Database::new(&db_path).expect("Failed to create search database");
    let search_config = EmbeddingConfig::default();
    let search_embedder = LocalEmbedder::new(search_config)
        .await
        .expect("Failed to create search embedder");
    let search_engine = SearchEngine::new(search_database, Some(search_embedder));

    // Test semantic search for programming-related content
    let options = SearchOptions {
        search_mode: Some("semantic".to_string()),
        min_score: 0.2,
        max_results: 3,
        ..Default::default()
    };

    let results = search_engine
        .search(
            "computer models",
            temp_dir.path().to_str().unwrap(),
            options,
        )
        .await
        .expect("Search failed");

    // Should find at least one semantic match
    assert!(
        !results.is_empty(),
        "Should find at least one semantic match"
    );

    // The AI document should be among the results
    let ai_file_path = temp_dir.path().join("ai.md").to_string_lossy().to_string();
    let found_ai = results.iter().any(|r| r.file_path == ai_file_path);
    assert!(
        found_ai,
        "AI document should be found for 'computer models' query"
    );

    println!("✅ Semantic search integration test passed");
    println!("   Found {} results for 'computer models'", results.len());
    for (i, result) in results.iter().enumerate() {
        println!(
            "   {}. {} (score: {:.3})",
            i + 1,
            result.file_path,
            result.score.unwrap_or(0.0)
        );
    }
}

#[tokio::test]
#[cfg(feature = "neural-embeddings")]
async fn test_semantic_vs_keyword_search() {
    let onnx_path = find_onnx_runtime();
    if let Some(path) = onnx_path {
        std::env::set_var("ORT_DYLIB_PATH", &path);
    } else {
        eprintln!("⚠️  Skipping test_semantic_vs_keyword_search: ONNX Runtime not found in ORT_DYLIB_PATH or LD_LIBRARY_PATH");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_db.sqlite");

    let database = Database::new(&db_path).expect("Failed to create database");

    let config = EmbeddingConfig::default();
    let indexer_embedder = LocalEmbedder::new(config)
        .await
        .expect("Failed to create indexer embedder");

    let indexer_config = IndexerConfig {
        chunk_size: 16,
        ..Default::default()
    };

    let indexer = FileIndexer::with_embedder(database, indexer_config, indexer_embedder);

    // Create test content in a subdirectory to avoid indexing the database file
    let test_dir = temp_dir.path().join("test_files");
    std::fs::create_dir(&test_dir).expect("Failed to create test directory");

    let content = "Machine learning algorithms are used in artificial intelligence research.";
    std::fs::write(test_dir.join("test.md"), content).expect("Failed to write test file");

    // Index the test directory
    let stats = indexer
        .index_directory(&test_dir)
        .expect("Failed to index directory");
    assert!(stats.chunks_created > 0, "Chunks should have been created");

    // Create a new database connection and embedder for search engine
    let search_database = Database::new(&db_path).expect("Failed to create search database");
    let search_config = EmbeddingConfig::default();
    let search_embedder = LocalEmbedder::new(search_config)
        .await
        .expect("Failed to create search embedder");
    let search_engine = SearchEngine::new(search_database, Some(search_embedder));

    // Test semantic search for a term not in the content
    let semantic_options = SearchOptions {
        search_mode: Some("semantic".to_string()),
        min_score: 0.1,
        ..Default::default()
    };

    let semantic_results = search_engine
        .search("programming", test_dir.to_str().unwrap(), semantic_options)
        .await
        .expect("Semantic search failed");

    // Test keyword search for the same term
    let keyword_options = SearchOptions {
        search_mode: Some("keyword".to_string()),
        min_score: 0.1,
        ..Default::default()
    };

    let keyword_results = search_engine
        .search("programming", test_dir.to_str().unwrap(), keyword_options)
        .await
        .expect("Keyword search failed");

    // Semantic search should find results, keyword search should not
    assert!(
        !semantic_results.is_empty(),
        "Semantic search should find related content"
    );
    assert!(
        keyword_results.is_empty(),
        "Keyword search should not find unrelated terms"
    );

    println!("✅ Semantic vs keyword search test passed");
    println!(
        "   Semantic search found {} results",
        semantic_results.len()
    );
    println!("   Keyword search found {} results", keyword_results.len());
}

#[tokio::test]
#[cfg(feature = "neural-embeddings")]
async fn test_onnx_runtime_detection() {
    let onnx_path = find_onnx_runtime();
    if onnx_path.is_some() {
        println!("✅ ONNX Runtime found at: {}", onnx_path.unwrap());
    } else {
        println!("⚠️  ONNX Runtime not found - this is expected in some environments");
    }

    // Test completed successfully
}

#[tokio::test]
#[cfg(feature = "neural-embeddings")]
async fn test_embedder_capability_detection() {
    let capability = LocalEmbedder::detect_capabilities();
    println!("✅ System embedding capability: {:?}", capability);

    // Test should pass regardless of capability
    assert!(matches!(
        capability,
        search::core::embedder::EmbeddingCapability::Full
            | search::core::embedder::EmbeddingCapability::TfIdf
            | search::core::embedder::EmbeddingCapability::None
    ));
}
