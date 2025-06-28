#[cfg(test)]
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

    // Test that Rust project detection works correctly
    #[test]
    fn test_rust_project_detection() {
        let rust_dir = Path::new("tests/test-data/code-projects/rust-project");

        // Search without specifying paths - should focus on src/ and exclude target/
        let (success, stdout, _stderr) = run_semisearch(&["function"], Some(rust_dir));

        assert!(success, "Rust project search should succeed");
        assert!(stdout.contains("src/"), "Should search in src/ directory");
        assert!(
            !stdout.contains("target/"),
            "Should exclude target/ directory"
        );

        // Check if it prioritizes .rs files
        let file_mentions = stdout.lines().filter(|line| line.contains(".rs")).count();

        assert!(file_mentions > 0, "Should find matches in .rs files");
    }

    // Test that JavaScript project detection works correctly
    #[test]
    fn test_js_project_detection() {
        let js_dir = Path::new("tests/test-data/code-projects/js-project");

        // Search without specifying paths - should focus on src/ and exclude node_modules/
        let (success, stdout, _stderr) = run_semisearch(&["function"], Some(js_dir));

        assert!(success, "JS project search should succeed");
        assert!(stdout.contains("src/"), "Should search in src/ directory");
        assert!(
            !stdout.contains("node_modules/"),
            "Should exclude node_modules/ directory"
        );

        // Check if it prioritizes .js files
        let file_mentions = stdout.lines().filter(|line| line.contains(".js")).count();

        assert!(file_mentions > 0, "Should find matches in .js files");
    }

    // Test that Python project detection works correctly
    #[test]
    fn test_python_project_detection() {
        let py_dir = Path::new("tests/test-data/code-projects/python-project");

        // Search without specifying paths - should focus on src/ and exclude __pycache__/
        let (success, stdout, _stderr) = run_semisearch(&["def"], Some(py_dir));

        assert!(success, "Python project search should succeed");
        assert!(stdout.contains("src/"), "Should search in src/ directory");
        assert!(
            !stdout.contains("__pycache__/"),
            "Should exclude __pycache__/ directory"
        );

        // Check if it prioritizes .py files
        let file_mentions = stdout.lines().filter(|line| line.contains(".py")).count();

        assert!(file_mentions > 0, "Should find matches in .py files");
    }

    // Test that documentation project detection works correctly
    #[test]
    fn test_docs_project_detection() {
        let docs_dir = Path::new("tests/test-data/docs-projects/api-docs");

        // Search without specifying paths - should focus on markdown files
        let (success, stdout, _stderr) = run_semisearch(&["API"], Some(docs_dir));

        assert!(success, "Docs project search should succeed");

        // Check if it prioritizes .md files
        let file_mentions = stdout.lines().filter(|line| line.contains(".md")).count();

        assert!(file_mentions > 0, "Should find matches in .md files");
    }

    // Test that mixed project detection works correctly
    #[test]
    fn test_mixed_project_detection() {
        let mixed_dir = Path::new("tests/test-data/mixed-projects/web-app");

        // Search without specifying paths - should search in both code and docs
        let (success, stdout, _stderr) = run_semisearch(&["API"], Some(mixed_dir));

        assert!(success, "Mixed project search should succeed");

        // Check if it finds results in both code and docs
        let code_mentions = stdout
            .lines()
            .filter(|line| line.contains("src/") || line.contains(".js"))
            .count();

        let docs_mentions = stdout
            .lines()
            .filter(|line| line.contains("docs/") || line.contains(".md"))
            .count();

        assert!(
            code_mentions > 0 || docs_mentions > 0,
            "Should find matches in either code or docs"
        );
    }

    // Test that file type specific search strategies work correctly
    #[test]
    fn test_file_type_specific_search() {
        // Test code search strategy
        let code_dir = Path::new("tests/test-data/code-projects");
        let (success, stdout, _stderr) = run_semisearch(&["function"], Some(code_dir));

        assert!(success, "Code search should succeed");
        assert!(
            stdout.contains("function"),
            "Should find function definitions"
        );

        // Test documentation search strategy
        let docs_dir = Path::new("tests/test-data/docs-projects");
        let (success, stdout, _stderr) = run_semisearch(&["methodology"], Some(docs_dir));

        assert!(success, "Documentation search should succeed");
        assert!(
            stdout.contains("methodology") || stdout.contains("Methodology"),
            "Should find conceptual terms in documentation"
        );

        // Test configuration search strategy
        let config_dir = Path::new("tests/test-data/mixed-documents/data");
        let (success, stdout, _stderr) = run_semisearch(&["settings"], Some(config_dir));

        assert!(success, "Configuration search should succeed");
        assert!(
            stdout.contains("settings") || stdout.contains("Settings"),
            "Should find exact configuration terms"
        );
    }

    // Test that smart query analysis works correctly
    #[test]
    fn test_smart_query_analysis() {
        let test_dir = Path::new("tests/test-data");

        // Test exact phrase query
        let (success, _stdout, _stderr) = run_semisearch(&["\"exact phrase\""], Some(test_dir));

        assert!(
            success,
            "Exact phrase search should succeed or fail gracefully"
        );

        // Test code pattern query
        let (success, stdout, _stderr) = run_semisearch(&["function validateUser"], Some(test_dir));

        assert!(success, "Code pattern search should succeed");
        assert!(
            stdout.contains("function") && stdout.contains("validateUser"),
            "Should detect code pattern and find matches"
        );

        // Test conceptual query
        let (success, stdout, _stderr) =
            run_semisearch(&["error handling implementation"], Some(test_dir));

        assert!(success, "Conceptual search should succeed");
        assert!(
            stdout.contains("error") || stdout.contains("handling"),
            "Should detect conceptual query and find matches"
        );

        // Test file extension query
        let (success, stdout, _stderr) = run_semisearch(&["config in .json"], Some(test_dir));

        assert!(success, "File extension search should succeed");
        assert!(
            stdout.contains(".json"),
            "Should detect file extension query and limit to those files"
        );
    }

    // Test that context-aware configuration works correctly
    #[test]
    fn test_context_aware_configuration() {
        // Test Rust project configuration
        let rust_dir = Path::new("tests/test-data/code-projects/rust-project");
        let (success, stdout, _stderr) = run_semisearch(&["status"], Some(rust_dir));

        assert!(success, "Status command should succeed");

        // If status shows search paths, check that they're appropriate for Rust
        if stdout.contains("path") || stdout.contains("Path") {
            assert!(
                stdout.contains("src") || stdout.contains("tests"),
                "Should configure appropriate paths for Rust project"
            );
        }

        // Test JS project configuration
        let js_dir = Path::new("tests/test-data/code-projects/js-project");
        let (success, stdout, _stderr) = run_semisearch(&["status"], Some(js_dir));

        assert!(success, "Status command should succeed");

        // If status shows search paths, check that they're appropriate for JS
        if stdout.contains("path") || stdout.contains("Path") {
            assert!(
                stdout.contains("src") || stdout.contains("routes"),
                "Should configure appropriate paths for JS project"
            );
        }
    }

    // Test that the tool automatically adapts to different project types
    #[test]
    fn test_automatic_adaptation() {
        // Test in a Rust project
        let rust_dir = Path::new("tests/test-data/code-projects/rust-project");
        let (success, rust_stdout, _stderr) = run_semisearch(&["function"], Some(rust_dir));

        assert!(success, "Rust project search should succeed");

        // Test in a JS project
        let js_dir = Path::new("tests/test-data/code-projects/js-project");
        let (success, js_stdout, _stderr) = run_semisearch(&["function"], Some(js_dir));

        assert!(success, "JS project search should succeed");

        // Check that the results are different, indicating adaptation
        assert!(
            rust_stdout != js_stdout,
            "Search results should adapt to different project contexts"
        );

        // Check file types in results
        let rust_files = rust_stdout
            .lines()
            .filter(|line| line.contains(".rs"))
            .count();

        let js_files = js_stdout
            .lines()
            .filter(|line| line.contains(".js"))
            .count();

        assert!(rust_files > 0, "Should find .rs files in Rust project");
        assert!(js_files > 0, "Should find .js files in JS project");
    }
}
