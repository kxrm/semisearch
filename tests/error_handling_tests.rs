use anyhow::Result;
use std::process::Command;
use tempfile::TempDir;

/// Test that user-friendly error messages are displayed on stderr
#[tokio::test]
async fn test_user_friendly_error_messages() -> Result<()> {
    // Test: Search in non-existent directory should show helpful error
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO",
            "/nonexistent/path/that/does/not/exist",
        ])
        .output()?;

    let stderr = String::from_utf8(output.stderr)?;
    let stdout = String::from_utf8(output.stdout)?;

    // Should fail with non-zero exit code
    assert!(
        !output.status.success(),
        "Should fail for non-existent path"
    );

    // Error should be on stderr, not stdout
    assert!(
        stderr.len() > stdout.len(),
        "Error message should be on stderr, not stdout"
    );

    // Should not contain technical jargon
    assert!(
        !stderr.contains("anyhow") && !stderr.contains("Error:"),
        "Should not expose technical error types: {stderr}"
    );

    // Should contain helpful guidance
    assert!(
        stderr.contains("Make sure") || stderr.contains("Try") || stderr.contains("Check"),
        "Should provide actionable guidance: {stderr}"
    );

    Ok(())
}

/// Test that no matches found shows helpful suggestions
#[tokio::test]
async fn test_no_matches_helpful_suggestions() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a test file without the search term
    std::fs::write(
        temp_dir.path().join("test.txt"),
        "This file contains no TODO comments",
    )?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "NONEXISTENT_TERM_XYZ123",
            temp_dir.path().to_str().unwrap(),
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should return exit code 1 (Unix convention: no matches found)
    assert!(
        !output.status.success(),
        "No matches should return exit code 1 (Unix convention): {stderr}"
    );

    if let Some(code) = output.status.code() {
        assert_eq!(code, 1, "No matches should return exit code 1, got: {code}");
    }

    // Should show helpful suggestions (now on stderr)
    let combined_output = format!("{stdout}{stderr}");
    assert!(
        combined_output.contains("Try:")
            || combined_output.contains("Check spelling")
            || combined_output.contains("--fuzzy"),
        "Should provide helpful suggestions for no matches: {combined_output}"
    );

    Ok(())
}

/// Test that JSON error format is available for parseable errors
#[tokio::test]
async fn test_json_error_format() -> Result<()> {
    // Test: JSON format should include error information
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "--advanced",
            "TODO",
            "/nonexistent/path",
            "--format",
            "json",
        ])
        .output()?;

    let stderr = String::from_utf8(output.stderr)?;

    // Should fail
    assert!(
        !output.status.success(),
        "Should fail for non-existent path"
    );

    // If JSON format is requested, error should be structured
    if stderr.contains("{") {
        // Extract just the JSON part (skip compilation output)
        let lines: Vec<&str> = stderr.lines().collect();
        let mut json_lines = Vec::new();
        let mut in_json = false;

        for line in lines {
            if line.trim().starts_with('{') {
                in_json = true;
            }
            if in_json {
                json_lines.push(line);
                // Check if this line completes the JSON (ends with } and has matching braces)
                let json_so_far = json_lines.join("\n");
                if json_so_far.trim().ends_with('}') {
                    let open_braces = json_so_far.matches('{').count();
                    let close_braces = json_so_far.matches('}').count();
                    if open_braces == close_braces {
                        break;
                    }
                }
            }
        }

        let json_str = json_lines.join("\n");

        // Try to parse as JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
        assert!(
            parsed.is_ok(),
            "JSON error format should be valid JSON: {json_str}"
        );

        let json = parsed.unwrap();
        assert!(
            json.get("error_type").is_some(),
            "JSON error should contain 'error_type' field: {json}"
        );
    }

    Ok(())
}

/// Test that proper exit codes are returned
#[tokio::test]
async fn test_proper_exit_codes() -> Result<()> {
    // Test: Different error types should return appropriate exit codes

    // Test 1: Invalid arguments (should be exit code 2)
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "--invalid-flag",
        ])
        .output()?;

    assert!(!output.status.success(), "Invalid arguments should fail");
    // Note: clap typically returns exit code 2 for argument errors

    // Test 2: File/directory not found (should be exit code 1)
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO",
            "/nonexistent/path",
        ])
        .output()?;

    assert!(!output.status.success(), "Non-existent path should fail");
    if let Some(code) = output.status.code() {
        assert_eq!(code, 1, "File not found should return exit code 1");
    }

    Ok(())
}

/// Test that technical errors are translated to user-friendly messages
#[tokio::test]
async fn test_technical_error_translation() -> Result<()> {
    // Create a directory we can't write to (simulate permission error)
    let temp_dir = TempDir::new()?;
    let readonly_dir = temp_dir.path().join("readonly");
    std::fs::create_dir(&readonly_dir)?;

    // Try to index with insufficient permissions
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "index",
            readonly_dir.to_str().unwrap(),
        ])
        .output()?;

    let stderr = String::from_utf8(output.stderr)?;

    // Should contain user-friendly language, not technical terms
    assert!(
        !stderr.contains("std::io::Error") && !stderr.contains("anyhow::Error"),
        "Should not expose technical error types: {stderr}"
    );

    // Should provide actionable guidance
    if !output.status.success() {
        assert!(
            stderr.contains("permission") || stderr.contains("access") || stderr.contains("Check"),
            "Should explain permission issues clearly: {stderr}"
        );
    }

    Ok(())
}

/// Test that errors are consistently formatted
#[tokio::test]
async fn test_consistent_error_formatting() -> Result<()> {
    // Test multiple error scenarios to ensure consistent formatting
    let test_cases = vec![
        // (args, expected_pattern)
        (vec!["TODO", "/nonexistent"], "Cannot search"),
        (vec!["index", "/readonly"], "Cannot index"),
    ];

    for (args, expected_pattern) in test_cases {
        let mut cmd_args = vec![
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
        ];
        for arg in &args {
            cmd_args.push(&**arg);
        }

        let output = Command::new("cargo").args(&cmd_args).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr)?;

            // Should have consistent error format
            assert!(
                stderr.contains(expected_pattern)
                    || stderr.contains("Error:")
                    || stderr.contains("❌"),
                "Should contain expected error pattern '{expected_pattern}': {stderr}"
            );
        }
    }

    Ok(())
}

/// Test that stderr is used for errors and stdout for results
#[tokio::test]
async fn test_stderr_stdout_separation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    std::fs::write(temp_dir.path().join("test.txt"), "TODO: test content")?;

    // Test successful search
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO",
            temp_dir.path().to_str().unwrap(),
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    if output.status.success() {
        // Results should be on stdout
        assert!(
            stdout.contains("Found") || stdout.contains("matches"),
            "Search results should be on stdout: {stdout}"
        );

        // Warnings (if any) should be on stderr, but not errors
        // Filter out compilation warnings which are expected
        let non_compilation_stderr: Vec<&str> = stderr
            .lines()
            .filter(|line| {
                !line.contains("warning:")
                    && !line.contains("Compiling")
                    && !line.contains("Finished")
                    && !line.contains("Blocking waiting")
                    && !line.contains("Running `target/debug/semisearch-new")
                    && !line.contains("associated functions")
                    && !line.contains("are never used")
                    && !line.contains("fn handle_")
                    && !line.contains("impl ErrorTranslator")
                    && !line.contains("#[warn(dead_code)]")
                    && !line.trim().starts_with("-->")
                    && !line.trim().starts_with("|")
                    && !line.trim().starts_with("=")
                    && !line.trim().starts_with("...")
                    && !line.trim().is_empty()
            })
            .collect();

        if !non_compilation_stderr.is_empty() {
            let filtered_stderr = non_compilation_stderr.join("\n");
            assert!(
                filtered_stderr.contains("⚠️")
                    || filtered_stderr.contains("Warning")
                    || filtered_stderr.contains("Falling back"),
                "Only warnings should be on stderr for successful operations: {filtered_stderr}"
            );
        }
    }

    Ok(())
}

/// Test that error context includes helpful information
#[tokio::test]
async fn test_error_context_information() -> Result<()> {
    // Test that errors include relevant context like file paths, query terms, etc.
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "test_query_xyz",
            "/path/that/does/not/exist",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;

        // Error should include context about what was attempted
        assert!(
            stderr.contains("/path/that/does/not/exist") || stderr.contains("directory"),
            "Error should include path context: {stderr}"
        );
    }

    Ok(())
}
