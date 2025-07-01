# SemiSearch Semantic Search Accessibility Assessment

**Date**: June 2025
**Author**: AI Assistant  
**Purpose**: Address the gap between semisearch's powerful semantic capabilities and user accessibility

## Executive Summary

The semisearch tool has full semantic search capabilities powered by ONNX runtime and neural embeddings, but these features are effectively hidden from novice users. The tool defaults to keyword-based search, and users must know to use `--advanced` mode and explicitly request `--mode semantic` to access AI-powered search. This assessment analyzes the current barriers and proposes solutions to make semantic search the default experience for capable systems.

**Key Finding**: The user assessment shows that novice users expect semantic behavior (typo correction, concept understanding) but receive literal keyword matching, leading to frustration and the perception that the tool is "not truly semantic."

## Current State Analysis

### 1. Hidden Capabilities

**Problem**: Semantic search is locked behind multiple barriers:
- Requires `--advanced` flag to even see the option
- Requires explicit `--mode semantic` selection
- No automatic detection or suggestion to use semantic mode
- Users don't know what they're missing

**Evidence**: From the user assessment:
> "Not truly 'semantic' - When I searched for 'authentification' (misspelled), it didn't find 'authentication' results like I expected"

The user expected semantic behavior but got keyword matching because they didn't know to enable advanced features.

### 2. Technical Jargon Barriers

**Problem**: The tool uses technical terms that novice users don't understand:
- "ONNX Runtime"
- "Neural embeddings"
- "TF-IDF"
- "Semantic search mode"

**Current doctor output**:
```
🔧 Capability Check:
✅ System supports full neural embeddings
🧪 Testing embedder initialization... ✅ Success
```

This means nothing to a non-technical user who just wants better search results.

### 3. Manual Discovery Required

**Problem**: Users must:
1. Run `doctor` to see capabilities
2. Understand what "neural embeddings" means
3. Know to use `--advanced` flag
4. Know to select `--mode semantic`
5. Know to run `index --semantic` to download models

This is an unreasonable expectation for novice users.

### 4. Capability Detection vs. Enablement

**Current flow**:
```rust
// System detects ONNX is available
EmbeddingCapability::Full => {
    println!("✅ System supports full neural embeddings");
}
```

But it doesn't automatically enable or suggest using these capabilities.

## Proposed Solutions

### 1. Automatic Semantic Mode for Capable Systems

**Implementation**: Make semantic search the default when available:

```rust
// In execute_search() - main.rs
async fn execute_search(query: &str, path: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
    // Auto-detect and use best available mode
    let strategy = match AutoStrategy::with_best_available().await {
        Ok(strategy) => strategy,
        Err(_) => AutoStrategy::new() // Fallback to basic
    };
    
    strategy.search(query, path, options).await
}

// New AutoStrategy method
impl AutoStrategy {
    pub async fn with_best_available() -> Result<Self> {
        // Try semantic first if system is capable
        if CapabilityDetector::can_use_semantic() {
            match Self::with_semantic_search().await {
                Ok(strategy) => return Ok(strategy),
                Err(_) => {} // Fall through to basic
            }
        }
        
        Ok(Self::new())
    }
}
```

### 2. Automatic Model Download on First Use

**Implementation**: When semantic search would help but model is missing:

```rust
// In LocalEmbedder::new()
pub async fn new(config: EmbeddingConfig) -> Result<Self> {
    match CapabilityDetector::detect_neural_capability() {
        NeuralCapability::ModelMissing => {
            // Prompt user in a friendly way
            println!("🎯 Better search results are available!");
            println!("   SemiSearch can understand concepts like 'login' finding 'authentication'");
            println!("   This requires downloading a small AI model (25MB).");
            println!();
            print!("   Download now for smarter search? [Y/n] ");
            
            if user_confirms() {
                Self::download_and_initialize(config).await
            } else {
                Self::new_tfidf_only(config).await
            }
        }
        // ... rest of implementation
    }
}
```

### 3. User-Friendly Status Messages

**Replace technical jargon with benefits**:

```rust
// In status command
println!("🔍 Search capabilities:");
println!("  • Basic search: ✅ Ready");
println!("  • Spell correction: ✅ Ready (--fuzzy)");

match capability {
    EmbeddingCapability::Full => {
        println!("  • Smart search: ✅ Ready");
        println!("    Understands concepts - 'login' finds 'authentication'");
    }
    EmbeddingCapability::TfIdf => {
        println!("  • Smart search: ⚠️ Limited");
        println!("    Missing: Concept understanding");
        println!("    → Run 'semisearch doctor' to enable");
    }
    EmbeddingCapability::None => {
        println!("  • Smart search: ❌ Not available");
        println!("    Your system doesn't support AI features");
    }
}
```

### 4. Progressive Enhancement in Doctor Command

**Make doctor command actionable**:

```rust
async fn run_doctor() -> Result<()> {
    println!("🩺 SemiSearch Health Check");
    
    match CapabilityDetector::detect_neural_capability() {
        NeuralCapability::Available => {
            println!("✅ Smart search is ready!");
            println!("   You can search for concepts:");
            println!("   • 'authentication' finds login code");
            println!("   • 'error handling' finds try/catch");
        }
        
        NeuralCapability::ModelMissing => {
            println!("🎯 Smart search available but not enabled");
            println!();
            println!("   Enable it now? This will:");
            println!("   ✓ Download a 25MB AI model");
            println!("   ✓ Understand typos better");
            println!("   ✓ Find related concepts");
            println!();
            print!("   Enable smart search? [Y/n] ");
            
            if user_confirms() {
                download_and_enable_semantic().await?;
            }
        }
        
        NeuralCapability::Unavailable(reason) => {
            println!("⚠️ Smart search not available: {}", 
                translate_technical_reason(reason));
        }
    }
}
```

### 5. Automatic Mode Selection Based on Query

**Detect when semantic would help**:

```rust
// In main.rs
fn should_use_semantic_search(query: &str) -> bool {
    // Existing conceptual indicators...
    
    // Add: Detect potential typos
    if looks_like_typo(query) {
        return true;
    }
    
    // Add: Multi-word natural language queries
    if query.split_whitespace().count() >= 3 {
        return true;
    }
    
    // Add: Questions or natural language patterns
    if query.starts_with("how") || query.starts_with("what") || 
       query.starts_with("where") || query.contains(" for ") {
        return true;
    }
    
    false
}
```

### 6. First-Run Experience

**Guide users on first use**:

```rust
// Check if first run
if is_first_run() {
    println!("👋 Welcome to SemiSearch!");
    println!();
    
    if CapabilityDetector::can_use_semantic() {
        println!("🎯 Your system supports smart search!");
        println!("   This helps find what you mean, not just what you type");
        println!();
        println!("   Examples:");
        println!("   • Typos: 'authentification' → finds 'authentication'");
        println!("   • Concepts: 'login' → finds auth code");
        println!();
        print!("   Enable smart search? [Y/n] ");
        
        if user_confirms() {
            setup_semantic_search().await?;
        }
    }
}
```

### 7. Remove --advanced Flag for Common Features

**Make semantic search accessible by default**:

```rust
// In CLI builder - show semantic options always when available
fn build_cli() -> Command {
    let mut cmd = Command::new("semisearch")
        .about("Semantic search across local files");
    
    // Basic options always visible
    cmd = cmd
        .arg(Arg::new("fuzzy").long("fuzzy"))
        .arg(Arg::new("exact").long("exact"));
    
    // Add semantic options if system supports it
    if CapabilityDetector::can_use_semantic() {
        cmd = cmd.arg(
            Arg::new("smart")
                .long("smart")
                .help("Use AI to understand what you mean (default when available)")
                .conflicts_with("exact")
        );
    }
    
    cmd
}
```

### 8. Better Error Messages

**When semantic search fails**:

```rust
// Instead of:
// "Semantic search explicitly requested but not available"

// Show:
"Smart search needs to download a small AI model first.
Run 'semisearch doctor' to set this up (takes 30 seconds)."
```

## Implementation Priority

### Phase 1: Automatic Detection (Week 1)
1. Make `AutoStrategy::with_best_available()` the default
2. Remove need for `--advanced` flag for semantic features
3. Auto-detect when semantic search would help

### Phase 2: Friendly Setup (Week 2)
1. Implement friendly model download prompts
2. Create first-run experience
3. Update status/doctor commands with plain language

### Phase 3: Seamless Experience (Week 3)
1. Background model download during idle
2. Progressive enhancement (start with basic, upgrade to semantic)
3. Cache semantic results for instant subsequent searches

## Success Metrics

1. **Discovery Rate**: % of capable systems using semantic search (target: >80%)
2. **Setup Completion**: % of users who complete semantic setup when prompted (target: >60%)
3. **User Satisfaction**: "The tool understands what I meant" (target: >70% agree)
4. **Search Quality**: Reduction in "no results found" for conceptual queries (target: 50% reduction)

## Conclusion

The core issue is that semisearch has powerful AI capabilities that are hidden behind technical barriers. By making semantic search the default experience on capable systems, using plain language, and guiding users through setup, we can deliver on the promise of truly semantic search at the command line.

The key insight: **Users don't need to know about ONNX, neural embeddings, or TF-IDF. They just need search that works better.**

## Immediate Implementation Guide

### Quick Win #1: Enable Semantic by Default in AutoStrategy

**File**: `src/main.rs`
```rust
// Change execute_search to always try semantic first
async fn execute_search(
    query: &str,
    path: &str,
    options: &SearchOptions,
    advanced_mode: bool,
) -> Result<Vec<SearchResult>> {
    use search::search::auto_strategy::AutoStrategy;

    // Always try semantic first for capable systems
    let auto_strategy = match AutoStrategy::with_semantic_search().await {
        Ok(strategy) => {
            // Silently succeeded - user gets semantic search
            strategy
        }
        Err(_) => {
            // Silently fall back to basic search
            AutoStrategy::new()
        }
    };

    auto_strategy.search(query, path, options_to_pass).await
}
```

### Quick Win #2: Friendly Model Download Prompt

**File**: `src/core/embedder.rs`
```rust
impl LocalEmbedder {
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        match CapabilityDetector::detect_neural_capability() {
            NeuralCapability::ModelMissing => {
                // Check if running in interactive mode
                if std::io::stdout().is_terminal() {
                    eprintln!("🎯 First-time setup detected!");
                    eprintln!("   SemiSearch can be smarter with a small AI model.");
                    eprintln!("   • Find concepts: 'login' → 'authentication'");
                    eprintln!("   • Handle typos: 'pasword' → 'password'");
                    eprintln!("   • Size: ~25MB (one-time download)");
                    eprintln!();
                    eprint!("   Enable smart search? [Y/n] ");
                    
                    std::io::stderr().flush()?;
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    
                    if input.trim().is_empty() || input.trim().to_lowercase() == "y" {
                        eprintln!("📥 Downloading AI model...");
                        // Download model
                        Self::download_model(&model_path, &config.model_name).await?;
                        Self::download_tokenizer(&tokenizer_path, &config.model_name).await?;
                        eprintln!("✅ Smart search enabled!");
                        
                        // Continue with neural initialization
                        return Self::initialize_neural(config).await;
                    }
                }
                
                // Fall back to TF-IDF
                Self::new_tfidf_only(config).await
            }
            // ... rest of cases
        }
    }
}
```

### Quick Win #3: Update Status Command

**File**: `src/main.rs`
```rust
async fn handle_simple_status() -> Result<()> {
    println!("🏥 SemiSearch Health Check");
    println!();

    // Check search capabilities with user-friendly language
    println!("🔍 Search capabilities:");
    
    match LocalEmbedder::detect_capabilities() {
        #[cfg(feature = "neural-embeddings")]
        EmbeddingCapability::Full => {
            println!("  • Basic search: ✅ Ready");
            println!("  • Typo correction: ✅ Ready"); 
            println!("  • Smart search: ✅ Ready");
            println!("    → Finds related concepts automatically");
        }
        EmbeddingCapability::TfIdf => {
            println!("  • Basic search: ✅ Ready");
            println!("  • Typo correction: ✅ Ready (--fuzzy)");
            println!("  • Smart search: ⚠️  Basic only");
            
            // Check if we can upgrade
            if CapabilityDetector::detect_neural_capability() == NeuralCapability::ModelMissing {
                println!("    → Run 'semisearch enable-smart-search' to upgrade");
            }
        }
        EmbeddingCapability::None => {
            println!("  • Basic search: ✅ Ready");
            println!("  • Typo correction: ✅ Ready (--fuzzy)");
            println!("  • Smart search: ❌ Not supported on this system");
        }
    }
    
    // ... rest of status
}
```

### Quick Win #4: Add User-Friendly Command

**File**: `src/cli/mod.rs`
```rust
// Add new subcommand for enabling smart search
.subcommand(
    Command::new("enable-smart-search")
        .about("Enable AI-powered search that understands concepts")
        .visible_alias("enable-ai")
)
```

**File**: `src/main.rs`
```rust
Commands::EnableSmartSearch => {
    match CapabilityDetector::detect_neural_capability() {
        NeuralCapability::Available => {
            println!("✅ Smart search is already enabled!");
        }
        NeuralCapability::ModelMissing => {
            println!("🎯 Setting up smart search...");
            println!("   This will download a 25MB AI model.");
            
            let config = EmbeddingConfig::default();
            match LocalEmbedder::new(config).await {
                Ok(_) => println!("✅ Smart search enabled!"),
                Err(e) => println!("❌ Setup failed: {}", e),
            }
        }
        NeuralCapability::Unavailable(reason) => {
            println!("❌ Smart search not available: {}", 
                user_friendly_reason(reason));
        }
    }
}
```

## Additional Recommendations

### 1. Progressive Disclosure in Help

Instead of hiding semantic options behind `--advanced`, show them progressively:

```
$ semisearch --help
Semantic search across local files

USAGE:
    semisearch [OPTIONS] <QUERY> [PATH]

OPTIONS:
    --fuzzy     Handle typos and similar words
    --exact     Find exact matches only
    --smart     Use AI to understand concepts (if available) ← Show only on capable systems

MORE OPTIONS:
    Use --advanced to see all options
```

### 2. Contextual Feature Discovery

When users get poor results, suggest semantic search:

```rust
// In display_simple_results()
if results.is_empty() && !semantic_was_used {
    if CapabilityDetector::can_use_semantic() {
        println!("💡 Tip: Enable smart search for better results:");
        println!("   semisearch enable-smart-search");
    }
}
```

### 3. Silent Upgrades

For users who have already used the tool, silently enable semantic when available:

```rust
// In first run after update
if user_has_search_history() && !semantic_enabled() && can_use_semantic() {
    // Silently download model in background
    tokio::spawn(async {
        let _ = download_semantic_model().await;
    });
}
```

### 4. Natural Language Aliases

Add user-friendly command aliases:

```bash
semisearch "find all login functions"  # Triggers semantic mode
semisearch "what handles errors"       # Triggers semantic mode  
semisearch "TODO"                      # Uses keyword mode
```

### 5. Performance Indicators

Show when smart search is being used:

```
$ semisearch "authentication logic"
🧠 Using smart search...

Found 5 results in 0.23s:
  src/auth.rs:42  fn login(username: &str, password: &str)
  src/security.rs:15  impl Authentication for User
  ...
```

## Intelligent Query Analysis - Beyond Word Matching

Simple word matching is fragile and misses many cases. Here's a more sophisticated approach to detecting when semantic search would be beneficial:

### 1. Query Structure Analysis

```rust
fn looks_conceptual(query: &str) -> bool {
    // Combine multiple signals rather than relying on specific words
    let signals = QuerySignals::analyze(query);
    
    // Weighted scoring system
    let score = 0.0
        + signals.has_natural_language_pattern * 0.3
        + signals.entity_relationship_score * 0.25
        + signals.ambiguity_score * 0.2
        + signals.contains_abstract_concepts * 0.15
        + signals.typo_likelihood * 0.1;
    
    score > 0.5 // Threshold for semantic benefit
}

struct QuerySignals {
    has_natural_language_pattern: f32,
    entity_relationship_score: f32,
    ambiguity_score: f32,
    contains_abstract_concepts: f32,
    typo_likelihood: f32,
}
```

### 2. Natural Language Pattern Detection

```rust
impl QuerySignals {
    fn detect_natural_language_patterns(query: &str) -> f32 {
        let mut score = 0.0;
        
        // Question patterns (without looking for specific words)
        let tokens: Vec<&str> = query.split_whitespace().collect();
        
        // Check for question-like structure
        if tokens.len() >= 3 {
            // Verb-noun patterns: "handle errors", "process data"
            if looks_like_verb_noun(&tokens) {
                score += 0.4;
            }
            
            // Descriptor patterns: "fast sorting algorithm"
            if has_adjective_noun_pattern(&tokens) {
                score += 0.3;
            }
            
            // Relationship patterns: "X for Y", "X with Y", "X in Y"
            if has_preposition_pattern(&tokens) {
                score += 0.3;
            }
        }
        
        score.min(1.0)
    }
}
```

### 3. Entity-Relationship Detection

```rust
fn calculate_entity_relationship_score(query: &str) -> f32 {
    // Look for queries that describe relationships between concepts
    let tokens: Vec<&str> = query.split_whitespace().collect();
    let mut score = 0.0;
    
    // Multi-concept queries benefit from semantic understanding
    let concept_count = estimate_concept_count(&tokens);
    if concept_count >= 2 {
        score += 0.5;
    }
    
    // Queries with implicit relationships
    // "user authentication" -> relationship between user and auth process
    if has_compound_concepts(&tokens) {
        score += 0.5;
    }
    
    score.min(1.0)
}

fn estimate_concept_count(tokens: &[&str]) -> usize {
    // Count potential concept boundaries
    let mut concepts = 1;
    
    for window in tokens.windows(2) {
        if is_concept_boundary(window[0], window[1]) {
            concepts += 1;
        }
    }
    
    concepts
}
```

### 4. Ambiguity and Context Detection

```rust
fn calculate_ambiguity_score(query: &str) -> f32 {
    let mut score = 0.0;
    
    // Shortened words or acronyms benefit from semantic expansion
    if has_abbreviations(query) {
        score += 0.3; // "auth" -> "authentication", "config" -> "configuration"
    }
    
    // Domain-specific terms that have multiple meanings
    if has_polysemous_terms(query) {
        score += 0.4; // "service" (web service? customer service?), "model" (data model? ML model?)
    }
    
    // Incomplete phrases that need context
    if looks_incomplete(query) {
        score += 0.3; // "login flow" -> needs understanding of authentication process
    }
    
    score.min(1.0)
}
```

### 5. Typo Detection Without Dictionary

```rust
fn estimate_typo_likelihood(query: &str) -> f32 {
    let mut score = 0.0;
    
    for word in query.split_whitespace() {
        // Unusual character patterns
        if has_unusual_char_patterns(word) {
            score += 0.2;
        }
        
        // Common typo patterns (double letters, transpositions)
        if matches_common_typo_patterns(word) {
            score += 0.3;
        }
        
        // Keyboard proximity errors
        if has_keyboard_proximity_anomalies(word) {
            score += 0.2;
        }
    }
    
    (score / query.split_whitespace().count() as f32).min(1.0)
}

fn has_unusual_char_patterns(word: &str) -> bool {
    // Unusual consonant clusters, vowel patterns
    let consonant_clusters = count_consonant_clusters(word);
    let vowel_ratio = calculate_vowel_ratio(word);
    
    consonant_clusters > 2 || vowel_ratio < 0.2 || vowel_ratio > 0.8
}
```

### 6. Query Intent Classification

```rust
enum QueryIntent {
    NavigationalExact,  // "main.rs", "config.json" - exact file/location
    Conceptual,         // "error handling", "authentication flow"
    Exploratory,        // "how does X work", "examples of Y"
    Definitional,       // "what is X", "meaning of Y"
    Relational,         // "X related to Y", "X vs Y"
}

fn classify_query_intent(query: &str) -> QueryIntent {
    let signals = QuerySignals::analyze(query);
    
    // Exact matches don't benefit from semantic
    if is_exact_target(query) {
        return QueryIntent::NavigationalExact;
    }
    
    // High conceptual signals
    if signals.entity_relationship_score > 0.7 {
        return QueryIntent::Conceptual;
    }
    
    // Other classifications...
    QueryIntent::Conceptual
}
```

### 7. Practical Implementation

```rust
pub fn should_suggest_semantic_search(
    query: &str, 
    initial_results: &[SearchResult]
) -> bool {
    // Quick exit for obvious exact searches
    if is_exact_file_search(query) {
        return false;
    }
    
    // Calculate composite score
    let query_score = analyze_query_complexity(query);
    let result_quality = assess_result_quality(initial_results);
    
    // Factors that increase semantic benefit:
    // 1. Complex/conceptual query structure
    // 2. Poor initial results
    // 3. Query ambiguity
    // 4. Natural language patterns
    
    let should_suggest = 
        query_score > 0.4 || 
        (query_score > 0.3 && result_quality < 0.5) ||
        initial_results.is_empty();
    
    should_suggest
}

fn analyze_query_complexity(query: &str) -> f32 {
    let signals = QuerySignals::analyze(query);
    
    // Weighted combination of all signals
    let weights = ComplexityWeights {
        natural_language: 0.25,
        relationships: 0.20,
        ambiguity: 0.20,
        abstraction: 0.20,
        typo_likelihood: 0.15,
    };
    
    signals.calculate_weighted_score(&weights)
}
```

### 8. Examples of Detection

| Query | Detection Reasoning | Score |
|-------|-------------------|--------|
| "TODO" | Single token, exact match pattern | 0.1 (keyword) |
| "find todos" | Verb-noun pattern, simple | 0.3 (maybe semantic) |
| "authentication flow" | Compound concept, abstract | 0.7 (semantic) |
| "user login process" | 3 related concepts | 0.8 (semantic) |
| "fast sorting algorithm" | Adjective-noun-concept | 0.6 (semantic) |
| "pasword reset" | Typo detected | 0.7 (semantic) |
| "main.rs" | Exact file reference | 0.0 (keyword) |
| "error handling patterns" | Abstract + compound | 0.8 (semantic) |

### Key Principles

1. **Multiple Signals** - Never rely on single indicators
2. **Graceful Degradation** - Works even with short queries
3. **No Hard-coded Words** - Pattern-based, not vocabulary-based
4. **Context Aware** - Considers result quality too
5. **Fast Computation** - All checks are lightweight

This approach is much more robust than word matching and adapts to different query styles and languages.

## Testing the Changes

### User Journey Tests

1. **First-time user with capable system**:
   - Run any search → Prompted to enable smart search
   - Accept → Model downloads → Search uses semantic mode

2. **Returning user after update**:
   - Run search → Semantic mode auto-enabled if available
   - No prompts, just better results

3. **User on limited system**:
   - Run search → Works with keyword/fuzzy
   - No confusing messages about unavailable features

### Success Criteria

- 80% of users on capable systems have semantic search enabled within first 3 searches
- 0% of users see technical terms like "ONNX" or "neural embeddings"  
- 90% of semantic searches return relevant results for conceptual queries

## Summary

The path to making semantic search accessible is:

1. **Remove barriers**: No --advanced flag needed
2. **Use plain language**: "Smart search" not "neural embeddings"
3. **Make it automatic**: Enable by default on capable systems
4. **Guide gently**: Simple prompts when setup is needed
5. **Work silently**: No technical output unless debugging

The goal is for users to get semantic search without knowing it exists as a separate feature - it should just work better when available.

## Critical Implementation Constraints

### The Distribution Challenge

You've correctly identified the core challenge: When users download semisearch, they get:
- ✅ The semisearch binary
- ❌ No ONNX runtime library 
- ❌ No AI models

This creates a chicken-and-egg problem for "automatic" enablement.

### Realistic Solutions

#### 1. ONNX Runtime Distribution Strategy

**Option A: Runtime Detection + Guided Installation**
```rust
// On first run or when semantic search would help
match CapabilityDetector::detect_neural_capability() {
    NeuralCapability::Unavailable("ONNX Runtime not found") => {
        if query_needs_semantic_search(query) {
            println!("💡 Better results available with smart search!");
            println!();
            
            // Platform-specific instructions
            #[cfg(target_os = "linux")]
            println!("   Install: sudo apt install libonnxruntime");
            
            #[cfg(target_os = "macos")]
            println!("   Install: brew install onnxruntime");
            
            #[cfg(target_os = "windows")]
            println!("   Download from: https://github.com/microsoft/onnxruntime/releases");
            
            println!();
            println!("   After installing, run 'semisearch enable-smart-search'");
        }
    }
    // ... other cases
}
```

**Option B: Bundled Runtime (Platform-Specific Releases)**
- Create platform-specific releases:
  - `semisearch-linux-x64-full.tar.gz` (includes libonnxruntime.so)
  - `semisearch-macos-arm64-full.tar.gz` (includes libonnxruntime.dylib)
  - `semisearch-windows-x64-full.zip` (includes onnxruntime.dll)
- Size increase: ~15-20MB per release
- Installation script extracts runtime to `~/.semisearch/lib/`

**Option C: Runtime Downloader**
```rust
// In capability detector
async fn ensure_onnx_runtime() -> Result<()> {
    let runtime_dir = dirs::home_dir()
        .unwrap()
        .join(".semisearch")
        .join("runtime");
    
    let runtime_path = runtime_dir.join("libonnxruntime.so");
    
    if !runtime_path.exists() {
        println!("📥 First-time setup: Downloading AI runtime...");
        
        // Download appropriate runtime for platform
        let url = get_onnx_runtime_url_for_platform()?;
        download_file(&url, &runtime_path).await?;
        
        // Set environment variable for current session
        std::env::set_var("ORT_DYLIB_PATH", runtime_path);
    }
    
    Ok(())
}
```

#### 2. Model Management Strategy

**Progressive Model Download**
```rust
impl LocalEmbedder {
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        match CapabilityDetector::detect_neural_capability() {
            NeuralCapability::ModelMissing => {
                // Don't prompt immediately - wait for a query that needs it
                return Self::new_tfidf_only(config).await;
            }
            // ... other cases
        }
    }
    
    pub async fn upgrade_if_beneficial(
        &mut self, 
        query: &str, 
        tfidf_results: &[SearchResult]
    ) -> Result<bool> {
        // Only prompt for upgrade if:
        // 1. Query looks conceptual/has typos
        // 2. TF-IDF returned poor/no results
        // 3. User hasn't declined recently
        
        if should_suggest_semantic_upgrade(query, tfidf_results) {
            if prompt_for_model_download().await? {
                self.download_and_upgrade().await?;
                return Ok(true);
            }
        }
        Ok(false)
    }
}
```

#### 3. Three-Tier Implementation

**Tier 1: Basic (Always Available)**
- Keyword search
- Fuzzy matching with edit distance
- Regex patterns
- No external dependencies

**Tier 2: Enhanced (Built-in)**
- TF-IDF embeddings
- Better relevance ranking
- Some concept understanding
- No external dependencies

**Tier 3: Smart (Requires Setup)**
- Full semantic search
- Typo correction via embeddings
- Concept understanding
- Requires ONNX + models

#### 4. First-Run Detection

```rust
// In main.rs
async fn check_first_run() -> Result<()> {
    let config_dir = dirs::home_dir()
        .unwrap()
        .join(".semisearch");
    
    let first_run_marker = config_dir.join(".first_run_complete");
    
    if !first_run_marker.exists() {
        // Check system capabilities
        match CapabilityDetector::detect_neural_capability() {
            NeuralCapability::Available => {
                // System has ONNX, just needs models
                println!("🎯 Welcome to SemiSearch!");
                println!("   Your system supports smart search.");
                println!("   Download AI model now? (25MB) [Y/n]");
                
                if user_confirms() {
                    download_models().await?;
                }
            }
            NeuralCapability::Unavailable(_) => {
                // Show quick start guide
                println!("🚀 Welcome to SemiSearch!");
                println!("   Basic search is ready to use.");
                println!("   For smarter search, run 'semisearch doctor'");
            }
            _ => {}
        }
        
        // Mark first run complete
        fs::write(first_run_marker, "")?;
    }
    
    Ok(())
}
```

#### 5. Smart Defaults Based on Query

```rust
// Don't require setup for basic queries
async fn execute_search(query: &str, path: &str) -> Result<Vec<SearchResult>> {
    // Always start with what's available
    let mut results = basic_search(query, path).await?;
    
    // If results are poor AND query would benefit from semantic
    if results.is_empty() || results_look_poor(&results) {
        if query_needs_semantic(query) {
            // Try to upgrade transparently
            if let Ok(semantic_available) = try_enable_semantic().await {
                if semantic_available {
                    results = semantic_search(query, path).await?;
                }
            }
        }
    }
    
    Ok(results)
}
```

### Recommended Approach

1. **Ship with TF-IDF as default** - Works everywhere, no dependencies
2. **Detect ONNX at runtime** - Check if system has it installed
3. **Lazy model download** - Only when user would benefit
4. **Progressive enhancement** - Start search immediately, upgrade in background
5. **Platform packages** - Offer "full" versions with bundled runtime

### Example User Journeys

**Journey 1: User with ONNX installed**
```
$ semisearch "authentication logic"
🔍 Searching...
💡 Smart search available! Download AI model? [Y/n] y
📥 Downloading model (25MB)...
✅ Smart search enabled!

[Shows semantic results]
```

**Journey 2: User without ONNX**
```
$ semisearch "authentication logic"  
🔍 Searching...
[Shows TF-IDF results]

💡 Tip: Install ONNX Runtime for smarter search
   Ubuntu: sudo apt install libonnxruntime
   Details: semisearch doctor
```

**Journey 3: Progressive Enhancement**
```
$ semisearch "pasword reset"  # typo
🔍 Searching...
No results found.

💡 Smart search can handle typos better!
   Enable now? [Y/n] y
   
[Guides through ONNX + model setup]
```

### Key Principles

1. **Never block search** - Always return results with what's available
2. **Suggest only when beneficial** - Don't prompt for every search
3. **Remember user choices** - Don't nag if they declined
4. **Make it reversible** - Easy to disable/uninstall
5. **Show value first** - Demonstrate why it's worth the setup

This approach acknowledges the distribution constraints while still making semantic search discoverable and accessible to users who would benefit from it.

## Ideal Implementation Flow

Based on your suggestion, here's the recommended user flow that balances automation with user control:

### Step-by-Step User Journey

```
User: semisearch "find the auth routines"
```

**Step 1: Intelligent Query Analysis**
```rust
// Detect semantic-friendly query patterns
if query.contains_any(["find the", "find all", "show me", "where is"]) ||
   query.word_count() >= 3 ||
   looks_conceptual(query) {
    // This query would benefit from semantic search
    check_semantic_availability().await
}
```

**Step 2: Capability Check & First Prompt**
```
🔍 Searching...
[Shows basic results first - never block]

💡 Smart search may work better with queries like this.
   It understands concepts like 'auth' → 'authentication', 'login'
   
   Would you like to enable smart search? [Y/n]
```

**Step 3: If User Accepts - Installation Options**
```
User: y

Checking system...

✨ Your system can support smart search!
   Two things needed:
   • ONNX Runtime library (5MB)
   • AI model (25MB)

Would you like to automate the installation? [Y/n]
```

**Step 4A: Automated Installation Path**
```
User: y

📥 Installing ONNX Runtime...
   ✓ Downloaded to ~/.semisearch/lib/
   ✓ Library path configured

📥 Downloading AI model...
   ✓ Model ready at ~/.semisearch/models/

✅ Smart search enabled!

🔍 Re-running your search with smart mode...
[Shows enhanced semantic results]
```

**Step 4B: Manual Installation Path**
```
User: n

No problem! Here's how to set it up manually:

📦 System-wide installation (recommended):
   Ubuntu/Debian: sudo apt install libonnxruntime
   macOS: brew install onnxruntime
   Fedora: sudo dnf install onnxruntime

📁 User-level installation:
   1. Download from: https://github.com/microsoft/onnxruntime/releases
   2. Extract to: ~/.semisearch/lib/
   3. Run: semisearch doctor

After installing, run your search again to use smart mode.
```

### Implementation Details

```rust
// In capability_detector.rs
pub async fn prompt_for_smart_search() -> Result<SmartSearchSetup> {
    match detect_neural_capability() {
        NeuralCapability::Available => {
            // Just need model
            Ok(SmartSearchSetup::ModelOnly)
        }
        NeuralCapability::ModelMissing => {
            // Need model, runtime is ready
            Ok(SmartSearchSetup::ModelOnly)
        }
        NeuralCapability::Unavailable("ONNX Runtime not found") => {
            // Need both runtime and model
            Ok(SmartSearchSetup::FullSetup)
        }
        NeuralCapability::Insufficient(reason) => {
            // Can't use semantic search
            Err(anyhow!("System requirements not met: {}", reason))
        }
    }
}

// In main.rs or search handler
async fn handle_semantic_upgrade(query: &str) -> Result<bool> {
    // Show initial prompt
    if !prompt_yes_no("Smart search may work better with queries like this.\n   \
                       It understands concepts like 'auth' → 'authentication', 'login'\n   \n   \
                       Would you like to enable smart search?") {
        return Ok(false);
    }
    
    match prompt_for_smart_search().await? {
        SmartSearchSetup::ModelOnly => {
            // Just download model
            download_model_with_progress().await?;
            Ok(true)
        }
        SmartSearchSetup::FullSetup => {
            // Offer automated vs manual
            if prompt_yes_no("Would you like to automate the installation?") {
                install_onnx_runtime().await?;
                download_model_with_progress().await?;
                Ok(true)
            } else {
                show_manual_instructions();
                Ok(false)
            }
        }
    }
}
```

### Key Features

1. **Never Blocks Search** - Always shows results with available capabilities first
2. **Progressive Consent** - Two clear decision points with escape hatches
3. **Clear Benefits** - Explains what smart search does in plain language
4. **Flexible Installation** - Both automated and manual paths
5. **Persistent Choice** - Remember if user declined to avoid nagging

### Configuration Storage

```rust
// Store user preferences
#[derive(Serialize, Deserialize)]
struct UserPreferences {
    smart_search_prompted: bool,
    smart_search_declined: Option<DateTime<Utc>>,
    smart_search_enabled: bool,
    last_prompt_version: String,
}

// Don't prompt again if:
// - User declined in last 30 days
// - User already has it enabled
// - System can't support it
```

This approach provides the best of both worlds - easy automation for users who want it, and full control for those who prefer manual setup. 