#!/bin/bash

# Interactive setup guide for new developers
# Referenced in SEMANTIC_SEARCH_ARCHITECTURE_PLAN.md

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🚀 Welcome to Semantic Search CLI Development!${NC}"
echo ""
echo "This guide will help you get started with the project."
echo ""

# Check if we're in the right directory
if [ ! -f "/workspaces/search/SEMANTIC_SEARCH_ARCHITECTURE_PLAN.md" ]; then
    echo -e "${YELLOW}⚠️  Warning: Not in the expected workspace directory${NC}"
    echo "Expected: /workspaces/search"
    echo "Current: $(pwd)"
    echo ""
fi

# Show development approach options
echo -e "${BLUE}📚 Choose your development approach:${NC}"
echo ""
echo "1) MVP - Basic keyword search (recommended for beginners)"
echo "2) Enhanced - Add fuzzy matching and caching"
echo "3) Full - Complete semantic search with ML"
echo "4) Custom - I'll configure it myself"
echo ""
read -p "Select approach (1-4): " approach

case $approach in
    1)
        echo -e "\n${GREEN}✅ Setting up MVP development${NC}"
        echo "You'll start with basic keyword search - perfect for learning!"
        
        # Create MVP-focused Cargo.toml
        cat > /workspaces/search/Cargo.toml << 'EOF'
[package]
name = "semisearch"
version = "0.1.0"
edition = "2021"

# MVP dependencies only
[dependencies]
clap = { version = "4.0", features = ["derive"] }
walkdir = "2.3"
anyhow = "1.0"
EOF
        
        echo -e "\n${BLUE}📝 Your first tasks:${NC}"
        echo "1. Implement basic file scanning in src/main.rs"
        echo "2. Add simple keyword matching"
        echo "3. Format output nicely"
        echo ""
        echo "Run: cargo run -- search 'TODO' ."
        ;;
        
    2)
        echo -e "\n${GREEN}✅ Setting up Enhanced Search${NC}"
        echo "You'll have fuzzy matching and basic caching!"
        
        # Add more dependencies
        cat > /workspaces/search/Cargo.toml << 'EOF'
[package]
name = "semisearch"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.0", features = ["derive"] }
walkdir = "2.3"
anyhow = "1.0"
fuzzy-matcher = "0.3"
rusqlite = { version = "0.29", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
EOF
        ;;
        
    3)
        echo -e "\n${GREEN}✅ Setting up Full Semantic Search${NC}"
        echo "Complete setup with ML capabilities!"
        
        # Full dependencies from architecture plan
        cp /workspaces/search/.devcontainer/templates/Cargo-full.toml /workspaces/search/Cargo.toml 2>/dev/null || \
        cat > /workspaces/search/Cargo.toml << 'EOF'
[package]
name = "semisearch"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
rusqlite = { version = "0.29", features = ["bundled"] }
ort = { version = "1.15", features = ["copy-dylibs"] }
tokenizers = "0.13"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
walkdir = "2.3"
anyhow = "1.0"
thiserror = "1.0"
indicatif = "0.17"
EOF
        ;;
        
    4)
        echo -e "\n${BLUE}ℹ️  Manual setup selected${NC}"
        echo "You can customize Cargo.toml and the project structure yourself."
        ;;
esac

echo ""
echo -e "${BLUE}📚 Learning resources:${NC}"
echo "• Architecture: cat SEMANTIC_SEARCH_ARCHITECTURE_PLAN.md"
echo "• Dev Guide: cat .devcontainer/README.md"
echo "• Quick help: search_help"
echo ""
echo -e "${GREEN}🎯 Next steps:${NC}"
echo "1. Review the architecture plan"
echo "2. Start with the checkpoint matching your chosen approach"
echo "3. Run 'just' to see available commands"
echo ""
echo "Happy coding! 🦀✨" 