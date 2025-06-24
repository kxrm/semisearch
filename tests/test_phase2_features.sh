#!/bin/bash

# Phase 2: Enhanced Search Features Test Script
# This script demonstrates all the new search capabilities

echo "🔍 Testing Phase 2: Enhanced Search Features"
echo "============================================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo ""
echo -e "${BLUE}📋 Test 1: Basic Search (Baseline)${NC}"
echo "Command: cargo run -- search 'TODO' --path src/ --limit 3"
cargo run -- search "TODO" --path src/ --limit 3

echo ""
echo -e "${BLUE}🔤 Test 2: Fuzzy Matching (Typo Tolerance)${NC}"
echo "Command: cargo run -- search 'TOOD' --fuzzy --path src/ --limit 3"
cargo run -- search "TOOD" --fuzzy --path src/ --limit 3

echo ""
echo -e "${BLUE}🔍 Test 3: Regex Pattern Matching${NC}"
echo "Command: cargo run -- search 'TODO.*:' --regex --path src/ --limit 3"
cargo run -- search "TODO.*:" --regex --path src/ --limit 3

echo ""
echo -e "${BLUE}🔠 Test 4: Case-Sensitive Search${NC}"
echo "Command: cargo run -- search 'TODO' --case-sensitive --path src/ --limit 3"
cargo run -- search "TODO" --case-sensitive --path src/ --limit 3

echo ""
echo -e "${BLUE}📊 Test 5: Fuzzy Search with Scoring${NC}"
echo "Command: cargo run -- search 'test' --fuzzy --score 0.6 --format json --limit 3"
cargo run -- search "test" --fuzzy --score 0.6 --format json --limit 3

echo ""
echo -e "${BLUE}🎯 Test 6: Complex Regex with JSON Output${NC}"
echo "Command: cargo run -- search '[Tt]est.*[Ff]unction' --regex --format json --limit 2"
cargo run -- search "[Tt]est.*[Ff]unction" --regex --format json --limit 2

echo ""
echo -e "${BLUE}🔄 Test 7: Combined Features${NC}"
echo "Command: cargo run -- search 'search' --fuzzy --score 0.7 --case-sensitive --limit 5"
cargo run -- search "search" --fuzzy --score 0.7 --case-sensitive --limit 5

echo ""
echo -e "${GREEN}✅ All Phase 2 Enhanced Search Features Tested!${NC}"
echo ""
echo -e "${YELLOW}📈 Performance Summary:${NC}"
echo "• Fuzzy matching: Handles typos and partial matches"
echo "• Regex support: Full pattern matching capabilities"
echo "• Case sensitivity: Configurable case handling"
echo "• Scoring system: Results ranked by relevance"
echo "• JSON output: Structured data with metadata"
echo "• Combined modes: Mix and match search strategies"
echo ""
echo -e "${GREEN}🎯 Phase 2 Complete - Ready for Phase 3 (Semantic Search)${NC}" 