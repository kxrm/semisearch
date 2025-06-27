#!/bin/bash

# CI Validation Script
# Ensures the test environment is properly configured for GitHub Actions

set -euo pipefail

# Colors for output
RED='\033[0;31m'
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

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

echo "🔍 Validating CI Environment"
echo "============================"

# Check Rust installation
log_info "Checking Rust installation..."
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    log_success "Rust found: $RUST_VERSION"
else
    log_error "Rust not found"
    exit 1
fi

if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version)
    log_success "Cargo found: $CARGO_VERSION"
else
    log_error "Cargo not found"
    exit 1
fi

# Check required tools
log_info "Checking required tools..."

TOOLS_MISSING=0

if command -v jq &> /dev/null; then
    log_success "jq found: $(jq --version)"
else
    log_warning "jq not found - JSON tests may be limited"
    TOOLS_MISSING=$((TOOLS_MISSING + 1))
fi

if command -v timeout &> /dev/null; then
    log_success "timeout command available"
else
    log_warning "timeout command not available - tests may run longer"
    TOOLS_MISSING=$((TOOLS_MISSING + 1))
fi

# Check project structure
log_info "Validating project structure..."

REQUIRED_FILES=(
    "Cargo.toml"
    "src/main.rs"
    "src/lib.rs"
    "tests/integration_tests.rs"
    "tests/run-all.sh"
    "tests/test-search.sh"
    "tests/test-performance.sh"
    "docs/SEMANTIC_SEARCH_ARCHITECTURE_PLAN.md"
)

MISSING_FILES=0

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        log_success "Found: $file"
    else
        log_error "Missing: $file"
        MISSING_FILES=$((MISSING_FILES + 1))
    fi
done

# Check test script permissions
log_info "Checking test script permissions..."

TEST_SCRIPTS=(
    "tests/run-all.sh"
    "tests/test-search.sh"
    "tests/test-performance.sh"
    "tests/test_phase2_features.sh"
)

PERMISSION_ISSUES=0

for script in "${TEST_SCRIPTS[@]}"; do
    if [ -f "$script" ]; then
        if [ -x "$script" ]; then
            log_success "Executable: $script"
        else
            log_warning "Not executable: $script (fixing...)"
            chmod +x "$script"
            if [ -x "$script" ]; then
                log_success "Fixed permissions: $script"
            else
                log_error "Failed to fix permissions: $script"
                PERMISSION_ISSUES=$((PERMISSION_ISSUES + 1))
            fi
        fi
    fi
done

# Check dependencies
log_info "Checking project dependencies..."
if cargo check --quiet; then
    log_success "All dependencies resolve correctly"
else
    log_error "Dependency resolution failed"
    exit 1
fi

# Quick build test
log_info "Testing build process..."
if cargo build --quiet; then
    log_success "Project builds successfully"
else
    log_error "Build failed"
    exit 1
fi

# Quick test run
log_info "Running quick test validation..."
if cargo test --lib --quiet > /dev/null 2>&1; then
    log_success "Unit tests pass"
else
    log_error "Unit tests failed"
    exit 1
fi

# Summary
echo ""
echo "📋 Validation Summary"
echo "===================="

if [ $MISSING_FILES -eq 0 ] && [ $PERMISSION_ISSUES -eq 0 ]; then
    log_success "All critical validations passed!"

    if [ $TOOLS_MISSING -gt 0 ]; then
        log_warning "$TOOLS_MISSING optional tools missing (tests will adapt)"
    fi

    echo ""
    log_success "✨ CI environment is ready for testing!"
    exit 0
else
    log_error "Validation failed:"
    [ $MISSING_FILES -gt 0 ] && echo "  - $MISSING_FILES required files missing"
    [ $PERMISSION_ISSUES -gt 0 ] && echo "  - $PERMISSION_ISSUES permission issues"
    exit 1
fi
