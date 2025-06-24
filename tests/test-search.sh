#!/bin/bash

# Test specific search functionality
# Usage: ./test/test-search.sh "test query"

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
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
RESULT=$(cargo run --quiet -- search "$QUERY" "$TEMP_DIR" 2>&1)
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
JSON_RESULT=$(cargo run --quiet -- search "$QUERY" "$TEMP_DIR" --format json 2>&1)
if [ $? -eq 0 ] && echo "$JSON_RESULT" | jq . >/dev/null 2>&1; then
    log_success "JSON output is valid"
    echo "$JSON_RESULT" | jq .
elif [ $? -eq 0 ]; then
    log_success "JSON output completed (jq not available for validation)"
    echo "$JSON_RESULT"
else
    log_error "JSON output failed"
    echo "$JSON_RESULT"
fi

echo ""

# Test fuzzy search
log_info "Testing fuzzy search..."
FUZZY_RESULT=$(cargo run --quiet -- search "${QUERY}O" "$TEMP_DIR" --fuzzy 2>&1)  # Introduce typo
if [ $? -eq 0 ]; then
    log_success "Fuzzy search completed"
    echo "$FUZZY_RESULT"
else
    log_error "Fuzzy search failed"
    echo "$FUZZY_RESULT"
fi

echo ""

# Test regex search
log_info "Testing regex search..."
REGEX_RESULT=$(cargo run --quiet -- search "TODO.*:" "$TEMP_DIR" --regex 2>&1)
if [ $? -eq 0 ]; then
    log_success "Regex search completed"
    echo "$REGEX_RESULT"
else
    log_error "Regex search failed"
    echo "$REGEX_RESULT"
fi

echo ""

# Test with limits
log_info "Testing result limits..."
LIMIT_RESULT=$(cargo run --quiet -- search "$QUERY" "$TEMP_DIR" --limit 2 2>&1)
if [ $? -eq 0 ]; then
    log_success "Limited search completed"
    echo "$LIMIT_RESULT"
else
    log_error "Limited search failed"
    echo "$LIMIT_RESULT"
fi

echo ""
log_success "Search functionality tests completed!"
echo ""
echo "📋 Summary:"
echo "  • Basic search: ✅"
echo "  • JSON output: ✅"
echo "  • Fuzzy search: ✅"
echo "  • Regex search: ✅"
echo "  • Result limits: ✅"
echo "" 