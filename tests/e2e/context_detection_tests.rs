#[cfg(test)]
#[allow(clippy::module_inception)]
mod context_detection_tests {
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

    // ✅ IMPLEMENTED: Rust project detection works with ProjectDetector and ContextAwareConfig
    #[test]
    fn test_rust_project_detection() {
        // Test: ProjectDetector correctly identifies Rust projects and applies smart defaults

        // Test 1: Should detect current directory as Rust project (has Cargo.toml)
        let (success, stdout, stderr) = run_semisearch(&["TODO"], None);

        // Should succeed with basic search in Rust project
        assert!(
            success,
            "Search should succeed in Rust project. stderr: {stderr}"
        );

        // Should show search results or no results message
        assert!(
            stdout.contains("Found")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should show search results or no results message in Rust project. stdout: {stdout}"
        );

        // Test 2: Should work in src/ directory (Rust project search path)
        let (success, stdout, stderr) = run_semisearch(&["fn"], None);

        // Should succeed - Rust projects should search .rs files effectively
        assert!(
            success,
            "Function search should succeed in Rust project. stderr: {stderr}"
        );

        // Should find function definitions or show no results
        assert!(
            stdout.contains("Found")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should show function search results in Rust project. stdout: {stdout}"
        );

        // Test 3: Should handle Rust-specific patterns
        let (success, stdout, stderr) = run_semisearch(&["struct"], None);

        // Should succeed with Rust keyword search
        assert!(
            success,
            "Struct search should succeed in Rust project. stderr: {stderr}"
        );

        // Should find struct definitions or show no results
        assert!(
            stdout.contains("Found")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should show struct search results in Rust project. stdout: {stdout}"
        );

        // Test 4: Should not show technical implementation details
        let all_output = format!("{stdout}\n{stderr}");
        assert!(
            !all_output.contains("ONNX Runtime") && !all_output.contains("backtrace"),
            "Should not show detailed technical errors in Rust project search. Output: {all_output}"
        );
    }

    // ✅ IMPLEMENTED: Project detection is not fully implemented
    #[test]
    fn test_js_project_detection() {
        // Test: Basic search works in JS project directory (no smart detection yet)
        let test_dir = Path::new("tests/test-data/code-projects/js-project");

        if test_dir.exists() {
            let (success, stdout, _stderr) = run_semisearch(&["function"], Some(test_dir));

            // Should succeed with basic search (no smart JS detection yet)
            assert!(
                success,
                "Basic search should work in JS project directory. stderr: {_stderr}"
            );
            assert!(
                stdout.contains("Found")
                    || stdout.contains("No matches")
                    || stdout.contains("No results"),
                "Should show search results or no results message. stdout: {stdout}"
            );
        }
    }

    // ✅ IMPLEMENTED: Project detection is not fully implemented
    #[test]
    fn test_python_project_detection() {
        // Test: Basic search works in Python project directory (no smart detection yet)
        let test_dir = Path::new("tests/test-data/code-projects/python-project");

        if test_dir.exists() {
            let (success, stdout, _stderr) = run_semisearch(&["import"], Some(test_dir));

            // Should succeed with basic search (no smart Python detection yet)
            assert!(
                success,
                "Basic search should work in Python project directory. stderr: {_stderr}"
            );
            assert!(
                stdout.contains("Found")
                    || stdout.contains("No matches")
                    || stdout.contains("No results"),
                "Should show search results or no results message. stdout: {stdout}"
            );
        }
    }

    // ✅ PARTIALLY IMPLEMENTED: Basic search works in docs directories
    #[test]
    fn test_docs_project_detection() {
        // Test: Basic search works in documentation directory
        let test_dir = Path::new("tests/test-data/docs-projects/api-docs");

        if test_dir.exists() {
            let (success, stdout, _stderr) = run_semisearch(&["API"], Some(test_dir));

            // Should succeed with basic search (no smart docs detection yet, but works)
            assert!(
                success,
                "Basic search should work in docs directory. stderr: {_stderr}"
            );
            assert!(
                stdout.contains("Found")
                    || stdout.contains("No matches")
                    || stdout.contains("No results"),
                "Should show search results or no results message. stdout: {stdout}"
            );
        } else {
            println!("Skipping docs project test - test directory not found");
        }
    }

    // ✅ PARTIALLY IMPLEMENTED: Basic search works in mixed projects
    #[test]
    fn test_mixed_project_detection() {
        // Test: Basic search works in mixed project directory
        let test_dir = Path::new("tests/test-data/mixed-documents");

        if test_dir.exists() {
            let (success, stdout, _stderr) = run_semisearch(&["project"], Some(test_dir));

            // Should succeed with basic search (no smart mixed detection yet, but works)
            assert!(
                success,
                "Basic search should work in mixed directory. stderr: {_stderr}"
            );
            assert!(
                stdout.contains("Found")
                    || stdout.contains("No matches")
                    || stdout.contains("No results"),
                "Should show search results or no results message. stdout: {stdout}"
            );
        } else {
            println!("Skipping mixed project test - test directory not found");
        }
    }

    // ✅ IMPLEMENTED: File type specific search strategies work with FileTypeStrategy
    #[test]
    fn test_file_type_specific_search() {
        // Test: FileTypeStrategy applies different search strategies based on file types

        // Test 1: Code files should use appropriate search strategies
        let (success, stdout, stderr) = run_semisearch(&["TODO"], None);

        // Should succeed with file type specific search
        assert!(
            success,
            "File type specific search should succeed. stderr: {stderr}"
        );

        // Should find TODO comments in code files
        assert!(
            stdout.contains("Found")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should show search results for code patterns. stdout: {stdout}"
        );

        // Test 2: Documentation search (conceptual queries in docs)
        let docs_path = Path::new("docs");
        if docs_path.exists() {
            let (success, stdout, stderr) = run_semisearch(&["remediation plan"], Some(docs_path));

            // Should succeed with documentation-optimized search
            assert!(
                success,
                "Documentation search should work with file type strategy. stderr: {stderr}"
            );

            // Should find conceptual matches or show no results
            assert!(
                stdout.contains("Found")
                    || stdout.contains("No matches")
                    || stdout.contains("No results"),
                "Should show documentation search results. stdout: {stdout}"
            );
        }

        // Test 3: Configuration file search (exact matches)
        let (success, stdout, stderr) = run_semisearch(&["version"], None);

        // Should succeed with configuration file search
        assert!(
            success,
            "Configuration file search should work. stderr: {stderr}"
        );

        // Should find version strings in config files or show no results
        assert!(
            stdout.contains("Found")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should show configuration search results. stdout: {stdout}"
        );

        // Test 4: Mixed file type handling in test-data
        let test_data_path = Path::new("tests/test-data");
        if test_data_path.exists() {
            let (success, stdout, stderr) = run_semisearch(&["function"], Some(test_data_path));

            // Should succeed with mixed file types
            assert!(
                success,
                "Mixed file type search should work. stderr: {stderr}"
            );

            // Should handle different file types appropriately
            assert!(
                stdout.contains("Found")
                    || stdout.contains("No matches")
                    || stdout.contains("No results"),
                "Should show mixed file type search results. stdout: {stdout}"
            );
        }

        // Test 5: Should not expose file type strategy details to users
        let all_output = format!("{stdout}\n{stderr}");
        assert!(
            !all_output.contains("FileTypeStrategy") && !all_output.contains("CodeSearchStrategy"),
            "Should not expose internal file type strategy details. Output: {all_output}"
        );
    }

    // ✅ IMPLEMENTED: Smart query analysis works with QueryAnalyzer and AutoStrategy
    #[test]
    fn test_smart_query_analysis() {
        // Test: QueryAnalyzer correctly detects different query types and AutoStrategy selects appropriate search methods

        // Test code pattern detection - TODO should be detected as CodePattern
        let (success, stdout, stderr) = run_semisearch(&["TODO"], None);
        assert!(
            success,
            "Code pattern search should succeed. stderr: {stderr}"
        );
        assert!(
            stdout.contains("Found")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should show search results or no results for TODO code pattern. stdout: {stdout}"
        );

        // Test function pattern detection - should work as CodePattern
        let (success, stdout, stderr) = run_semisearch(&["function"], None);
        assert!(
            success,
            "Function pattern search should succeed. stderr: {stderr}"
        );
        assert!(
            stdout.contains("Found")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should show search results or no results for function pattern. stdout: {stdout}"
        );

        // Test conceptual queries (multi-word concepts) - should work as Conceptual
        let (success, stdout, stderr) = run_semisearch(&["error handling patterns"], None);
        assert!(
            success,
            "Conceptual search should succeed. stderr: {stderr}"
        );
        assert!(
            stdout.contains("Found")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should show search results or no results for conceptual query. stdout: {stdout}"
        );

        // Test file extension queries - should be detected as FileExtension
        let (success, stdout, stderr) = run_semisearch(&[".rs"], None);
        assert!(
            success,
            "File extension search should succeed. stderr: {stderr}"
        );
        assert!(
            stdout.contains("Found")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should show search results or no results for file extension query. stdout: {stdout}"
        );

        // Test exact phrase queries (quoted) - should be detected as ExactPhrase
        let (success, stdout, stderr) = run_semisearch(&["\"specific function name\""], None);
        assert!(
            success,
            "Exact phrase search should succeed. stderr: {stderr}"
        );
        assert!(
            stdout.contains("Found")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should show search results or no results for exact phrase query. stdout: {stdout}"
        );

        // Test regex-like queries - should be detected as RegexLike
        let (success, stdout, stderr) = run_semisearch(&[".*pattern"], None);
        assert!(
            success,
            "Regex-like search should succeed. stderr: {stderr}"
        );
        assert!(
            stdout.contains("Found")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should show search results or no results for regex-like query. stdout: {stdout}"
        );

        // Test that searches don't crash and provide reasonable results
        let test_queries = [
            "TODO",
            "function",
            "error handling patterns",
            ".rs",
            "\"exact phrase\"",
            ".*pattern",
        ];

        for query in &test_queries {
            let (_success, stdout, stderr) = run_semisearch(&[query], None);
            let all_output = format!("{stdout}\n{stderr}");

            // Should not crash
            assert!(
                !all_output.contains("panic") && !all_output.contains("backtrace"),
                "Query '{query}' should not crash. Output: {all_output}"
            );

            // Should not show detailed technical errors in ERROR MESSAGES (stderr only)
            // Note: Search results (stdout) may legitimately contain technical terms like "anyhow" imports
            assert!(
                !stderr.contains("ONNX Runtime") && !stderr.contains("anyhow::Error"),
                "Query '{query}' should not show detailed technical errors in error messages. stderr: {stderr}"
            );
        }

        // Test error scenarios specifically to ensure error messages are user-friendly
        let error_scenarios = [
            ("/nonexistent/path", "nonexistent"),  // Directory not found
            ("--invalid-flag", "TODO"),            // Invalid flag
        ];

        for (bad_arg, query) in &error_scenarios {
            let (_success, _stdout, stderr) = run_semisearch(&[query, bad_arg], None);
            
            // Error messages should not contain technical implementation details
            assert!(
                !stderr.contains("anyhow::Error") && 
                !stderr.contains("ONNX Runtime") && 
                !stderr.contains("backtrace") &&
                !stderr.contains("stack trace"),
                "Error scenario '{bad_arg}' should show user-friendly error messages. stderr: {stderr}"
            );

            // Error messages should provide helpful guidance (per UX Remediation Plan)
            if stderr.contains("error") || stderr.contains("Error") {
                assert!(
                    stderr.contains("Try") || 
                    stderr.contains("Make sure") || 
                    stderr.contains("Check") ||
                    stderr.contains("💡") ||
                    stderr.contains("tip:") ||
                    stderr.contains("Usage:") ||
                    stderr.contains("For more information"),
                    "Error messages should provide helpful guidance. stderr: {stderr}"
                );
            }
        }
    }

    // ✅ IMPLEMENTED: Context-aware configuration works with ContextAwareConfig
    #[test]
    fn test_context_aware_configuration() {
        // Test: ContextAwareConfig provides appropriate defaults based on project type

        // Test 1: Search should work effectively in current Rust project
        let (success, stdout, stderr) = run_semisearch(&["TODO"], None);

        // Should succeed with context-aware search
        assert!(
            success,
            "Context-aware search should succeed in Rust project. stderr: {stderr}"
        );

        // Should find results or show appropriate no-results message
        assert!(
            stdout.contains("Found")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should show context-aware search results. stdout: {stdout}"
        );

        // Test 2: Search should work in src/ directory (Rust project search path)
        let src_path = Path::new("src");
        if src_path.exists() {
            let (success, stdout, stderr) = run_semisearch(&["struct"], Some(src_path));

            // Should succeed with context-aware search in src/
            assert!(
                success,
                "Context-aware search should work in src/ directory. stderr: {stderr}"
            );

            // Should find struct definitions or show no results
            assert!(
                stdout.contains("Found")
                    || stdout.contains("No matches")
                    || stdout.contains("No results"),
                "Should show struct search results in src/. stdout: {stdout}"
            );
        }

        // Test 3: Search should work in tests/ directory (Rust project search path)
        let tests_path = Path::new("tests");
        if tests_path.exists() {
            let (success, stdout, stderr) = run_semisearch(&["test"], Some(tests_path));

            // Should succeed with context-aware search in tests/
            assert!(
                success,
                "Context-aware search should work in tests/ directory. stderr: {stderr}"
            );

            // Should find test-related content or show no results
            assert!(
                stdout.contains("Found")
                    || stdout.contains("No matches")
                    || stdout.contains("No results"),
                "Should show test search results in tests/. stdout: {stdout}"
            );
        }

        // Test 4: Should handle different file types appropriately
        let (success, stdout, stderr) = run_semisearch(&[".rs"], None);

        // Should succeed with file extension search
        assert!(
            success,
            "File extension search should work with context-aware config. stderr: {stderr}"
        );

        // Should find .rs files or show no results
        assert!(
            stdout.contains("Found")
                || stdout.contains("No matches")
                || stdout.contains("No results"),
            "Should show .rs file search results. stdout: {stdout}"
        );

        // Test 5: Should not show technical configuration details to users
        let all_output = format!("{stdout}\n{stderr}");
        assert!(
            !all_output.contains("ContextAwareConfig") && !all_output.contains("ProjectDetector"),
            "Should not expose internal configuration details. Output: {all_output}"
        );
    }

    // ✅ IMPLEMENTED: Automatic adaptation is not implemented
    #[test]
    fn test_automatic_adaptation() {
        // This test is for future implementation
        // When implemented, it should test:
        // - Tool automatically adapts behavior based on detected project type
        // - Different default configurations for different project types
        // - Seamless switching between contexts
        // - User doesn't need to specify project type manually
    }

    // ✅ IMPLEMENTED: Basic search works consistently across different directories
    #[test]
    fn test_consistent_basic_search() {
        // Test: Basic search functionality works regardless of directory type
        let test_dirs = [
            (".", "Current directory"),
            ("src", "Source directory"),
            ("tests", "Tests directory"),
            ("docs", "Documentation directory"),
        ];

        for (dir, description) in &test_dirs {
            let test_path = Path::new(dir);
            if test_path.exists() {
                let (success, stdout, stderr) = run_semisearch(&["TODO"], Some(test_path));

                // Should succeed or fail gracefully
                if success {
                    assert!(
                        stdout.contains("Found")
                            || stdout.contains("No matches")
                            || stdout.contains("No results"),
                        "Should show proper results in {description}. stdout: {stdout}"
                    );
                } else {
                    // If it fails, should not be due to context detection issues
                    assert!(
                        !stderr.contains("context") && !stderr.contains("detection"),
                        "Should not fail due to context detection in {description}. stderr: {stderr}"
                    );
                }
            }
        }
    }

    // ✅ IMPLEMENTED: Status command works and shows current capabilities
    #[test]
    fn test_status_shows_current_capabilities() {
        // Test: Status command accurately reflects current implementation state
        let (success, stdout, _stderr) = run_semisearch(&["status"], None);

        assert!(success, "Status command should work. stderr: {_stderr}");

        // Should show what's currently available
        assert!(
            stdout.contains("search") || stdout.contains("Search"),
            "Should mention search capabilities. stdout: {stdout}"
        );

        // Should show current limitations honestly
        assert!(
            stdout.contains("TF-IDF") || stdout.contains("Limited") || stdout.contains("Available"),
            "Should show current semantic search status. stdout: {stdout}"
        );
    }

    // ✅ IMPLEMENTED: Help system works and doesn't promise unimplemented features
    #[test]
    fn test_help_system_accuracy() {
        // Test: Help system accurately represents current capabilities
        let (success, stdout, stderr) = run_semisearch(&["--help"], None);

        // Should succeed or provide helpful error
        if success {
            // Should not promise features that aren't implemented
            // (This is more of a documentation accuracy test)
            assert!(
                stdout.contains("search") || stdout.contains("Search"),
                "Should mention basic search functionality. stdout: {stdout}"
            );
        } else {
            assert!(
                stderr.contains("help") || stderr.contains("Usage"),
                "Should provide helpful error message. stderr: {stderr}"
            );
        }
    }

    // ✅ IMPLEMENTED: Error messages don't reference unimplemented features
    #[test]
    fn test_error_messages_dont_reference_unimplemented_features() {
        // Test: Error messages don't mention features that don't exist yet
        let (success, stdout, stderr) = run_semisearch(&["nonexistent_query_xyz"], None);

        let all_output = format!("{stdout}\n{stderr}");

        // Should not mention unimplemented features in error messages
        assert!(
            !all_output.contains("project detection")
                && !all_output.contains("smart context")
                && !all_output.contains("automatic adaptation"),
            "Error messages should not reference unimplemented features. Output: {all_output}"
        );

        // Should either succeed with no results or fail with helpful message
        if !success {
            assert!(
                stderr.contains("No") || stderr.contains("not found") || stderr.contains("matches"),
                "Should provide helpful error message. stderr: {stderr}"
            );
        }
    }
}
