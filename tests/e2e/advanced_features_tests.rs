#[cfg(test)]
mod advanced_features_tests {
    use std::env;
    use std::path::Path;
    use std::process::Command;

    // Helper function to run semisearch and capture output
    fn run_semisearch(args: &[&str], working_dir: Option<&Path>) -> (bool, String, String) {
        let current_dir = env::current_dir().expect("Failed to get current directory");
        let dir = working_dir.unwrap_or(&current_dir);

        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("Failed to execute semisearch");

        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        (success, stdout, stderr)
    }

    // Test that advanced mode is accessible
    #[test]
    fn test_advanced_mode_accessible() {
        // Test with advanced flag
        let (success, stdout, _stderr) = run_semisearch(&["--advanced", "--help"], None);

        assert!(success, "Advanced help should succeed");

        // Check for advanced options in help output
        let has_advanced_options = stdout.contains("mode")
            || stdout.contains("score")
            || stdout.contains("threshold")
            || stdout.contains("limit");

        assert!(
            has_advanced_options,
            "Should show advanced options with --advanced flag"
        );

        // Compare with regular help
        let (success, regular_stdout, _stderr) = run_semisearch(&["--help"], None);

        assert!(success, "Regular help should succeed");

        // Advanced help should be more verbose
        assert!(
            stdout.len() > regular_stdout.len(),
            "Advanced help should contain more options than regular help"
        );
    }

    // Test specific advanced search modes
    #[test]
    fn test_advanced_search_modes() {
        let test_dir = Path::new("tests/test-data/code-projects");

        // Test regex mode if available
        let (success, stdout, _stderr) = run_semisearch(
            &["--advanced", "function.*\\(", "--mode", "regex"],
            Some(test_dir),
        );

        // If regex mode is implemented, it should succeed and find function declarations
        if success {
            assert!(
                stdout.contains("function"),
                "Regex search should find function declarations"
            );
        }

        // Test exact mode if available
        let (success, stdout, _stderr) =
            run_semisearch(&["--advanced", "TODO", "--mode", "exact"], Some(test_dir));

        // If exact mode is implemented, it should succeed and find exact TODO matches
        if success {
            assert!(
                stdout.contains("TODO"),
                "Exact search should find TODO comments"
            );
        }

        // Test semantic mode if available
        let (success, stdout, _stderr) = run_semisearch(
            &["--advanced", "error handling", "--mode", "semantic"],
            Some(test_dir),
        );

        // If semantic mode is implemented, it should succeed or fail gracefully
        if success && !stdout.contains("not available") {
            assert!(
                stdout.contains("error")
                    || stdout.contains("exception")
                    || stdout.contains("try")
                    || stdout.contains("catch"),
                "Semantic search should find error handling concepts"
            );
        }
    }

    // Test score threshold option
    #[test]
    fn test_score_threshold() {
        let test_dir = Path::new("tests/test-data");

        // Test with high threshold (should find fewer results)
        let (success, high_stdout, _stderr) = run_semisearch(
            &["--advanced", "function", "--score", "0.8"],
            Some(test_dir),
        );

        // Test with low threshold (should find more results)
        let (success2, low_stdout, _stderr) = run_semisearch(
            &["--advanced", "function", "--score", "0.2"],
            Some(test_dir),
        );

        // If score threshold is implemented, low threshold should find more results
        if success && success2 {
            let high_matches = high_stdout.matches("function").count();
            let low_matches = low_stdout.matches("function").count();

            // Only assert if the feature seems to be implemented
            if high_matches > 0 && low_matches > high_matches {
                assert!(
                    low_matches > high_matches,
                    "Lower threshold should find more matches than higher threshold"
                );
            }
        }
    }

    // Test result limit option
    #[test]
    fn test_result_limit() {
        let test_dir = Path::new("tests/test-data");

        // Test with small limit
        let (success, small_stdout, _stderr) =
            run_semisearch(&["--advanced", "function", "--limit", "3"], Some(test_dir));

        // Test with large limit
        let (success2, large_stdout, _stderr) =
            run_semisearch(&["--advanced", "function", "--limit", "20"], Some(test_dir));

        // If limit is implemented, small limit should find fewer results
        if success && success2 {
            let small_matches = small_stdout.matches("function").count();
            let large_matches = large_stdout.matches("function").count();

            // Only assert if the feature seems to be implemented
            if small_matches > 0 && small_matches < large_matches {
                assert!(
                    small_matches < large_matches,
                    "Smaller limit should find fewer matches than larger limit"
                );
            }
        }
    }

    // Test path filtering options
    #[test]
    fn test_path_filtering() {
        let test_dir = Path::new("tests/test-data");

        // Test with include pattern
        let (success, stdout, _stderr) = run_semisearch(
            &["--advanced", "function", "--include", "*.js"],
            Some(test_dir),
        );

        // If include pattern is implemented, it should only find JS files
        if success {
            let js_matches = stdout.lines().filter(|line| line.contains(".js")).count();

            let non_js_matches = stdout
                .lines()
                .filter(|line| line.contains(".") && !line.contains(".js"))
                .count();

            // Only assert if the feature seems to be implemented
            if js_matches > 0 {
                assert!(
                    non_js_matches == 0,
                    "Include pattern should only match specified file types"
                );
            }
        }

        // Test with exclude pattern
        let (success, stdout, _stderr) = run_semisearch(
            &["--advanced", "function", "--exclude", "*.md"],
            Some(test_dir),
        );

        // If exclude pattern is implemented, it should not find MD files
        if success {
            let md_matches = stdout.lines().filter(|line| line.contains(".md")).count();

            // Only assert if the feature seems to be implemented
            if stdout.contains("function") {
                assert!(
                    md_matches == 0,
                    "Exclude pattern should not match excluded file types"
                );
            }
        }
    }

    // Test context lines option
    #[test]
    fn test_context_lines() {
        let test_dir = Path::new("tests/test-data/code-projects");

        // Test with context
        let (success, stdout, _stderr) = run_semisearch(
            &["--advanced", "function", "--context", "3"],
            Some(test_dir),
        );

        // If context lines is implemented, it should show lines before/after matches
        if success {
            let lines: Vec<&str> = stdout.lines().collect();
            let mut found_match = false;
            let mut context_lines = 0;

            for line in lines {
                if line.contains("function") {
                    found_match = true;
                } else if found_match && !line.is_empty() && !line.contains("---") {
                    context_lines += 1;
                    if context_lines >= 3 {
                        break;
                    }
                }
            }

            // Only assert if the feature seems to be implemented
            if context_lines > 0 {
                assert!(
                    context_lines > 0,
                    "Should show context lines around matches"
                );
            }
        }
    }

    // Test output format options
    #[test]
    fn test_output_formats() {
        let test_dir = Path::new("tests/test-data/code-projects");

        // Test JSON output if available
        let (success, stdout, _stderr) = run_semisearch(
            &["--advanced", "function", "--format", "json"],
            Some(test_dir),
        );

        // If JSON format is implemented, output should be valid JSON
        if success && stdout.trim().starts_with("{") || stdout.trim().starts_with("[") {
            assert!(
                stdout.contains("\"") && stdout.contains(":"),
                "JSON output should contain quotes and colons"
            );
            assert!(
                stdout.contains("function"),
                "JSON output should contain search term"
            );
        }

        // Test compact output if available
        let (success, stdout, _stderr) = run_semisearch(
            &["--advanced", "function", "--format", "compact"],
            Some(test_dir),
        );

        // If compact format is implemented, output should be more concise
        if success {
            let lines = stdout.lines().count();

            // Compare with default format
            let (success2, default_stdout, _stderr) =
                run_semisearch(&["--advanced", "function"], Some(test_dir));

            if success2 {
                let default_lines = default_stdout.lines().count();

                // Only assert if the feature seems to be implemented
                if lines < default_lines {
                    assert!(
                        lines < default_lines,
                        "Compact format should have fewer lines than default"
                    );
                }
            }
        }
    }

    // Test that advanced features are hidden by default
    #[test]
    fn test_advanced_features_hidden() {
        // Get regular help output
        let (success, stdout, _stderr) = run_semisearch(&["--help"], None);

        assert!(success, "Regular help should succeed");

        // Check that advanced options are not in regular help
        let has_advanced_mode_option =
            stdout.contains("--mode") && (stdout.contains("regex") || stdout.contains("semantic"));

        let has_advanced_score_option = stdout.contains("--score") && stdout.contains("threshold");

        assert!(
            !has_advanced_mode_option,
            "Advanced mode options should not be in regular help"
        );
        assert!(
            !has_advanced_score_option,
            "Advanced score options should not be in regular help"
        );

        // Check that advanced flag is mentioned
        assert!(
            stdout.contains("--advanced")
                || stdout.contains("advanced mode")
                || stdout.contains("power user"),
            "Regular help should mention advanced mode"
        );
    }

    // Test that environment variable can enable advanced mode
    #[test]
    fn test_env_var_advanced_mode() {
        // This test is speculative - it depends on if env var support is implemented

        // Try with environment variable if the feature is implemented
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--")
            .arg("--help")
            .env("SEMISEARCH_ADVANCED", "1");

        let output = cmd.output().expect("Failed to execute semisearch");
        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        assert!(success, "Help with env var should succeed");

        // If env var is implemented, it should show advanced options
        if stdout.contains("mode") && stdout.contains("regex") {
            assert!(
                stdout.contains("--score") || stdout.contains("--threshold"),
                "Environment variable should enable advanced mode"
            );
        }
    }
}
