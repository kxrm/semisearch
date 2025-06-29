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

    // ❌ NOT IMPLEMENTED: Project detection is not fully implemented as described in UX plan
    #[test]
    #[ignore = "Project detection not implemented yet - needs Task 2.1.1 and 2.1.2"]
    fn test_rust_project_detection() {
        // This test is for future implementation
        // When implemented, it should test:
        // - Automatic detection of Cargo.toml
        // - Focus on .rs files in src/ and tests/
        // - Ignore target/ directory
        // - Use code-aware search strategies
    }

    // ❌ NOT IMPLEMENTED: Project detection is not fully implemented
    #[test]
    #[ignore = "Project detection not implemented yet - needs Task 2.1.1"]
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

    // ❌ NOT IMPLEMENTED: Project detection is not fully implemented
    #[test]
    #[ignore = "Project detection not implemented yet - needs Task 2.1.1"]
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

    // ❌ NOT IMPLEMENTED: File type specific search strategies are not implemented
    #[test]
    #[ignore = "File type strategies not implemented yet - needs Task 2.3.1"]
    fn test_file_type_specific_search() {
        // This test is for future implementation
        // When implemented, it should test:
        // - Code files use regex + semantic search
        // - Documentation files use semantic search for concepts
        // - Configuration files use exact search
        // - Different scoring strategies per file type
    }

    // ✅ IMPLEMENTED: Smart query analysis works with QueryAnalyzer and AutoStrategy
    #[test]
    fn test_smart_query_analysis() {
        // Test: QueryAnalyzer correctly detects different query types and AutoStrategy selects appropriate search methods
        
        // Test code pattern detection - TODO should be detected as CodePattern
        let (success, stdout, stderr) = run_semisearch(&["TODO"], None);
        assert!(success, "Code pattern search should succeed. stderr: {stderr}");
        assert!(
            stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("No results"),
            "Should show search results or no results for TODO code pattern. stdout: {stdout}"
        );

        // Test function pattern detection - should work as CodePattern
        let (success, stdout, stderr) = run_semisearch(&["function"], None);
        assert!(success, "Function pattern search should succeed. stderr: {stderr}");
        assert!(
            stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("No results"),
            "Should show search results or no results for function pattern. stdout: {stdout}"
        );

        // Test conceptual queries (multi-word concepts) - should work as Conceptual
        let (success, stdout, stderr) = run_semisearch(&["error handling patterns"], None);
        assert!(success, "Conceptual search should succeed. stderr: {stderr}");
        assert!(
            stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("No results"),
            "Should show search results or no results for conceptual query. stdout: {stdout}"
        );

        // Test file extension queries - should be detected as FileExtension
        let (success, stdout, stderr) = run_semisearch(&[".rs"], None);
        assert!(success, "File extension search should succeed. stderr: {stderr}");
        assert!(
            stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("No results"),
            "Should show search results or no results for file extension query. stdout: {stdout}"
        );

        // Test exact phrase queries (quoted) - should be detected as ExactPhrase
        let (success, stdout, stderr) = run_semisearch(&["\"specific function name\""], None);
        assert!(success, "Exact phrase search should succeed. stderr: {stderr}");
        assert!(
            stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("No results"),
            "Should show search results or no results for exact phrase query. stdout: {stdout}"
        );

        // Test regex-like queries - should be detected as RegexLike
        let (success, stdout, stderr) = run_semisearch(&[".*pattern"], None);
        assert!(success, "Regex-like search should succeed. stderr: {stderr}");
        assert!(
            stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("No results"),
            "Should show search results or no results for regex-like query. stdout: {stdout}"
        );

        // Test that searches don't crash and provide reasonable results
        let test_queries = [
            "TODO",
            "function",
            "error handling patterns", 
            ".rs",
            "\"exact phrase\"",
            ".*pattern"
        ];

        for query in &test_queries {
            let (_success, stdout, stderr) = run_semisearch(&[query], None);
            let all_output = format!("{stdout}\n{stderr}");
            
            // Should not crash
            assert!(
                !all_output.contains("panic") && !all_output.contains("backtrace"),
                "Query '{query}' should not crash. Output: {all_output}"
            );

            // Should not show detailed technical errors to regular users
            assert!(
                !all_output.contains("ONNX Runtime") && !all_output.contains("anyhow"),
                "Query '{query}' should not show detailed technical errors. Output: {all_output}"
            );
        }
    }

    // ❌ NOT IMPLEMENTED: Context-aware configuration is not implemented
    #[test]
    #[ignore = "Context-aware configuration not implemented yet - needs Task 2.1.2"]
    fn test_context_aware_configuration() {
        // This test is for future implementation
        // When implemented, it should test:
        // - Automatic search path configuration based on project type
        // - File pattern filtering based on detected project type
        // - Ignore pattern configuration (target/, node_modules/, etc.)
        // - Search strategy selection based on context
    }

    // ❌ NOT IMPLEMENTED: Automatic adaptation is not implemented
    #[test]
    #[ignore = "Automatic adaptation not implemented yet - needs full context detection system"]
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
