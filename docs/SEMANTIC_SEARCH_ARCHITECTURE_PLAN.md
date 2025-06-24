# Semantic Search CLI Tool - Architectural Plan

## Project Overview

**Tool Name:** `semisearch` (Semantic Search CLI)  
**Purpose:** Privacy-first CLI tool for semantic search across local files  
**Target Users:** Developers, researchers, knowledge workers  
**Privacy Model:** 100% local processing, no network requests after initial setup  
**Development Philosophy:** Progressive enhancement - start simple, add features based on system capabilities  
**Hardware Requirements:** Minimal - works on systems as low-powered as Raspberry Pi

## High-Level Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────┐
│   User Query    │───▶│   CLI Interface  │───▶│   Query Processor   │
│  "Jim Carrey"   │    │  (clap + tokio)  │    │                     │
└─────────────────┘    └──────────────────┘    └─────────────────────┘
                                                           │
                       ┌─────────────────────────────────────┴─────────────┐
                       ▼                                                   ▼
           ┌─────────────────────┐                           ┌─────────────────────┐
           │   Keyword Search    │                           │   Semantic Search   │
           │  (TF-IDF + Fuzzy)   │                           │ (Local Embeddings)  │
           └─────────────────────┘                           └─────────────────────┘
                       │                                                   │
                       └─────────────────┬─────────────────────────────────┘
                                         ▼
                              ┌─────────────────────┐
                              │   Result Merger    │
                              │  (Score + Rank)    │
                              └─────────────────────┘
                                         │
                                         ▼
                              ┌─────────────────────┐
                              │  Output Formatter  │
                              │ (JSON/Plain Text)  │
                              └─────────────────────┘
```

## Implementation Approach

### Progressive Enhancement Strategy
The tool is designed with a "start simple, enhance gradually" approach:

1. **MVP (Basic Keyword Search)** - Works on any system
2. **Enhanced Search (Fuzzy Matching)** - Still lightweight
3. **Semantic Search (ML Models)** - For systems with more resources
4. **Full Features** - All capabilities enabled

### Language Options

#### Primary: Rust Implementation
Best for production use with optimal performance.

#### Alternative: Python Implementation  
For junior developers or rapid prototyping:
- **CLI:** `typer` (simpler than clap)
- **Database:** `sqlite3` (built-in, no external deps)
- **Search:** `whoosh` or custom TF-IDF
- **ML:** `sentence-transformers` (optional)

## Technology Stack

### Core Technologies
- **Language:** Rust (2021 edition)
- **CLI Framework:** clap v4 (argument parsing)
- **Async Runtime:** tokio (file I/O and concurrency)
- **Database:** SQLite (rusqlite crate)
- **ML Inference:** ONNX Runtime (ort crate)
- **Text Processing:** tokenizers, unicode-segmentation

### Key Dependencies
```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
rusqlite = { version = "0.29", features = ["bundled"] }
ort = { version = "1.15", features = ["copy-dylibs"] }
tokenizers = "0.13"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
walkdir = "2.3"
uuid = { version = "1.0", features = ["v4"] }
anyhow = "1.0"
thiserror = "1.0"
indicatif = "0.17"
```

## Development Environment Setup

### DevContainer Configuration

Create `.devcontainer/devcontainer.json`:
```json
{
  "name": "Semantic Search CLI Development",
  "image": "mcr.microsoft.com/devcontainers/rust:1-1-bullseye",
  "features": {
    "ghcr.io/devcontainers/features/common-utils:2": {
      "installZsh": true,
      "configureZshAsDefaultShell": true,
      "installOhMyZsh": true
    },
    "ghcr.io/devcontainers/features/git:1": {},
    "ghcr.io/devcontainers/features/github-cli:1": {}
  },
  "customizations": {
    "vscode": {
      "extensions": [
        "rust-lang.rust-analyzer",
        "serayuzgur.crates",
        "vadimcn.vscode-lldb",
        "tamasfe.even-better-toml"
      ]
    }
  },
  "postCreateCommand": "rustup component add clippy rustfmt",
  "remoteUser": "vscode"
}
```

### Initial Project Setup

1. **Create new Rust project:**
```bash
cargo new semisearch --bin
cd semisearch
```

2. **Setup project structure:**
```
semisearch/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli/
│   │   ├── mod.rs
│   │   └── commands.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── indexer.rs
│   │   ├── searcher.rs
│   │   └── embedder.rs
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── database.rs
│   │   └── vector_store.rs
│   ├── text/
│   │   ├── mod.rs
│   │   ├── processor.rs
│   │   └── tokenizer.rs
│   └── config/
│       ├── mod.rs
│       └── settings.rs
├── tests/
├── benches/
├── models/  (gitignored - downloaded at runtime)
└── README.md
```

### Getting Started Quickly (MVP Approach)

For developers who want to start with a minimal working version:

```bash
# Step 1: Create basic keyword search in < 100 lines
cargo new semisearch --bin
cd semisearch

# Step 2: Add minimal dependencies
cargo add clap --features derive
cargo add rusqlite --features bundled
cargo add walkdir

# Step 3: Start with keyword search only
# Implement basic file scanning and text matching
# Add ML features later when it's working
```

**MVP Features (Week 1):**
- Basic CLI with search command
- Simple file traversal
- Keyword matching with line numbers
- JSON output format

This approach lets you have a working tool quickly, then enhance it progressively.

## Detailed Implementation Plan

### Phase 1: Foundation (Week 1-2)

#### 1.1 CLI Interface Setup
```rust
// src/cli/mod.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "semisearch")]
#[command(about = "Semantic search across local files")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Search for semantic matches
    Search {
        /// Search query
        query: String,
        
        /// Target directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        path: String,
        
        /// Minimum similarity score (0.0-1.0)
        #[arg(short, long, default_value = "0.7")]
        score: f32,
        
        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
        
        /// Output format
        #[arg(short, long, default_value = "plain")]
        format: String,
    },
    
    /// Index files in directory
    Index {
        /// Directory to index
        path: String,
    },
    
    /// Show configuration
    Config,
}
```

#### 1.2 Configuration Management
```rust
// src/config/settings.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub privacy: PrivacyConfig,
    pub performance: PerformanceConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub never_index_patterns: Vec<String>,
    pub exclude_directories: Vec<String>,
    pub encrypt_index: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_file_size_mb: u64,
    pub max_results: usize,
    pub embedding_model: String,
    pub chunk_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            privacy: PrivacyConfig {
                never_index_patterns: vec![
                    "*.key".to_string(),
                    "*.pem".to_string(),
                    "*.pfx".to_string(),
                    "password*".to_string(),
                    "secrets*".to_string(),
                    "credentials*".to_string(),
                    ".env*".to_string(),
                    "id_rsa*".to_string(),
                    "*.sqlite*".to_string(),
                    "*.db".to_string(),
                ],
                exclude_directories: vec![
                    ".git".to_string(),
                    "node_modules".to_string(),
                    ".venv".to_string(),
                    "target".to_string(),
                    ".aws".to_string(),
                    ".ssh".to_string(),
                ],
                encrypt_index: false,
            },
            performance: PerformanceConfig {
                max_file_size_mb: 50,
                max_results: 100,
                embedding_model: "all-MiniLM-L6-v2".to_string(),
                chunk_size: 512,
            },
        }
    }
}

impl Config {
    /// Auto-detect system capabilities and adjust configuration
    pub fn auto_detect() -> Self {
        let available_memory = sys_info::mem_info().unwrap_or_default().total;
        let cpu_count = num_cpus::get();
        
        if available_memory < 2_000_000 { // Less than 2GB RAM
            Config::minimal()
        } else if cpu_count < 4 {
            Config::conservative()
        } else {
            Config::default()
        }
    }
    
    pub fn minimal() -> Self {
        Self {
            privacy: Config::default().privacy,
            performance: PerformanceConfig {
                max_file_size_mb: 10,
                max_results: 50,
                embedding_model: "none".to_string(), // Keyword search only
                chunk_size: 256,
            },
        }
    }
    
    pub fn conservative() -> Self {
        Self {
            privacy: Config::default().privacy,
            performance: PerformanceConfig {
                max_file_size_mb: 25,
                max_results: 75,
                embedding_model: "tfidf".to_string(), // Use TF-IDF instead of neural
                chunk_size: 384,
            },
        }
    }
}
```

### Phase 2: Storage Layer (Week 2-3)

#### 2.1 Database Schema
```sql
-- migrations/001_initial.sql
CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    hash TEXT NOT NULL,
    modified_at INTEGER NOT NULL,
    size_bytes INTEGER NOT NULL,
    indexed_at INTEGER NOT NULL
);

CREATE TABLE chunks (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    line_number INTEGER NOT NULL,
    start_char INTEGER NOT NULL,
    end_char INTEGER NOT NULL,
    content TEXT NOT NULL,
    embedding BLOB,
    FOREIGN KEY (file_id) REFERENCES files(id)
);

CREATE INDEX idx_files_path ON files(path);
CREATE INDEX idx_chunks_file_id ON chunks(file_id);
CREATE INDEX idx_chunks_content ON chunks(content);

-- Query cache for improved performance
CREATE TABLE query_cache (
    query_hash TEXT PRIMARY KEY,
    results TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    access_count INTEGER DEFAULT 1,
    last_accessed INTEGER NOT NULL
);

CREATE INDEX idx_query_cache_accessed ON query_cache(last_accessed);
```

#### 2.2 Database Implementation
```rust
// src/storage/database.rs
use rusqlite::{Connection, Result, params};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(include_str!("../../migrations/001_initial.sql"))?;
        Ok(Self { conn })
    }
    
    pub fn insert_file(&self, path: &str, hash: &str, modified_at: i64, size_bytes: i64) -> Result<i64> {
        let mut stmt = self.conn.prepare(
            "INSERT OR REPLACE INTO files (path, hash, modified_at, size_bytes, indexed_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)"
        )?;
        
        stmt.execute(params![path, hash, modified_at, size_bytes, chrono::Utc::now().timestamp()])?;
        Ok(self.conn.last_insert_rowid())
    }
    
    pub fn insert_chunk(&self, file_id: i64, line_number: usize, start_char: usize, 
                       end_char: usize, content: &str, embedding: Option<&[f32]>) -> Result<()> {
        let embedding_bytes = embedding.map(|e| bytemuck::cast_slice(e));
        
        let mut stmt = self.conn.prepare(
            "INSERT INTO chunks (file_id, line_number, start_char, end_char, content, embedding) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )?;
        
        stmt.execute(params![file_id, line_number, start_char, end_char, content, embedding_bytes])?;
        Ok(())
    }
}
```

### Phase 3: Text Processing (Week 3-4)

#### 3.1 Modular Search Architecture

The search system uses a plugin-based architecture for maximum flexibility:

```rust
// Core trait that all search strategies must implement
trait SearchStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>>;
    fn required_resources(&self) -> ResourceRequirements;
}

// Different search implementations
struct KeywordSearch;      // Basic string matching
struct FuzzySearch;        // Edit distance based
struct RegexSearch;        // Regular expressions
struct TfIdfSearch;        // Statistical ranking
struct SemanticSearch;     // ML-based understanding

// Developers can implement one at a time
impl SearchStrategy for KeywordSearch {
    fn name(&self) -> &str { "keyword" }
    
    fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        // Simple implementation - just 50 lines of code
    }
    
    fn required_resources(&self) -> ResourceRequirements {
        ResourceRequirements {
            min_memory_mb: 10,
            requires_ml: false,
            requires_index: false,
        }
    }
}
```

This allows developers to:
1. Start with just `KeywordSearch`
2. Add `FuzzySearch` when ready
3. Implement `SemanticSearch` only if needed
4. Mix and match strategies based on system capabilities

#### 3.2 Text Processor
```rust
// src/text/processor.rs
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

pub struct TextProcessor {
    stop_words: HashSet<String>,
}

impl TextProcessor {
    pub fn new() -> Self {
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by"
        ].iter().map(|&s| s.to_string()).collect();
        
        Self { stop_words }
    }
    
    pub fn process_file(&self, content: &str) -> Vec<TextChunk> {
        content
            .lines()
            .enumerate()
            .filter_map(|(line_num, line)| {
                let cleaned = self.clean_text(line);
                if cleaned.len() > 10 {  // Skip very short lines
                    Some(TextChunk {
                        line_number: line_num + 1,
                        content: cleaned,
                        tokens: self.tokenize(&cleaned),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
    
    fn clean_text(&self, text: &str) -> String {
        text.trim()
            .replace('\t', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.unicode_words()
            .map(|w| w.to_lowercase())
            .filter(|w| !self.stop_words.contains(w))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct TextChunk {
    pub line_number: usize,
    pub content: String,
    pub tokens: Vec<String>,
}
```

### Phase 4: Local Embeddings (Week 4-5)

#### 4.1 Embedding Options

The tool supports multiple embedding strategies based on system capabilities:

1. **Full Neural Embeddings** (Default for capable systems)
   - Uses ONNX Runtime with all-MiniLM-L6-v2
   - Best semantic understanding
   - Requires ~100MB model download

2. **TF-IDF Embeddings** (For limited systems)
   - Pure Rust implementation
   - No external model needed
   - Good for keyword-based semantic similarity

3. **Cached Embeddings** (Hybrid approach)
   - Pre-compute embeddings during indexing
   - No runtime ML needed for search
   - Larger index size but faster search

4. **No Embeddings** (Minimal systems)
   - Keyword and fuzzy matching only
   - Works on any system
   - Still provides good results for exact matches

#### 4.2 Model Downloader
```rust
// src/core/embedder.rs
use ort::{Environment, ExecutionProvider, Session, SessionBuilder, Value};
use std::path::Path;
use tokio::fs;

pub struct LocalEmbedder {
    session: Session,
    tokenizer: tokenizers::Tokenizer,
}

impl LocalEmbedder {
    pub async fn new(model_path: &Path) -> anyhow::Result<Self> {
        // Download model if it doesn't exist
        if !model_path.exists() {
            Self::download_model(model_path).await?;
        }
        
        let environment = Environment::builder()
            .with_name("semantic_search")
            .with_execution_providers([ExecutionProvider::CPU(Default::default())])
            .build()?;
            
        let session = SessionBuilder::new(&environment)?
            .with_model_from_file(model_path)?;
            
        let tokenizer = tokenizers::Tokenizer::from_pretrained("sentence-transformers/all-MiniLM-L6-v2", None)?;
        
        Ok(Self { session, tokenizer })
    }
    
    async fn download_model(model_path: &Path) -> anyhow::Result<()> {
        println!("Downloading embedding model (first time setup)...");
        
        // Create progress bar
        let pb = indicatif::ProgressBar::new(100);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
        );
        
        // Download from HuggingFace
        let url = "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx";
        let response = reqwest::get(url).await?;
        let total_size = response.content_length().unwrap_or(0);
        pb.set_length(total_size);
        
        let mut file = fs::File::create(model_path).await?;
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }
        
        pb.finish_with_message("Model downloaded successfully");
        Ok(())
    }
    
    pub fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let encoding = self.tokenizer.encode(text, false)?;
        let tokens = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();
        
        let input_ids = Value::from_array(self.session.allocator(), &[1, tokens.len()], tokens)?;
        let attention_mask = Value::from_array(self.session.allocator(), &[1, attention_mask.len()], attention_mask)?;
        
        let outputs = self.session.run([input_ids, attention_mask])?;
        let embeddings = outputs[0].try_extract::<f32>()?;
        
        // Mean pooling
        let embedding_dim = embeddings.len() / tokens.len();
        let mut pooled = vec![0.0; embedding_dim];
        
        for i in 0..tokens.len() {
            for j in 0..embedding_dim {
                pooled[j] += embeddings[i * embedding_dim + j];
            }
        }
        
        // Normalize
        let norm: f32 = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
        pooled.iter_mut().for_each(|x| *x /= norm);
        
        Ok(pooled)
    }
}
```

### Phase 5: Search Implementation (Week 5-6)

#### 5.1 Multi-Strategy Search
```rust
// src/core/searcher.rs
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum SearchMode {
    Full,      // Keyword + Semantic (requires ML models)
    Hybrid,    // Keyword + Cached embeddings only
    Basic,     // Keyword only (no ML required)
}

pub struct SearchEngine {
    database: Database,
    embedder: Option<LocalEmbedder>,
    config: Config,
    mode: SearchMode,
}

impl SearchEngine {
    pub fn new(database: Database, embedder: Option<LocalEmbedder>, config: Config) -> Self {
        let mode = match (&embedder, config.performance.embedding_model.as_str()) {
            (Some(_), _) => SearchMode::Full,
            (None, "none") => SearchMode::Basic,
            (None, _) => SearchMode::Hybrid,
        };
        
        Self { database, embedder, config, mode }
    }
    
    pub async fn search(&self, query: &str, options: SearchOptions) -> anyhow::Result<Vec<SearchResult>> {
        match self.mode {
            SearchMode::Full => {
                // Generate query embedding
                let query_embedding = self.embedder.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Embedder not available"))?
                    .embed(query)?;
                
                // Parallel search strategies
                let (keyword_results, semantic_results) = tokio::try_join!(
                    self.keyword_search(query, &options),
                    self.semantic_search(&query_embedding, &options)
                )?;
                
                // Merge and rank results
                Ok(self.merge_results(keyword_results, semantic_results))
            }
            SearchMode::Hybrid => {
                // Use cached embeddings only
                let keyword_results = self.keyword_search(query, &options).await?;
                let cached_semantic = self.search_cached_embeddings(query, &options).await?;
                Ok(self.merge_results(keyword_results, cached_semantic))
            }
            SearchMode::Basic => {
                // Keyword search only
                self.keyword_search(query, &options).await
            }
        }
    }
    
    async fn keyword_search(&self, query: &str, options: &SearchOptions) -> anyhow::Result<Vec<SearchResult>> {
        let query_tokens = self.tokenize_query(query);
        let mut results = Vec::new();
        
        for chunk in self.database.get_all_chunks()? {
            let score = self.calculate_keyword_score(&query_tokens, &chunk.tokens);
            if score >= options.min_score {
                results.push(SearchResult {
                    file_path: chunk.file_path,
                    line_number: chunk.line_number,
                    content: chunk.content,
                    score,
                    match_type: MatchType::Keyword,
                });
            }
        }
        
        Ok(results)
    }
    
    async fn semantic_search(&self, query_embedding: &[f32], options: &SearchOptions) -> anyhow::Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        
        for chunk in self.database.get_chunks_with_embeddings()? {
            if let Some(embedding) = chunk.embedding {
                let similarity = cosine_similarity(query_embedding, &embedding);
                if similarity >= options.min_score {
                    results.push(SearchResult {
                        file_path: chunk.file_path,
                        line_number: chunk.line_number,
                        content: chunk.content,
                        score: similarity,
                        match_type: MatchType::Semantic,
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    fn merge_results(&self, keyword_results: Vec<SearchResult>, semantic_results: Vec<SearchResult>) -> Vec<SearchResult> {
        let mut combined = HashMap::new();
        
        // Add keyword results
        for result in keyword_results {
            let key = format!("{}:{}", result.file_path, result.line_number);
            combined.insert(key, result);
        }
        
        // Add semantic results, boosting score if already exists
        for result in semantic_results {
            let key = format!("{}:{}", result.file_path, result.line_number);
            if let Some(existing) = combined.get_mut(&key) {
                existing.score = (existing.score + result.score) / 2.0; // Average scores
                existing.match_type = MatchType::Hybrid;
            } else {
                combined.insert(key, result);
            }
        }
        
        let mut results: Vec<_> = combined.into_values().collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(self.config.performance.max_results);
        
        results
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    dot_product / (norm_a * norm_b)
}
```

### Phase 6: File Indexing (Week 6-7)

#### 6.1 Background Indexer
```rust
// src/core/indexer.rs
use walkdir::WalkDir;
use std::collections::HashSet;
use tokio::fs;

pub struct FileIndexer {
    database: Database,
    embedder: LocalEmbedder,
    text_processor: TextProcessor,
    config: Config,
}

impl FileIndexer {
    pub async fn index_directory(&self, path: &str) -> anyhow::Result<IndexStats> {
        let mut stats = IndexStats::default();
        let excluded_dirs: HashSet<_> = self.config.privacy.exclude_directories.iter().collect();
        
        println!("Scanning directory: {}", path);
        let pb = indicatif::ProgressBar::new_spinner();
        pb.set_style(indicatif::ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap());
        
        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !self.should_exclude_directory(e, &excluded_dirs))
        {
            let entry = entry?;
            pb.set_message(format!("Processing: {}", entry.path().display()));
            
            if entry.file_type().is_file() {
                match self.process_file(entry.path()).await {
                    Ok(file_stats) => {
                        stats.files_processed += 1;
                        stats.chunks_created += file_stats.chunks_created;
                    }
                    Err(e) => {
                        eprintln!("Error processing {}: {}", entry.path().display(), e);
                        stats.files_skipped += 1;
                    }
                }
            }
        }
        
        pb.finish_with_message(format!("Indexing complete: {} files processed", stats.files_processed));
        Ok(stats)
    }
    
    async fn process_file(&self, path: &Path) -> anyhow::Result<FileStats> {
        let content = fs::read_to_string(path).await?;
        let metadata = fs::metadata(path).await?;
        
        // Skip large files
        if metadata.len() > self.config.performance.max_file_size_mb * 1024 * 1024 {
            return Err(anyhow::anyhow!("File too large"));
        }
        
        // Check if file needs reindexing
        let file_hash = self.calculate_file_hash(&content);
        if !self.database.needs_reindexing(path.to_str().unwrap(), &file_hash)? {
            return Ok(FileStats { chunks_created: 0 });
        }
        
        // Process text into chunks
        let chunks = self.text_processor.process_file(&content);
        let file_id = self.database.insert_file(
            path.to_str().unwrap(),
            &file_hash,
            metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as i64,
            metadata.len() as i64,
        )?;
        
        // Generate embeddings and store chunks
        for chunk in chunks {
            let embedding = self.embedder.embed(&chunk.content)?;
            self.database.insert_chunk(
                file_id,
                chunk.line_number,
                0, // start_char - simplified for now
                chunk.content.len(),
                &chunk.content,
                Some(&embedding),
            )?;
        }
        
        Ok(FileStats { chunks_created: chunks.len() })
    }
}
```

### Phase 7: Output Formatting & Testing (Week 7-8)

#### 7.1 Output Formatter
```rust
// src/cli/formatter.rs
use serde_json;

pub struct OutputFormatter;

impl OutputFormatter {
    pub fn format_results(results: &[SearchResult], format: &str) -> String {
        match format {
            "json" => Self::format_json(results),
            "plain" => Self::format_plain(results),
            _ => Self::format_plain(results),
        }
    }
    
    fn format_plain(results: &[SearchResult]) -> String {
        results
            .iter()
            .map(|r| format!("{}:{}:{:.2} {}", 
                r.file_path, 
                r.line_number, 
                r.score,
                r.content.chars().take(100).collect::<String>()
            ))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    fn format_json(results: &[SearchResult]) -> String {
        serde_json::to_string_pretty(results).unwrap_or_default()
    }
}
```

#### 7.2 Integration Tests
```rust
// tests/integration_tests.rs
use semisearch::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_end_to_end_search() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create test files
    std::fs::write(
        temp_dir.path().join("movies.txt"),
        "Ghostbusters\nSilence of the Lambs\nAce Ventura: Pet Detective\nIt's a Wonderful Life"
    ).unwrap();
    
    // Initialize search engine
    let config = Config::default();
    let db_path = temp_dir.path().join("test.db");
    let database = Database::new(&db_path).unwrap();
    let embedder = LocalEmbedder::new(&temp_dir.path().join("model.onnx")).await.unwrap();
    let search_engine = SearchEngine::new(database, embedder, config);
    
    // Index files
    let indexer = FileIndexer::new(/* ... */);
    indexer.index_directory(temp_dir.path().to_str().unwrap()).await.unwrap();
    
    // Search
    let results = search_engine.search("Jim Carrey", SearchOptions::default()).await.unwrap();
    
    // Verify results
    assert!(!results.is_empty());
    assert!(results.iter().any(|r| r.content.contains("Ace Ventura")));
}
```

## Development Timeline

### Development Checkpoints (Progressive Enhancement)

Each checkpoint produces a working, useful tool:

#### Checkpoint 1: MVP (Day 1-3)
**Goal:** Basic working search in < 300 lines of code
- [x] Simple CLI with search command
- [x] File traversal with walkdir
- [x] Line-by-line keyword matching
- [x] Display results with file:line:content
- **Deliverable:** `semisearch "TODO" ./src`

#### Checkpoint 2: Enhanced Search (Week 1)
**Goal:** Better search quality
- [ ] Fuzzy matching with edit distance
- [ ] Case-insensitive search
- [ ] Regex support
- [ ] Ignore files based on .gitignore
- **Deliverable:** `semisearch "TODo" --fuzzy`

#### Checkpoint 3: Persistent Index (Week 2)
**Goal:** Faster repeated searches
- [ ] SQLite storage for file metadata
- [ ] Basic caching of search results
- [ ] Incremental indexing (only new/changed files)
- [ ] Progress indicators
- **Deliverable:** `semisearch index . && semisearch "query"`

#### Checkpoint 4: Smart Search (Week 3-4)
**Goal:** Context-aware results
- [ ] TF-IDF scoring for better ranking
- [ ] Multi-word query support
- [ ] Snippet extraction with context
- [ ] Configuration file support
- **Deliverable:** `semisearch "error handling" --context=2`

#### Checkpoint 5: Semantic Search (Week 5-6)
**Goal:** Understanding meaning (optional based on system)
- [ ] Auto-detect system capabilities
- [ ] Download models if capable
- [ ] Fallback to TF-IDF if not
- [ ] Hybrid search mode
- **Deliverable:** `semisearch "Jim Carrey movies" --semantic`

#### Checkpoint 6: Production Ready (Week 7-8)
**Goal:** Polish and optimization
- [ ] Performance profiling and optimization
- [ ] Comprehensive error handling
- [ ] Multiple output formats
- [ ] Cross-platform testing
- [ ] Documentation and examples
- **Deliverable:** Production-ready binary

### Traditional Timeline (Reference)

#### Week 1-2: Foundation
- [ ] Setup development environment
- [ ] Create project structure  
- [ ] Implement CLI interface with clap
- [ ] Setup configuration management
- [ ] Basic error handling framework

#### Week 3-4: Storage & Text Processing
- [ ] SQLite database implementation
- [ ] Migration system setup
- [ ] Text processing pipeline
- [ ] File system scanning
- [ ] Basic keyword search

#### Week 5-6: ML Integration
- [ ] ONNX runtime integration
- [ ] Model download system
- [ ] Embedding generation
- [ ] Vector similarity search
- [ ] Result merging logic

#### Week 7-8: Polish & Testing
- [ ] Output formatting
- [ ] Performance optimization
- [ ] Comprehensive testing
- [ ] Documentation
- [ ] Binary packaging

## Testing Strategy

### Simplified Testing for Junior Developers

#### Test Data Repository
A companion repository `semisearch-test-data` provides:
```
test-data/
├── small/          # 10 files, good for unit tests
├── medium/         # 1000 files, integration tests  
├── large/          # 10000 files, performance tests
├── edge-cases/     # Special characters, encodings
└── scripts/        # Test automation scripts
```

#### Simple Test Scripts
```bash
# Run all tests with one command
./test/run-all.sh

# Test specific functionality
./test/test-search.sh "test query"
./test/test-index.sh ./sample-data
./test/test-performance.sh
```

#### Docker-Based Testing
```bash
# Run tests in isolated environment
docker run -v $(pwd):/app semisearch-test

# Test on different platforms
docker-compose up test-suite
```

### Traditional Testing Approach

#### Unit Tests
- Text processing functions
- Database operations
- Similarity calculations
- Configuration parsing
- Mock external dependencies

#### Integration Tests
- End-to-end search workflows
- File indexing processes
- Multi-format output testing
- Resource limit testing

#### Performance Tests
- Large file handling
- Concurrent search operations
- Memory usage profiling
- Search speed benchmarks
- Index size growth

## Deployment & Distribution

### Binary Packaging
```bash
# Release build with optimizations
cargo build --release

# Cross-compilation for different platforms
cargo install cross
cross build --target x86_64-unknown-linux-gnu --release
cross build --target x86_64-pc-windows-gnu --release
cross build --target x86_64-apple-darwin --release
```

### Installation Methods
1. **Direct download:** Pre-built binaries from releases
2. **Cargo install:** `cargo install semisearch` 
3. **Package managers:** Homebrew, APT, etc.

## Platform-Specific Considerations

### Raspberry Pi / ARM Devices
- Default to keyword-only mode
- Use pre-quantized models if available
- Reduce chunk size to 128 bytes
- Limit concurrent file operations

### Docker / Containers
- Pre-download models in Dockerfile
- Mount index directory as volume
- Use multi-stage builds for smaller images
- Example Dockerfile provided in repo

### Windows
- Handle long path names (> 260 chars)
- Use case-insensitive file matching by default
- Account for different line endings (CRLF)
- Provide PowerShell installation script

### macOS
- Handle quarantine attributes on downloaded models
- Respect Spotlight exclusions
- Use Accelerate framework for vector operations (if available)
- Provide Homebrew formula

### Linux Distributions
- Respect XDG base directory spec
- Provide .deb and .rpm packages
- systemd service for background indexing
- AppImage for universal compatibility

## Security & Privacy Considerations

### Data Protection
- All processing happens locally
- No network requests after initial model download
- Option to encrypt local index
- Respect `.gitignore` and similar exclusion files

### File Access
- Read-only access to source files
- Configurable exclusion patterns
- Size limits to prevent resource exhaustion

## Performance Targets

### Adaptive Performance Goals

Performance targets adjust based on system capabilities:

#### Minimal Systems (Raspberry Pi, <2GB RAM)
- **Startup Time:** < 1s (keyword mode)
- **Search Performance:**
  - Small projects (< 1000 files): < 2s
  - Medium projects (< 10000 files): < 10s
  - Large projects: Use incremental results
- **Memory Usage:**
  - Base memory: < 50MB
  - Per 1000 files: < 20MB

#### Standard Systems (4GB+ RAM)
- **Startup Time:** 
  - Cold start: < 500ms
  - Warm start: < 100ms
- **Search Performance:**
  - Small projects: < 1s
  - Medium projects: < 5s
  - Large projects: < 30s
- **Memory Usage:**
  - Base memory: < 100MB
  - Per 1000 files: < 50MB

#### High-End Systems (16GB+ RAM)
- **Startup Time:** < 50ms with pre-loaded models
- **Search Performance:**
  - Instant for cached queries
  - < 500ms for new queries
  - Parallel processing enabled
- **Memory Usage:**
  - Can cache entire index in memory
  - Pre-load embeddings for speed

## Future Enhancements

### Phase 2 Features
- Real-time file watching and incremental indexing
- Custom embedding models
- Query expansion and suggestions
- Integration with popular editors (VSCode, Vim)

### Advanced Features
- Multi-language support
- Custom similarity functions
- Distributed search across multiple machines
- Web interface for remote access

## Troubleshooting Guide

### Common Issues

#### Model/ML Related
1. **Model download fails:** 
   - Check internet connection and disk space
   - Use `--no-semantic` flag to skip ML features
   - Manually download model to `~/.semisearch/models/`

2. **ONNX Runtime errors:**
   - Fall back to keyword mode: `semisearch --mode=basic`
   - Try TF-IDF mode instead: `semisearch --mode=tfidf`
   - Check CPU architecture compatibility

#### Performance Issues
3. **Large files cause OOM:** 
   - Adjust `max_file_size_mb` in config
   - Use `--streaming` mode for large files
   - Enable swap space on low-memory systems

4. **Slow search:**
   - Check if running in correct mode for your hardware
   - Use `semisearch config --auto-detect` to optimize
   - Limit search scope: `semisearch "query" ./specific/dir`
   - Clear old cache: `semisearch cache --clear`

#### Search Quality
5. **No results found:**
   - Try fuzzy search: `semisearch "query" --fuzzy`
   - Lower similarity threshold: `--score=0.5`
   - Check indexed files: `semisearch status`
   - Use simpler queries without special characters

#### Platform-Specific
6. **Raspberry Pi issues:**
   - Ensure using keyword mode by default
   - Increase chunk processing delay
   - Use external USB storage for index

7. **Docker container issues:**
   - Mount index volume: `-v ~/.semisearch:/index`
   - Allocate enough memory: `--memory=1g`
   - Use pre-built images with models included

### Debug Commands
```bash
# Check system compatibility
semisearch doctor

# Show current configuration
semisearch config

# Test with minimal resources
semisearch search "query" --mode=basic --no-cache

# Benchmark your system
semisearch benchmark

# Rebuild index incrementally
semisearch index --incremental /path/to/directory

# Verbose debugging
RUST_LOG=debug semisearch search "query" --verbose
```

## Summary

This architectural plan provides a comprehensive roadmap for building a privacy-first semantic search CLI tool that works on ANY system - from a Raspberry Pi to a high-end workstation. 

### Key Principles

1. **Progressive Enhancement:** Start with basic keyword search that works everywhere, then add features based on available resources.

2. **Accessibility:** Written for junior/mid-level developers with clear checkpoints and working code at each stage.

3. **Adaptive Performance:** Automatically adjusts to system capabilities without user intervention.

4. **True Offline:** After optional initial model download, everything works completely offline.

5. **Modular Architecture:** Implement one search strategy at a time, test it, then move to the next.

### Implementation Path

**Week 1:** Get a working MVP with basic search  
**Week 2:** Add persistence and caching  
**Week 3-4:** Enhance search quality with fuzzy/TF-IDF  
**Week 5-6:** Add semantic search for capable systems  
**Week 7-8:** Polish, optimize, and package  

Remember: Each week produces a useful, working tool. You don't need to implement everything to have something valuable.

The modular design allows for incremental development and testing, while the clear phases ensure steady progress toward a production-ready tool that serves users across the entire spectrum of hardware capabilities.