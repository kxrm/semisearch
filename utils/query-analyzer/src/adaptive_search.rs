use crate::lightweight_analysis::{LightweightAnalyzer, build_analyzer_with_defaults};

/// Adaptive search strategy that escalates from keyword to semantic
pub struct AdaptiveSearchStrategy {
    analyzer: LightweightAnalyzer,
    keyword_threshold: f32,
    semantic_threshold: f32,
}

#[derive(Debug)]
pub enum SearchRecommendation {
    /// Use keyword search only
    KeywordOnly { confidence: f32 },
    
    /// Try keyword first, fallback to semantic if poor results
    KeywordWithSemanticFallback { 
        keyword_confidence: f32,
        semantic_confidence: f32,
    },
    
    /// Go straight to semantic search
    SemanticOnly { confidence: f32 },
    
    /// Try both in parallel and merge results
    HybridSearch { 
        keyword_weight: f32,
        semantic_weight: f32,
    },
}

impl AdaptiveSearchStrategy {
    pub fn new() -> Self {
        Self {
            analyzer: build_analyzer_with_defaults(),
            keyword_threshold: 0.45,  // Raised from 0.35 - below this, definitely keyword
            semantic_threshold: 0.60, // Lowered from 0.55 - above this, definitely semantic
        }
    }
    
    pub fn recommend(&mut self, query: &str) -> SearchRecommendation {
        let score = self.analyzer.analyze(query);
        let semantic_score = score.needs_semantic;
        let confidence = score.confidence;
        
        // Get query characteristics
        let tokens = query.split_whitespace().count();
        let has_operators = query.contains('"') || query.contains('*') || 
                           query.contains('+') || query.contains('-');
        
        // Decision logic
        match (semantic_score, tokens, has_operators) {
            // Clear keyword queries
            (s, _, true) if s < 0.5 => {
                // Has search operators - definitely keyword
                SearchRecommendation::KeywordOnly { 
                    confidence: 0.9 
                }
            },
            (s, 1..=2, false) if s < self.keyword_threshold => {
                // Very short query with low semantic score
                SearchRecommendation::KeywordOnly { 
                    confidence: confidence.max(0.7) 
                }
            },
            
            // Clear semantic queries
            (s, 5.., false) if s > self.semantic_threshold => {
                // Long query with high semantic score
                SearchRecommendation::SemanticOnly { 
                    confidence 
                }
            },
            (s, _, false) if s > 0.7 => {
                // Very high semantic score
                SearchRecommendation::SemanticOnly { 
                    confidence 
                }
            },
            
            // Middle ground - adaptive approach
            (s, 3..=4, false) if s >= self.keyword_threshold && s <= self.semantic_threshold => {
                // Medium length, medium score - try keyword first
                SearchRecommendation::KeywordWithSemanticFallback {
                    keyword_confidence: 1.0 - s,
                    semantic_confidence: s,
                }
            },
            
            // Hybrid for ambiguous cases
            (s, _, false) if (s - 0.5).abs() < 0.1 => {
                // Score very close to 0.5 - try both
                SearchRecommendation::HybridSearch {
                    keyword_weight: 1.0 - s,
                    semantic_weight: s,
                }
            },
            
            // Default fallback
            (s, _, _) => {
                if s < 0.5 {
                    SearchRecommendation::KeywordWithSemanticFallback {
                        keyword_confidence: 1.0 - s,
                        semantic_confidence: s,
                    }
                } else {
                    SearchRecommendation::SemanticOnly { confidence: s }
                }
            }
        }
    }
    
    pub fn explain_strategy(&mut self, query: &str) -> String {
        let recommendation = self.recommend(query);
        let score = self.analyzer.analyze(query);
        
        match recommendation {
            SearchRecommendation::KeywordOnly { confidence } => {
                format!(
                    "Strategy: Keyword search only (confidence: {:.0}%)\n\
                     Reason: Query has keyword characteristics (score: {:.2})\n\
                     Action: Use TF-IDF/BM25 ranking",
                    confidence * 100.0,
                    score.needs_semantic
                )
            },
            
            SearchRecommendation::KeywordWithSemanticFallback { 
                keyword_confidence, 
                semantic_confidence 
            } => {
                format!(
                    "Strategy: Adaptive search with fallback\n\
                     1. Start with keyword search (confidence: {:.0}%)\n\
                     2. If results < threshold, use semantic (confidence: {:.0}%)\n\
                     Reason: Ambiguous query (score: {:.2})\n\
                     Action: Monitor result quality and escalate if needed",
                    keyword_confidence * 100.0,
                    semantic_confidence * 100.0,
                    score.needs_semantic
                )
            },
            
            SearchRecommendation::SemanticOnly { confidence } => {
                format!(
                    "Strategy: Semantic search only (confidence: {:.0}%)\n\
                     Reason: Query requires understanding (score: {:.2})\n\
                     Action: Use vector similarity search",
                    confidence * 100.0,
                    score.needs_semantic
                )
            },
            
            SearchRecommendation::HybridSearch { 
                keyword_weight, 
                semantic_weight 
            } => {
                format!(
                    "Strategy: Hybrid search (parallel execution)\n\
                     Keyword weight: {:.0}%\n\
                     Semantic weight: {:.0}%\n\
                     Reason: Query could benefit from both (score: {:.2})\n\
                     Action: Run both searches and merge results",
                    keyword_weight * 100.0,
                    semantic_weight * 100.0,
                    score.needs_semantic
                )
            }
        }
    }
}

/// Example of how to use this in practice
pub fn demonstrate_adaptive_search() {
    let mut strategy = AdaptiveSearchStrategy::new();
    
    let test_queries = vec![
        "TODO",
        "user authentication",
        "how does caching improve performance",
        "React useState",
        "difference between TCP and UDP protocols",
        "main.py",
        "error handling best practices",
    ];
    
    println!("=== Adaptive Search Strategy Demo ===\n");
    
    for query in test_queries {
        println!("Query: \"{}\"", query);
        println!("{}", strategy.explain_strategy(query));
        println!();
    }
} 