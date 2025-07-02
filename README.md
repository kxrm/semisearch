# SemiSearch - Smart Local File Search

**Find what you're looking for in your files, even when you don't know the exact words.**

SemiSearch is a privacy-focused command-line tool that helps you search through your local files using intelligent text analysis. Unlike traditional search tools that only match exact keywords, SemiSearch understands relationships between words and concepts.

## What Makes SemiSearch Different?

### 🧠 **Intelligent Text Analysis**
Traditional search tools look for exact word matches. If you search for "car", you won't find documents about "automobile" or "vehicle". SemiSearch uses statistical analysis (TF-IDF) to understand relationships between words and find related concepts.

**Example**: Searching for "error handling" will find:
- Code with try/catch blocks  
- Documentation about exception management
- Comments mentioning "error recovery"
- Functions named "handleFailure" or "processException"

### 🔒 **100% Private**
Everything happens on your computer. No data is sent to the cloud. No AI services are called. Your files stay yours.

### ⚡ **Works Immediately**
SemiSearch adapts to your system:
- **Any computer**: Statistical text analysis (TF-IDF) that works great
- **Zero configuration**: Just install and start searching
- **Progressive enhancement**: Advanced features unlock as you learn

## Quick Start

### Installation

**Option 1: Build from Source (Recommended)**
```bash
# Clone repository
git clone https://github.com/kxrm/semisearch.git
cd semisearch

# Build release version
cargo build --release --features neural-embeddings

# The binary will be in target/release/semisearch
./target/release/semisearch --help
```

**Option 2: Download Pre-built Binaries**
Download the pre-built binary for your system from the [releases page](https://github.com/kxrm/semisearch/releases) (when available).

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

# Detailed system diagnostics
semisearch doctor

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

## Understanding Search Capabilities

SemiSearch automatically chooses the best search strategy based on your query:

### 🎯 **Smart Query Analysis**
The tool automatically detects what kind of search you need:
- **Simple terms** → Fast keyword search
- **Conceptual queries** → Semantic search for meaning and relationships
- **Poor keyword results** → Automatic semantic fallback
- **Code patterns** → Code-aware search
- **Typos detected** → Automatic fuzzy matching

### 🔤 **Keyword Search**
Traditional exact word matching. Fast and precise.

**When used**: For exact text you know exists (like "TODO" comments)

### 🔀 **Fuzzy Search**  
Handles typos and partial matches.

```bash
semisearch "authentcation" --fuzzy
```

**When used**: When you add `--fuzzy` or when typos are detected

### 📊 **TF-IDF Analysis**
Statistical ranking based on word importance and relationships.

**When used**: For conceptual searches like "error handling patterns"

### 📐 **Regex Search** (Advanced Mode)
Pattern matching for complex searches.

```bash
semisearch --advanced "user_[0-9]+" --mode regex
```

**When used**: In advanced mode for pattern matching

## Common Use Cases

### For Developers

Find code examples and patterns:
```bash
# Find all error handling code
semisearch "error handling"

# Find async patterns  
semisearch "async await"

# Find TODO comments
semisearch "TODO"

# Find function definitions
semisearch "fn main"
```

### For Researchers

Search through papers and notes:
```bash
# Find content about a research topic
semisearch "machine learning" docs/

# Find related concepts
semisearch "neural networks" --fuzzy
```

### For Content Creators

Search through drafts and documents:
```bash
# Find all mentions of a topic
semisearch "climate change"

# Find similar content
semisearch "renewable energy"
```

## Advanced Features (Power Users)

### Indexing for Faster Searches

For large directories, create an index for instant searches:

```bash
# Create an index
semisearch index ./my-project

# Now searches are much faster
semisearch "database queries"
```

### Advanced Mode

Access power-user features:

```bash
# Enable advanced options
semisearch --advanced --help

# Use specific search modes
semisearch --advanced "pattern" --mode regex

# Include/exclude file patterns
semisearch --advanced "TODO" --include "*.rs"
semisearch --advanced "test" --exclude "*test*"

# Fine-tune relevance
semisearch --advanced "query" --semantic-threshold 0.8
```

### Fine-tuning Results

Control what results you see:

```bash
# Show surrounding context lines
./target/release/semisearch --advanced "password" --context 3

# Use semantic search for conceptual queries
./target/release/semisearch --advanced "authentication" --mode semantic

# Output as JSON
./target/release/semisearch --advanced "config" --format json

# Show only file paths
./target/release/semisearch --advanced "function" --files-only
```

### System Capabilities

Check what search capabilities your system supports:

```bash
semisearch doctor
```

This shows:
- Available search methods
- Database status and indexed files
- Performance metrics
- Recommendations for your system

## Progressive Learning

SemiSearch grows with you:

### For Beginners
- **Encouraging tips**: "Great start! Keep exploring"
- **Clear guidance**: When searches fail, you know exactly what to try next
- **Zero setup**: Works immediately without configuration

### For Intermediate Users  
- **Feature discovery**: "Try --fuzzy for spelling variations"
- **Contextual suggestions**: Based on your actual usage patterns
- **Learning progression**: Tips become more advanced as you use the tool

### For Experienced Users
- **Power features**: "Try --advanced for more options"
- **Efficiency tips**: "You're using semisearch a lot! Here are advanced features..."
- **All capabilities**: Full access to regex, filtering, and advanced modes

## How Text Analysis Works

### In Simple Terms

Imagine you're looking for a book in a library:

1. **Traditional search**: You can only find books if you know the exact title
2. **SemiSearch**: A librarian who understands what you're looking for and shows you related books

SemiSearch acts like that smart librarian for your files.

### The Technical Magic

1. **Text Understanding**: SemiSearch reads your files and understands what they're about
2. **Statistical Analysis**: It uses TF-IDF to find relationships between words and concepts
3. **Similarity Matching**: When you search, it finds files with related meanings
4. **Smart Ranking**: Results are sorted by how closely they match your intent

### Privacy-First Approach

- **Local Processing**: All analysis happens on your computer
- **No Cloud Services**: Never sends your data anywhere  
- **Offline Operation**: Works without internet
- **Your Data Stays Yours**: No tracking, no analytics, no external APIs

## Installation Guide

### Option 1: Build from Source (Required)

```bash
# Clone the repository
git clone https://github.com/kxrm/semisearch.git
cd semisearch

# Build with neural embeddings support
cargo build --release --features neural-embeddings

# Run the tool
./target/release/semisearch "your query"
```

### Option 2: Pre-built Binaries (When Available)

Pre-built binaries may be available on the [releases page](https://github.com/kxrm/semisearch/releases) for some versions.

### Option 3: Development Build

```bash
# Clone repository
git clone https://github.com/kxrm/semisearch.git
cd semisearch

# Build release version
cargo build --release

# The binary will be in target/release/semisearch
```

## Command Reference

### Basic Commands
```bash
# Search (default command)
semisearch "your query"
semisearch "query" path/to/search/

# Get help and status
semisearch help-me     # Interactive help
semisearch status      # Quick health check  
semisearch doctor      # Detailed diagnostics
semisearch --help      # Command reference
```

### Simple Options
```bash
# Handle typos
semisearch "databse" --fuzzy

# Find exact matches
semisearch "exact phrase" --exact

# Show more context
semisearch "function" --context 2

# JSON output
semisearch "config" --format json
```

### Advanced Mode (Power Users)
```bash
# Enable all options
semisearch --advanced --help

# Specific search modes
semisearch --advanced "query" --mode semantic
semisearch --advanced "pattern" --mode regex

# File filtering
semisearch --advanced "TODO" --include "*.rs"
semisearch --advanced "test" --exclude "*test*"

# Fine-tune relevance
semisearch --advanced "query" --semantic-threshold 0.8
```

### Indexing Commands
```bash
semisearch index .              # Index current directory
semisearch index ./src          # Index specific directory
semisearch status               # Check indexed files
```

## Troubleshooting

### No results found

When no matches are found, SemiSearch provides helpful suggestions:
```bash
semisearch "nonexistent"
# Shows: Try different words, check spelling, search broader locations
```

Common fixes:
1. **Try fuzzy search**: `semisearch "query" --fuzzy`
2. **Use simpler terms**: Break complex searches into parts
3. **Check the location**: Make sure you're searching the right directory

### Too many results

SemiSearch automatically provides tips:
```bash
semisearch "function" 
# Shows: "Many results found. Use more specific terms or search in specific folders"
```

Try:
1. **Be more specific**: `semisearch "function validateUser"`
2. **Search specific folders**: `semisearch "TODO" src/`
3. **Use exact phrases**: `semisearch "exact phrase" --exact`

### Slow searches

Speed up searches:
1. **Create an index**: `semisearch index .`
2. **Search specific folders**: `semisearch "query" ./src`
3. **Check system status**: `semisearch doctor`

### Getting Help

SemiSearch provides contextual help based on what you're trying to do:
- **Interactive guidance**: `semisearch help-me`
- **System status**: `semisearch status`
- **Detailed diagnostics**: `semisearch doctor`
- **Command reference**: `semisearch --help`

## Performance & Capabilities

### Speed Optimization

1. **Index large directories**:
   ```bash
   semisearch index ./large-project
   # Subsequent searches will be much faster
   ```

2. **Use specific paths**:
   ```bash
   # Faster: searches only src/
   semisearch "function" src/
   ```

3. **Leverage automatic optimization**:
   - SemiSearch automatically chooses the fastest method for your query
   - Simple searches use fast keyword matching
   - Complex searches use statistical analysis

### Current Capabilities

SemiSearch provides these search capabilities:
- ✅ **Keyword search**: Fast exact matching for precise queries
- ✅ **Fuzzy search**: Handles typos and similar words (`--fuzzy`)
- ✅ **Semantic search**: Automatically used for conceptual queries and as fallback when keyword search fails
- ✅ **TF-IDF analysis**: Statistical text analysis for concept matching
- ✅ **Regex patterns**: Advanced pattern matching (`--advanced --mode regex`)
- ✅ **Context lines**: Show surrounding lines around matches (`--advanced --context N`)
- ✅ **Project detection**: Automatically adapts to Rust, JavaScript, Python, and documentation projects
- ✅ **File filtering**: Include/exclude patterns (`--include "*.rs"`)
- ✅ **Multiple output formats**: Plain text and JSON (`--format json`)

**Note**: Semantic search works automatically in basic searches - no `--advanced` flag needed!

Check your specific capabilities: `./target/release/semisearch doctor`

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Built with Rust for performance and safety
- TF-IDF implementation for intelligent text analysis
- Inspired by the need for private, intelligent search

---

**Remember**: Your files stay on your computer. Your privacy is preserved. Search smarter, not harder. 🔍
