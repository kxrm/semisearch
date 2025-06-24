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

### ✅ **Phase 4: Local Embeddings - COMPLETED**

**Local Embedding Capabilities:**
- ✅ **LocalEmbedder** - TF-IDF based embedding implementation with vocabulary building
- ✅ **SemanticSearch** - Cosine similarity search with reranking capabilities
- ✅ **Capability Detection** - Progressive enhancement based on system resources (Full/TfIdf/None)
- ✅ **Vocabulary Persistence** - Save/load embedding models for reuse across sessions
- ✅ **Batch Processing** - Efficient embedding generation for multiple texts
- ✅ **Integration Ready** - Hooks for indexer integration and storage layer support

**Semantic Search Features:**
- ✅ **Cosine Similarity Matching** - Mathematical similarity calculations for semantic relevance
- ✅ **Semantic Reranking** - Advanced result boosting with exact match preferences
- ✅ **Configurable Thresholds** - Adjustable similarity thresholds for result filtering
- ✅ **Normalized TF-IDF Vectors** - Proper vector normalization for accurate similarity
- ✅ **Thread-safe Design** - Arc-wrapped components for concurrent access
- ✅ **Privacy-First Architecture** - All processing remains completely local

**Architecture Compliance:**
- ✅ **Progressive Enhancement** - Graceful degradation from semantic → TF-IDF → keyword search
- ✅ **Minimal System Support** - Works on any system with basic keyword search fallback
- ✅ **Standard System Enhancement** - TF-IDF embeddings on capable systems
- ✅ **Future-Ready Design** - Architecture prepared for neural embedding upgrades
- ✅ **Offline-First Privacy** - No network requests, all data stays local

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

### Semantic Search (Phase 4)
```bash
# Semantic search with TF-IDF embeddings
cargo run -- search "error handling" --semantic

# Semantic search with custom similarity threshold
cargo run -- search "database connection" --semantic --similarity-threshold 0.3

# Combined semantic and traditional search
cargo run -- search "async function" --semantic --fuzzy --format json
```

### Available Options
- `--path, -p`: Target directory (default: current directory)
- `--format, -f`: Output format - `plain` or `json` (default: plain)
- `--limit, -l`: Maximum number of results (default: 10)
- `--score, -s`: Minimum similarity score (0.0-1.0, default: 0.0)
- `--fuzzy`: Enable fuzzy matching for typos and partial matches
- `--typo-tolerance`: Enable typo tolerance using edit distance
- `--max-edit-distance`: Maximum edit distance for typos (default: 2)
- `--regex`: Use regex pattern matching
- `--case-sensitive`: Perform case-sensitive search (default: case-insensitive)
- `--whole-words`: Match whole words only (works with regex)
- `--semantic`: Enable semantic search using local embeddings
- `--similarity-threshold`: Minimum semantic similarity (0.0-1.0, default: 0.1)

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

### Phase 4 Local Embeddings Dependencies
- `indicatif` - Progress bars for embedding operations
- `num_cpus` - CPU detection for capability assessment

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

#### Phase 4 Embeddings Tests (8 tests - 100% passing)
- End-to-end embedding workflow testing
- Vocabulary building and persistence
- Embedding generation and normalization
- Similarity calculations and edge cases
- Capability detection across systems
- Batch processing functionality
- Semantic search with reranking
- Error handling for empty vocabularies

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
- **Startup Time:** < 1s for basic keyword search
- **Indexing Speed:** 29 files in 9.26s (initial), 0.01s (incremental)
- **Search Speed:** Handles thousands of files efficiently with parallel processing
- **Memory Usage:** Minimal memory footprint with efficient data structures
- **Storage:** Efficient SQLite storage (784KB for 30 files, 5,797 chunks)
- **Caching:** Regex compilation caching for improved performance
- **Scalability:** Resource-aware search strategies with configurable limits
- **Semantic Search:** TF-IDF embedding generation with vocabulary persistence

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

With all core phases complete, potential future enhancements include:

1. **Neural Embeddings**:
   - ONNX Runtime integration for transformer models
   - Pre-trained embedding models (BERT, sentence-transformers)
   - GPU acceleration support
   - Advanced semantic understanding

2. **Advanced Features**:
   - Real-time file watching for automatic re-indexing
   - Query expansion and synonym handling
   - Document clustering and similarity
   - Multi-language semantic search improvements

3. **Production Optimizations**:
   - Background indexing processes
   - Advanced caching strategies
   - Query performance optimization
   - Cross-platform distribution

## CI/CD Pipeline

### GitHub Actions Workflows

- **CI Pipeline** (`.github/workflows/ci.yml`):
  - Multi-platform testing (Linux, Windows, macOS, ARM64)
  - Rust version matrix (stable, beta, MSRV 1.80.0)
  - Security auditing with cargo-audit and cargo-deny
  - Code quality checks (clippy, formatting)
  - Architecture plan compliance validation
  - All phase integration tests

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

### Local Embeddings & Semantic Search (Phase 4)
- **TF-IDF Embeddings**: Mathematical vector representations of text content
- **Cosine Similarity**: Accurate semantic similarity calculations
- **Vocabulary Management**: Persistent vocabulary building and model storage
- **Progressive Enhancement**: Automatic capability detection and graceful degradation
- **Privacy-First ML**: All machine learning processing remains completely local
- **Semantic Reranking**: Advanced result boosting with multiple scoring strategies

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