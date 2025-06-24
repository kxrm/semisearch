#!/bin/bash

# Install additional development tools after minimal setup
# Run this after the container is working to add more features

echo "🔧 Installing additional development tools..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Ask user what to install
echo "What would you like to install?"
echo "1. Basic Rust tools (cargo-edit, cargo-audit, etc.)"
echo "2. Modern CLI tools (ripgrep, bat, exa, fd)"
echo "3. Performance tools (hyperfine, tokei)"
echo "4. Cross-compilation tools"
echo "5. All of the above"
echo "6. Python ML libraries"

read -p "Enter your choice (1-6): " choice

case $choice in
    1|5)
        log_info "Installing Rust development tools..."
        rust_tools=("cargo-edit" "cargo-audit" "cargo-outdated" "cargo-tree" "cargo-update" "cargo-cache")
        for tool in "${rust_tools[@]}"; do
            log_info "Installing $tool..."
            cargo install --locked "$tool" && log_success "$tool installed" || log_warning "Failed to install $tool"
        done
        ;;& # Continue to next case
    2|5)
        log_info "Installing modern CLI tools..."
        cli_tools=("ripgrep" "bat" "exa" "fd-find")
        for tool in "${cli_tools[@]}"; do
            log_info "Installing $tool..."
            cargo install --locked "$tool" && log_success "$tool installed" || log_warning "Failed to install $tool"
        done
        ;;& # Continue to next case
    3|5)
        log_info "Installing performance tools..."
        perf_tools=("hyperfine" "tokei")
        for tool in "${perf_tools[@]}"; do
            log_info "Installing $tool..."
            cargo install --locked "$tool" && log_success "$tool installed" || log_warning "Failed to install $tool"
        done
        ;;& # Continue to next case
    4|5)
        log_info "Installing cross-compilation tools..."
        cargo install --locked cross && log_success "cross installed" || log_warning "Failed to install cross"
        
        log_info "Adding cross-compilation targets..."
        rustup target add x86_64-unknown-linux-musl
        rustup target add aarch64-unknown-linux-gnu
        rustup target add x86_64-pc-windows-gnu
        ;;& # Continue to next case
    6|5)
        log_info "Installing Python ML libraries..."
        pip3 install --user \
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
        ;;
esac

# Update aliases based on what's now available
log_info "Updating shell aliases..."
cat >> ~/.zshrc << 'EOF'

# Updated aliases based on installed tools
if command -v exa >/dev/null 2>&1; then
    alias ll='exa -la --group-directories-first'
    alias la='exa -la'
    alias tree='exa --tree'
fi

if command -v bat >/dev/null 2>&1; then
    alias cat='bat --style=plain'
fi

if command -v rg >/dev/null 2>&1; then
    alias grep='rg'
fi

if command -v fd >/dev/null 2>&1; then
    alias find='fd'
fi

if command -v hyperfine >/dev/null 2>&1; then
    alias perf='hyperfine --warmup 3'
fi
EOF

log_success "Additional tools installation complete!"
echo ""
echo "🎉 Your development environment is now enhanced!"
echo "Run 'source ~/.zshrc' to activate new aliases"
echo "Or just restart your terminal." 