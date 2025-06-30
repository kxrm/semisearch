use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Represents different patterns in user queries
#[derive(Debug, Clone, PartialEq)]
pub enum QueryPattern {
    Simple,         // "TODO"
    RegexLike,      // "TODO.*Fix"
    PotentialTypo,  // "databse"
    Conceptual,     // "error handling patterns"
    FileFiltering,  // "TODO .py files"
}

impl QueryPattern {
    /// Analyze a query to determine its pattern
    pub fn analyze(query: &str) -> Self {
        // Check for regex patterns
        if query.contains(".*") || query.contains("\\d") || query.contains("[") || query.contains("(") {
            return Self::RegexLike;
        }

        // Check for file extension filtering
        if query.contains(".py") || query.contains(".rs") || query.contains(".js") || query.contains(".md") {
            return Self::FileFiltering;
        }

        // Check for potential typos (common misspellings)
        let typo_patterns = [
            "databse", "functoin", "recieve", "seperate", "occurance", "accomodate",
            "arguement", "begining", "definately", "existance", "independant",
        ];
        if typo_patterns.iter().any(|&typo| query.contains(typo)) {
            return Self::PotentialTypo;
        }

        // Check for conceptual queries (multi-word, descriptive)
        if query.split_whitespace().count() > 3 {
            return Self::Conceptual;
        }

        Self::Simple
    }
}

/// Statistics about user behavior patterns
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageStats {
    pub total_searches: u32,
    pub advanced_mode_used: bool,
    pub fuzzy_mode_used: bool,
    pub recent_queries: Vec<String>,
    pub complex_queries: Vec<String>,
}

/// Tracks user behavior to enable progressive feature discovery
#[derive(Debug)]
pub struct UsageTracker {
    stats: UsageStats,
    usage_file: PathBuf,
    max_recent_queries: usize,
}

impl UsageTracker {
    /// Create a new usage tracker with the specified file path
    pub fn new(usage_file: PathBuf) -> Self {
        Self {
            stats: UsageStats::default(),
            usage_file,
            max_recent_queries: 10,
        }
    }

    /// Load usage tracker from file, creating new one if file doesn't exist
    pub fn load(usage_file: PathBuf) -> Result<Self> {
        let stats = if usage_file.exists() {
            let content = fs::read_to_string(&usage_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            UsageStats::default()
        };

        Ok(Self {
            stats,
            usage_file,
            max_recent_queries: 10,
        })
    }

    /// Record a search query with context
    pub fn record_search(&mut self, query: &str, fuzzy_used: bool, advanced_used: bool, _result_count: usize) {
        self.stats.total_searches += 1;
        
        if fuzzy_used {
            self.stats.fuzzy_mode_used = true;
        }
        
        if advanced_used {
            self.stats.advanced_mode_used = true;
        }

        // Add to recent queries (keep only the most recent)
        self.stats.recent_queries.push(query.to_string());
        if self.stats.recent_queries.len() > self.max_recent_queries {
            self.stats.recent_queries.remove(0);
        }

        // Track complex queries
        let pattern = QueryPattern::analyze(query);
        if matches!(pattern, QueryPattern::RegexLike | QueryPattern::Conceptual) {
            self.stats.complex_queries.push(query.to_string());
            // Keep only recent complex queries
            if self.stats.complex_queries.len() > 5 {
                self.stats.complex_queries.remove(0);
            }
        }
    }

    /// Get current usage statistics
    pub fn get_stats(&self) -> &UsageStats {
        &self.stats
    }

    /// Save usage statistics to file
    pub fn save(&self) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.usage_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.stats)?;
        fs::write(&self.usage_file, content)?;
        Ok(())
    }

    /// Get the default usage file path
    pub fn default_usage_file() -> Result<PathBuf> {
        // Check environment variable first (for tests and custom setups)
        let home_dir = if let Ok(home_env) = std::env::var("HOME") {
            PathBuf::from(home_env)
        } else {
            dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        };
        
        Ok(home_dir.join(".semisearch").join("usage.json"))
    }

    /// Get user experience level based on usage patterns
    pub fn get_experience_level(&self) -> UserExperienceLevel {
        match self.stats.total_searches {
            0..=3 => UserExperienceLevel::Beginner,
            4..=10 => UserExperienceLevel::Intermediate,
            11..=25 => UserExperienceLevel::Experienced,
            _ => UserExperienceLevel::Expert,
        }
    }

    /// Check if user has used advanced features
    pub fn has_used_advanced_features(&self) -> bool {
        self.stats.advanced_mode_used || self.stats.fuzzy_mode_used
    }

    /// Get recent query patterns
    pub fn get_recent_patterns(&self) -> Vec<QueryPattern> {
        self.stats.recent_queries
            .iter()
            .map(|q| QueryPattern::analyze(q))
            .collect()
    }
}

/// User experience levels for progressive disclosure
#[derive(Debug, Clone, PartialEq)]
pub enum UserExperienceLevel {
    Beginner,     // 0-3 searches
    Intermediate, // 4-10 searches
    Experienced,  // 11-25 searches
    Expert,       // 25+ searches
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_query_pattern_analysis() {
        assert_eq!(QueryPattern::analyze("TODO"), QueryPattern::Simple);
        assert_eq!(QueryPattern::analyze("TODO.*Fix"), QueryPattern::RegexLike);
        assert_eq!(QueryPattern::analyze("databse"), QueryPattern::PotentialTypo);
        assert_eq!(QueryPattern::analyze("error handling patterns in authentication"), QueryPattern::Conceptual);
        assert_eq!(QueryPattern::analyze("TODO .py files"), QueryPattern::FileFiltering);
    }

    #[test]
    fn test_usage_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let usage_file = temp_dir.path().join("usage.json");
        
        let mut tracker = UsageTracker::new(usage_file.clone());
        
        // Record some searches
        tracker.record_search("TODO", false, false, 5);
        tracker.record_search("function.*login", false, true, 2);
        tracker.record_search("databse", true, false, 0);
        
        let stats = tracker.get_stats();
        assert_eq!(stats.total_searches, 3);
        assert!(stats.fuzzy_mode_used);
        assert!(stats.advanced_mode_used);
        assert_eq!(stats.recent_queries.len(), 3);
        assert_eq!(stats.complex_queries.len(), 1);
        
        // Test save and load
        tracker.save().unwrap();
        let loaded_tracker = UsageTracker::load(usage_file).unwrap();
        let loaded_stats = loaded_tracker.get_stats();
        
        assert_eq!(loaded_stats.total_searches, 3);
        assert!(loaded_stats.fuzzy_mode_used);
        assert!(loaded_stats.advanced_mode_used);
    }

    #[test]
    fn test_experience_levels() {
        let temp_dir = TempDir::new().unwrap();
        let usage_file = temp_dir.path().join("usage.json");
        
        let mut tracker = UsageTracker::new(usage_file);
        
        assert_eq!(tracker.get_experience_level(), UserExperienceLevel::Beginner);
        
        // Add some searches
        for i in 0..5 {
            tracker.record_search(&format!("query{i}"), false, false, 1);
        }
        assert_eq!(tracker.get_experience_level(), UserExperienceLevel::Intermediate);
        
        // Add more searches
        for i in 5..15 {
            tracker.record_search(&format!("query{i}"), false, false, 1);
        }
        assert_eq!(tracker.get_experience_level(), UserExperienceLevel::Experienced);
    }

    #[test]
    fn test_recent_queries_limit() {
        let temp_dir = TempDir::new().unwrap();
        let usage_file = temp_dir.path().join("usage.json");
        
        let mut tracker = UsageTracker::new(usage_file);
        
        // Add more than max_recent_queries
        for i in 0..15 {
            tracker.record_search(&format!("query{i}"), false, false, 1);
        }
        
        let stats = tracker.get_stats();
        assert_eq!(stats.recent_queries.len(), 10); // Should be limited to max_recent_queries
        assert_eq!(stats.recent_queries[0], "query5"); // Should have removed older queries
        assert_eq!(stats.recent_queries[9], "query14"); // Should have latest query
    }
} 