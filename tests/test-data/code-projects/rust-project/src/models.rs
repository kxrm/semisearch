use std::path::PathBuf;

/// A search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The path to the file containing the match
    pub path: PathBuf,
    
    /// The line number where the match was found (1-based)
    pub line_number: usize,
    
    /// The content of the line containing the match
    pub content: String,
    
    /// The relevance score of the match (0.0 to 1.0)
    pub score: f64,
}

/// The type of match
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    /// Exact match
    Exact,
    
    /// Fuzzy match
    Fuzzy,
    
    /// Semantic match
    Semantic,
    
    /// Regex match
    Regex,
}

/// A search query
#[derive(Debug, Clone)]
pub struct Query {
    /// The raw query string
    pub raw: String,
    
    /// The normalized query string
    pub normalized: String,
    
    /// The type of query
    pub query_type: QueryType,
}

/// The type of query
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    /// A simple text query
    Text,
    
    /// A regex query
    Regex,
    
    /// A semantic query
    Semantic,
}

impl Query {
    /// Create a new query
    pub fn new(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let normalized = raw.to_lowercase();
        
        let query_type = if raw.starts_with('/') && raw.ends_with('/') {
            QueryType::Regex
        } else if raw.contains(' ') && raw.split_whitespace().count() > 3 {
            QueryType::Semantic
        } else {
            QueryType::Text
        };
        
        Self {
            raw,
            normalized,
            query_type,
        }
    }
} 