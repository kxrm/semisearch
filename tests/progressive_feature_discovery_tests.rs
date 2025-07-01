use tempfile::TempDir;

/// Tests for Progressive Feature Discovery (UX Remediation Plan Task 3.3.2)
/// Ensures novice users naturally learn about advanced features
#[cfg(test)]
mod progressive_feature_discovery_tests {
    use super::*;
    use search::core::patterns::{utils, QueryPattern};
    use search::user::feature_discovery::FeatureDiscovery;
    use search::user::usage_tracker::{UsageStats, UsageTracker};

    /// Test: New users get basic, encouraging tips
    #[test]
    fn test_new_user_gets_basic_tips() {
        let stats = UsageStats {
            total_searches: 2,
            advanced_mode_used: false,
            fuzzy_mode_used: false,
            recent_queries: vec!["TODO".to_string(), "function".to_string()],
            complex_queries: vec![],
        };

        let suggestion = FeatureDiscovery::suggest_next_step(&stats, "TODO", 5);

        // New users should get encouraging, basic tips
        assert!(suggestion.is_some());
        let tip = suggestion.unwrap();
        assert!(
            tip.contains("Great start") || tip.contains("Keep exploring") || tip.contains("💡"),
            "New users should get encouraging tips: {tip}"
        );
        assert!(
            !tip.contains("--advanced") && !tip.contains("regex"),
            "New users shouldn't see advanced features: {tip}"
        );
    }

    /// Test: Experienced users get advanced feature suggestions
    #[test]
    fn test_experienced_user_gets_advanced_suggestions() {
        let stats = UsageStats {
            total_searches: 15,
            advanced_mode_used: false,
            fuzzy_mode_used: true,
            recent_queries: vec![
                "TODO".to_string(),
                "function login".to_string(),
                "error handling".to_string(),
            ],
            complex_queries: vec!["TODO.*Fix".to_string()],
        };

        let suggestion = FeatureDiscovery::suggest_next_step(&stats, "TODO", 3);

        // Experienced users should learn about advanced features
        assert!(suggestion.is_some());
        let tip = suggestion.unwrap();
        assert!(
            tip.contains("--advanced") || tip.contains("power") || tip.contains("more options"),
            "Experienced users should learn about advanced features: {tip}"
        );
    }

    /// Test: Complex queries trigger appropriate hints
    #[test]
    fn test_complex_query_triggers_regex_hint() {
        let stats = UsageStats {
            total_searches: 8,
            advanced_mode_used: false,
            fuzzy_mode_used: false,
            recent_queries: vec!["TODO.*Fix".to_string()],
            complex_queries: vec!["TODO.*Fix".to_string()],
        };

        let suggestion = FeatureDiscovery::suggest_next_step(&stats, "TODO.*Fix", 0);

        // Complex queries should suggest regex features
        assert!(suggestion.is_some());
        let tip = suggestion.unwrap();
        assert!(
            tip.contains("regex") || tip.contains("pattern") || tip.contains("--advanced"),
            "Complex queries should suggest regex features: {tip}"
        );
    }

    /// Test: Users who haven't used fuzzy get typo suggestions
    #[test]
    fn test_suggests_fuzzy_for_potential_typos() {
        let stats = UsageStats {
            total_searches: 6,
            advanced_mode_used: false,
            fuzzy_mode_used: false,
            recent_queries: vec!["databse".to_string(), "functoin".to_string()],
            complex_queries: vec![],
        };

        let suggestion = FeatureDiscovery::suggest_next_step(&stats, "databse", 0);

        // Should suggest fuzzy search for potential typos
        assert!(suggestion.is_some());
        let tip = suggestion.unwrap();
        assert!(
            tip.contains("--fuzzy") || tip.contains("typo") || tip.contains("spelling"),
            "Should suggest fuzzy search for typos: {tip}"
        );
    }

    /// Test: Progressive disclosure - no overwhelming suggestions
    #[test]
    fn test_no_overwhelming_suggestions() {
        let stats = UsageStats {
            total_searches: 3,
            advanced_mode_used: false,
            fuzzy_mode_used: false,
            recent_queries: vec!["TODO".to_string()],
            complex_queries: vec![],
        };

        let suggestion = FeatureDiscovery::suggest_next_step(&stats, "TODO", 10);

        // Early users shouldn't get overwhelming suggestions
        if let Some(tip) = suggestion {
            assert!(
                !tip.contains("--advanced") && !tip.contains("regex") && !tip.contains("complex"),
                "Early users shouldn't get overwhelming suggestions: {tip}"
            );
        }
    }

    /// Test: Usage tracking works correctly
    #[test]
    fn test_usage_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let usage_file = temp_dir.path().join("usage.json");

        let mut tracker = UsageTracker::new(usage_file.clone());

        // Track some searches
        tracker.record_search("TODO", false, false, 5);
        tracker.record_search("function.*login", false, false, 0);
        tracker.record_search("error", true, false, 3);

        // Save and reload
        tracker.save().unwrap();
        let reloaded_tracker = UsageTracker::load(usage_file).unwrap();
        let stats = reloaded_tracker.get_stats();

        assert_eq!(stats.total_searches, 3);
        assert!(stats.fuzzy_mode_used);
        assert!(!stats.advanced_mode_used);
        assert_eq!(stats.recent_queries.len(), 3);
        assert_eq!(stats.complex_queries.len(), 1);
        assert!(stats
            .complex_queries
            .contains(&"function.*login".to_string()));
    }

    /// Test: Query pattern detection
    #[test]
    fn test_query_pattern_detection() {
        assert_eq!(utils::analyze_query_pattern("TODO"), QueryPattern::Simple);
        assert_eq!(
            utils::analyze_query_pattern("TODO.*Fix"),
            QueryPattern::RegexLike
        );
        assert_eq!(
            utils::analyze_query_pattern("databse"),
            QueryPattern::PotentialTypo
        );
        assert_eq!(
            utils::analyze_query_pattern("error handling patterns in authentication"),
            QueryPattern::Conceptual
        );
        assert_eq!(
            utils::analyze_query_pattern("TODO .py files"),
            QueryPattern::FileFiltering
        );
    }

    /// Test: Contextual tips based on search results
    #[test]
    fn test_contextual_tips_for_results() {
        let stats = UsageStats {
            total_searches: 7,
            advanced_mode_used: false,
            fuzzy_mode_used: false,
            recent_queries: vec!["TODO".to_string()],
            complex_queries: vec![],
        };

        // Many results should suggest filtering
        let suggestion = FeatureDiscovery::suggest_next_step(&stats, "TODO", 50);
        assert!(suggestion.is_some());
        let tip = suggestion.unwrap();
        assert!(
            tip.contains("specific") || tip.contains("filter") || tip.contains("narrow"),
            "Many results should suggest filtering: {tip}"
        );

        // Few results should suggest different approaches
        let suggestion = FeatureDiscovery::suggest_next_step(&stats, "very_specific_term", 1);
        if let Some(tip) = suggestion {
            assert!(
                tip.contains("broader") || tip.contains("simpler") || tip.contains("different"),
                "Few results should suggest broader search: {tip}"
            );
        }
    }

    /// Test: Learning progression over time
    #[test]
    fn test_learning_progression() {
        // Beginner (1-3 searches)
        let beginner_stats = UsageStats {
            total_searches: 2,
            advanced_mode_used: false,
            fuzzy_mode_used: false,
            recent_queries: vec!["TODO".to_string()],
            complex_queries: vec![],
        };

        let beginner_tip = FeatureDiscovery::suggest_next_step(&beginner_stats, "TODO", 5);
        if let Some(tip) = beginner_tip {
            assert!(
                tip.contains("Great") || tip.contains("start") || tip.contains("exploring"),
                "Beginners should get encouraging tips: {tip}"
            );
        }

        // Intermediate (5-10 searches)
        let intermediate_stats = UsageStats {
            total_searches: 7,
            advanced_mode_used: false,
            fuzzy_mode_used: false,
            recent_queries: vec!["TODO".to_string(), "function".to_string()],
            complex_queries: vec![],
        };

        let intermediate_tip =
            FeatureDiscovery::suggest_next_step(&intermediate_stats, "databse", 0);
        if let Some(tip) = intermediate_tip {
            assert!(
                tip.contains("--fuzzy") || tip.contains("typo"),
                "Intermediate users should learn about fuzzy search: {tip}"
            );
        }

        // Advanced (10+ searches)
        let advanced_stats = UsageStats {
            total_searches: 15,
            advanced_mode_used: false,
            fuzzy_mode_used: true,
            recent_queries: vec!["TODO".to_string(), "function".to_string()],
            complex_queries: vec!["TODO.*Fix".to_string()],
        };

        let advanced_tip = FeatureDiscovery::suggest_next_step(&advanced_stats, "TODO", 5);
        if let Some(tip) = advanced_tip {
            assert!(
                tip.contains("--advanced") || tip.contains("power") || tip.contains("more options"),
                "Advanced users should learn about advanced features: {tip}"
            );
        }
    }
}
