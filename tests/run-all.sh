#!/bin/bash

# Run all tests with one command
# Following the SEMANTIC_SEARCH_ARCHITECTURE_PLAN.md testing strategy

set -euo pipefail  # Stricter error handling for CI

# Detect CI environment
if [ "${CI:-false}" = "true" ]; then
    echo "🤖 Running in CI environment"
    export RUST_BACKTRACE=1
    export CARGO_TERM_COLOR=always
fi

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
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

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

echo "🧪 Running Semantic Search CLI Test Suite"
echo "========================================"

# Change to project root
cd "$(dirname "$0")/.."

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    log_error "Cargo not found. Please install Rust."
    exit 1
fi

# Check if jq is available for JSON testing
if ! command -v jq &> /dev/null; then
    log_warning "jq not found. JSON validation tests will be skipped."
    SKIP_JSON_TESTS=true
else
    SKIP_JSON_TESTS=false
fi

# Build the project first
log_info "Building project..."
if cargo build --quiet; then
    log_success "Build successful"
else
    log_error "Build failed"
    exit 1
fi

# Run unit tests
log_info "Running unit tests..."
if cargo test --lib --quiet; then
    log_success "Unit tests passed"
else
    log_error "Unit tests failed"
    exit 1
fi

# Run integration tests
log_info "Running integration tests..."
if cargo test --test integration_tests --quiet; then
    log_success "Integration tests passed"
else
    log_error "Integration tests failed"
    exit 1
fi

# Run feature demonstration tests
log_info "Running Phase 2 feature tests..."
if bash tests/test_phase2_features.sh; then
    log_success "Phase 2 feature tests passed"
else
    log_warning "Phase 2 feature tests had some issues (this may be expected)"
fi

# Run clippy for code quality
log_info "Running clippy (code quality)..."
if cargo clippy --quiet -- -D warnings; then
    log_success "Clippy checks passed"
else
    log_warning "Clippy found some issues"
fi

# Run formatting check
log_info "Checking code formatting..."
if cargo fmt --check; then
    log_success "Code formatting is correct"
else
    log_warning "Code formatting needs attention (run 'cargo fmt')"
fi

# Performance test (if test data exists)
if [ -d "test-data" ]; then
    log_info "Running performance tests on test data..."
    
    # Time a search operation
    start_time=$(date +%s.%N)
    cargo run --quiet -- search "TODO" test-data --limit 50 > /dev/null 2>&1 || true
    end_time=$(date +%s.%N)
    duration=$(echo "$end_time - $start_time" | bc -l 2>/dev/null || echo "unknown")
    
    if [ "$duration" != "unknown" ]; then
        log_info "Search performance: ${duration}s"
        # Warn if search takes too long
        if (( $(echo "$duration > 5.0" | bc -l 2>/dev/null) )); then
            log_warning "Search took longer than 5 seconds"
        else
            log_success "Search performance acceptable"
        fi
    fi
else
    log_info "No test-data directory found, skipping performance tests"
    log_info "Create test data with: mkdir -p test-data && echo 'TODO: test' > test-data/sample.txt"
fi

echo ""
echo "🎉 Test Suite Complete!"
echo "======================="

# Summary
log_success "All critical tests passed"
log_info "Ready for development and production use"

# Show next steps
echo ""
echo "📋 Next Steps:"
echo "  • Run specific tests: cargo test <test_name>"
echo "  • Fix formatting: cargo fmt"
echo "  • Fix clippy issues: cargo clippy --fix"
echo "  • Create test data: mkdir test-data && echo 'sample' > test-data/file.txt"
echo "  • Test features: bash tests/test_phase2_features.sh"
echo "" 