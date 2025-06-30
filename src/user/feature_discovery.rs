use crate::user::usage_tracker::{QueryPattern, UsageStats, UserExperienceLevel};

/// Generates contextual tips to help users discover advanced features progressively
pub struct FeatureDiscovery;

impl FeatureDiscovery {
    /// Suggest the next step for a user based on their usage patterns and current search
    pub fn suggest_next_step(stats: &UsageStats, current_query: &str, result_count: usize) -> Option<String> {
        let experience_level = Self::determine_experience_level(stats);
        let query_pattern = QueryPattern::analyze(current_query);

        // Progressive disclosure based on experience level
        match experience_level {
            UserExperienceLevel::Beginner => Self::suggest_for_beginner(stats, &query_pattern, result_count),
            UserExperienceLevel::Intermediate => Self::suggest_for_intermediate(stats, &query_pattern, result_count),
            UserExperienceLevel::Experienced => Self::suggest_for_experienced(stats, &query_pattern, result_count),
            UserExperienceLevel::Expert => Self::suggest_for_expert(stats, &query_pattern, result_count),
        }
    }

    /// Determine user experience level from usage stats
    fn determine_experience_level(stats: &UsageStats) -> UserExperienceLevel {
        match stats.total_searches {
            0..=3 => UserExperienceLevel::Beginner,
            4..=10 => UserExperienceLevel::Intermediate,
            11..=25 => UserExperienceLevel::Experienced,
            _ => UserExperienceLevel::Expert,
        }
    }

    /// Suggestions for beginner users (0-3 searches)
    fn suggest_for_beginner(stats: &UsageStats, _pattern: &QueryPattern, result_count: usize) -> Option<String> {
        match result_count {
            0 => Some("💡 No matches found? Try simpler terms or check spelling.".to_string()),
            1..=5 => {
                if stats.total_searches <= 1 {
                    Some("💡 Great start! Keep exploring your codebase.".to_string())
                } else {
                    Some("💡 Great! You're getting the hang of searching. Keep exploring!".to_string())
                }
            },
            6..=20 => Some("💡 Good results! Try more specific terms to narrow down.".to_string()),
            _ => Some("💡 Many matches found. Try searching in a specific folder or use more specific terms.".to_string()),
        }
    }

    /// Suggestions for intermediate users (4-10 searches)
    fn suggest_for_intermediate(stats: &UsageStats, pattern: &QueryPattern, result_count: usize) -> Option<String> {
        // Introduce fuzzy search for typos
        if matches!(pattern, QueryPattern::PotentialTypo) && !stats.fuzzy_mode_used {
            return Some("💡 Looks like a typo? Try adding --fuzzy to handle spelling variations.".to_string());
        }

        // Introduce advanced mode for complex patterns (intermediate users can learn about regex)
        if matches!(pattern, QueryPattern::RegexLike) && !stats.advanced_mode_used {
            return Some("💡 Complex pattern detected! Try --advanced for regex and more powerful search options.".to_string());
        }

        // Suggest file filtering for specific patterns
        if matches!(pattern, QueryPattern::FileFiltering) {
            return Some("💡 Searching specific file types? Try: semisearch \"your term\" --include \"*.py\"".to_string());
        }

        // Result-based suggestions
        match result_count {
            0 => Some("💡 No results? Try --fuzzy for typos or broader search terms.".to_string()),
            1..=3 => Some("💡 Few results. Try broader terms or search in parent directories.".to_string()),
            20..=50 => Some("💡 Many results. Try more specific terms or search in specific folders.".to_string()),
            _ => Some("💡 Too many results. Be more specific or search in a particular directory.".to_string()),
        }
    }

    /// Suggestions for experienced users (11-25 searches)
    fn suggest_for_experienced(stats: &UsageStats, pattern: &QueryPattern, result_count: usize) -> Option<String> {
        // Introduce advanced mode for complex patterns
        if matches!(pattern, QueryPattern::RegexLike) && !stats.advanced_mode_used {
            return Some("💡 Complex pattern detected! Try --advanced for regex and more powerful search options.".to_string());
        }

        // Suggest advanced features for power users (lowered threshold to 11 for experienced users)
        if stats.total_searches >= 11 && !stats.advanced_mode_used {
            return Some("💡 You're using semisearch a lot! Try --advanced for more powerful search options.".to_string());
        }

        // Advanced filtering suggestions
        match result_count {
            0 => Some("💡 No results? Try --advanced for regex patterns or --fuzzy for typos.".to_string()),
            50..=100 => Some("💡 Many results. Try --advanced with --exclude patterns to filter out unwanted files.".to_string()),
            _ => None,
        }
    }

    /// Suggestions for expert users (25+ searches)
    fn suggest_for_expert(_stats: &UsageStats, pattern: &QueryPattern, result_count: usize) -> Option<String> {
        // Only provide suggestions for specific scenarios
        match (pattern, result_count) {
            (QueryPattern::RegexLike, 0) => Some("💡 Complex regex with no results? Check pattern syntax or try simpler alternatives.".to_string()),
            (_, 100..) => Some("💡 Many results. Consider using --exclude patterns or searching in specific directories.".to_string()),
            _ => None, // Expert users don't need constant tips
        }
    }

    /// Generate contextual tips based on search patterns
    pub fn generate_contextual_tip(stats: &UsageStats, recent_patterns: &[QueryPattern]) -> Option<String> {
        // Look for learning opportunities in recent patterns
        let has_typos = recent_patterns.iter().any(|p| matches!(p, QueryPattern::PotentialTypo));
        let has_regex = recent_patterns.iter().any(|p| matches!(p, QueryPattern::RegexLike));
        let has_file_filtering = recent_patterns.iter().any(|p| matches!(p, QueryPattern::FileFiltering));

        match Self::determine_experience_level(stats) {
            UserExperienceLevel::Intermediate => {
                if has_typos && !stats.fuzzy_mode_used {
                    Some("💡 I noticed some typos in your searches. Try --fuzzy to handle spelling variations automatically.".to_string())
                } else if has_file_filtering {
                    Some("💡 Searching specific file types? Use --include \"*.ext\" for better filtering.".to_string())
                } else {
                    None
                }
            },
            UserExperienceLevel::Experienced => {
                if has_regex && !stats.advanced_mode_used {
                    Some("💡 I see you're using complex patterns. Try --advanced for full regex support and more options.".to_string())
                } else if stats.total_searches > 15 && !stats.advanced_mode_used {
                    Some("💡 You're a power user! Try --advanced to unlock regex, filtering, and more advanced features.".to_string())
                } else {
                    None
                }
            },
            _ => None,
        }
    }

    /// Generate tips based on search effectiveness
    pub fn suggest_search_improvement(query: &str, result_count: usize, stats: &UsageStats) -> Option<String> {
        let experience = Self::determine_experience_level(stats);
        let pattern = QueryPattern::analyze(query);

        match (result_count, pattern, experience) {
            // No results - provide helpful suggestions
            (0, QueryPattern::PotentialTypo, _) => {
                Some("💡 No results found. This looks like a typo - try --fuzzy to find similar matches.".to_string())
            },
            (0, QueryPattern::RegexLike, UserExperienceLevel::Experienced) | 
            (0, QueryPattern::RegexLike, UserExperienceLevel::Expert) => {
                Some("💡 No results for this pattern. Try --advanced for full regex support.".to_string())
            },
            (0, _, UserExperienceLevel::Beginner) => {
                Some("💡 No results found. Try simpler terms or check spelling.".to_string())
            },
            (0, _, _) => {
                Some("💡 No results found. Try broader terms, check spelling, or search in parent directories.".to_string())
            },

            // Too many results - suggest filtering
            (50.., _, UserExperienceLevel::Experienced) |
            (50.., _, UserExperienceLevel::Expert) => {
                Some("💡 Many results found. Try --advanced with --exclude patterns or search in specific directories.".to_string())
            },
            (20.., _, _) => {
                Some("💡 Many results found. Use more specific terms or search in a specific folder.".to_string())
            },

            // Good number of results - encourage exploration
            (1..=10, _, UserExperienceLevel::Beginner) => {
                Some("💡 Great! You found some matches. Try different search terms to explore more.".to_string())
            },

            _ => None,
        }
    }

    /// Check if it's appropriate to show a tip (avoid overwhelming users)
    pub fn should_show_tip(stats: &UsageStats) -> bool {
        // Show tips more frequently for beginners, less for experts
        match Self::determine_experience_level(stats) {
            UserExperienceLevel::Beginner => true, // Always show tips for beginners
            UserExperienceLevel::Intermediate => stats.total_searches % 2 == 0, // Every other search
            UserExperienceLevel::Experienced => stats.total_searches % 3 == 0, // Every third search
            UserExperienceLevel::Expert => stats.total_searches % 5 == 0, // Every fifth search
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beginner_suggestions() {
        let stats = UsageStats {
            total_searches: 2,
            advanced_mode_used: false,
            fuzzy_mode_used: false,
            recent_queries: vec!["TODO".to_string()],
            complex_queries: vec![],
        };

        let suggestion = FeatureDiscovery::suggest_next_step(&stats, "TODO", 5);
        assert!(suggestion.is_some());
        let tip = suggestion.unwrap();
        println!("Actual tip: {tip}");
        assert!(tip.contains("Great") || tip.contains("Nice") || tip.contains("start"));
        assert!(!tip.contains("--advanced"));
    }

    #[test]
    fn test_intermediate_typo_suggestion() {
        let stats = UsageStats {
            total_searches: 6,
            advanced_mode_used: false,
            fuzzy_mode_used: false,
            recent_queries: vec!["databse".to_string()],
            complex_queries: vec![],
        };

        let suggestion = FeatureDiscovery::suggest_next_step(&stats, "databse", 0);
        assert!(suggestion.is_some());
        let tip = suggestion.unwrap();
        assert!(tip.contains("--fuzzy") || tip.contains("typo"));
    }

    #[test]
    fn test_experienced_advanced_suggestion() {
        let stats = UsageStats {
            total_searches: 15,
            advanced_mode_used: false,
            fuzzy_mode_used: true,
            recent_queries: vec!["TODO.*Fix".to_string()],
            complex_queries: vec!["TODO.*Fix".to_string()],
        };

        let suggestion = FeatureDiscovery::suggest_next_step(&stats, "TODO.*Fix", 3);
        assert!(suggestion.is_some());
        let tip = suggestion.unwrap();
        assert!(tip.contains("--advanced") || tip.contains("power"));
    }

    #[test]
    fn test_expert_minimal_suggestions() {
        let stats = UsageStats {
            total_searches: 30,
            advanced_mode_used: true,
            fuzzy_mode_used: true,
            recent_queries: vec!["TODO".to_string()],
            complex_queries: vec![],
        };

        let suggestion = FeatureDiscovery::suggest_next_step(&stats, "TODO", 5);
        // Expert users should get minimal suggestions
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_tip_frequency() {
        let beginner_stats = UsageStats {
            total_searches: 2,
            advanced_mode_used: false,
            fuzzy_mode_used: false,
            recent_queries: vec![],
            complex_queries: vec![],
        };

        let expert_stats = UsageStats {
            total_searches: 30,
            advanced_mode_used: true,
            fuzzy_mode_used: true,
            recent_queries: vec![],
            complex_queries: vec![],
        };

        // Beginners should always see tips
        assert!(FeatureDiscovery::should_show_tip(&beginner_stats));

        // Experts should see tips less frequently
        let expert_tip_frequency = (0..10).map(|i| {
            let mut stats = expert_stats.clone();
            stats.total_searches = 30 + i;
            FeatureDiscovery::should_show_tip(&stats)
        }).filter(|&x| x).count();

        assert!(expert_tip_frequency < 5); // Should show tips less than half the time
    }

    #[test]
    fn test_contextual_tips() {
        let stats = UsageStats {
            total_searches: 8,
            advanced_mode_used: false,
            fuzzy_mode_used: false,
            recent_queries: vec!["databse".to_string(), "functoin".to_string()],
            complex_queries: vec![],
        };

        let patterns = vec![QueryPattern::PotentialTypo, QueryPattern::PotentialTypo];
        let tip = FeatureDiscovery::generate_contextual_tip(&stats, &patterns);

        assert!(tip.is_some());
        let tip_text = tip.unwrap();
        assert!(tip_text.contains("--fuzzy") || tip_text.contains("typo"));
    }
} 