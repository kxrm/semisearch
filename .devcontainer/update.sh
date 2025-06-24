#!/bin/bash

# Semantic Search CLI Development Environment Update Script
# This script updates tools and dependencies when the devcontainer is updated

set -e

echo "🔄 Updating Semantic Search CLI development environment..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Update system packages
log_info "Updating system packages..."
sudo apt-get update -qq
sudo apt-get upgrade -y

# Update Rust toolchain
log_info "Updating Rust toolchain..."
rustup update

# Update Rust tools
log_info "Updating Rust development tools..."
if command -v cargo-install-update >/dev/null 2>&1; then
    cargo install-update -a 2>/dev/null || log_warning "Some tools may need manual update"
else
    log_warning "cargo-install-update not found, skipping tool updates"
fi

# Update Python packages
log_info "Updating Python ML libraries..."
pip3 install --user --upgrade \
    sentence-transformers \
    torch \
    numpy \
    matplotlib \
    jupyter \
    pandas \
    scikit-learn \
    typer \
    rich \
    tqdm

# Clean up
log_info "Cleaning up..."
sudo apt-get autoremove -y
sudo apt-get autoclean

# Only run cargo cache if it's installed
if command -v cargo-cache >/dev/null 2>&1; then
    cargo cache --autoclean
else
    log_warning "cargo-cache not installed, skipping cache cleanup"
fi

log_success "Development environment updated successfully!"
echo ""
echo "🎯 Ready for development!"
echo "Run 'search_help' for available commands." 