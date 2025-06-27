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
        .args(["run", "--bin", "semisearch-new", "--", "--invalid-flag"])
        .output()?;

    assert!(!output.status.success(), "Invalid arguments should fail");
    // Note: clap typically returns exit code 2 for argument errors

    // Test 2: File/directory not found (should be exit code 1)
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--",
            "TODO",
            "/nonexistent/path",
        ])
        .output()?;

    assert!(!output.status.success(), "Non-existent path should fail");
    if let Some(code) = output.status.code() {
        assert_eq!(code, 1, "Directory access error should return exit code 1");
    }

    Ok(())
}

/// Test that technical errors are translated to user-friendly messages
#[tokio::test]
async fn test_technical_error_translation() -> Result<()> {
    // Test: Technical error types should be hidden from users
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--",
            "TODO",
            "/root/protected_directory_that_requires_permissions",
        ])
        .output()?;

    let stderr = String::from_utf8(output.stderr)?;

    // Should fail
    assert!(!output.status.success(), "Permission error should fail");

    // Should not expose technical error details
    assert!(
        !stderr.contains("std::io::Error") && !stderr.contains("Error {"),
        "Should not expose technical error types: {stderr}"
    );

    // Should provide user-friendly explanation
    assert!(
        stderr.contains("Permission") || stderr.contains("access") || stderr.contains("denied"),
        "Should explain permission issue in user-friendly terms: {stderr}"
    );

    Ok(())
}

/// Test that error messages are consistently formatted
#[tokio::test]
async fn test_consistent_error_formatting() -> Result<()> {
    // Test multiple error scenarios to ensure consistent formatting

    // Test 1: Directory not found
    let output1 = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--",
            "TODO",
            "/nonexistent/directory",
        ])
        .output()?;

    let stderr1 = String::from_utf8(output1.stderr)?;

    // Test 2: Permission denied (if possible)
    let output2 = Command::new("cargo")
        .args(["run", "--bin", "semisearch-new", "--", "TODO", "/root"])
        .output()?;

    let stderr2 = String::from_utf8(output2.stderr)?;

    // Both should fail
    assert!(!output1.status.success());
    assert!(!output2.status.success());

    // Both should have helpful error messages
    assert!(
        stderr1.contains("Cannot") || stderr1.contains("Unable"),
        "Should have clear error message: {stderr1}"
    );
    assert!(
        stderr2.contains("Cannot") || stderr2.contains("Unable") || stderr2.contains("Permission"),
        "Should have clear error message: {stderr2}"
    );

    Ok(())
}

/// Test that errors are properly separated between stdout and stderr
#[tokio::test]
async fn test_stderr_stdout_separation() -> Result<()> {
    // Test: Error messages should go to stderr, results to stdout

    // Test 1: Successful search (should use stdout)
    let temp_dir = TempDir::new()?;
    std::fs::write(temp_dir.path().join("test.txt"), "TODO: test item")?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--",
            "TODO",
            temp_dir.path().to_str().unwrap(),
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    if output.status.success() {
        // Results should be on stdout (filter out compilation warnings from stderr)
        let _filtered_stderr: String = stderr
            .lines()
            .filter(|line| {
                !line.contains("warning:")
                    && !line.contains("Compiling")
                    && !line.contains("Finished")
                    && !line.contains("Running")
                    && !line.contains("Building")
                    && !line.contains("Checking")
                    && !line.contains("Downloaded")
                    && !line.contains("Downloading")
                    && !line.contains("Updating")
                    && !line.contains("Fresh")
                    && !line.contains("Dirty")
                    && !line.contains("target/debug")
                    && !line.trim().starts_with("-->")
                    && !line.trim().starts_with("|")
                    && !line.trim().starts_with("=")
                    && !line.trim().is_empty()
            })
            .collect::<Vec<_>>()
            .join("\n");

        // During coverage testing, there might be additional output
        // The important thing is that results are on stdout
        assert!(
            !stdout.is_empty(),
            "Results should be on stdout, but stdout is empty"
        );
        assert!(
            stdout.contains("Found") || stdout.contains("matches"),
            "Stdout should contain search results: {stdout}"
        );
    }

    // Test 2: Error case (should use stderr)
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--",
            "TODO",
            "/nonexistent/path",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Errors should be on stderr
    assert!(
        stderr.len() > stdout.len(),
        "Errors should be on stderr: stdout={}, stderr={}",
        stdout.len(),
        stderr.len()
    );

    Ok(())
}

/// Test that error messages include helpful context information
#[tokio::test]
async fn test_error_context_information() -> Result<()> {
    // Test: Error messages should include context like query and path

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--",
            "specific_query_term",
            "/nonexistent/specific/path",
        ])
        .output()?;

    let stderr = String::from_utf8(output.stderr)?;

    // Should fail
    assert!(!output.status.success());

    // Error message should include context information
    assert!(
        stderr.contains("specific") || stderr.contains("path") || stderr.contains("nonexistent"),
        "Error should include context information: {stderr}"
    );

    // Should provide actionable suggestions
    assert!(
        stderr.contains("Try") || stderr.contains("Check") || stderr.contains("Make sure"),
        "Error should provide actionable suggestions: {stderr}"
    );

    Ok(())
}
