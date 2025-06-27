#!/bin/bash

# Performance testing script
# Tests search speed with different dataset sizes

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
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

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if bc is available for calculations
if ! command -v bc &> /dev/null; then
    log_warning "bc not available, timing calculations may be limited"
    CALC_AVAILABLE=false
else
    CALC_AVAILABLE=true
fi

echo "⚡ Performance Testing Suite"
echo "============================"

# Change to project root
cd "$(dirname "$0")/.."

# Build optimized version
log_info "Building optimized version..."
cargo build --release --quiet

# Determine binary path
if [ -f "./target/release/semisearch" ]; then
    BINARY_PATH="./target/release/semisearch"
elif [ -f "/tmp/target/release/semisearch" ]; then
    BINARY_PATH="/tmp/target/release/semisearch"
else
    log_error "Could not find semisearch binary"
    exit 1
fi

log_info "Using binary: $BINARY_PATH"

# Create test data directory
TEST_DATA_DIR="test-perf-data"
rm -rf "$TEST_DATA_DIR"
mkdir -p "$TEST_DATA_DIR"

trap "rm -rf $TEST_DATA_DIR" EXIT

# Performance targets from architecture plan
echo ""
echo "📊 Performance Targets:"
echo "  • Small projects (< 1000 files): < 2s"
echo "  • Medium projects (< 10000 files): < 10s"
echo "  • Cold start: < 500ms"
echo ""

# Test 1: Small dataset (100 files)
log_info "Creating small dataset (100 files)..."
for i in $(seq 1 100); do
    cat > "$TEST_DATA_DIR/file_$i.txt" << EOF
This is test file number $i
TODO: Implement feature $i
FIXME: Bug in module $i
Some random content here
Another line with content
Final line of file $i
EOF
done

log_info "Testing small dataset performance..."
start_time=$(date +%s.%N)
$BINARY_PATH search "TODO" --path "$TEST_DATA_DIR" --limit 50 > /dev/null 2>&1
end_time=$(date +%s.%N)

if [ "$CALC_AVAILABLE" = true ]; then
    duration=$(echo "$end_time - $start_time" | bc -l)
    echo "Small dataset (100 files): ${duration}s"

    if (( $(echo "$duration < 2.0" | bc -l) )); then
        log_success "Small dataset performance: EXCELLENT (< 2s target)"
    elif (( $(echo "$duration < 5.0" | bc -l) )); then
        log_warning "Small dataset performance: ACCEPTABLE (but > 2s target)"
    else
        log_error "Small dataset performance: POOR (> 5s)"
    fi
else
    echo "Small dataset test completed (timing unavailable)"
fi

# Test 2: Medium dataset (1000 files)
log_info "Creating medium dataset (1000 files)..."
for i in $(seq 101 1100); do
    cat > "$TEST_DATA_DIR/file_$i.txt" << EOF
Medium test file $i with more content
TODO: Feature implementation for $i
FIXME: Critical bug in component $i
WARNING: Performance issue in $i
INFO: Documentation needed for $i
DEBUG: Trace information for $i
ERROR: Exception handling in $i
Multiple lines of content here
Some additional text content
More content to make files larger
Final content line for file $i
EOF
done

log_info "Testing medium dataset performance..."
start_time=$(date +%s.%N)
$BINARY_PATH search "TODO" --path "$TEST_DATA_DIR" --limit 100 > /dev/null 2>&1
end_time=$(date +%s.%N)

if [ "$CALC_AVAILABLE" = true ]; then
    duration=$(echo "$end_time - $start_time" | bc -l)
    echo "Medium dataset (1000 files): ${duration}s"

    if (( $(echo "$duration < 10.0" | bc -l) )); then
        log_success "Medium dataset performance: TARGET MET (< 10s)"
    elif (( $(echo "$duration < 30.0" | bc -l) )); then
        log_warning "Medium dataset performance: ACCEPTABLE (but > 10s target)"
    else
        log_error "Medium dataset performance: POOR (> 30s)"
    fi
else
    echo "Medium dataset test completed (timing unavailable)"
fi

# Test 3: Cold start performance
log_info "Testing cold start performance..."
# Clear any potential caches
sync
echo 3 > /proc/sys/vm/drop_caches 2>/dev/null || true

start_time=$(date +%s.%N)
$BINARY_PATH search "TODO" --path "$TEST_DATA_DIR" --limit 10 > /dev/null 2>&1
end_time=$(date +%s.%N)

if [ "$CALC_AVAILABLE" = true ]; then
    duration=$(echo "$end_time - $start_time" | bc -l)
    echo "Cold start: ${duration}s"

    if (( $(echo "$duration < 0.5" | bc -l) )); then
        log_success "Cold start performance: EXCELLENT (< 500ms target)"
    elif (( $(echo "$duration < 2.0" | bc -l) )); then
        log_warning "Cold start performance: ACCEPTABLE (but > 500ms target)"
    else
        log_error "Cold start performance: NEEDS IMPROVEMENT (> 2s)"
    fi
else
    echo "Cold start test completed (timing unavailable)"
fi

# Test 4: Different search modes performance
log_info "Testing different search modes..."

# Basic search
start_time=$(date +%s.%N)
$BINARY_PATH search "TODO" --path "$TEST_DATA_DIR" --limit 20 > /dev/null 2>&1
end_time=$(date +%s.%N)
if [ "$CALC_AVAILABLE" = true ]; then
    basic_duration=$(echo "$end_time - $start_time" | bc -l)
    echo "Basic search: ${basic_duration}s"
fi

# Fuzzy search
start_time=$(date +%s.%N)
$BINARY_PATH search "TODO" --path "$TEST_DATA_DIR" --mode fuzzy --limit 20 > /dev/null 2>&1
end_time=$(date +%s.%N)
if [ "$CALC_AVAILABLE" = true ]; then
    fuzzy_duration=$(echo "$end_time - $start_time" | bc -l)
    echo "Fuzzy search: ${fuzzy_duration}s"
fi

# Regex search
start_time=$(date +%s.%N)
$BINARY_PATH search "TODO.*:" --path "$TEST_DATA_DIR" --mode regex --limit 20 > /dev/null 2>&1
end_time=$(date +%s.%N)
if [ "$CALC_AVAILABLE" = true ]; then
    regex_duration=$(echo "$end_time - $start_time" | bc -l)
    echo "Regex search: ${regex_duration}s"
fi

# Semantic search (if available)
start_time=$(date +%s.%N)
if $BINARY_PATH search "TODO implementation" --path "$TEST_DATA_DIR" --mode semantic --limit 20 > /dev/null 2>&1; then
    end_time=$(date +%s.%N)
    if [ "$CALC_AVAILABLE" = true ]; then
        semantic_duration=$(echo "$end_time - $start_time" | bc -l)
        echo "Semantic search: ${semantic_duration}s"
    fi
else
    echo "Semantic search: Not available on this system"
fi

# Test 5: Memory usage (if available)
if command -v ps &> /dev/null; then
    log_info "Testing memory usage..."
    $BINARY_PATH search "TODO" --path "$TEST_DATA_DIR" --limit 50 &
    SEARCH_PID=$!
    sleep 0.5  # Give it time to start

    if ps -p $SEARCH_PID > /dev/null; then
        MEMORY_KB=$(ps -o rss= -p $SEARCH_PID 2>/dev/null || echo "unknown")
        if [ "$MEMORY_KB" != "unknown" ]; then
            MEMORY_MB=$((MEMORY_KB / 1024))
            echo "Peak memory usage: ${MEMORY_MB}MB"

            if [ $MEMORY_MB -lt 100 ]; then
                log_success "Memory usage: EXCELLENT (< 100MB)"
            elif [ $MEMORY_MB -lt 200 ]; then
                log_warning "Memory usage: ACCEPTABLE (< 200MB)"
            else
                log_error "Memory usage: HIGH (> 200MB)"
            fi
        fi
    fi

    wait $SEARCH_PID 2>/dev/null || true
fi

echo ""
echo "🎯 Performance Summary"
echo "====================="

if [ "$CALC_AVAILABLE" = true ]; then
    echo "Timing Results:"
    echo "  • Small dataset: Available"
    echo "  • Medium dataset: Available"
    echo "  • Cold start: Available"
    echo "  • Search modes: Available"
else
    echo "Timing Results: Limited (bc not available)"
fi

echo ""
echo "💡 Performance Tips:"
echo "  • Use --limit to reduce result processing time"
echo "  • Basic search is fastest, fuzzy/regex are slower"
echo "  • Release build is significantly faster than debug"
echo "  • Consider using .gitignore to exclude large directories"
echo ""

log_success "Performance testing completed!"
echo ""
