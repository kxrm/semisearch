# Semantic Search CLI Tool

A privacy-first CLI tool for semantic search across local files, built with Rust.

## MVP Features (Checkpoint 1) ✅

All MVP features from the architecture plan have been implemented:

### ✅ **COMPLETED Features:**

1. **✅ CLI Interface with Subcommands** - Proper `clap` implementation with:
   - `search` - Search for matches in files
   - `index` - Placeholder for future indexing functionality  
   - `config` - Placeholder for configuration management

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

5. **✅ Comprehensive Testing** - 10 unit tests covering:
   - File matching functionality
   - Directory traversal
   - Result limiting
   - Edge cases (empty directories, no matches)
   - Fuzzy matching with typos
   - Regex pattern matching
   - Case-sensitive search
   - Search result scoring and ranking

## Usage

### Search Command
```bash
# Basic search
cargo run -- search "TODO" --path ./src

# JSON output with limit
cargo run -- search "query" --format json --limit 5

# Fuzzy matching (handles typos)
cargo run -- search "TOOD" --fuzzy

# Regex pattern matching
cargo run -- search "TODO.*:" --regex

# Case-sensitive search
cargo run -- search "TODO" --case-sensitive

# Combined options with scoring
cargo run -- search "error" --fuzzy --score 0.5 --format json
```

### Available Options
- `--path, -p`: Target directory (default: current directory)
- `--format, -f`: Output format - `plain` or `json` (default: plain)
- `--limit, -l`: Maximum number of results (default: 10)
- `--score, -s`: Minimum similarity score (0.0-1.0, default: 0.0)
- `--fuzzy`: Enable fuzzy matching for typos and partial matches
- `--regex`: Use regex pattern matching
- `--case-sensitive`: Perform case-sensitive search (default: case-insensitive)

### Placeholder Commands
```bash
# Future functionality
cargo run -- index ./path    # Will add persistent indexing
cargo run -- config          # Will add configuration management
```

## Architecture

Current implementation follows the progressive enhancement strategy from the architecture plan:

- **Phase 1: Foundation** ✅ - CLI interface, basic search, file traversal
- **Phase 2: Enhanced Search** ✅ - Fuzzy matching, regex support, case sensitivity, scoring
- **Phase 3: Semantic Search** 🔄 - Coming next: ML-based semantic understanding, TF-IDF scoring
- **Phase 4: Production Ready** 📋 - Future: optimization, cross-platform support

## Dependencies

- `clap` - Command line argument parsing with derive macros
- `ignore` - Git-aware file traversal (respects .gitignore)
- `serde` + `serde_json` - JSON serialization for output
- `anyhow` - Better error handling and propagation
- `fuzzy-matcher` - Fuzzy string matching for typo tolerance
- `regex` - Regular expression pattern matching
- `edit-distance` - String similarity calculations

## Testing

```bash
# Run all tests
cargo test

# Build and test search functionality
cargo build
cargo run -- search "test query" --path ./
```

## Performance

Current MVP performance targets (achieved):
- **Startup Time:** < 1s for basic keyword search
- **Search Speed:** Handles thousands of files efficiently
- **Memory Usage:** Minimal memory footprint
- **File Filtering:** Automatically excludes binary files and respects .gitignore

## Next Steps (Phase 3)

Based on the architecture plan, the next features to implement are:

1. **Semantic Search & ML Integration**:
   - TF-IDF scoring for better ranking
   - Local embedding models (ONNX Runtime)
   - Multi-word query understanding
   - Context-aware search results

2. **Persistent Index**:
   - SQLite storage for file metadata
   - Incremental indexing
   - Search result caching
   - Background file watching

3. **Configuration System**:
   - User preferences and profiles
   - File exclusion patterns
   - Performance tuning options
   - Auto-detection of system capabilities

## Project Structure

```
src/
├── main.rs    # CLI interface and command handling
└── lib.rs     # Core search functionality and types
```

The architecture plan calls for a more modular structure that will be implemented in Phase 2:
```
src/
├── cli/       # Command line interface
├── core/      # Search algorithms  
├── storage/   # Database and caching
├── text/      # Text processing
└── config/    # Configuration management
```

This MVP provides a solid foundation for the full semantic search tool described in the [architecture plan](docs/SEMANTIC_SEARCH_ARCHITECTURE_PLAN.md).

## Documentation

- [Architecture Plan](docs/SEMANTIC_SEARCH_ARCHITECTURE_PLAN.md) - Complete technical specification and implementation roadmap 