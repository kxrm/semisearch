# Semantic Search CLI - Development Container

This devcontainer provides a complete development environment for the semantic search CLI project, designed to support developers of all experience levels.

## 🚀 Quick Start

1. **Open in VS Code with Dev Containers extension**
   - Install the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
   - Open this folder in VS Code
   - Click "Reopen in Container" when prompted

2. **Run the setup script**
   ```bash
   # After the container opens, run:
   bash .devcontainer/setup.sh
   ```

3. **Start developing**
   ```bash
   # Show all available commands
   just
   
   # Build the project
   just build
   
   # Get help with development commands
   search_help
   ```

## 📦 What's Included

### Core Development Tools
- **Rust**: Latest stable with clippy, rustfmt, and rust-src
- **Database**: SQLite3 with development tools
- **ML Support**: ONNX Runtime dependencies and Python 3.11 with ML libraries
- **Git**: Latest version with GitHub CLI
- **Node.js**: LTS version for potential web interfaces
- **Docker**: Docker-in-Docker for containerization

### VS Code Extensions
- **Rust Development**: rust-analyzer, crates, LLDB debugger, TOML support
- **Python Support**: Python extension with debugpy and Jupyter
- **General Development**: GitLens, spell checker, code runner, YAML/JSON support
- **AI Assistance**: GitHub Copilot and Copilot Chat (if available)

### System Tools (Available via install-more-tools.sh)
- `ripgrep` (rg) - Fast text search
- `fd` - Fast file finder
- `bat` - Enhanced cat with syntax highlighting
- `exa` - Modern ls replacement
- `hyperfine` - Command-line benchmarking
- `tokei` - Code statistics

### Rust Development Tools (Available via install-more-tools.sh)
- `cargo-edit` - Easy dependency management
- `cargo-audit` - Security vulnerability scanning
- `cargo-outdated` - Dependency update checking
- `cross` - Cross-compilation made easy

### Built-in Development Scripts
- `.devcontainer/setup.sh` - Main environment setup
- `.devcontainer/scripts/new-developer-setup.sh` - Interactive onboarding
- `.devcontainer/install-more-tools.sh` - Install additional optional tools
- `.devcontainer/update.sh` - Update all development tools
- `.devcontainer/test-setup.sh` - Validate environment setup

## 🛠️ Development Workflow

### For Beginners

1. **Get oriented**
   ```bash
   # Run the setup script first
   bash .devcontainer/setup.sh
   
   # Get help and see available commands
   search_help
   
   # Create test data to work with
   create_test_data
   
   # Use the interactive setup guide
   bash .devcontainer/scripts/new-developer-setup.sh
   ```

2. **Follow the progressive development approach**
   - Start with MVP (basic keyword search)
   - Add features incrementally
   - Test each checkpoint thoroughly

### For Experienced Developers

1. **Quick setup**
   ```bash
   # Run setup and install additional tools
   bash .devcontainer/setup.sh
   bash .devcontainer/install-more-tools.sh
   
   # Start building immediately
   just build
   just run search "hello world"
   ```

2. **Use advanced workflows**
   ```bash
   # Install cross-compilation tools
   bash .devcontainer/install-more-tools.sh
   
   # Use Just for task automation
   just watch    # Continuous development
   just clippy   # Code quality checks
   just stats    # Project statistics
   ```

## 🔍 Architecture Requirements Coverage

### ✅ Fully Supported
- **Core Dependencies**: All key dependencies from the architecture plan are available
  - `clap` v4 for CLI parsing
  - `tokio` for async runtime
  - `rusqlite` for SQLite database
  - `serde` for serialization
  - `walkdir` for file traversal
  - `anyhow` for error handling

- **ML Infrastructure**: 
  - ONNX Runtime support via Python
  - Python 3.11 with ML libraries (sentence-transformers, torch, etc.)
  - Model storage volumes configured

- **Development Tools**:
  - Comprehensive VS Code setup with rust-analyzer
  - Build caching with persistent volumes
  - Cross-compilation support (installable)

### 🔧 Optional Enhancements
- **Advanced Rust Tools**: Install via `bash .devcontainer/install-more-tools.sh`
- **Performance Tools**: hyperfine, tokei available for installation
- **Modern CLI Tools**: ripgrep, bat, exa, fd available for installation

## 📁 Project Structure

The setup script creates this structure:
```
/workspaces/semisearch/
├── .devcontainer/          # Development environment
├── src/                    # Source code (created by setup)
├── tests/                  # Test files
├── benches/                # Benchmarks
├── docs/                   # Documentation
├── .models/                # ML models (volume mounted)
├── .cache/                 # Cache data (volume mounted)
├── Cargo.toml              # Project manifest (created by setup)
├── Justfile                # Task automation (created by setup)
└── test-data/              # Sample data (created by create_test_data)
```

## 🚀 Progressive Development Paths

### Path 1: MVP (Recommended for beginners)
```bash
bash .devcontainer/scripts/new-developer-setup.sh
# Choose option 1: MVP
cargo run -- search "TODO" .
```

### Path 2: Enhanced Search
```bash
bash .devcontainer/scripts/new-developer-setup.sh
# Choose option 2: Enhanced
# Adds fuzzy matching and caching
```

### Path 3: Full Semantic Search
```bash
bash .devcontainer/scripts/new-developer-setup.sh
# Choose option 3: Full
# Complete ML-powered semantic search
```

## 🔧 Environment Variables

The container sets these environment variables:
- `CARGO_TARGET_DIR=/tmp/target` - Shared build cache
- `SEMISEARCH_MODELS_DIR=/workspaces/semisearch/.models` - ML models storage
- `SEMISEARCH_CACHE_DIR=/workspaces/semisearch/.cache` - Application cache
- `RUST_BACKTRACE=1` - Enhanced error reporting
- `RUST_LOG=info` - Default logging level

## 📚 Available Commands

After running the setup script, these aliases are available:
```bash
# Cargo shortcuts
cb          # cargo build
ct          # cargo test
cr          # cargo run
cc          # cargo check
fmt         # cargo fmt
clippy      # cargo clippy
cw          # cargo watch (if installed)

# Just commands
j           # just
jl          # just --list

# Helper functions
search_help      # Show development guidance
create_test_data # Create sample test files
```

## 🐛 Troubleshooting

### First Time Setup
1. **Run the setup script**: `bash .devcontainer/setup.sh`
2. **Check environment**: `bash .devcontainer/test-setup.sh`
3. **Install optional tools**: `bash .devcontainer/install-more-tools.sh`

### Common Issues
- **Missing tools**: Run `bash .devcontainer/install-more-tools.sh`
- **Build errors**: Ensure Cargo.toml exists with `ls -la Cargo.toml`
- **Path issues**: All scripts use `/workspaces/semisearch/` consistently

### Getting Help
- **Architecture guidance**: `cat SEMANTIC_SEARCH_ARCHITECTURE_PLAN.md`
- **Development help**: `search_help`
- **Available tasks**: `just --list`
- **Script documentation**: `cat .devcontainer/SCRIPTS.md`

---

**Ready to build an amazing semantic search tool!** 🦀✨

Start with the MVP approach and progressively enhance your way to a full-featured semantic search CLI that works on any system. 