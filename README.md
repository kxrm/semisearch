# Semantic Search CLI Tool

A privacy-first CLI tool for semantic search across local files, built with Rust.

## Current Status: All Phases Complete ✅

All Phases 1, 2, 3, and 4 from the architecture plan have been implemented with comprehensive test coverage!

### ✅ **Phase 1: Foundation (MVP) - COMPLETED**

1. **✅ CLI Interface with Subcommands** - Proper `clap` implementation with:
   - `search` - Search for matches in files
   - `index` - Directory indexing with persistent storage
   - `config` - Configuration and database statistics

2. **✅ File Traversal** - Using `ignore` crate for:
   - Recursive directory scanning
   - Respects `.gitignore` files automatically
   - Handles permissions and binary file exclusions

3. **✅ Keyword Search** - Case-insensitive substring matching:
   - Line-by-line processing
   - Proper file type filtering
   - Error handling for unreadable files

4. **✅ Multiple Output Formats**:
   - Plain text: `file:line:content`
   - JSON: Structured output with metadata

### ✅ **Phase 2: Storage Layer - COMPLETED**

1. **✅ SQLite Database** - Persistent local storage:
   - Schema with files, chunks, and query cache tables
   - Database stored in `~/.semisearch/index.db`
   - Proper indexes and foreign key constraints
   - Automatic migration system

2. **✅ Incremental Indexing** - Smart change detection:
   - SHA-256 hash-based file change detection
   - Only processes modified files on re-indexing
   - **920x faster** re-indexing (0.01s vs 9.26s for unchanged files)
   - Configurable exclusion patterns

3. **✅ File Processing & Storage** - Efficient content management:
   - Text chunking with configurable sizes
   - Metadata tracking (file size, modification time, etc.)
   - Integration with existing Phase 3 text processing
   - Thread-safe operations with proper error handling

4. **✅ CLI Integration** - Production-ready commands:
   - `semisearch index <directory>` - Index files with progress feedback
   - `semisearch config` - Show database stats and configuration
   - User-friendly error messages and status reporting

### ✅ **Phase 3: Text Processing & Advanced Search - COMPLETED**

**New Modular Architecture:**
- ✅ **Trait-based Search Strategies** - Plugin architecture for extensible search
- ✅ **Text Processing Pipeline** - Comprehensive text analysis and preparation
- ✅ **Multiple Search Algorithms** - Keyword, Fuzzy, Regex, and TF-IDF search
- ✅ **Unicode Support** - Full internationalization and multi-language text processing

**Advanced Text Processing:**
- ✅ **TextProcessor** with configurable chunk lengths and stop words
- ✅ **Language Detection** - Automatic detection of programming languages (Rust, Python, JavaScript, Java, C, HTML, SQL)
- ✅ **Text Complexity Analysis** - Vocabulary diversity and readability scoring
- ✅ **Phrase Extraction** - 2-word and 3-word phrase combinations
- ✅ **Overlapping Chunks** - Better semantic coverage for large files
- ✅ **Unicode-aware Tokenization** - Proper handling of international characters

**Enhanced Search Strategies:**
- ✅ **Keyword Search** - Basic string matching with phrase bonuses and whole word matching
- ✅ **Fuzzy Search** - Multi-algorithm approach with SkimMatcherV2, edit distance, and typo tolerance
- ✅ **Regex Search** - Pattern detection, compilation caching, wildcard support, and complex patterns
- ✅ **TF-IDF Search** - Statistical ranking with document frequency analysis and index building

**Performance & Quality:**
- ✅ **Regex Compilation Caching** - Improved performance for repeated patterns
- ✅ **Parallel Processing Support** - Rayon integration for concurrent operations
- ✅ **Resource Management** - Each strategy specifies its computational requirements
- ✅ **Error Handling** - Comprehensive error handling with detailed messages

### ✅ **Phase 4: Neural Embeddings - COMPLETED**

**Neural Embedding Capabilities:**
- ✅ **ONNX Runtime Integration** - Local transformer model execution with all-MiniLM-L6-v2
- ✅ **LocalEmbedder** - Both TF-IDF and neural embedding implementations with vocabulary building
- ✅ **SemanticSearch** - Advanced cosine similarity search with neural and TF-IDF reranking
- ✅ **Capability Detection** - Progressive enhancement based on system resources (Neural/TfIdf/Keyword)
- ✅ **Model Download System** - Automatic model download with progress bars and caching
- ✅ **Vocabulary Persistence** - Save/load embedding models for reuse across sessions
- ✅ **Batch Processing** - Efficient embedding generation for multiple texts
- ✅ **Cross-platform Support** - Linux, macOS, and Windows compatibility (with platform-specific handling)

**Advanced Semantic Search Features:**
- ✅ **384-dimensional Neural Embeddings** - High-quality semantic representations using transformer models
- ✅ **Cosine Similarity Matching** - Mathematical similarity calculations for semantic relevance
- ✅ **Semantic Reranking** - Advanced result boosting with exact match preferences
- ✅ **Configurable Thresholds** - Adjustable similarity thresholds for result filtering
- ✅ **Normalized Vectors** - Proper vector normalization for accurate similarity calculations
- ✅ **Thread-safe Design** - Arc-wrapped components for concurrent access
- ✅ **Privacy-First Architecture** - All ML processing remains completely local after initial model download

**Production-Ready Features:**
- ✅ **Progressive Enhancement** - Graceful degradation from neural → TF-IDF → keyword search
- ✅ **System Requirements Detection** - 4GB+ RAM threshold for neural embeddings
- ✅ **Local Model Caching** - Models stored in `~/.semisearch/models/` for offline use
- ✅ **Doctor Command** - System capability analysis and troubleshooting
- ✅ **Platform Compatibility** - Neural embeddings on Linux/macOS, graceful fallback on Windows
- ✅ **Offline-First Privacy** - No network requests after initial model download

## Usage

### Storage & Indexing Commands
```bash
# Index your project for persistent storage
cargo run -- index ./src

# Check database stats and configuration
cargo run -- config

# Re-index (lightning fast with incremental updates)
cargo run -- index ./src
```

### Basic Search Commands
```bash
# Basic keyword search
cargo run -- search "TODO" --path ./src

# JSON output with limit
cargo run -- search "query" --format json --limit 5

# Case-sensitive search
cargo run -- search "TODO" --case-sensitive
```

### Advanced Search Features
```bash
# Fuzzy matching (handles typos and partial matches)
cargo run -- search "TOOD" --fuzzy

# Enhanced typo tolerance with edit distance
cargo run -- search "TODO" --typo-tolerance --max-edit-distance 2

# Regex pattern matching
cargo run -- search "TODO.*:" --regex

# Email pattern matching
cargo run -- search "\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b" --regex

# Wildcard patterns
cargo run -- search "*.txt" --regex

# Combined options with scoring
cargo run -- search "error" --fuzzy --score 0.5 --format json

# Whole word matching
cargo run -- search "test" --whole-words
```

### Neural Semantic Search (Phase 4)
```bash
# Neural semantic search (auto-detects system capabilities)
cargo run -- search "error handling" --mode semantic

# Neural semantic search with custom similarity threshold
cargo run -- search "database connection" --mode semantic --semantic-threshold 0.3

# Force neural embeddings (if system supports it)
cargo run -- search "async function" --semantic

# TF-IDF fallback semantic search
cargo run -- search "database query" --mode tfidf

# Combined semantic and traditional search modes
cargo run -- search "async function" --mode hybrid --format json

# Check system capabilities for neural embeddings
cargo run -- doctor
```

### Available Options
- `--path, -p`: Target directory (default: current directory)
- `--format, -f`: Output format - `plain` or `json` (default: plain)
- `--limit, -l`: Maximum number of results (default: 10)
- `--score, -s`: Minimum similarity score (0.0-1.0, default: 0.0)
- `--mode, -m`: Search mode - `auto`, `semantic`, `keyword`, `fuzzy`, `regex`, `tfidf`, `hybrid` (default: auto)
- `--fuzzy`: Enable fuzzy matching for typos and partial matches
- `--typo-tolerance`: Enable typo tolerance using edit distance
- `--max-edit-distance`: Maximum edit distance for typos (default: 2)
- `--regex`: Use regex pattern matching
- `--case-sensitive`: Perform case-sensitive search (default: case-insensitive)
- `--whole-words`: Match whole words only (works with regex)
- `--semantic`: Force neural semantic search (if system supports it)
- `--no-semantic`: Disable neural embeddings, use TF-IDF fallback
- `--semantic-threshold`: Minimum semantic similarity (0.0-1.0, default: 0.1)

## Architecture

The project follows a modular, trait-based architecture with progressive enhancement and persistent storage:

### Current Module Structure
```
src/
├── main.rs              # CLI interface and command handling
├── lib.rs               # Core library exports
├── core/                # Core functionality
│   ├── mod.rs          # Core module exports
│   ├── indexer.rs      # File indexing with change detection
│   └── embedder.rs     # Phase 4: Local embedding implementation
├── storage/             # Persistent storage layer
│   ├── mod.rs          # Storage module exports
│   └── database.rs     # SQLite database integration
├── search/              # Search strategies and algorithms
│   ├── mod.rs          # Search engine and trait definitions
│   ├── keyword.rs      # Keyword search implementation
│   ├── fuzzy.rs        # Fuzzy search with typo tolerance
│   ├── regex_search.rs # Regex pattern matching
│   ├── tfidf.rs        # TF-IDF statistical ranking
│   ├── semantic.rs     # Phase 4: Semantic search with embeddings
│   └── strategy.rs     # Helper functions for search strategies
└── text/               # Text processing pipeline
    ├── mod.rs          # Text processing exports
    ├── processor.rs    # Main text processing logic
    └── tokenizer.rs    # Unicode-aware tokenization

migrations/
└── 001_initial.sql     # Database schema with indexes
```

### Architecture Phases
- **Phase 1: Foundation** ✅ - CLI interface, basic search, file traversal
- **Phase 2: Storage Layer** ✅ - SQLite persistence, incremental indexing, change detection
- **Phase 3: Text Processing** ✅ - Modular architecture, advanced text processing, multiple search strategies
- **Phase 4: Local Embeddings** ✅ - TF-IDF embeddings, semantic search, privacy-first ML

## Dependencies

### Core Dependencies
- `clap` - Command line argument parsing with derive macros
- `ignore` - Git-aware file traversal (respects .gitignore)
- `serde` + `serde_json` - JSON serialization for output
- `anyhow` - Better error handling and propagation

### Phase 2 Storage Dependencies
- `rusqlite` - SQLite database integration with bundled SQLite
- `sha2` - SHA-256 hashing for file change detection
- `chrono` - Date/time handling for metadata
- `dirs` - Cross-platform user directory detection

### Phase 3 Text Processing Dependencies
- `fuzzy-matcher` - Advanced fuzzy string matching with SkimMatcherV2
- `regex` - Regular expression pattern matching with caching
- `edit-distance` - String similarity calculations
- `unicode-segmentation` - Unicode-aware text processing
- `thiserror` - Custom error types
- `rayon` - Parallel processing capabilities
- `rustc-hash` - High-performance hash maps

### Phase 4 Neural Embeddings Dependencies
- `ort` - ONNX Runtime for transformer model execution
- `tokenizers` - HuggingFace tokenizers for text preprocessing
- `ndarray` - N-dimensional arrays for tensor operations
- `reqwest` - HTTP client for model downloads
- `indicatif` - Progress bars for model downloads and embedding operations
- `num_cpus` - CPU detection for capability assessment
- `sys-info` - System information for RAM detection

### Development Dependencies
- `criterion` - Performance benchmarking and regression testing
- `proptest` - Property-based testing for fuzzy scenarios
- `tempfile` - Temporary file handling for tests
- `tokio` - Async runtime for test infrastructure

## Testing

### Comprehensive Test Coverage: 131 Tests ✅

The project maintains industry-standard test coverage with multiple test categories:

#### Core Library Tests (123 tests - 100% passing)
- **Search Module**: Enhanced with semantic search tests
- **Text Processor**: All processing methods, language detection, complexity
- **Keyword Search**: Scoring, case sensitivity, phrase matching
- **Fuzzy Search**: Typo tolerance, edit distance, multi-algorithm scoring
- **Regex Search**: Pattern detection, caching, complex patterns
- **TF-IDF Search**: Indexing, scoring, statistics
- **Semantic Search**: Embedding generation, similarity calculations, reranking
- **Tokenizer**: Classification, Unicode handling, position tracking
- **Strategy Helper**: Context, merging, highlighting
- **Integration**: Cross-module functionality, file processing

#### Integration Tests (8 tests - 100% passing)
- End-to-end search workflows
- File system integration
- Performance testing
- Output format verification
- Case sensitivity handling
- Large directory processing
- Fuzzy and regex search integration

#### Phase 2 Storage Tests (9 tests - 100% passing)
- Database operations and schema
- Incremental indexing with change detection
- File exclusion patterns and size limits
- Text processing integration
- Error handling and edge cases
- End-to-end indexing workflows

#### Phase 3 Comprehensive Tests (19 tests - 100% passing)
- Multi-strategy search coordination
- Text processing comprehensive scenarios
- Language detection across multiple languages
- Performance with large content
- Edge cases and error handling
- File integration scenarios

#### Phase 4 Neural Embeddings Tests (8 tests - 100% passing on Linux/macOS)
- End-to-end neural embedding workflow testing
- ONNX Runtime integration and model loading
- Vocabulary building and persistence
- 384-dimensional embedding generation and normalization
- Similarity calculations and edge cases
- Capability detection across systems
- Batch processing functionality
- Semantic search with neural reranking
- Error handling for empty vocabularies
- **Note**: Neural embedding tests are excluded on Windows due to ONNX Runtime compatibility issues

### Test Categories Covered
- ✅ **Functionality Tests**: All public methods and core features
- ✅ **Edge Case Tests**: Empty inputs, special characters, boundary conditions
- ✅ **Performance Tests**: Large content handling, concurrent access, incremental indexing
- ✅ **Integration Tests**: Multi-strategy coordination, file processing, database operations
- ✅ **Configuration Tests**: All search options and parameters
- ✅ **Error Handling**: Invalid inputs, malformed patterns, database errors
- ✅ **Unicode Tests**: International character support
- ✅ **Concurrent Tests**: Thread safety verification
- ✅ **Semantic Tests**: Embedding generation, similarity calculations, vocabulary management

### Running Tests

```bash
# Run all tests (unit + integration + storage + embeddings)
cargo test

# Run core library tests only
cargo test --lib

# Run integration tests only
cargo test --test integration_tests

# Run Phase 2 storage tests
cargo test --test phase2_storage_tests

# Run Phase 3 comprehensive tests
cargo test --test phase3_text_processing_tests

# Run Phase 4 embeddings tests
cargo test --test phase4_embeddings_tests

# Run with output for debugging
cargo test -- --nocapture

# Run specific test categories
cargo test search::keyword::tests    # Keyword search tests
cargo test storage::database::tests  # Database tests
cargo test core::indexer::tests      # Indexer tests
cargo test core::embedder::tests     # Embedder tests
cargo test search::semantic::tests   # Semantic search tests
```

## Performance

### Current Performance Characteristics
- **Startup Time:** < 1s for basic keyword search, 2-3s for neural embedding initialization
- **Indexing Speed:** 29 files in 9.26s (initial), 0.01s (incremental)
- **Search Speed:** 9-140ms for semantic search, handles thousands of files efficiently
- **Neural Embeddings:** 384-dimensional vectors, ~90MB model download (one-time)
- **Memory Usage:** 4GB+ RAM recommended for neural embeddings, graceful degradation below
- **Storage:** Efficient SQLite storage (784KB for 30 files, 5,797 chunks) + model cache
- **Caching:** Model caching in `~/.semisearch/models/`, regex compilation caching
- **Scalability:** Resource-aware search strategies with automatic capability detection
- **Cross-platform:** Full neural support on Linux/macOS, TF-IDF fallback on Windows

### Performance Features
- **Incremental Indexing:** SHA-256 change detection provides 920x speedup for unchanged files
- **Parallel Processing:** Rayon integration for concurrent file processing
- **Efficient Data Structures:** rustc-hash for high-performance hash maps
- **Database Optimization:** Proper indexes and query optimization
- **Regex Caching:** Compiled patterns cached for repeated use
- **Resource Management:** Each search strategy specifies computational requirements
- **Configurable Limits:** Adjustable result limits and scoring thresholds
- **Embedding Persistence:** Vocabulary models saved for reuse across sessions
- **Progressive Enhancement:** Automatic capability detection and graceful degradation

## Future Enhancements

With all core phases complete including neural embeddings, potential future enhancements include:

1. **Advanced Neural Features**:
   - GPU acceleration support for faster embedding generation
   - Additional pre-trained models (sentence-transformers, domain-specific models)
   - Multi-language transformer models for international content
   - Fine-tuning capabilities for domain-specific search

2. **Enhanced Functionality**:
   - Real-time file watching for automatic re-indexing
   - Query expansion and synonym handling
   - Document clustering and similarity analysis
   - Vector database integration for large-scale deployments
   - Semantic query suggestions and auto-completion

3. **Production Optimizations**:
   - Background indexing processes with incremental neural embedding updates
   - Advanced caching strategies for embedding vectors
   - Query performance optimization with vector indexes
   - Cross-platform distribution with pre-built binaries
   - Windows ONNX Runtime compatibility improvements

## CI/CD Pipeline

### GitHub Actions Workflows

- **CI Pipeline** (`.github/workflows/ci.yml`):
  - Multi-platform testing (Linux, Windows, macOS, ARM64)
  - Rust version matrix (stable, beta, MSRV 1.85.0)
  - Neural embedding tests (Linux/macOS only, Windows excluded due to ONNX Runtime issues)
  - Security auditing with cargo-audit and cargo-deny
  - Code quality checks (clippy, formatting)
  - Architecture plan compliance validation
  - All phase integration tests with platform-specific handling

- **Auto-merge Pipeline** (`.github/workflows/auto-merge-solo.yml`):
  - Automatic PR merging for solo development
  - Post-merge CI validation on main branch
  - Comprehensive quality gates

- **Release Pipeline** (`.github/workflows/release.yml`):
  - Automated releases on git tags
  - Multi-platform binary builds
  - Crates.io publishing (when configured)
  - Release notes generation

### Status Badges

[![CI](https://github.com/kxrm/semisearch/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/kxrm/semisearch/actions)

## Project Structure

```
src/
├── main.rs              # CLI interface and command handling
├── lib.rs               # Core library exports and integration
├── core/                # Core functionality
│   ├── mod.rs          # Core module exports
│   ├── indexer.rs      # File indexing with change detection (418 lines)
│   └── embedder.rs     # Local embedding implementation (485 lines)
├── storage/             # Persistent storage layer
│   ├── mod.rs          # Storage module exports
│   └── database.rs     # SQLite database integration (439 lines)
├── search/              # Search strategies and algorithms
│   ├── mod.rs          # Search engine and trait definitions
│   ├── keyword.rs      # Keyword search implementation
│   ├── fuzzy.rs        # Fuzzy search with typo tolerance
│   ├── regex_search.rs # Regex pattern matching
│   ├── tfidf.rs        # TF-IDF statistical ranking
│   ├── semantic.rs     # Semantic search with embeddings (382 lines)
│   └── strategy.rs     # Helper functions for search strategies
└── text/               # Text processing pipeline
    ├── mod.rs          # Text processing exports
    ├── processor.rs    # Main text processing logic
    └── tokenizer.rs    # Unicode-aware tokenization

migrations/
└── 001_initial.sql     # Database schema with indexes (41 lines)

tests/
├── integration_tests.rs           # End-to-end testing (8 tests)
├── phase2_storage_tests.rs        # Phase 2 storage tests (9 tests)
├── phase3_text_processing_tests.rs # Comprehensive Phase 3 tests (19 tests)
├── phase4_embeddings_tests.rs     # Phase 4 embeddings tests (8 tests)
├── run-all.sh                     # Comprehensive test runner
├── test-search.sh                 # Search functionality tests
├── test-performance.sh            # Performance benchmarking
└── validate-ci.sh                 # CI environment validation

.github/
├── workflows/
│   ├── ci.yml              # CI/CD pipeline
│   ├── auto-merge-solo.yml # Auto-merge workflow
│   └── release.yml         # Release automation
└── README.md               # CI/CD documentation

docs/
└── SEMANTIC_SEARCH_ARCHITECTURE_PLAN.md  # Complete technical specification
```

## Key Features Implemented

### Persistent Storage Layer (Phase 2)
- **SQLite Integration**: Complete database schema with files, chunks, and query cache
- **Incremental Indexing**: SHA-256 hash-based change detection for efficiency
- **File Processing**: Integration with Phase 3 text processing pipeline
- **CLI Commands**: `index` and `config` commands for database management
- **Privacy-First**: All data stored locally in `~/.semisearch/index.db`

### Advanced Text Processing (Phase 3)
- **Multi-language Support**: Automatic detection of Rust, Python, JavaScript, Java, C, HTML, SQL
- **Stop Word Filtering**: Customizable stop word lists for better search relevance
- **Phrase Extraction**: Automatic extraction of 2-word and 3-word meaningful phrases
- **Text Complexity Scoring**: Vocabulary diversity analysis for content assessment
- **Overlapping Chunk Processing**: Better semantic coverage for large documents

### Sophisticated Search Algorithms
- **Multi-Algorithm Fuzzy Matching**: SkimMatcherV2, edit distance, and substring bonuses
- **TF-IDF Statistical Ranking**: Document frequency analysis with phrase bonuses
- **Advanced Regex Processing**: Pattern detection, caching, and wildcard support
- **Contextual Scoring**: Position bonuses, length penalties, and relevance factors

### Neural Embeddings & Semantic Search (Phase 4)
- **Neural Embeddings**: 384-dimensional transformer-based semantic representations using all-MiniLM-L6-v2
- **ONNX Runtime Integration**: Local execution of pre-trained transformer models
- **TF-IDF Fallback**: Mathematical vector representations for systems without neural capability
- **Cosine Similarity**: Accurate semantic similarity calculations for both neural and TF-IDF vectors
- **Model Management**: Automatic model download, caching, and version management
- **Progressive Enhancement**: Automatic capability detection with graceful degradation (Neural → TF-IDF → Keyword)
- **Privacy-First ML**: All machine learning processing remains completely local after initial model download
- **Advanced Reranking**: Multi-strategy result boosting with neural and statistical scoring
- **Cross-platform Support**: Linux/macOS neural embeddings with Windows TF-IDF fallback

### Developer Experience
- **Trait-based Architecture**: Easy to extend with new search strategies
- **Comprehensive Error Handling**: Detailed error messages with context
- **Unicode-aware Processing**: Full internationalization support
- **Performance Monitoring**: Resource requirements and benchmarking capabilities
- **Production Ready**: Comprehensive CLI with persistent storage and semantic capabilities

This implementation provides a complete semantic search solution with all four phases of the architecture plan implemented. The system seamlessly integrates traditional search algorithms with modern semantic understanding, while maintaining privacy-first principles and providing a robust foundation for future enhancements.

## Documentation

- [Architecture Plan](docs/SEMANTIC_SEARCH_ARCHITECTURE_PLAN.md) - Complete technical specification and implementation roadmap
- [CI/CD Documentation](.github/CI.md) - GitHub Actions setup and troubleshooting 