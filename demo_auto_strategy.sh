#!/bin/bash

echo "🚀 SemiSearch AutoStrategy Demonstration"
echo "========================================"
echo

# Build the project
echo "📦 Building project..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo

# Test 1: Exact phrase detection
echo "🔍 Test 1: Exact Phrase Detection"
echo "Query: \"TODO: implement feature\""
echo "Expected: Exact phrase search (keyword-based)"
echo "Running..."
./target/release/semisearch search "\"TODO: implement feature\"" --path ./src
echo

# Test 2: Code pattern detection
echo "🔍 Test 2: Code Pattern Detection"
echo "Query: TODO"
echo "Expected: Code pattern search (regex-based)"
echo "Running..."
./target/release/semisearch search "TODO" --path ./src
echo

# Test 3: Conceptual query detection
echo "🔍 Test 3: Conceptual Query Detection"
echo "Query: error handling patterns"
echo "Expected: Conceptual search (fuzzy-based, fallback from semantic)"
echo "Running..."
./target/release/semisearch search "error handling patterns" --path ./src
echo

# Test 4: File extension detection
echo "🔍 Test 4: File Extension Detection"
echo "Query: .rs"
echo "Expected: File extension search (keyword-based)"
echo "Running..."
./target/release/semisearch search ".rs" --path ./src
echo

# Test 5: Regex-like pattern detection
echo "🔍 Test 5: Regex-like Pattern Detection"
echo "Query: TODO.*:"
echo "Expected: Regex search"
echo "Running..."
./target/release/semisearch search "TODO.*:" --path ./src
echo

# Test 6: Typo tolerance (fallback to fuzzy)
echo "🔍 Test 6: Typo Tolerance"
echo "Query: databse"
echo "Expected: Fuzzy search for typo tolerance"
echo "Running..."
./target/release/semisearch search "databse" --path ./src
echo

# Test 7: Project context detection
echo "🔍 Test 7: Project Context Detection"
echo "Testing different project types..."
echo

echo "Current directory (should be Mixed - has Cargo.toml and docs):"
./target/release/semisearch search "TODO" --path .
echo

echo "Docs directory (should be Documentation):"
./target/release/semisearch search "TODO" --path ./docs
echo

echo "Src directory (should be Unknown - no Cargo.toml):"
./target/release/semisearch search "TODO" --path ./src
echo

echo "🎉 AutoStrategy Demonstration Complete!"
echo
echo "Summary of AutoStrategy Features:"
echo "✅ Query pattern detection (exact phrases, code patterns, conceptual, file extensions, regex)"
echo "✅ Project context detection (code, documentation, mixed, unknown)"
echo "✅ Automatic strategy selection based on query type and project context"
echo "✅ Fallback mechanisms for semantic search when unavailable"
echo "✅ Typo tolerance through fuzzy search"
echo
echo "The AutoStrategy automatically chooses the best search method based on:"
echo "1. Query content analysis (QueryAnalyzer)"
echo "2. Project type detection (ProjectContext)"
echo "3. Available search capabilities"
echo
echo "This provides a seamless user experience where users don't need to choose"
echo "search modes - the tool intelligently selects the optimal strategy!" 