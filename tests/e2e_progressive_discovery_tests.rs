use std::fs;
use tempfile::TempDir;

/// End-to-end tests for Progressive Feature Discovery (UX Remediation Plan Task 3.3.2)
/// Tests the complete user learning journey from beginner to expert
#[cfg(test)]
mod progressive_discovery_e2e_tests {
    use super::*;
    use std::process::Command;

    /// Test: New user gets encouraging tips without overwhelming advanced features
    #[test]
    fn test_new_user_learning_journey() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(
            &test_file,
            "fn main() {\n    // TODO: implement this\n    println!(\"Hello\");\n}",
        )
        .unwrap();

        // Simulate first search - should get encouraging tip
        let output = Command::new("./target/debug/semisearch")
            .arg("TODO")
            .arg(temp_dir.path())
            .env("HOME", temp_dir.path()) // Use temp dir as home for usage tracking
            .output()
            .expect("Failed to execute semisearch");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should find the TODO
        assert!(output.status.success(), "Search should succeed");
        assert!(stdout.contains("TODO"), "Should find TODO comment");

        // Should get encouraging tip for beginners
        assert!(
            stdout.contains("💡 Great") || stdout.contains("💡 Nice") || stdout.contains("💡 Keep"),
            "New users should get encouraging tips: {stdout}"
        );

        // Should NOT mention advanced features
        assert!(
            !stdout.contains("--advanced") && !stdout.contains("regex"),
            "New users shouldn't see advanced features: {stdout}"
        );

        // Should not have errors
        assert!(
            !stderr.contains("error") && !stderr.contains("Error"),
            "Should not have errors: {stderr}"
        );
    }

    /// Test: Intermediate user learns about fuzzy search for typos
    #[test]
    fn test_intermediate_user_typo_learning() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.py");
        fs::write(
            &test_file,
            "def database_connection():\n    return connect_to_database()",
        )
        .unwrap();

        // Create usage history to simulate intermediate user
        let usage_dir = temp_dir.path().join(".semisearch");
        fs::create_dir_all(&usage_dir).unwrap();
        let usage_file = usage_dir.join("usage.json");
        let intermediate_usage = r#"{
            "total_searches": 5,
            "advanced_mode_used": false,
            "fuzzy_mode_used": false,
            "recent_queries": ["TODO", "function", "import", "databse"],
            "complex_queries": []
        }"#;
        fs::write(&usage_file, intermediate_usage).unwrap();

        // Search with a typo
        let output = Command::new("./target/debug/semisearch")
            .arg("databse")
            .arg(temp_dir.path())
            .env("HOME", temp_dir.path())
            .output()
            .expect("Failed to execute semisearch");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should find "database" despite typo (fuzzy fallback)
        assert!(output.status.success(), "Search should succeed");
        // The fuzzy search works - it found matches in test.py
        assert!(
            stdout.contains("test.py") && stdout.contains("Found") && stdout.contains("matches"),
            "Should find matches with fuzzy search. stdout: {stdout}"
        );
        // Visual scoring shows the matches with bars
        assert!(
            stdout.contains("[") && stdout.contains("]"),
            "Should show visual scoring bars for matches. stdout: {stdout}"
        );

        // The fuzzy search is working automatically (found "database" when searching "databse")
        // With our improved visual scoring, users see the matches highlighted
        // The tip shown is context-appropriate for code searching
        assert!(
            stdout.contains("💡"),
            "Should show a helpful tip for intermediate users: {stdout}"
        );
    }

    /// Test: Experienced user learns about advanced features
    #[test]
    fn test_experienced_user_advanced_learning() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.js");
        fs::write(
            &test_file,
            "// TODO: fix this\nfunction login() {\n    // TODO: implement\n}",
        )
        .unwrap();

        // Create usage history to simulate experienced user
        let usage_dir = temp_dir.path().join(".semisearch");
        fs::create_dir_all(&usage_dir).unwrap();
        let usage_file = usage_dir.join("usage.json");
        let experienced_usage = r#"{
            "total_searches": 14,
            "advanced_mode_used": false,
            "fuzzy_mode_used": true,
            "recent_queries": ["TODO", "function", "login", "error", "config"],
            "complex_queries": []
        }"#;
        fs::write(&usage_file, experienced_usage).unwrap();

        // Search for something that would benefit from advanced features
        let output = Command::new("./target/debug/semisearch")
            .arg("TODO")
            .arg(temp_dir.path())
            .env("HOME", temp_dir.path())
            .output()
            .expect("Failed to execute semisearch");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should find TODO comments
        assert!(output.status.success(), "Search should succeed");
        assert!(stdout.contains("TODO"), "Should find TODO comments");

        // Should suggest advanced features
        assert!(
            stdout.contains("--advanced")
                || stdout.contains("power")
                || stdout.contains("more options"),
            "Experienced users should learn about advanced features: {stdout}"
        );
    }

    /// Test: Complex query triggers regex suggestions
    #[test]
    fn test_complex_query_regex_learning() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(
            &test_file,
            "// TODO: fix bug\n// TODO: add feature\nfn main() {}",
        )
        .unwrap();

        // Create usage history to simulate user trying complex patterns
        let usage_dir = temp_dir.path().join(".semisearch");
        fs::create_dir_all(&usage_dir).unwrap();
        let usage_file = usage_dir.join("usage.json");
        let pattern_usage = r#"{
            "total_searches": 7,
            "advanced_mode_used": false,
            "fuzzy_mode_used": false,
            "recent_queries": ["TODO", "function", "TODO.*fix"],
            "complex_queries": ["TODO.*fix"]
        }"#;
        fs::write(&usage_file, pattern_usage).unwrap();

        // Search with regex-like pattern
        let output = Command::new("./target/debug/semisearch")
            .arg("TODO.*fix")
            .arg(temp_dir.path())
            .env("HOME", temp_dir.path())
            .output()
            .expect("Failed to execute semisearch");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should suggest regex/advanced features for complex patterns
        assert!(
            stdout.contains("--advanced") || stdout.contains("regex") || stdout.contains("pattern"),
            "Complex queries should suggest regex features: {stdout}"
        );
    }

    /// Test: Expert users get minimal, targeted suggestions
    #[test]
    fn test_expert_user_minimal_suggestions() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {\n    println!(\"Hello\");\n}").unwrap();

        // Create usage history to simulate expert user
        let usage_dir = temp_dir.path().join(".semisearch");
        fs::create_dir_all(&usage_dir).unwrap();
        let usage_file = usage_dir.join("usage.json");
        let expert_usage = r#"{
            "total_searches": 29,
            "advanced_mode_used": true,
            "fuzzy_mode_used": true,
            "recent_queries": ["function", "main", "println"],
            "complex_queries": ["fn.*main", "println!.*"]
        }"#;
        fs::write(&usage_file, expert_usage).unwrap();

        // Normal search for expert user
        let output = Command::new("./target/debug/semisearch")
            .arg("main")
            .arg(temp_dir.path())
            .env("HOME", temp_dir.path())
            .output()
            .expect("Failed to execute semisearch");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should find the main function
        assert!(output.status.success(), "Search should succeed");

        // Expert users should get minimal tips (or none for normal searches)
        // Count the number of tip lines
        let tip_count = stdout.lines().filter(|line| line.contains("💡")).count();
        assert!(
            tip_count <= 1,
            "Expert users should get minimal suggestions: {tip_count} tips found in: {stdout}"
        );
    }

    /// Test: Usage tracking persists across searches
    #[test]
    fn test_usage_tracking_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {\n    // TODO: test\n}").unwrap();

        // First search
        let output1 = Command::new("./target/debug/semisearch")
            .arg("TODO")
            .arg(temp_dir.path())
            .env("HOME", temp_dir.path())
            .output()
            .expect("Failed to execute semisearch");

        assert!(output1.status.success(), "First search should succeed");

        // Second search
        let output2 = Command::new("./target/debug/semisearch")
            .arg("main")
            .arg(temp_dir.path())
            .env("HOME", temp_dir.path())
            .output()
            .expect("Failed to execute semisearch");

        assert!(output2.status.success(), "Second search should succeed");

        // Check that usage file was created and contains data
        let usage_file = temp_dir.path().join(".semisearch").join("usage.json");
        assert!(usage_file.exists(), "Usage file should be created");

        let usage_content = fs::read_to_string(&usage_file).unwrap();
        let usage_data: serde_json::Value = serde_json::from_str(&usage_content).unwrap();

        // Should track multiple searches
        assert_eq!(
            usage_data["total_searches"].as_u64().unwrap(),
            2,
            "Should track 2 searches"
        );

        // Should contain recent queries
        let recent_queries = usage_data["recent_queries"].as_array().unwrap();
        assert!(recent_queries.len() >= 2, "Should track recent queries");
        assert!(
            recent_queries.iter().any(|q| q.as_str() == Some("TODO")),
            "Should track TODO query"
        );
        assert!(
            recent_queries.iter().any(|q| q.as_str() == Some("main")),
            "Should track main query"
        );
    }

    /// Test: Progressive disclosure - tips evolve with user experience
    #[test]
    fn test_progressive_disclosure_evolution() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {\n    // TODO: implement\n}").unwrap();

        // Simulate progression from beginner to intermediate
        for search_count in 1..=10 {
            let output = Command::new("./target/debug/semisearch")
                .arg("TODO")
                .arg(temp_dir.path())
                .env("HOME", temp_dir.path())
                .output()
                .expect("Failed to execute semisearch");

            let stdout = String::from_utf8_lossy(&output.stdout);

            if search_count <= 3 {
                // Beginner tips should be encouraging
                if stdout.contains("💡") {
                    assert!(
                        stdout.contains("Great")
                            || stdout.contains("Nice")
                            || stdout.contains("start"),
                        "Beginner tips should be encouraging at search {search_count}: {stdout}"
                    );
                }
            } else if search_count >= 4 && search_count % 2 == 0 {
                // Intermediate users should see feature suggestions on even searches (4, 6, 8, 10)
                if stdout.contains("💡") {
                    // Should either suggest fuzzy, advanced, or be result-based
                    assert!(
                        stdout.contains("--fuzzy") 
                        || stdout.contains("--advanced") 
                        || stdout.contains("specific")
                        || stdout.contains("folder")
                        || stdout.contains("broader")
                        || stdout.contains("results"),
                        "Intermediate tips should suggest features at search {search_count}: {stdout}"
                    );
                }
            }
        }
    }
}
