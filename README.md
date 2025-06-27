# SemiSearch - Smart Local File Search

**Find what you're looking for in your files, even when you don't know the exact words.**

SemiSearch is a privacy-focused command-line tool that helps you search through your local files using natural language. Unlike traditional search tools that only match exact keywords, SemiSearch understands the *meaning* behind your search queries.

## What Makes SemiSearch Different?

### 🧠 **Semantic Understanding**
Traditional search tools look for exact word matches. If you search for "car", you won't find documents about "automobile" or "vehicle". SemiSearch understands that these words are related and finds them all.

**Example**: Searching for "error handling" will find:
- Code with try/catch blocks
- Documentation about exception management
- Comments mentioning "error recovery"
- Functions named "handleFailure" or "processException"

### 🔒 **100% Private**
Everything happens on your computer. No data is sent to the cloud. No AI services are called. Your files stay yours.

### ⚡ **Works Everywhere**
SemiSearch adapts to your system:
- **High-end computer?** Uses advanced AI models for the best semantic understanding
- **Older laptop?** Falls back to statistical methods that still work great
- **Raspberry Pi?** Runs in lightweight mode with basic but effective search

## Quick Start

### Installation

Download the pre-built binary for your system from the [releases page](https://github.com/kxrm/semisearch/releases), or install with cargo:

```bash
# Install from crates.io (when published)
cargo install semisearch

# Or build from source
git clone https://github.com/kxrm/semisearch.git
cd semisearch
cargo build --release
```

### Basic Usage

```bash
# Simple search (no subcommand needed)
semisearch "database connection"

# Search in a specific directory
semisearch "user authentication" src/

# Handle typos automatically
semisearch "databse" --fuzzy

# Get interactive help
semisearch help-me
```

### Getting Help

SemiSearch includes a comprehensive help system:

```bash
# Interactive help with examples and guidance
semisearch help-me

# Check if everything is working
semisearch status

# Quick command reference
semisearch --help

# Advanced options for power users
semisearch --advanced --help
```

The interactive help (`semisearch help-me`) is perfect for beginners. It provides:
- Real-time examples based on your queries
- Personalized suggestions
- Step-by-step guidance
- Common use cases for your situation

## Understanding Search Modes

SemiSearch offers different search strategies, each with its own strengths:

### 🎯 **Semantic Search** (Recommended)
Understands meaning and context. Best for finding conceptually related content.

```bash
semisearch search "user authentication" --mode semantic
```

**When to use**: When you want to find all content related to a concept, even if it uses different words.

### 🔤 **Keyword Search**
Traditional exact word matching. Fast and precise.

```bash
semisearch search "TODO" --mode keyword
```

**When to use**: When you know the exact text you're looking for.

### 🔀 **Fuzzy Search**
Handles typos and partial matches.

```bash
semisearch search "authentcation" --mode fuzzy
```

**When to use**: When you might have typos or only remember part of a word.

### 📐 **Regex Search**
Pattern matching for complex searches.

```bash
semisearch search "user_[0-9]+" --mode regex
```

**When to use**: When you need to match patterns like email addresses, IDs, or structured text.

### 📊 **TF-IDF Search**
Statistical ranking based on word importance.

```bash
semisearch search "machine learning" --mode tfidf
```

**When to use**: When you want results ranked by how important the search terms are in each document.

### 🔄 **Hybrid Search**
Combines semantic understanding with keyword matching for best results.

```bash
semisearch search "async error handling" --mode hybrid
```

**When to use**: When you want both exact matches and semantically related content.

## Common Use Cases

### For Developers

Find code examples and patterns:
```bash
# Find all error handling code
semisearch search "error handling try catch exception"

# Find async/await patterns
semisearch search "asynchronous programming" --mode semantic

# Find TODO comments with context
semisearch search "TODO|FIXME" --mode regex --context 2
```

### For Researchers

Search through papers and notes:
```bash
# Find content about a research topic
semisearch search "machine learning algorithms" --path ./research

# Find related concepts
semisearch search "neural networks" --mode semantic --score 0.3
```

### For Content Creators

Search through drafts and documents:
```bash
# Find all mentions of a topic across documents
semisearch search "climate change" --path ./articles

# Find similar paragraphs
semisearch search "renewable energy solutions" --mode semantic
```

## Advanced Features

### Indexing for Faster Searches

For large directories, create an index for instant searches:

```bash
# Create an index
semisearch index ./my-project

# Now searches are much faster
semisearch search "database queries"
```

### Fine-tuning Results

Control what results you see:

```bash
# Only show highly relevant results (0.0 to 1.0)
semisearch search "authentication" --score 0.7

# Limit number of results
semisearch search "user login" --limit 5

# Show surrounding context
semisearch search "password" --context 3
```

### System Capabilities

Check what search capabilities your system supports:

```bash
semisearch doctor
```

This shows:
- Available memory and processing power
- Whether AI models can be used
- Current search capabilities
- Recommendations for your system

## How Semantic Search Works

### In Simple Terms

Imagine you're looking for a book in a library:

1. **Traditional search**: You can only find books if you know the exact title
2. **Semantic search**: A librarian who understands what you're looking for and shows you related books

SemiSearch acts like that smart librarian for your files.

### The Technical Magic

1. **Text Understanding**: SemiSearch reads your files and understands what they're about
2. **Meaning Extraction**: It converts text into mathematical representations (called embeddings)
3. **Similarity Matching**: When you search, it finds files with similar meanings
4. **Smart Ranking**: Results are sorted by how closely they match your intent

### Privacy-First Approach

- **Local Processing**: All analysis happens on your computer
- **No Cloud Services**: Never sends your data anywhere
- **Offline Operation**: Works without internet (after initial setup)
- **Your Data Stays Yours**: No tracking, no analytics, no external APIs

## Installation Guide

### Option 1: Pre-built Binaries (Easiest)

1. Go to the [releases page](https://github.com/kxrm/semisearch/releases)
2. Download the binary for your system
3. Make it executable and run:

```bash
# Linux/macOS
chmod +x semisearch
./semisearch search "your query"

# Windows
semisearch.exe search "your query"
```

### Option 2: Install with Cargo

```bash
# Requires Rust toolchain
cargo install semisearch
```

### Option 3: Build from Source

```bash
# Clone repository
git clone https://github.com/kxrm/semisearch.git
cd semisearch

# Build with all features
cargo build --release --features neural-embeddings

# The binary will be in target/release/semisearch
```

### Enabling Advanced AI Features (Optional)

For the best semantic search experience, you can install ONNX Runtime. This enables neural network-based search for superior results.

<details>
<summary>Click for ONNX Runtime installation instructions</summary>

#### Linux
```bash
# Download ONNX Runtime
wget https://github.com/microsoft/onnxruntime/releases/download/v1.16.0/onnxruntime-linux-x64-1.16.0.tgz
tar xzf onnxruntime-linux-x64-1.16.0.tgz

# Option 1: Install system-wide
sudo cp onnxruntime-linux-x64-1.16.0/lib/libonnxruntime.so* /usr/local/lib/
sudo ldconfig

# Option 2: Use environment variable
export LD_LIBRARY_PATH=$PWD/onnxruntime-linux-x64-1.16.0/lib:$LD_LIBRARY_PATH
```

#### macOS
```bash
# Download ONNX Runtime
wget https://github.com/microsoft/onnxruntime/releases/download/v1.16.0/onnxruntime-osx-x64-1.16.0.tgz
tar xzf onnxruntime-osx-x64-1.16.0.tgz

# Set library path
export DYLD_LIBRARY_PATH=$PWD/onnxruntime-osx-x64-1.16.0/lib:$DYLD_LIBRARY_PATH
```

</details>

## Command Reference

### Search Command
```bash
semisearch search [OPTIONS] <QUERY>
```

**Common Options:**
- `-p, --path <PATH>` - Where to search (default: current directory)
- `-m, --mode <MODE>` - How to search: semantic, keyword, fuzzy, regex, tfidf, hybrid, auto
- `-l, --limit <N>` - Show only N results (default: 10)
- `-f, --format <FMT>` - Output format: plain or json
- `--context <N>` - Show N lines around each match

**Advanced Options:**
- `-s, --score <0-1>` - Minimum relevance score (default: 0.3)
- `--semantic-threshold <0-1>` - Semantic similarity threshold
- `--case-sensitive` - Match case exactly
- `--typo-tolerance` - Allow typos in matches
- `--files-only` - Show only file paths, not content

### Other Commands

**Index files for faster searching:**
```bash
semisearch index <directory>
```

**Check system status:**
```bash
semisearch status    # Show indexed files and database info
semisearch doctor    # Test system capabilities
semisearch config    # Display configuration
```

## Troubleshooting

### "Neural embeddings unavailable"

This is normal! SemiSearch automatically uses TF-IDF (a statistical method) instead, which still provides good semantic search. To enable neural embeddings:
1. Install ONNX Runtime (see installation guide)
2. Ensure you have at least 4GB RAM
3. Run `semisearch doctor` to verify

### Slow searches

Try these solutions:
1. **Create an index**: `semisearch index ./your-directory`
2. **Search specific folders**: `--path ./src` instead of entire disk
3. **Limit results**: `--limit 20` to get results faster

### No results found

When no matches are found, SemiSearch returns exit code 1 (following Unix conventions like `grep`). Common fixes:
1. **Lower the threshold**: `--score 0.1` (default is 0.3)
2. **Try fuzzy mode**: `--mode fuzzy` for typos
3. **Use simpler queries**: Break complex searches into parts
4. **Check file types**: Binary files are automatically skipped

**Exit Code Reference:**
- `0`: Matches found (success)
- `1`: No matches found or other errors
- `2`: Invalid arguments or command syntax

### Memory issues

For systems with limited RAM:
1. Use `--mode tfidf` or `--mode keyword`
2. Index smaller directories at a time
3. Set `--no-semantic` to disable neural features

## Performance Tips

### Speed Optimization

1. **Always index large directories first:**
   ```bash
   semisearch index ./large-project
   # Subsequent searches will be instant
   ```

2. **Use specific paths:**
   ```bash
   # Slower: searches everything
   semisearch search "function"

   # Faster: searches only src/
   semisearch search "function" --path ./src
   ```

3. **Adjust result limits:**
   ```bash
   # Get top 5 results quickly
   semisearch search "error" --limit 5
   ```

### Memory Optimization

For systems with limited RAM:
```bash
# Use lightweight search modes
semisearch search "query" --mode keyword
semisearch search "query" --mode tfidf

# Disable semantic features
semisearch search "query" --no-semantic
```

## Understanding the Technology

### What is Semantic Search?

Traditional search looks for exact matches. If you search for "car", it only finds "car", not "automobile" or "vehicle".

Semantic search understands meaning. It knows that:
- "car", "automobile", and "vehicle" are related
- "error handling" relates to "exception management"
- "user authentication" connects to "login system"

### How Does It Work?

1. **Text Analysis**: SemiSearch reads your files and breaks them into meaningful chunks
2. **Embedding Generation**: Each chunk is converted into numbers that represent its meaning
3. **Similarity Calculation**: When you search, your query is converted the same way
4. **Smart Matching**: Files with similar number patterns (meanings) are found
5. **Ranking**: Results are sorted by how well they match your intent

### Privacy Guarantees

- **No Internet Required**: Works completely offline after setup
- **No Data Collection**: We don't track searches or results
- **No Cloud Processing**: All AI runs on your computer
- **Open Source**: You can verify the code yourself

---

## Technical Documentation

<details>
<summary>Architecture and Implementation Details</summary>

### Architecture Overview

SemiSearch uses a modular, trait-based architecture with progressive enhancement:

```
User Query → Search Engine → Multiple Search Strategies → Ranked Results
                ↓
        Capability Detection
                ↓
    Neural/TF-IDF/Keyword Fallback
```

### Module Structure

```
src/
├── main.rs              # CLI interface
├── lib.rs               # Core library
├── core/                # Core functionality
│   ├── indexer.rs      # File indexing
│   └── embedder.rs     # Embedding generation
├── search/              # Search strategies
│   ├── keyword.rs      # Exact matching
│   ├── fuzzy.rs        # Typo tolerance
│   ├── regex_search.rs # Pattern matching
│   ├── tfidf.rs        # Statistical ranking
│   └── semantic.rs     # Neural search
├── storage/             # Data persistence
│   └── database.rs     # SQLite integration
└── text/               # Text processing
    ├── processor.rs    # Chunking and analysis
    └── tokenizer.rs    # Word extraction
```

### Progressive Enhancement

The system automatically adapts to available resources:

1. **Full Neural Mode** (4GB+ RAM, ONNX Runtime)
   - Uses transformer models (all-MiniLM-L6-v2)
   - 384-dimensional embeddings
   - Best semantic understanding

2. **TF-IDF Mode** (2GB+ RAM)
   - Statistical text analysis
   - Good semantic approximation
   - No external dependencies

3. **Basic Mode** (Any system)
   - Keyword and fuzzy matching
   - Minimal resource usage
   - Still effective for many use cases

### Key Technologies

- **Language**: Rust (performance and safety)
- **Database**: SQLite (embedded, no server needed)
- **ML Runtime**: ONNX Runtime (optional, for neural features)
- **Text Processing**: Unicode-aware tokenization
- **Search Algorithms**: Multiple strategies for different needs

### Performance Characteristics

- **Startup Time**: < 1s for basic search, 2-3s with neural embeddings
- **Search Speed**: 9-140ms for semantic search
- **Memory Usage**: 50MB base, scales with index size
- **Index Size**: ~25KB per 100 text files
- **Model Size**: 90MB (one-time download)

</details>

<details>
<summary>Development Information</summary>

### Building from Source

```bash
# Clone the repository
git clone https://github.com/kxrm/semisearch.git
cd semisearch

# Build with all features
cargo build --release --features neural-embeddings

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Project Status

**Current Version**: v0.6.0

All core features are complete and production-ready:
- ✅ CLI interface with multiple commands
- ✅ Multiple search strategies (keyword, fuzzy, regex, TF-IDF, semantic)
- ✅ SQLite persistence with incremental indexing
- ✅ Unicode-aware text processing
- ✅ Neural embeddings with ONNX Runtime
- ✅ Progressive enhancement based on system capabilities
- ✅ 100% offline operation (after initial model download)
- ✅ Enhanced error handling with context-aware messages
- ✅ Proper Unix exit codes (0=success, 1=no matches/errors, 2=invalid arguments)

### Testing

The project maintains comprehensive test coverage:
- **131 tests** across all modules
- **100% pass rate** on Linux/macOS
- Integration tests for all features
- Performance benchmarks
- Cross-platform CI/CD

Run tests with:
```bash
cargo test                    # All tests
cargo test --lib             # Unit tests only
cargo test --test integration_tests  # Integration tests
```

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Code style guidelines
- Testing requirements
- PR process
- Development setup

### Documentation

- [Architecture Plan](docs/SEMANTIC_SEARCH_ARCHITECTURE_PLAN.md) - Detailed technical specification
- [CI/CD Documentation](.github/CI.md) - GitHub Actions setup
- API documentation: `cargo doc --open`

</details>

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Built with Rust for performance and safety
- ONNX Runtime for neural inference
- Transformer models from Hugging Face
- Inspired by the need for private, intelligent search

---

**Remember**: Your files stay on your computer. Your privacy is preserved. Search smarter, not harder. 🔍
