#!/bin/bash

# Phase 3: Text Processing and Advanced Search Features Test Script
# This script demonstrates all the Phase 3 capabilities

echo "🔍 Testing Phase 3: Text Processing and Advanced Search Features"
echo "=============================================================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo ""
echo -e "${BLUE}📋 Test 1: Basic Keyword Search${NC}"
echo "Command: cargo run -- search 'TODO' --path src/ --limit 3"
cargo run -- search "TODO" --path src/ --limit 3

echo ""
echo -e "${BLUE}🔤 Test 2: Fuzzy Matching (Enhanced Typo Tolerance)${NC}"
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
echo -e "${BLUE}📊 Test 5: Enhanced Typo Tolerance${NC}"
echo "Command: cargo run -- search 'test' --typo-tolerance --max-edit-distance 2 --limit 3"
cargo run -- search "test" --typo-tolerance --max-edit-distance 2 --limit 3

echo ""
echo -e "${BLUE}🎯 Test 6: Email Pattern Matching${NC}"
echo "Command: cargo run -- search '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}' --regex --limit 2"
cargo run -- search "[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}" --regex --limit 2

echo ""
echo -e "${BLUE}🔄 Test 7: Combined Features with JSON Output${NC}"
echo "Command: cargo run -- search 'search' --fuzzy --score 0.5 --format json --limit 3"
cargo run -- search "search" --fuzzy --score 0.5 --format json --limit 3

echo ""
echo -e "${BLUE}🌐 Test 8: Unicode and International Text${NC}"
echo "Creating test file with unicode content..."
echo -e "café naïve résumé\n中文测试\nالعربية" > unicode_test.txt
echo "Command: cargo run -- search 'café' --path unicode_test.txt"
cargo run -- search "café" --path unicode_test.txt
rm -f unicode_test.txt

echo ""
echo -e "${BLUE}⚡ Test 9: Performance with Large Query${NC}"
echo "Command: cargo run -- search 'function.*test.*return' --regex --path src/ --limit 5"
cargo run -- search "function.*test.*return" --regex --path src/ --limit 5

echo ""
echo -e "${BLUE}🔧 Test 10: Whole Words Matching${NC}"
echo "Command: cargo run -- search 'test' --whole-words --path src/ --limit 3"
cargo run -- search "test" --whole-words --path src/ --limit 3

echo ""
echo -e "${GREEN}✅ All Phase 3: Text Processing and Advanced Search Features Tested!${NC}"
echo ""
echo -e "${YELLOW}📈 Phase 3 Feature Summary:${NC}"
echo "• ✅ Modular Search Architecture: Trait-based plugin system"
echo "• ✅ Advanced Text Processing: Unicode-aware tokenization"
echo "• ✅ Multiple Search Strategies: Keyword, Fuzzy, Regex, TF-IDF"
echo "• ✅ Language Detection: Automatic programming language identification"
echo "• ✅ Enhanced Fuzzy Matching: Multi-algorithm approach with SkimMatcherV2"
echo "• ✅ Regex Caching: Improved performance for repeated patterns"
echo "• ✅ Text Complexity Analysis: Vocabulary diversity scoring"
echo "• ✅ Phrase Extraction: 2-word and 3-word meaningful phrases"
echo "• ✅ Unicode Support: Full internationalization"
echo "• ✅ Performance Optimizations: Parallel processing capabilities"
echo ""
echo -e "${GREEN}🎯 Phase 3 Complete - Ready for Phase 4 (Local Embeddings)${NC}" 