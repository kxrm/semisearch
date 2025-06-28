use crate::DEFAULT_LIMIT;

/// Configuration for the file processor
#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum number of results to return
    pub limit: usize,
    
    /// Whether to enable verbose output
    pub verbose: bool,
    
    /// Minimum score threshold for results
    pub score_threshold: f64,
    
    /// File extensions to include
    pub include_extensions: Vec<String>,
    
    /// File extensions to exclude
    pub exclude_extensions: Vec<String>,
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self {
            limit: DEFAULT_LIMIT,
            verbose: false,
            score_threshold: 0.1,
            include_extensions: Vec::new(),
            exclude_extensions: vec![
                "exe".to_string(),
                "dll".to_string(),
                "so".to_string(),
                "dylib".to_string(),
                "bin".to_string(),
            ],
        }
    }
    
    /// Set the maximum number of results to return
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
    
    /// Set whether to enable verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    /// Set the minimum score threshold for results
    pub fn with_score_threshold(mut self, threshold: f64) -> Self {
        self.score_threshold = threshold;
        self
    }
    
    /// Add file extensions to include
    pub fn with_include_extensions(mut self, extensions: Vec<String>) -> Self {
        self.include_extensions = extensions;
        self
    }
    
    /// Add file extensions to exclude
    pub fn with_exclude_extensions(mut self, extensions: Vec<String>) -> Self {
        self.exclude_extensions = extensions;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
} 