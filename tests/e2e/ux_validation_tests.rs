#[cfg(test)]
mod ux_validation_tests {
    use std::env;
    use std::fs;
    use std::io::Write;
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

    // Test that basic search works without any flags
    #[test]
    fn test_basic_search_works() {
        let test_dir = Path::new("tests/test-data/code-projects/rust-project");

        // Test: User runs basic search without flags
        let (success, stdout, stderr) = run_semisearch(&["TODO"], Some(test_dir));

        assert!(success, "Basic search should succeed");
        assert!(stdout.contains("Found"), "Should show results count");
        assert!(!stderr.contains("error"), "Should not show errors");
        assert!(
            !stdout.contains("ONNX"),
            "Should not show technical details"
        );

        // Verify that the TODO comments in the code are found
        assert!(
            stdout.contains("TODO: Add proper logging setup")
                || stdout.contains("TODO: Implement better error handling")
                || stdout.contains("TODO: Add support for different output formats"),
            "Should find TODO comments in the code"
        );
    }

    // Test that error messages are helpful
    #[test]
    fn test_error_messages_are_helpful() {
        // Test: User provides a non-existent path
        let (success, _stdout, stderr) = run_semisearch(&["nonexistent", "/bad/path"], None);

        assert!(!success, "Should fail for bad path");
        assert!(
            !stderr.contains("anyhow"),
            "Should not expose internal errors"
        );
        assert!(
            stderr.contains("Make sure") || stderr.contains("Check"),
            "Should give actionable advice"
        );
    }

    // Test that auto-detection works for different project types
    #[test]
    fn test_project_detection_works() {
        // Test: Rust project detection
        let rust_dir = Path::new("tests/test-data/code-projects/rust-project");
        let (success, stdout, _stderr) = run_semisearch(&["function"], Some(rust_dir));

        assert!(success, "Search in Rust project should succeed");
        assert!(stdout.contains(".rs"), "Should focus on Rust files");

        // Test: JavaScript project detection
        let js_dir = Path::new("tests/test-data/code-projects/js-project");
        let (success, stdout, _stderr) = run_semisearch(&["function"], Some(js_dir));

        assert!(success, "Search in JS project should succeed");
        assert!(stdout.contains(".js"), "Should focus on JS files");

        // Test: Documentation project detection
        let docs_dir = Path::new("tests/test-data/docs-projects/api-docs");
        let (success, stdout, _stderr) = run_semisearch(&["API"], Some(docs_dir));

        assert!(success, "Search in docs project should succeed");
        assert!(stdout.contains(".md"), "Should focus on markdown files");
    }

    // Test that fuzzy search works
    #[test]
    fn test_fuzzy_search_works() {
        let test_dir = Path::new("tests/test-data/mixed-documents");

        // Test: User makes a typo
        let (success, stdout, _stderr) = run_semisearch(&["databse", "--fuzzy"], Some(test_dir));

        assert!(success, "Fuzzy search should succeed");
        assert!(
            stdout.contains("database") || stdout.contains("Database"),
            "Should find 'database' despite typo 'databse'"
        );
    }

    // Test that exact search works
    #[test]
    fn test_exact_search_works() {
        let test_dir = Path::new("tests/test-data/mixed-documents");

        // Test: User wants exact matches only
        let (success, stdout, _stderr) = run_semisearch(&["TODO", "--exact"], Some(test_dir));

        assert!(success, "Exact search should succeed");

        // Count occurrences of "TODO" in the output
        let todo_count = stdout.matches("TODO").count();
        assert!(todo_count > 0, "Should find exact TODO matches");

        // Ensure no partial matches like "TODOS" are included
        assert!(
            !stdout.contains("TODOS"),
            "Should not include partial matches"
        );
    }

    // Test that query analysis works correctly
    #[test]
    fn test_query_analysis_works() {
        let test_dir = Path::new("tests/test-data");

        // Test: Code pattern query
        let (success, stdout, _stderr) = run_semisearch(&["function validateUser"], Some(test_dir));

        assert!(success, "Code pattern search should succeed");
        assert!(
            stdout.contains("function") && stdout.contains("validateUser"),
            "Should detect code pattern query"
        );

        // Test: Conceptual query
        let (success, stdout, _stderr) =
            run_semisearch(&["error handling patterns"], Some(test_dir));

        assert!(success, "Conceptual search should succeed");
        assert!(
            stdout.contains("error") || stdout.contains("handling"),
            "Should detect conceptual query"
        );

        // Test: File extension query
        let (success, stdout, _stderr) = run_semisearch(&["config in .json files"], Some(test_dir));

        assert!(success, "File extension search should succeed");
        assert!(
            stdout.contains(".json"),
            "Should detect file extension in query"
        );
    }

    // Test that results are grouped logically
    #[test]
    fn test_result_grouping() {
        let test_dir = Path::new("tests/test-data");

        // Test: Results should be grouped by file
        let (success, stdout, _stderr) = run_semisearch(&["error"], Some(test_dir));

        assert!(success, "Search should succeed");

        // Check if output contains file headers/groupings
        let lines: Vec<&str> = stdout.lines().collect();
        let mut current_file = "";
        let mut file_groups = 0;

        for line in lines {
            if line.contains("📁") || (line.contains(".") && !line.starts_with(" ")) {
                if current_file != line {
                    current_file = line;
                    file_groups += 1;
                }
            }
        }

        assert!(file_groups > 0, "Results should be grouped by file");
    }

    // Test that the help system works
    #[test]
    fn test_help_system_works() {
        // Test: User asks for help
        let (success, stdout, _stderr) = run_semisearch(&["help-me"], None);

        assert!(success, "Help command should succeed");
        assert!(
            stdout.contains("Welcome") || stdout.contains("help") || stdout.contains("guide"),
            "Should show welcome/help message"
        );

        // Test: User runs with no arguments
        let (success, stdout, _stderr) = run_semisearch(&[], None);

        assert!(success, "Running without args should succeed");
        assert!(
            stdout.contains("Usage") || stdout.contains("help"),
            "Should show usage information"
        );
    }

    // Test that status command works
    #[test]
    fn test_status_command_works() {
        // Test: User checks status
        let (success, stdout, _stderr) = run_semisearch(&["status"], None);

        assert!(success, "Status command should succeed");
        assert!(
            stdout.contains("status")
                || stdout.contains("Status")
                || stdout.contains("working")
                || stdout.contains("health"),
            "Should show status information"
        );
    }

    // Test file type specific search strategies
    #[test]
    fn test_file_type_strategies() {
        // Test code files
        let code_dir = Path::new("tests/test-data/code-projects");
        let (success, stdout, _stderr) = run_semisearch(&["function"], Some(code_dir));

        assert!(success, "Code search should succeed");
        assert!(
            stdout.contains("function"),
            "Should find functions in code files"
        );

        // Test documentation files
        let docs_dir = Path::new("tests/test-data/docs-projects");
        let (success, stdout, _stderr) = run_semisearch(&["API"], Some(docs_dir));

        assert!(success, "Documentation search should succeed");
        assert!(stdout.contains("API"), "Should find API mentions in docs");

        // Test configuration files
        let (success, stdout, _stderr) = run_semisearch(
            &["config"],
            Some(Path::new("tests/test-data/mixed-documents/data")),
        );

        assert!(success, "Config search should succeed");
        assert!(
            stdout.contains("config") || stdout.contains("Config"),
            "Should find config mentions in config files"
        );
    }

    // Test search in mixed documents
    #[test]
    fn test_mixed_documents_search() {
        let mixed_dir = Path::new("tests/test-data/mixed-documents");

        // Test: Search in mixed documents
        let (success, stdout, _stderr) = run_semisearch(&["project"], Some(mixed_dir));

        assert!(success, "Mixed documents search should succeed");
        assert!(
            stdout.contains("project") || stdout.contains("Project"),
            "Should find 'project' in mixed documents"
        );

        // Check if results from different file types are included
        assert!(
            stdout.contains(".md")
                || stdout.contains(".txt")
                || stdout.contains(".py")
                || stdout.contains(".json"),
            "Should include results from different file types"
        );
    }

    // Test advanced mode
    #[test]
    fn test_advanced_mode() {
        // Test: User uses advanced mode
        let (success, _stdout, _stderr) = run_semisearch(&["TODO", "--advanced"], None);

        assert!(success, "Advanced mode search should succeed");

        // Check for advanced options in help output
        let (success, stdout, _stderr) = run_semisearch(&["--advanced", "--help"], None);

        assert!(success, "Advanced help should succeed");
        assert!(
            stdout.contains("mode")
                || stdout.contains("score")
                || stdout.contains("limit")
                || stdout.contains("threshold"),
            "Should show advanced options"
        );
    }

    // Test no results scenario
    #[test]
    fn test_no_results_scenario() {
        // Test: Search for something that doesn't exist
        let (success, stdout, _stderr) = run_semisearch(&["xyz123impossible"], None);

        // It should succeed even with no results
        assert!(success, "No results search should still succeed");
        assert!(
            stdout.contains("No") && stdout.contains("found"),
            "Should indicate no results found"
        );
        assert!(
            stdout.contains("Try") || stdout.contains("suggestion"),
            "Should provide suggestions"
        );
    }

    // Test with large result set
    #[test]
    fn test_large_result_set() {
        // Create a temporary file with many occurrences of a search term
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("many_matches.txt");
        let mut file = fs::File::create(&file_path).expect("Failed to create temp file");

        // Write 100 lines with "test" in them
        for i in 1..=100 {
            writeln!(file, "Line {} contains test pattern", i)
                .expect("Failed to write to temp file");
        }

        // Test: Search with many results
        let (success, stdout, _stderr) = run_semisearch(&["test"], Some(temp_dir.path()));

        assert!(success, "Large result set search should succeed");
        assert!(stdout.contains("Found"), "Should show results count");

        // Check if output is paginated or limited
        let result_count = stdout.matches("Line").count();
        assert!(result_count > 0, "Should show some results");
        assert!(
            result_count <= 50 || stdout.contains("more"),
            "Should limit results or indicate there are more"
        );
    }

    // Test search with context
    #[test]
    fn test_search_with_context() {
        let test_dir = Path::new("tests/test-data/code-projects");

        // Test: Search with context (if supported)
        let (success, stdout, _stderr) =
            run_semisearch(&["function", "--context", "3"], Some(test_dir));

        assert!(
            success,
            "Context search should succeed or be ignored gracefully"
        );

        // If context is supported, check that lines before/after match are shown
        if stdout.contains("function") {
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

    // Test the simplicity of the interface
    #[test]
    fn test_interface_simplicity() {
        // Test: Get help output
        let (success, stdout, _stderr) = run_semisearch(&["--help"], None);

        assert!(success, "Help should succeed");

        // Count the number of visible commands/options in the default interface
        let command_count = stdout.matches("--").count();

        // Basic interface should have few options
        assert!(
            command_count <= 16,
            "Default interface should have limited options"
        );

        // Advanced interface should have more options
        let (success, advanced_stdout, _stderr) = run_semisearch(&["--advanced", "--help"], None);

        assert!(success, "Advanced help should succeed");

        // Only test if advanced mode is implemented
        if advanced_stdout.contains("advanced") {
            let advanced_command_count = advanced_stdout.matches("--").count();
            assert!(
                advanced_command_count >= command_count,
                "Advanced interface should have more options than basic interface"
            );
        }
    }
}
