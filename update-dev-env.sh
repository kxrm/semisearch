#!/bin/bash

# Manual development environment update script
# Run this when you want to update your development tools

echo "🔄 Updating development environment..."

# Check if we're in a container
if [ -f /.dockerenv ]; then
    echo "Running in container - using update script"
    bash .devcontainer/update.sh
else
    echo "Running outside container - updating local tools"
    
    # Update Rust
    rustup update
    
    # Update cargo tools if they exist
    if command -v cargo-install-update >/dev/null 2>&1; then
        cargo install-update -a
    else
        echo "⚠️  cargo-install-update not found. Install it with: cargo install cargo-update"
    fi
    
    # Clean up if cargo-cache exists
    if command -v cargo-cache >/dev/null 2>&1; then
        cargo cache --autoclean
    fi
fi

echo "✅ Environment update complete!" 