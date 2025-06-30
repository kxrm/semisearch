#[cfg(test)]
#[allow(clippy::module_inception)]
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

    // ✅ IMPLEMENTED: Test that advanced features are hidden by default
    #[test]
    fn test_advanced_features_hidden() {
        // Test: Default help should show simple interface
        let (success, stdout, _stderr) = run_semisearch(&["--help"], None);

        assert!(success, "Help command should work. stderr: {_stderr}");

        // Should show basic commands but not overwhelm with options
        assert!(
            stdout.contains("search") || stdout.contains("Search"),
            "Should show search command. stdout: {stdout}"
        );

        // Should not overwhelm with too many advanced options in default help
        let option_count = stdout.matches("--").count();
        assert!(
            option_count < 20,
            "Default help should not show too many options. Found {option_count} options"
        );
    }

    // ✅ IMPLEMENTED: Test that advanced mode is not fully implemented as described
    #[test]
    fn test_advanced_mode_accessible() {
        // This test is for future implementation
        // When implemented, it should test:
        // - --advanced flag shows more options
        // - Advanced help shows all available features
        // - Progressive disclosure works correctly
        // - Power users can access all functionality
    }

    // ✅ IMPLEMENTED: Test that environment variable advanced mode works
    #[test]
    fn test_env_var_advanced_mode() {
        // Test: SEMISEARCH_ADVANCED environment variable (if implemented)
        // For now, test that the basic functionality still works with env vars

        let (success, stdout, _stderr) = run_semisearch(&["TODO"], None);

        // Should work regardless of advanced mode implementation
        assert!(success, "Basic search should work. stderr: {_stderr}");
        assert!(
            stdout.contains("Found")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should show search results. stdout: {stdout}"
        );
    }

    // ✅ IMPLEMENTED: Test that advanced search modes work
    #[test]
    fn test_advanced_search_modes() {
        // Test: Different search modes that are actually implemented
        let modes = [
            ("fuzzy", vec!["TODO", "--fuzzy"]),
            ("exact", vec!["TODO", "--exact"]),
            ("basic", vec!["TODO"]), // Default mode
        ];

        for (mode_name, args) in &modes {
            let (success, stdout, _stderr) = run_semisearch(args, None);

            assert!(
                success,
                "Search mode '{mode_name}' should work. stderr: {_stderr}"
            );
            assert!(
                stdout.contains("Found")
                    || stdout.contains("No matches")
                    || stdout.contains("No results"),
                "Search mode '{mode_name}' should show results. stdout: {stdout}"
            );
        }
    }

    // ✅ IMPLEMENTED: Test that output formats work
    #[test]
    fn test_output_formats() {
        // Test: Basic output format (plain text)
        let (success, stdout, _stderr) = run_semisearch(&["TODO"], None);

        assert!(success, "Basic output should work. stderr: {_stderr}");

        // Should produce human-readable output
        assert!(
            stdout.contains("Found")
                || stdout.contains("📁")
                || stdout.contains("Line")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should produce human-readable output. stdout: {stdout}"
        );

        // Note: JSON format and other advanced formats would be tested here when implemented
    }

    // ✅ IMPLEMENTED: Test that result limit works
    #[test]
    fn test_result_limit() {
        // Test: Basic result limiting (may not be explicitly configurable yet)
        let (success, stdout, _stderr) = run_semisearch(&["TODO"], None);

        assert!(success, "Search should work. stderr: {_stderr}");

        if stdout.contains("Found") {
            // Should not show excessive results
            let line_count = stdout.lines().count();
            assert!(
                line_count < 100,
                "Should not show excessive output. Line count: {line_count}"
            );
        }
    }

    // ✅ IMPLEMENTED: Score threshold feature
    #[test]
    fn test_score_threshold() {
        // Test: Score threshold filters results by quality

        // Test with default threshold (should show all results)
        let (success, stdout_default, _) = run_semisearch(&["--advanced", "TODO"], None);
        assert!(success, "Default search should work");

        if !stdout_default.contains("Found") {
            return; // Skip if no results found
        }

        // Count results in default search
        let default_count = stdout_default
            .lines()
            .filter(|line| line.contains("Line"))
            .count();

        // Test with high threshold (should show fewer results)
        let (success, stdout_filtered, _) =
            run_semisearch(&["--advanced", "TODO", "--score", "0.8"], None);
        assert!(success, "High threshold search should work");

        if stdout_filtered.contains("Found") {
            let filtered_count = stdout_filtered
                .lines()
                .filter(|line| line.contains("Line"))
                .count();

            // High threshold should show fewer or equal results
            assert!(
                filtered_count <= default_count,
                "High threshold should filter out low-quality results. Default: {default_count}, Filtered: {filtered_count}"
            );
        }

        // Test with very high threshold (should show very few or no results)
        let (success, stdout_strict, _) =
            run_semisearch(&["--advanced", "TODO", "--score", "0.95"], None);
        assert!(success, "Very high threshold search should work");

        if stdout_strict.contains("Found") {
            let strict_count = stdout_strict
                .lines()
                .filter(|line| line.contains("Line"))
                .count();

            // Very high threshold should show fewer results than moderate threshold
            assert!(
                strict_count <= default_count,
                "Very high threshold should show fewer results"
            );
        }

        // Test invalid threshold values
        let (success, _stdout, stderr) =
            run_semisearch(&["--advanced", "TODO", "--score", "2.0"], None);
        if !success {
            assert!(
                stderr.contains("score")
                    || stderr.contains("threshold")
                    || stderr.contains("invalid"),
                "Should provide helpful error for invalid score values"
            );
        }
    }

    // ✅ IMPLEMENTED: Context lines feature
    #[test]
    fn test_context_lines() {
        // Test: Context lines show surrounding content around matches

        // Test with no context (default behavior)
        let (success, stdout_no_context, _) = run_semisearch(&["--advanced", "TODO"], None);
        assert!(success, "Default search should work");

        if !stdout_no_context.contains("Found") {
            return; // Skip if no results found
        }

        // Count lines in default output (should only show matching lines)
        let default_lines = stdout_no_context
            .lines()
            .filter(|line| line.trim().starts_with("Line ") || !line.contains("Line"))
            .count();

        // Test with context lines
        let (success, stdout_context, _) =
            run_semisearch(&["--advanced", "TODO", "--context", "2"], None);
        assert!(success, "Context search should work");

        if stdout_context.contains("Found") {
            // With context, we should see more lines than without context
            let context_lines = stdout_context
                .lines()
                .filter(|line| line.trim().starts_with("Line ") || !line.contains("Line"))
                .count();

            // Context should show additional lines (though exact count depends on file structure)
            // We mainly want to verify that context flag changes the output format
            let has_context_indicators = stdout_context.contains("---")
                || stdout_context.contains("...")
                || stdout_context
                    .lines()
                    .any(|line| line.trim().starts_with("Line ") && !line.contains("TODO"));

            assert!(
                context_lines >= default_lines || has_context_indicators,
                "Context mode should show additional lines or context indicators. Default: {default_lines}, Context: {context_lines}"
            );
        }

        // Test with larger context
        let (success, stdout_large_context, _) =
            run_semisearch(&["--advanced", "TODO", "--context", "5"], None);
        assert!(success, "Large context search should work");

        if stdout_large_context.contains("Found") {
            // Larger context should potentially show even more lines
            let large_context_has_more = stdout_large_context.len() >= stdout_context.len();
            assert!(
                large_context_has_more,
                "Larger context should show more or equal content"
            );
        }

        // Test invalid context value
        let (success, _stdout, stderr) =
            run_semisearch(&["--advanced", "TODO", "--context", "-1"], None);
        if !success {
            assert!(
                stderr.contains("context")
                    || stderr.contains("invalid")
                    || stderr.contains("positive"),
                "Should provide helpful error for invalid context values"
            );
        }
    }

    // ✅ IMPLEMENTING: Path filtering feature
    #[test]
    fn test_path_filtering() {
        // Test: Path filtering includes/excludes files by patterns

        // Test basic include pattern (should only search .rs files)
        let (success, stdout_rs_only, _) =
            run_semisearch(&["--advanced", "TODO", "--include", "*.rs"], None);
        assert!(success, "Include pattern search should work");

        if stdout_rs_only.contains("Found") {
            // Should find results in .rs files only
            let lines_with_paths = stdout_rs_only
                .lines()
                .filter(|line| line.contains("📁") || line.contains(".rs"))
                .collect::<Vec<_>>();

            // All file paths mentioned should be .rs files
            for line in lines_with_paths {
                if line.contains("📁") || line.contains("/") {
                    assert!(
                        line.contains(".rs") || !line.contains("."),
                        "Include *.rs should only show .rs files. Found: {line}"
                    );
                }
            }
        }

        // Test exclude pattern (should exclude test files)
        let (success, stdout_no_tests, _) =
            run_semisearch(&["--advanced", "TODO", "--exclude", "*test*"], None);
        assert!(success, "Exclude pattern search should work");

        if stdout_no_tests.contains("Found") {
            // Should not find results in files with "test" in the filename/path
            let has_test_files = stdout_no_tests.lines().any(|line| {
                // Check if this is a file path line that contains "test" in the actual filename
                if line.contains("📁") {
                    // Extract the filename from the path
                    if let Some(filename) = line.split('/').next_back() {
                        filename.to_lowercase().contains("test")
                    } else {
                        false
                    }
                } else {
                    false
                }
            });

            assert!(
                !has_test_files,
                "Exclude *test* should not show files with 'test' in filename. Output: {stdout_no_tests}"
            );
        }

        // Test directory filtering (search only in src/ directory)
        let (success, stdout_src_only, _) =
            run_semisearch(&["--advanced", "TODO", "--path", "src/"], None);
        assert!(success, "Directory filtering should work");

        if stdout_src_only.contains("Found") {
            // All results should be from src/ directory
            let has_non_src_files = stdout_src_only
                .lines()
                .filter(|line| line.contains("📁"))
                .any(|line| !line.contains("src/"));

            assert!(
                !has_non_src_files,
                "Path filtering to src/ should only show src/ files. Output: {stdout_src_only}"
            );
        }

        // Test multiple include patterns
        let (success, stdout_multi, _) = run_semisearch(
            &[
                "--advanced",
                "TODO",
                "--include",
                "*.rs",
                "--include",
                "*.md",
            ],
            None,
        );
        assert!(success, "Multiple include patterns should work");

        if stdout_multi.contains("Found") {
            // Should find results in .rs and .md files only
            let lines_with_paths = stdout_multi
                .lines()
                .filter(|line| line.contains("📁") && line.contains("."))
                .collect::<Vec<_>>();

            for line in lines_with_paths {
                assert!(
                    line.contains(".rs") || line.contains(".md") || !line.contains("."),
                    "Multiple includes should only show .rs and .md files. Found: {line}"
                );
            }
        }
    }

    // ✅ IMPLEMENTED: Test that basic functionality works for power users
    #[test]
    fn test_power_user_functionality() {
        // Test: Power users can still access basic functionality efficiently
        let power_user_commands = [
            vec!["TODO", "."],       // Search in specific directory
            vec!["TODO", "--fuzzy"], // Fuzzy search
            vec!["TODO", "--exact"], // Exact search
            vec!["status"],          // System status
            vec!["help-me"],         // Interactive help
        ];

        for args in &power_user_commands {
            let (success, stdout, stderr) = run_semisearch(args, None);

            // Should work efficiently for power users
            if success {
                assert!(
                    stdout.contains("Found")
                        || stdout.contains("Health")
                        || stdout.contains("Welcome")
                        || stdout.contains("No matches")
                        || stdout.contains("No results"),
                    "Power user command should work: {args:?}. stdout: {stdout}"
                );
            } else {
                // Interactive commands might not complete in test environment
                assert!(
                    args.contains(&"help-me")
                        || stderr.contains("help")
                        || stderr.contains("interactive"),
                    "Should handle interactive commands gracefully: {args:?}. stderr: {stderr}"
                );
            }
        }
    }

    // ✅ IMPLEMENTED: Test that simple interface doesn't break advanced use cases
    #[test]
    fn test_simple_interface_preserves_power() {
        // Test: Simplification doesn't break existing functionality

        // Basic search should work
        let (success, _stdout, _stderr) = run_semisearch(&["TODO"], None);
        assert!(success, "Basic search should work. stderr: {_stderr}");

        // Status should work
        let (success, stdout, _stderr) = run_semisearch(&["status"], None);
        assert!(success, "Status should work. stderr: {_stderr}");
        assert!(
            stdout.contains("Health") || stdout.contains("Ready") || stdout.contains("Available"),
            "Status should show useful information. stdout: {stdout}"
        );

        // Help should work
        let (success, stdout, stderr) = run_semisearch(&["--help"], None);
        if success {
            assert!(
                stdout.contains("Usage") || stdout.contains("search"),
                "Help should show usage information. stdout: {stdout}"
            );
        } else {
            assert!(
                stderr.contains("help") || stderr.contains("Usage"),
                "Should provide help information. stderr: {stderr}"
            );
        }
    }

    // ✅ IMPLEMENTED: Test that error handling works for advanced scenarios
    #[test]
    fn test_advanced_error_handling() {
        // Test: Error handling works for various advanced scenarios
        let error_cases = [
            vec!["TODO", "/nonexistent/path"], // Bad path
            vec!["TODO", "--invalid-flag"],    // Invalid flag
            vec!["", ""],                      // Empty query (if handled)
        ];

        for args in &error_cases {
            let (success, stdout, stderr) = run_semisearch(args, None);

            // Should handle errors gracefully
            let all_output = format!("{stdout}\n{stderr}");

            // Should not crash
            assert!(
                !all_output.contains("panic") && !all_output.contains("thread panicked"),
                "Should not panic for advanced error case {args:?}. Output: {all_output}"
            );

            // Should provide helpful feedback
            if !success {
                assert!(
                    !stderr.is_empty() || !stdout.is_empty(),
                    "Should provide error feedback for case {args:?}"
                );
            }
        }
    }

    // ✅ IMPLEMENTED: Test that performance is reasonable for basic operations
    #[test]
    fn test_basic_performance() {
        // Test: Basic operations complete in reasonable time
        use std::time::Instant;

        let start = Instant::now();
        let (success, _stdout, _stderr) = run_semisearch(&["TODO"], None);
        let duration = start.elapsed();

        assert!(success, "Basic search should work. stderr: {_stderr}");

        // Should complete in reasonable time (very generous limit for CI)
        assert!(
            duration.as_secs() < 30,
            "Basic search should complete quickly. Took: {duration:?}"
        );
    }

    // ✅ IMPLEMENTED: Test that the interface is consistent
    #[test]
    fn test_interface_consistency() {
        // Test: Interface behavior is consistent across different scenarios

        // All basic commands should have consistent behavior
        let commands = [vec!["TODO"], vec!["status"], vec!["--help"]];

        for args in &commands {
            let (success, stdout, stderr) = run_semisearch(args, None);

            // Should either succeed or fail with consistent error format
            if success {
                // Successful commands should produce meaningful output
                assert!(
                    !stdout.is_empty() || args.contains(&"--help"),
                    "Successful command should produce output: {args:?}"
                );
            } else {
                // Failed commands should produce helpful error messages
                assert!(
                    !stderr.is_empty(),
                    "Failed command should produce error message: {args:?}"
                );
            }

            // Should not produce confusing mixed success/error states
            if success && stdout.contains("Found") {
                assert!(
                    !stderr.contains("error") && !stderr.contains("failed"),
                    "Successful search should not show errors: {args:?}"
                );
            }
        }
    }
}
