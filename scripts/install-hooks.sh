#!/bin/bash

# Install pre-commit hooks for semisearch
# This script sets up pre-commit hooks to ensure code quality

set -e

echo "🔧 Installing pre-commit hooks for semisearch..."

# Check if pre-commit is installed
if ! command -v pre-commit &> /dev/null; then
    echo "❌ pre-commit is not installed. Installing..."

    # Try different installation methods
    if command -v pip &> /dev/null; then
        pip install pre-commit
    elif command -v pip3 &> /dev/null; then
        pip3 install pre-commit
    elif command -v brew &> /dev/null; then
        brew install pre-commit
    else
        echo "❌ Could not install pre-commit. Please install it manually:"
        echo "   pip install pre-commit"
        echo "   or: brew install pre-commit"
        exit 1
    fi
fi

# Install the pre-commit hooks
echo "📦 Installing pre-commit hooks..."
pre-commit install

# Install additional hooks for different stages
echo "🔗 Installing additional hooks..."
pre-commit install --hook-type commit-msg
pre-commit install --hook-type pre-push

echo "✅ Pre-commit hooks installed successfully!"
echo ""
echo "🎯 Available hooks:"
echo "   • rust-fmt: Check Rust code formatting"
echo "   • rust-clippy: Run Rust linter"
echo "   • rust-test: Run Rust tests (manual only)"
echo "   • trailing-whitespace: Remove trailing whitespace"
echo "   • end-of-file-fixer: Ensure files end with newline"
echo "   • check-yaml: Validate YAML files"
echo "   • check-added-large-files: Prevent large files"
echo "   • check-merge-conflict: Detect merge conflicts"
echo ""
echo "💡 Usage:"
echo "   • Hooks run automatically on commit"
echo "   • Run manually: pre-commit run --all-files"
echo "   • Run specific hook: pre-commit run rust-clippy"
echo "   • Skip hooks: git commit --no-verify"
