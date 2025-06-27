#!/bin/bash

# Test script to validate devcontainer setup
echo "🧪 Testing devcontainer setup..."

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

test_passed=0
test_failed=0

check_command() {
    local cmd=$1
    local name=$2

    if command -v $cmd >/dev/null 2>&1; then
        echo -e "${GREEN}✅ $name found${NC}"
        ((test_passed++))
    else
        echo -e "${RED}❌ $name not found${NC}"
        ((test_failed++))
    fi
}

check_rust_tool() {
    local cmd=$1
    local name=$2

    if cargo --list | grep -q $cmd; then
        echo -e "${GREEN}✅ $name (cargo) found${NC}"
        ((test_passed++))
    else
        echo -e "${RED}❌ $name (cargo) not found${NC}"
        ((test_failed++))
    fi
}

echo "🔧 Checking core tools..."
check_command "rustc" "Rust compiler"
check_command "cargo" "Cargo"
check_command "git" "Git"
check_command "sqlite3" "SQLite3"
check_command "python3" "Python3"

echo ""
echo "🦀 Checking Rust tools..."
check_rust_tool "watch" "cargo-watch"
check_rust_tool "edit" "cargo-edit"
check_rust_tool "audit" "cargo-audit"
check_command "just" "just"
check_command "cross" "cross"

echo ""
echo "🛠️ Checking optional development tools..."
check_command "rg" "ripgrep"
check_command "fd" "fd-find"
check_command "bat" "bat"
check_command "exa" "exa"
check_command "hyperfine" "hyperfine"
check_command "tokei" "tokei"

echo ""
echo "📁 Checking project structure..."
if [ -d "/workspace" ]; then
    echo -e "${GREEN}✅ /workspace directory exists${NC}"
    ((test_passed++))
else
    echo -e "${RED}❌ /workspace directory missing${NC}"
    ((test_failed++))
fi

if [ -d "/workspace/scripts" ]; then
    echo -e "${GREEN}✅ Scripts directory exists${NC}"
    ((test_passed++))
else
    echo -e "${RED}❌ Scripts directory missing${NC}"
    ((test_failed++))
fi

echo ""
echo "📊 Test Results:"
echo -e "  Passed: ${GREEN}$test_passed${NC}"
echo -e "  Failed: ${RED}$test_failed${NC}"

if [ $test_failed -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed! Development environment is ready.${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠️  Some tools are missing but basic development should work.${NC}"
    echo "You can install missing tools later with cargo or the update script."
    exit 0
fi
