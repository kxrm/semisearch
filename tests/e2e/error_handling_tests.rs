#[cfg(test)]
mod error_handling_tests {
    use std::env;
    use std::fs;
    use std::io::Write;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
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

    // Test that error messages are user-friendly
    #[test]
    fn test_user_friendly_error_messages() {
        // Test: Non-existent directory
        let (success, _stdout, stderr) = run_semisearch(&["TODO", "/nonexistent"], None);

        assert!(!success, "Should fail for non-existent directory");
        assert!(
            !stderr.contains("anyhow"),
            "Should not expose internal errors"
        );
        assert!(!stderr.contains("panic"), "Should not show panic messages");
        assert!(
            stderr.contains("Make sure")
                || stderr.contains("Check")
                || stderr.contains("exists")
                || stderr.contains("permission"),
            "Should provide actionable advice for directory access"
        );

        // Test: Non-existent query that produces no results
        let (success, stdout, _stderr) = run_semisearch(&["xyz123impossible"], None);

        // This should succeed even with no results
        assert!(success, "No results search should still succeed");
        assert!(
            stdout.contains("No") && stdout.contains("found"),
            "Should indicate no results found"
        );
        assert!(
            stdout.contains("Try") || stdout.contains("Check") || stdout.contains("suggestion"),
            "Should provide suggestions for no results"
        );
    }

    // Test that technical jargon is hidden
    #[test]
    fn test_no_technical_jargon() {
        // Test: Basic search should not expose technical details
        let (success, stdout, stderr) = run_semisearch(&["TODO"], None);

        assert!(success, "Basic search should succeed");

        // Check for absence of technical jargon
        assert!(!stdout.contains("ONNX"), "Should not mention ONNX");
        assert!(
            !stdout.contains("embeddings"),
            "Should not mention embeddings"
        );
        assert!(!stdout.contains("TF-IDF"), "Should not mention TF-IDF");
        assert!(!stdout.contains("neural"), "Should not mention neural");
        assert!(!stdout.contains("vector"), "Should not mention vector");
        assert!(!stderr.contains("panic"), "Should not show panic messages");
        assert!(
            !stderr.contains("unwrap"),
            "Should not show Rust unwrap errors"
        );
        assert!(
            !stderr.contains("anyhow"),
            "Should not expose anyhow errors"
        );
    }

    // Test that fallback modes are handled gracefully
    #[test]
    fn test_fallback_mode_handling() {
        // This test is speculative - it depends on how fallbacks are implemented
        // We'll check that if a fallback message appears, it's user-friendly

        let (success, stdout, stderr) = run_semisearch(&["concept search"], None);

        assert!(success, "Search should succeed or fail gracefully");

        let combined_output = format!("{}\n{}", stdout, stderr);

        if combined_output.contains("fall") || combined_output.contains("basic") {
            assert!(
                !combined_output.contains("ONNX"),
                "Should not mention ONNX in fallback"
            );
            assert!(
                !combined_output.contains("neural"),
                "Should not mention neural in fallback"
            );
            assert!(
                combined_output.contains("basic")
                    || combined_output.contains("simple")
                    || combined_output.contains("fast"),
                "Should use user-friendly terms for fallback mode"
            );
        }
    }

    // Test that error recovery suggestions are helpful
    #[test]
    fn test_error_recovery_suggestions() {
        // Test: No results case
        let (success, stdout, _stderr) = run_semisearch(&["xyz123impossible"], None);

        assert!(success, "No results search should still succeed");

        // Check for helpful suggestions
        assert!(
            stdout.contains("Try") || stdout.contains("suggestion"),
            "Should provide suggestions"
        );

        // Check for specific suggestions
        let has_spelling_suggestion =
            stdout.contains("spelling") || stdout.contains("typo") || stdout.contains("fuzzy");

        let has_simpler_terms_suggestion = stdout.contains("simpler")
            || stdout.contains("different")
            || stdout.contains("alternative");

        assert!(
            has_spelling_suggestion || has_simpler_terms_suggestion,
            "Should provide specific actionable suggestions"
        );
    }

    // Test handling of typos in queries
    #[test]
    fn test_typo_handling() {
        // Test with a common typo
        let (success, stdout, _stderr) = run_semisearch(&["functoin"], None);

        assert!(success, "Typo search should succeed");

        // Check if it found "function" despite the typo, or suggested it
        let found_correction = stdout.contains("function")
            || stdout.contains("Did you mean")
            || stdout.contains("fuzzy");

        assert!(
            found_correction,
            "Should find results despite typo or suggest correction"
        );

        // Test with explicit fuzzy flag
        let (success, stdout, _stderr) = run_semisearch(&["functoin", "--fuzzy"], None);

        assert!(success, "Fuzzy search should succeed");
        assert!(
            stdout.contains("function"),
            "Should find 'function' with fuzzy search despite typo 'functoin'"
        );
    }

    // Test handling of invalid flags or options
    #[test]
    fn test_invalid_flag_handling() {
        // Test with an invalid flag
        let (success, _stdout, stderr) = run_semisearch(&["TODO", "--invalid-flag"], None);

        assert!(!success, "Invalid flag should fail");
        assert!(
            stderr.contains("unknown")
                || stderr.contains("invalid")
                || stderr.contains("flag")
                || stderr.contains("option"),
            "Should indicate unknown flag"
        );
        assert!(
            stderr.contains("--help") || stderr.contains("help"),
            "Should suggest using help"
        );
    }

    // Test handling of empty queries
    #[test]
    fn test_empty_query_handling() {
        // Test with empty query
        let (success, stdout, stderr) = run_semisearch(&[""], None);

        // It should either provide help or a specific error
        if !success {
            assert!(
                stderr.contains("query")
                    || stderr.contains("search term")
                    || stderr.contains("empty"),
                "Should indicate empty query issue"
            );
            assert!(
                stderr.contains("help") || stderr.contains("example"),
                "Should suggest getting help"
            );
        } else {
            assert!(
                stdout.contains("help") || stdout.contains("usage") || stdout.contains("example"),
                "Should show help or usage information"
            );
        }
    }

    // Test handling of very large files
    #[test]
    fn test_large_file_handling() {
        // Create a temporary large file
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("large_file.txt");
        let mut file = fs::File::create(&file_path).expect("Failed to create temp file");

        // Write 10,000 lines to make a large file
        for i in 1..=10_000 {
            writeln!(file, "Line {} of large file test", i).expect("Failed to write to temp file");
        }

        // Add a unique search term near the end
        writeln!(file, "UNIQUE_SEARCH_TERM_12345").expect("Failed to write to temp file");

        // Test search in large file
        let (success, stdout, _stderr) =
            run_semisearch(&["UNIQUE_SEARCH_TERM_12345"], Some(temp_dir.path()));

        assert!(success, "Large file search should succeed");
        assert!(
            stdout.contains("UNIQUE_SEARCH_TERM_12345"),
            "Should find term in large file"
        );
    }

    // Test handling of binary files
    #[test]
    fn test_binary_file_handling() {
        // Create a temporary binary file
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("binary_file.bin");
        let mut file = fs::File::create(&file_path).expect("Failed to create temp file");

        // Write some binary data
        let binary_data = [0u8; 1024];
        file.write_all(&binary_data)
            .expect("Failed to write binary data");

        // Test search in binary file
        let (success, stdout, stderr) = run_semisearch(&["test"], Some(temp_dir.path()));

        assert!(
            success,
            "Binary file search should succeed or fail gracefully"
        );

        let combined_output = format!("{}\n{}", stdout, stderr);

        // If binary files are handled specially, check for appropriate messaging
        if combined_output.contains("binary") {
            assert!(
                combined_output.contains("skip") || combined_output.contains("ignore"),
                "Should indicate binary files are skipped"
            );
        }
    }

    // Test handling of permission denied errors
    #[test]
    fn test_permission_denied_handling() {
        // This test is OS-dependent and may not work in all environments
        // Create a temporary file with restricted permissions
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("restricted.txt");
        let mut file = fs::File::create(&file_path).expect("Failed to create temp file");

        writeln!(file, "This is a restricted file").expect("Failed to write to temp file");

        // Try to make the file unreadable - this may fail on some systems
        #[cfg(unix)]
        let _ = fs::set_permissions(&file_path, fs::Permissions::from_mode(0o000));

        // Test search in directory with permission issues
        let (success, _stdout, stderr) = run_semisearch(&["restricted"], Some(temp_dir.path()));

        // The search itself should succeed, but there might be warnings
        assert!(success, "Search with permission issues should succeed");

        // If permissions are an issue, check for user-friendly error
        if stderr.contains("permission") {
            assert!(
                stderr.contains("access") || stderr.contains("read"),
                "Should explain permission issue clearly"
            );
        }

        // Reset permissions to avoid issues
        #[cfg(unix)]
        let _ = fs::set_permissions(&file_path, fs::Permissions::from_mode(0o644));
    }

    // Test that the contextual help system works
    #[test]
    fn test_contextual_help_system() {
        // Test help-me command
        let (success, stdout, _stderr) = run_semisearch(&["help-me"], None);

        assert!(success, "Help-me command should succeed");
        assert!(
            stdout.contains("Welcome")
                || stdout.contains("help")
                || stdout.contains("guide")
                || stdout.contains("example"),
            "Should show welcome/help message"
        );

        // Test no results case for contextual help
        let (success, stdout, _stderr) = run_semisearch(&["xyz123impossible"], None);

        assert!(success, "No results search should still succeed");
        assert!(
            stdout.contains("Try") || stdout.contains("suggestion"),
            "Should provide contextual help for no results"
        );

        // Test too many results case (if implemented)
        let test_dir = Path::new("tests/test-data");
        let (success, stdout, _stderr) = run_semisearch(&["a"], Some(test_dir));

        assert!(success, "Search with common term should succeed");

        // If too many results handling is implemented, check for guidance
        if stdout.contains("many") || stdout.contains("lots") {
            assert!(
                stdout.contains("specific")
                    || stdout.contains("narrow")
                    || stdout.contains("refine"),
                "Should suggest narrowing search for too many results"
            );
        }
    }
}
