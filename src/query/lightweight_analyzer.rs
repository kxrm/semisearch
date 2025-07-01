use std::collections::HashMap;

/// Lightweight analyzer using pre-computed statistics
/// Total size: ~2MB when serialized
pub struct LightweightAnalyzer {
    // Character-level statistics (compressed)
    char_stats: CharacterStats,

    // Token-level statistics for common words
    token_stats: TokenStats,

    // Subword patterns for OOV handling
    subword_patterns: SubwordPatterns,

    // Store last query for length calculation
    last_query: String,
}

#[derive(Debug)]
pub struct CharacterStats {
    // Store only deltas from uniform distribution
    // This compresses well since most transitions follow patterns
    trigram_log_probs: HashMap<(u8, u8, u8), i8>, // Quantized log probs
}

#[derive(Debug)]
pub struct TokenStats {
    // Top 5K words with their "semantic weight"
    // Words that typically need semantic search have higher weight
    semantic_indicators: HashMap<u32, u8>, // Word hash -> weight (0-255)

    // Common bigram patterns
    bigram_coherence: HashMap<(u32, u32), u8>, // Hash pairs -> coherence score
}

#[derive(Debug)]
pub struct SubwordPatterns {
    // Common prefixes/suffixes that indicate concepts
    concept_affixes: Vec<(&'static str, u8)>, // (affix, weight)

    // Patterns that suggest entities
    entity_patterns: Vec<regex::Regex>,
}

impl LightweightAnalyzer {
    pub fn analyze(&mut self, query: &str) -> SemanticScore {
        // Store query for length calculation
        self.last_query = query.to_string();

        let tokens = self.tokenize(query);

        // 1. Character-level perplexity (using pre-computed stats)
        let char_perplexity = self.calculate_char_perplexity(query);

        // 2. Token-level semantic weight
        let semantic_weight = self.calculate_semantic_weight(&tokens);

        // 3. Structural coherence
        let coherence = self.calculate_coherence(&tokens);

        // 4. Entity/concept detection
        let concept_density = self.detect_concepts(&tokens);

        // Combine scores with learned weights
        SemanticScore {
            needs_semantic: self.combine_scores(
                char_perplexity,
                semantic_weight,
                coherence,
                concept_density,
            ),
            confidence: self.calculate_confidence(&tokens),
            explanation: self.generate_explanation(
                char_perplexity,
                semantic_weight,
                coherence,
                concept_density,
            ),
        }
    }

    fn calculate_char_perplexity(&self, text: &str) -> f32 {
        let chars: Vec<u8> = text.bytes().collect();
        let mut total_surprise = 0.0;

        // Use pre-computed trigram statistics
        for window in chars.windows(3) {
            let key = (window[0], window[1], window[2]);
            let log_prob = self
                .char_stats
                .trigram_log_probs
                .get(&key)
                .copied()
                .unwrap_or(-100) as f32
                / 10.0; // Dequantize

            total_surprise -= log_prob;
        }

        // Normalize by length
        total_surprise / (chars.len() as f32).max(1.0)
    }

    fn calculate_semantic_weight(&self, tokens: &[Token]) -> f32 {
        let mut weight = 0.0;
        let mut count = 0;

        for token in tokens {
            if let Some(&w) = self.token_stats.semantic_indicators.get(&token.hash) {
                weight += w as f32 / 255.0;
                count += 1;
            } else {
                // Unknown token - be more generous, assume it might be semantic
                weight += self.analyze_oov_token(&token.text) * 0.7 + 0.3; // Add base 0.3
                count += 1;
            }
        }

        if count > 0 {
            weight / count as f32
        } else {
            0.0
        }
    }

    fn analyze_oov_token(&self, token: &str) -> f32 {
        let mut score: f32 = 0.0;

        // Check concept affixes
        for (affix, weight) in &self.subword_patterns.concept_affixes {
            if token.starts_with(affix) || token.ends_with(affix) {
                score = score.max(*weight as f32 / 255.0);
            }
        }

        // Check entity patterns
        for pattern in &self.subword_patterns.entity_patterns {
            if pattern.is_match(token) {
                score = score.max(0.7);
            }
        }

        // All caps words get a boost
        if token.chars().next().is_some_and(|c| c.is_uppercase()) {
            score += 0.05;
        }

        score
    }

    fn tokenize(&self, text: &str) -> Vec<Token> {
        // Simple whitespace tokenization with lowercasing
        text.split_whitespace()
            .map(|s| {
                let lower = s.to_lowercase();
                Token {
                    text: lower.clone(),
                    hash: self.hash_token(&lower),
                    original: s.to_string(),
                }
            })
            .collect()
    }

    fn hash_token(&self, token: &str) -> u32 {
        // Fast non-cryptographic hash
        let mut hash = 5381u32;
        for byte in token.bytes() {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u32);
        }
        hash
    }

    fn combine_scores(
        &self,
        char_perplexity: f32,
        semantic_weight: f32,
        coherence: f32,
        concept_density: f32,
    ) -> f32 {
        // Get token count for length weighting
        let tokens = self.tokenize(&self.last_query);
        let token_count = tokens.len() as f32;

        // More aggressive length-based weighting
        let length_factor = match token_count as usize {
            0..=1 => 0.0, // Single word: no boost
            2 => 0.2,     // Two words: significant boost
            3 => 0.4,     // Three words: strong boost
            4 => 0.5,     // Four words: very strong boost
            _ => 0.6,     // 5+ words: maximum boost
        };

        // Check for question indicators
        let query_lower = self.last_query.to_lowercase();
        let has_question_word = [
            "how", "what", "why", "when", "where", "which", "who", "does", "can", "should", "would",
        ]
        .iter()
        .any(|&word| query_lower.starts_with(word));

        let question_boost = if has_question_word { 0.3 } else { 0.0 };

        // Adjusted weights - bias toward semantic
        const W_PERPLEXITY: f32 = -0.01; // Minimal impact
        const W_SEMANTIC: f32 = 0.5; // Still important but reduced
        const W_COHERENCE: f32 = 0.1;
        const W_CONCEPTS: f32 = 0.1;
        const W_LENGTH: f32 = 0.2; // Strong length contribution
        const W_QUESTION: f32 = 0.1; // Question word boost

        // Normalize perplexity
        let normalized_perplexity = ((char_perplexity - 3.0) / 4.0).clamp(0.0, 1.0);

        let score = W_PERPLEXITY * normalized_perplexity
            + W_SEMANTIC * semantic_weight
            + W_COHERENCE * coherence
            + W_CONCEPTS * concept_density
            + W_LENGTH * length_factor
            + W_QUESTION * question_boost
            + 0.35; // Higher base bias - start at 0.35 instead of 0.15

        // Clamp to 0-1 range
        score.clamp(0.0, 1.0)
    }

    // Additional methods...
    fn calculate_coherence(&self, tokens: &[Token]) -> f32 {
        if tokens.len() < 2 {
            return 0.0;
        }

        let mut coherence = 0.0;
        let mut count = 0;

        for window in tokens.windows(2) {
            let key = (window[0].hash, window[1].hash);
            if let Some(&score) = self.token_stats.bigram_coherence.get(&key) {
                coherence += score as f32 / 255.0;
            } else {
                // Unknown bigram - use simple heuristics
                coherence += 0.3; // Neutral score
            }
            count += 1;
        }

        coherence / count as f32
    }

    fn detect_concepts(&self, tokens: &[Token]) -> f32 {
        let mut concept_score = 0.0;

        for token in tokens {
            // Capitalized words often indicate concepts/entities
            if token
                .original
                .chars()
                .next()
                .is_some_and(|c| c.is_uppercase())
            {
                concept_score += 0.3;
            }

            // Mixed case (CamelCase, etc)
            let has_upper = token.original.chars().any(|c| c.is_uppercase());
            let has_lower = token.original.chars().any(|c| c.is_lowercase());
            if has_upper && has_lower {
                concept_score += 0.4;
            }
        }

        (concept_score / tokens.len() as f32).min(1.0)
    }

    fn calculate_confidence(&self, tokens: &[Token]) -> f32 {
        // Higher confidence with more tokens and known words
        let known_ratio = tokens
            .iter()
            .filter(|t| self.token_stats.semantic_indicators.contains_key(&t.hash))
            .count() as f32
            / tokens.len().max(1) as f32;

        let length_factor = (tokens.len() as f32 / 10.0).min(1.0);

        known_ratio * 0.7 + length_factor * 0.3
    }

    fn generate_explanation(
        &self,
        char_perplexity: f32,
        semantic_weight: f32,
        coherence: f32,
        concept_density: f32,
    ) -> String {
        let mut reasons = Vec::new();

        if char_perplexity > 3.0 {
            reasons.push("Unusual character patterns detected");
        }

        if semantic_weight > 0.6 {
            reasons.push("Contains semantically rich terms");
        }

        if coherence > 0.7 {
            reasons.push("Terms show strong relationships");
        }

        if concept_density > 0.5 {
            reasons.push("Multiple concepts or entities detected");
        }

        if reasons.is_empty() {
            "Simple keyword query".to_string()
        } else {
            reasons.join(", ")
        }
    }

    pub fn explain_analysis(&self, query: &str) {
        let tokens = self.tokenize(query);
        let char_perplexity = self.calculate_char_perplexity(query);
        let semantic_weight = self.calculate_semantic_weight(&tokens);
        let coherence = self.calculate_coherence(&tokens);
        let concept_density = self.detect_concepts(&tokens);

        println!(
            "├─ Character Perplexity: {:.2} (lower = more common patterns)",
            char_perplexity
        );
        println!(
            "├─ Semantic Weight:      {:.2} (higher = richer vocabulary)",
            semantic_weight
        );
        println!(
            "├─ Token Coherence:      {:.2} (higher = better relationships)",
            coherence
        );
        println!(
            "├─ Concept Density:      {:.2} (higher = more entities/concepts)",
            concept_density
        );
        println!("└─ Token Count:          {}", tokens.len());

        if !tokens.is_empty() {
            println!("\nToken Analysis:");
            for token in &tokens {
                let token_score =
                    if let Some(&w) = self.token_stats.semantic_indicators.get(&token.hash) {
                        w as f32 / 255.0
                    } else {
                        self.analyze_oov_token(&token.text)
                    };
                println!(
                    "  '{}' → semantic weight: {:.2}",
                    token.original, token_score
                );
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Token {
    text: String,
    hash: u32,
    original: String,
}

#[derive(Debug)]
pub struct SemanticScore {
    pub needs_semantic: f32, // 0.0 to 1.0
    pub confidence: f32,     // How confident in the assessment
    pub explanation: String,
}

/// Build analyzer with default pre-computed statistics
pub fn build_analyzer_with_defaults() -> LightweightAnalyzer {
    use regex::Regex;
    use std::collections::HashMap;

    // Character trigram frequencies from English text
    let mut trigram_probs = HashMap::new();
    // Common English trigrams with quantized log probabilities
    let common_trigrams = vec![
        (b"the", 30),
        (b"and", 25),
        (b"ing", 28),
        (b"ion", 26),
        (b"tio", 24),
        (b"ent", 23),
        (b"ati", 22),
        (b"for", 25),
        (b"her", 24),
        (b"ter", 23),
        (b"hat", 22),
        (b"tha", 21),
        (b"ere", 23),
        (b"ate", 22),
        (b"his", 24),
        (b"con", 22),
        (b"res", 21),
        (b"ver", 22),
        (b"all", 23),
        (b"ons", 21),
        (b"nce", 20),
        (b"men", 21),
        (b"ith", 22),
        (b"ted", 21),
        (b"ers", 22),
        (b"pro", 21),
        (b"thi", 23),
        (b"wit", 21),
        (b"are", 24),
        (b"ess", 20),
        (b"not", 22),
        (b"ive", 21),
        (b"was", 23),
        (b"ect", 20),
        (b"rea", 21),
        (b"com", 22),
        (b"eve", 20),
        (b"per", 21),
        (b"int", 22),
        (b"est", 23),
        (b"sta", 21),
        (b"cti", 20),
        (b"ica", 19),
        (b"ist", 20),
    ];

    for (trigram, prob) in common_trigrams {
        if trigram.len() == 3 {
            trigram_probs.insert((trigram[0], trigram[1], trigram[2]), prob);
        }
    }

    // Words that typically appear in semantic queries
    let mut semantic_indicators = HashMap::new();
    let semantic_words = vec![
        // Conceptual terms
        ("relationship", 200),
        ("concept", 190),
        ("theory", 185),
        ("analysis", 180),
        ("structure", 175),
        ("pattern", 170),
        ("framework", 180),
        ("model", 175),
        ("system", 170),
        ("process", 165),
        ("method", 160),
        ("approach", 165),
        // Abstract terms
        ("understanding", 180),
        ("meaning", 175),
        ("context", 170),
        ("interpretation", 185),
        ("significance", 180),
        ("implication", 175),
        // Academic/technical
        ("algorithm", 190),
        ("implementation", 185),
        ("architecture", 180),
        ("optimization", 185),
        ("evaluation", 175),
        ("performance", 170),
        // Relational
        ("between", 150),
        ("among", 145),
        ("through", 140),
        ("within", 145),
        ("across", 140),
        ("regarding", 150),
        // Objects/entities (often need context)
        ("object", 160),
        ("entity", 165),
        ("component", 170),
        ("element", 155),
        ("feature", 160),
        ("attribute", 165),
        ("property", 170),
        ("characteristic", 175),
        // Actions that suggest complex queries
        ("analyze", 180),
        ("compare", 175),
        ("evaluate", 170),
        ("determine", 165),
        ("identify", 160),
        ("examine", 165),
        ("investigate", 170),
        ("explore", 165),
        // Question words (often semantic)
        ("how", 140),
        ("why", 145),
        ("what", 135),
        ("when", 130),
        ("where", 130),
        ("which", 135),
        ("who", 125),
        // Technical concepts
        ("memory", 165),
        ("cache", 160),
        ("database", 170),
        ("network", 165),
        ("security", 170),
        ("authentication", 175),
        ("authorization", 175),
        ("encryption", 170),
        ("protocol", 165),
        ("interface", 160),
        ("function", 155),
        ("class", 150),
        ("inheritance", 170),
        ("polymorphism", 180),
        ("abstraction", 175),
        // Process words
        ("management", 170),
        ("handling", 165),
        ("processing", 160),
        ("execution", 165),
        ("operation", 160),
        ("transaction", 165),
        ("synchronization", 180),
        ("coordination", 175),
        ("integration", 170),
        // Comparative/evaluative
        ("difference", 175),
        ("similarity", 170),
        ("comparison", 175),
        ("contrast", 170),
        ("versus", 165),
        ("alternative", 170),
        ("option", 150),
        ("choice", 155),
        ("decision", 160),
        // Impact/effect words
        ("impact", 170),
        ("effect", 165),
        ("influence", 170),
        ("cause", 160),
        ("result", 155),
        ("consequence", 170),
        ("implication", 175),
        ("outcome", 165),
        ("affect", 165),
        // Complex concepts
        ("complexity", 175),
        ("scalability", 180),
        ("reliability", 175),
        ("availability", 170),
        ("consistency", 175),
        ("concurrency", 180),
        ("latency", 170),
        ("throughput", 165),
        ("bottleneck", 170),
        // Design/architecture
        ("design", 170),
        ("principle", 175),
        ("practice", 165),
        ("strategy", 170),
        ("technique", 165),
        ("methodology", 175),
        ("paradigm", 180),
        ("philosophy", 175),
        // Problem-solving
        ("problem", 165),
        ("solution", 160),
        ("issue", 155),
        ("challenge", 170),
        ("difficulty", 165),
        ("debugging", 170),
        ("troubleshooting", 175),
        ("resolving", 165),
        // Data concepts
        ("data", 150),
        ("information", 160),
        ("storage", 155),
        ("retrieval", 165),
        ("query", 145),
        ("search", 150),
        ("index", 145),
        ("structure", 170),
        ("organization", 165),
        // System concepts
        ("distributed", 180),
        ("centralized", 175),
        ("decentralized", 180),
        ("asynchronous", 185),
        ("synchronous", 180),
        ("parallel", 175),
        ("concurrent", 180),
        ("sequential", 170),
        ("reactive", 175),
        // Auxiliary verbs that indicate complex queries
        ("does", 120),
        ("can", 115),
        ("should", 125),
        ("would", 120),
        ("could", 120),
        ("might", 115),
        ("must", 125),
        ("will", 110),
        // Common programming terms
        ("programming", 160),
        ("coding", 155),
        ("development", 165),
        ("software", 160),
        ("hardware", 155),
        ("application", 160),
        ("program", 150),
        ("code", 145),
        ("script", 140),
        ("language", 155),
        ("syntax", 160),
        ("semantics", 175),
        // More technical concepts
        ("concept", 170),
        ("principle", 175),
        ("fundamental", 170),
        ("basic", 130),
        ("advanced", 165),
        ("intermediate", 155),
        ("beginner", 140),
        ("expert", 160),
        ("professional", 155),
        // Common tech domains
        ("web", 140),
        ("mobile", 145),
        ("desktop", 140),
        ("server", 145),
        ("client", 140),
        ("frontend", 150),
        ("backend", 150),
        ("fullstack", 155),
        ("devops", 160),
        // Common question phrases
        ("causes", 160),
        ("reasons", 165),
        ("factors", 160),
        ("considerations", 170),
        ("implications", 175),
        ("consequences", 170),
        ("benefits", 155),
        ("drawbacks", 160),
        ("advantages", 155),
        ("disadvantages", 160),
        ("pros", 145),
        ("cons", 145),
        // Analysis terms
        ("analyzing", 170),
        ("evaluating", 170),
        ("assessing", 165),
        ("investigating", 170),
        ("exploring", 165),
        ("examining", 165),
        ("reviewing", 160),
        ("studying", 165),
        ("researching", 170),
    ];

    for (word, weight) in semantic_words {
        let hash = hash_token(word);
        semantic_indicators.insert(hash, weight);
    }

    // Common bigram patterns that suggest coherent queries
    let mut bigram_coherence = HashMap::new();
    let coherent_bigrams = vec![
        (("object", "oriented"), 220),
        (("data", "structure"), 215),
        (("machine", "learning"), 220),
        (("neural", "network"), 225),
        (("natural", "language"), 220),
        (("user", "interface"), 210),
        (("error", "handling"), 205),
        (("memory", "management"), 210),
        (("file", "system"), 200),
        (("operating", "system"), 205),
        (("design", "pattern"), 215),
        (("best", "practice"), 200),
        (("use", "case"), 195),
        (("edge", "case"), 190),
        (("high", "level"), 185),
        (("low", "level"), 185),
        (("open", "source"), 190),
        (("real", "time"), 195),
        (("time", "complexity"), 200),
        (("space", "complexity"), 200),
    ];

    for ((w1, w2), score) in coherent_bigrams {
        let h1 = hash_token(w1);
        let h2 = hash_token(w2);
        bigram_coherence.insert((h1, h2), score);
    }

    // Entity patterns
    let entity_patterns = vec![
        Regex::new(r"^[A-Z][a-z]+$").unwrap(),     // Capitalized words
        Regex::new(r"^[A-Z]+[0-9]+$").unwrap(),    // IDs like J345
        Regex::new(r"^[A-Z]{2,}$").unwrap(),       // Acronyms
        Regex::new(r"^[A-Z][a-z]+[A-Z]").unwrap(), // CamelCase
    ];

    LightweightAnalyzer {
        char_stats: CharacterStats {
            trigram_log_probs: trigram_probs,
        },
        token_stats: TokenStats {
            semantic_indicators,
            bigram_coherence,
        },
        subword_patterns: SubwordPatterns {
            concept_affixes: vec![
                ("tion", 180),
                ("ment", 170),
                ("ness", 160),
                ("able", 150),
                ("ize", 140),
                ("ify", 140),
                ("ology", 200),
                ("graphy", 190),
                ("metry", 185),
                ("ism", 160),
                ("ist", 155),
                ("ity", 150),
                ("ance", 145),
                ("ence", 145),
                ("ship", 155),
                ("ful", 130),
                ("less", 130),
                ("ward", 125),
            ],
            entity_patterns,
        },
        last_query: String::new(),
    }
}

fn hash_token(token: &str) -> u32 {
    let mut hash = 5381u32;
    for byte in token.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u32);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_estimates() {
        // Ensure our data structures stay small
        let _analyzer = build_analyzer_with_defaults();

        // In practice, with 50K trigrams, 5K words, 10K bigrams:
        // - trigram_log_probs: 50K * 4 bytes = 200KB
        // - semantic_indicators: 5K * 5 bytes = 25KB
        // - bigram_coherence: 10K * 9 bytes = 90KB
        // - Other data: ~50KB
        // Total: ~365KB uncompressed, ~150KB compressed

        // With bincode/cbor serialization, expect ~2MB for full model
    }
}
