#!/bin/bash

# Simple setup for semantic search CLI development
# Works within the VS Code workspace directory

set -e

echo "🚀 Setting up development environment..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Install only essential system packages
log_info "Installing essential packages..."
sudo apt-get update -qq >/dev/null 2>&1
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    sqlite3 \
    git \
    curl >/dev/null 2>&1

# Verify Rust is working
log_info "Verifying Rust installation..."
rustc --version
cargo --version

# Install essential Rust components
log_info "Installing Rust components..."
rustup component add clippy rustfmt

# Create project structure in workspace (if not exists)
log_info "Setting up workspace structure..."
mkdir -p /workspaces/semisearch/{src,tests,benches,docs}

# Create .vscode settings if not exists
if [ ! -f /workspaces/semisearch/.vscode/settings.json ]; then
    mkdir -p /workspaces/semisearch/.vscode
    cat > /workspaces/semisearch/.vscode/settings.json << 'EOF'
{
    "rust-analyzer.linkedProjects": ["./Cargo.toml"],
    "rust-analyzer.checkOnSave.command": "clippy"
}
EOF
fi

# Create helpful aliases and functions
log_info "Setting up development shortcuts..."
cat >> ~/.zshrc << 'EOF'

# Semantic Search Development Environment
export CARGO_TARGET_DIR=/tmp/target
export SEMISEARCH_MODELS_DIR=/workspaces/semisearch/.models
export SEMISEARCH_CACHE_DIR=/workspaces/semisearch/.cache

# Always start in workspace
cd /workspaces/semisearch

# Handy aliases
alias ll='ls -la'
alias cb='cargo build'
alias ct='cargo test'
alias cr='cargo run'
alias cc='cargo check'
alias fmt='cargo fmt'
alias clippy='cargo clippy'
alias cw='cargo watch'

# Just commands (if Justfile exists)
alias j='just'
alias jl='just --list'

# Development helper
search_help() {
    echo "🔍 Semantic Search CLI Development"
    echo ""
    echo "📁 Workspace: /workspaces/semisearch"
    echo ""
    echo "🚀 Quick Commands:"
    echo "  cb       # cargo build"
    echo "  ct       # cargo test"
    echo "  cr       # cargo run"
    echo "  cc       # cargo check"
    echo "  fmt      # cargo fmt"
    echo "  clippy   # cargo clippy"
    echo "  cw       # cargo watch (if installed)"
    echo ""
    echo "📝 Getting Started:"
    echo "  1. Create Cargo.toml if not exists"
    echo "  2. cargo run"
    echo ""
    echo "📖 Architecture: see SEMANTIC_SEARCH_ARCHITECTURE_PLAN.md"
}

# Create test data helper
create_test_data() {
    echo "📝 Creating test data..."
    mkdir -p /workspaces/semisearch/test-data/{small,medium}

    # Create some test files
    echo "Ghostbusters is a classic movie" > /workspaces/semisearch/test-data/small/movies.txt
    echo "Jim Carrey stars in Ace Ventura" >> /workspaces/semisearch/test-data/small/movies.txt
    echo "The Silence of the Lambs won many awards" >> /workspaces/semisearch/test-data/small/movies.txt

    echo "✅ Test data created in /workspaces/semisearch/test-data/"
}

# Welcome message
echo "🦀 Semantic Search CLI Development Environment"
echo "📁 Workspace: /workspaces/semisearch"
echo "💡 Run 'search_help' for guidance"
echo ""
EOF

# Create initial Cargo.toml if it doesn't exist
if [ ! -f /workspaces/semisearch/Cargo.toml ]; then
    log_info "Creating initial Cargo.toml..."
    cat > /workspaces/semisearch/Cargo.toml << 'EOF'
[package]
name = "semisearch"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
walkdir = "2.3"
EOF

    # Create basic main.rs if it doesn't exist
    if [ ! -f /workspaces/semisearch/src/main.rs ]; then
        log_info "Creating initial src/main.rs..."
        mkdir -p /workspaces/semisearch/src
        cat > /workspaces/semisearch/src/main.rs << 'RUST'
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "semisearch")]
#[command(about = "Semantic search across local files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for matches
    Search {
        /// Search query
        query: String,

        /// Target directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        path: String,
    },

    /// Index files in directory
    Index {
        /// Directory to index
        path: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search { query, path } => {
            println!("🔍 Searching for '{}' in '{}'", query, path);
            println!("(This is where search logic will go)");
        }
        Commands::Index { path } => {
            println!("📚 Indexing files in '{}'", path);
            println!("(This is where indexing logic will go)");
        }
    }

    Ok(())
}
RUST
    fi
fi

# Create a Justfile for common tasks
if [ ! -f /workspaces/semisearch/Justfile ]; then
    log_info "Creating Justfile for task automation..."
    cat > /workspaces/semisearch/Justfile << 'EOF'
# List available commands
default:
    @just --list

# Build the project
build:
    cargo build

# Run tests
test:
    cargo test

# Run the CLI
run *ARGS:
    cargo run -- {{ARGS}}

# Format code
fmt:
    cargo fmt

# Run clippy
clippy:
    cargo clippy -- -D warnings

# Watch for changes and rebuild
watch:
    cargo watch -x check -x test -x run

# Clean build artifacts
clean:
    cargo clean
    rm -rf /tmp/target

# Create test data
test-data:
    @echo "Creating test data..."
    @mkdir -p test-data/{small,medium,large}
    @echo "Test data created!"

# Show project statistics
stats:
    @echo "📊 Project Statistics:"
    @tokei
EOF
fi

log_success "Setup complete!"
echo ""
echo "🎉 Development environment ready!"
echo ""
echo "📍 Workspace: /workspaces/semisearch"
echo ""
echo "🚀 Quick Start:"
echo "  cargo build"
echo "  cargo run -- --help"
echo ""
echo "💡 Or use Just commands:"
echo "  just        # List available commands"
echo "  just build  # Build the project"
echo "  just run search 'hello world'"
echo ""
echo "🎓 New to the project?"
echo "  Run: bash .devcontainer/scripts/new-developer-setup.sh"
echo ""
echo "Happy coding! 🦀✨"
