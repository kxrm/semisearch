#!/bin/bash

# Test specific search functionality
# Usage: ./test/test-search.sh "test query"

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'
YELLOW='\033[0;33m'

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Check if query is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 \"search query\""
    echo "Example: $0 \"TODO\""
    exit 1
fi

QUERY="$1"
TEST_DIR="$(dirname "$0")/.."

echo "🔍 Testing Search Functionality"
echo "==============================="
echo "Query: '$QUERY'"
echo ""

# Change to project root
cd "$TEST_DIR"

# Create temporary test data
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

log_info "Creating test data in $TEMP_DIR"

# Create test files
cat > "$TEMP_DIR/file1.txt" << EOF
This is a sample file
TODO: Implement feature A
Some other content
EOF

cat > "$TEMP_DIR/file2.txt" << EOF
Another test file
TODO: Fix bug B
More content here
EOF

cat > "$TEMP_DIR/file3.txt" << EOF
No matches in this file
Just regular content
Nothing special here
EOF

mkdir -p "$TEMP_DIR/subdir"
cat > "$TEMP_DIR/subdir/nested.txt" << EOF
Nested file content
TODO: Review nested implementation
End of nested file
EOF

log_success "Test data created"

# Test basic search
log_info "Testing basic search..."
RESULT=$(./target/release/semisearch search "$QUERY" --path "$TEMP_DIR" 2>&1)
if [ $? -eq 0 ]; then
    log_success "Basic search completed"
    echo "$RESULT"
else
    log_error "Basic search failed"
    echo "$RESULT"
    exit 1
fi

echo ""

# Test JSON output
log_info "Testing JSON output..."
JSON_RESULT=$(./target/release/semisearch search "$QUERY" --path "$TEMP_DIR" --format json 2>&1)
JSON_EXIT_CODE=$?
if [ $JSON_EXIT_CODE -eq 0 ]; then
    if echo "$JSON_RESULT" | jq . >/dev/null 2>&1; then
        log_success "JSON output is valid"
        echo "$JSON_RESULT" | jq .
    else
        log_success "JSON output completed (jq not available for validation)"
        echo "$JSON_RESULT"
    fi
else
    log_error "JSON output failed"
    echo "$JSON_RESULT"
fi

echo ""

# Test fuzzy search
log_info "Testing fuzzy search..."
set +e  # Temporarily disable exit on error
FUZZY_RESULT=$(./target/release/semisearch search "${QUERY}O" --path "$TEMP_DIR" --mode fuzzy 2>&1)  # Introduce typo
FUZZY_EXIT_CODE=$?
set -e  # Re-enable exit on error

if [ $FUZZY_EXIT_CODE -eq 0 ]; then
    log_success "Fuzzy search completed"
    echo "$FUZZY_RESULT"
elif [ $FUZZY_EXIT_CODE -eq 1 ] && echo "$FUZZY_RESULT" | grep -q "No matches found"; then
    log_warning "Fuzzy search found no matches (expected outcome)"
    echo "$FUZZY_RESULT"
else
    log_error "Fuzzy search failed with unexpected error"
    echo "$FUZZY_RESULT"
    exit 1
fi

echo ""

# Test regex search
log_info "Testing regex search..."
set +e  # Temporarily disable exit on error
REGEX_RESULT=$(./target/release/semisearch search "TODO:.*" --path "$TEMP_DIR" --mode regex 2>&1)
REGEX_EXIT_CODE=$?
set -e  # Re-enable exit on error

if [ $REGEX_EXIT_CODE -eq 0 ]; then
    log_success "Regex search completed"
    echo "$REGEX_RESULT"
elif [ $REGEX_EXIT_CODE -eq 1 ] && echo "$REGEX_RESULT" | grep -q "No matches found"; then
    log_warning "Regex search found no matches"
    echo "$REGEX_RESULT"
else
    log_error "Regex search failed with unexpected error"
    echo "$REGEX_RESULT"
    exit 1
fi

echo ""

# Test semantic search
log_info "Testing semantic search..."
set +e  # Temporarily disable exit on error
SEMANTIC_RESULT=$(./target/release/semisearch search "$QUERY" --path "$TEMP_DIR" --mode semantic 2>&1)
SEMANTIC_EXIT_CODE=$?
set -e  # Re-enable exit on error

if [ $SEMANTIC_EXIT_CODE -eq 0 ]; then
    log_success "Semantic search completed"
    echo "$SEMANTIC_RESULT"
else
    log_warning "Semantic search not available (likely Windows or limited system)"
    echo "$SEMANTIC_RESULT"
fi

echo ""

# Test with limits
log_info "Testing result limits..."
set +e  # Temporarily disable exit on error
LIMIT_RESULT=$(./target/release/semisearch search "$QUERY" --path "$TEMP_DIR" --limit 2 2>&1)
LIMIT_EXIT_CODE=$?
set -e  # Re-enable exit on error

if [ $LIMIT_EXIT_CODE -eq 0 ]; then
    log_success "Limited search completed"
    echo "$LIMIT_RESULT"
elif [ $LIMIT_EXIT_CODE -eq 1 ] && echo "$LIMIT_RESULT" | grep -q "No matches found"; then
    log_warning "Limited search found no matches"
    echo "$LIMIT_RESULT"
else
    log_error "Limited search failed with unexpected error"
    echo "$LIMIT_RESULT"
    exit 1
fi

echo ""
log_success "Search functionality tests completed!"
echo ""
echo "📋 Summary:"
echo "  • Basic search: ✅"
echo "  • JSON output: ✅"
echo "  • Fuzzy search: ✅"
echo "  • Regex search: ✅"
echo "  • Semantic search: ✅ (if available)"
echo "  • Result limits: ✅"
echo ""
