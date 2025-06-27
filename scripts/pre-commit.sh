#!/bin/bash

# Manual pre-commit checks for semisearch
# Run this script before committing to ensure code quality

set -e

echo "🔍 Running pre-commit checks..."

# Check Rust formatting
echo "📝 Checking Rust code formatting..."
cargo fmt --check

# Run Clippy linter
echo "🔧 Running Clippy linter..."
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
echo "🧪 Running tests..."
cargo test

# Check for large files
echo "📦 Checking for large files..."
find . -type f -size +1M -not -path "./target/*" -not -path "./.git/*" | head -10

# Check for merge conflicts
echo "🔍 Checking for merge conflicts..."
if grep -r "<<<<<<< HEAD" . --exclude-dir=target --exclude-dir=.git; then
    echo "❌ Found merge conflicts!"
    exit 1
fi

if grep -r "=======" . --exclude-dir=target --exclude-dir=.git; then
    echo "❌ Found merge conflicts!"
    exit 1
fi

if grep -r ">>>>>>>" . --exclude-dir=target --exclude-dir=.git; then
    echo "❌ Found merge conflicts!"
    exit 1
fi

echo "✅ All pre-commit checks passed!"
echo ""
echo "💡 You can now commit your changes safely."
echo "   git add ."
echo "   git commit -m \"your commit message\""
