use search::errors::{ErrorTranslator, UserError};
use std::process::Command;
use tempfile::TempDir;

#[tokio::test]
async fn test_enhanced_error_translation_coverage() {
    // Test that ErrorTranslator handles various technical error types

    // Test ONNX/Neural errors
    let onnx_error = anyhow::anyhow!("ONNX Runtime initialization failed");
    let translated = ErrorTranslator::translate_technical_error(&onnx_error);
    match translated {
        UserError::FallbackMode { .. } => {
            // Should provide helpful fallback information
            let display = format!("{translated}");
            assert!(display.contains("basic mode"));
            assert!(display.contains("tip") || display.contains("Tip"));
        }
        _ => panic!("Expected FallbackMode error for ONNX error"),
    }

    // Test permission errors
    let permission_error = anyhow::anyhow!("Permission denied accessing /restricted/path");
    let translated = ErrorTranslator::translate_technical_error(&permission_error);
    match translated {
        UserError::Permission { path, .. } => {
            assert!(path.contains("restricted") || path.contains("path"));
        }
        _ => panic!("Expected Permission error for permission denied"),
    }

    // Test database errors
    let db_error = anyhow::anyhow!("SQLite database is locked");
    let translated = ErrorTranslator::translate_technical_error(&db_error);
    match translated {
        UserError::Database { .. } => {
            let display = format!("{translated}");
            assert!(display.contains("database"));
        }
        _ => panic!("Expected Database error for SQLite error"),
    }
}

#[tokio::test]
async fn test_error_display_point_integration() {
    // Test that all major error display points use user-friendly errors

    let _temp_dir = TempDir::new().unwrap();

    // Test main binary error handling
    let output = Command::new("./target/debug/semisearch-new")
        .arg("test_query")
        .arg("/completely/nonexistent/path/12345")
        .output()
        .expect("Failed to execute command");

    // Should exit with error code
    assert!(!output.status.success());

    // Error should be user-friendly (on stderr)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("anyhow"));
    assert!(!stderr.contains("Error:"));
    assert!(stderr.contains("Cannot search") || stderr.contains("directory"));
    assert!(stderr.contains("Try:") || stderr.contains("suggestion"));
}

#[tokio::test]
async fn test_error_context_preservation() {
    // Test that error translation preserves important context

    let query = "test_query_12345";
    let path = "/test/path/nonexistent";

    // Create error with query context
    let error = anyhow::anyhow!("No results found for query: {}", query);
    let translated =
        ErrorTranslator::translate_technical_error_with_context(&error, Some(query), Some(path));

    match translated {
        UserError::NoMatches {
            query: preserved_query,
            ..
        } => {
            assert_eq!(preserved_query, query);
        }
        _ => {
            // Should at least preserve context in suggestions
            let display = format!("{translated}");
            assert!(display.contains(query) || display.contains("query"));
        }
    }
}

#[tokio::test]
async fn test_cascading_error_handling() {
    // Test that errors are properly handled at all levels

    // Test search operation errors
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    // Create a file with restricted permissions
    let restricted_file = temp_dir.path().join("restricted.txt");
    std::fs::write(&restricted_file, "test content").unwrap();

    // Try to search with invalid parameters
    let output = Command::new("./target/debug/semisearch-new")
        .arg("--advanced")
        .arg("search")
        .arg("test")
        .arg("--mode")
        .arg("invalid_mode")
        .arg(temp_path)
        .output()
        .expect("Failed to execute command");

    // Should handle gracefully with user-friendly error
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains("panic"));
        assert!(!stderr.contains("unwrap"));
        // Should provide helpful guidance
        assert!(
            stderr.contains("Try:") || stderr.contains("suggestion") || stderr.contains("help")
        );
    }
}

#[tokio::test]
async fn test_error_message_consistency() {
    // Test that error messages are consistent across different entry points

    let test_cases = vec![
        (
            "./target/debug/semisearch-new",
            vec!["nonexistent_query", "/bad/path"],
        ),
        (
            "./target/debug/semisearch-new",
            vec!["--advanced", "search", "query", "/bad/path"],
        ),
    ];

    for (binary, args) in test_cases {
        let output = Command::new(binary)
            .args(args)
            .output()
            .expect("Failed to execute command");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // All errors should be user-friendly
            assert!(!stderr.contains("anyhow::Error"));
            assert!(!stderr.contains("thread 'main' panicked"));

            // Should have consistent structure
            assert!(
                stderr.contains("Cannot")
                    || stderr.contains("No matches")
                    || stderr.contains("Invalid")
            );

            // Should provide suggestions
            assert!(stderr.contains("Try:") || stderr.contains("suggestion"));
        }
    }
}

#[tokio::test]
async fn test_json_error_format_enhanced() {
    // Test enhanced JSON error format with more error types

    let test_cases = vec![
        ("/nonexistent/path", "DirectoryAccess"),
        ("/proc/1/mem", "Permission"), // Likely to cause permission error
    ];

    for (path, expected_error_type) in test_cases {
        let output = Command::new("./target/debug/semisearch-new")
            .args(["--advanced", "search", "test", path, "--format", "json"])
            .output()
            .expect("Failed to execute command");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Should be valid JSON
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&stderr) {
                // Should have error_type field
                assert!(json_value.get("error_type").is_some());

                // Should have details
                assert!(json_value.get("details").is_some());

                // Error type should match expected
                if let Some(error_type) = json_value.get("error_type").and_then(|v| v.as_str()) {
                    if error_type == expected_error_type {
                        // Verify details structure
                        let details = json_value.get("details").unwrap();
                        assert!(details.get("suggestions").is_some());
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn test_error_recovery_suggestions() {
    // Test that error recovery suggestions are actionable and specific

    // Test 1: Directory access error
    let output = Command::new("./target/debug/semisearch-new")
        .args(["impossible_query_xyz_123", "/nonexistent"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should contain specific, actionable suggestions for directory errors
    let suggestions_found = stderr.contains("semisearch")
        && (stderr.contains("Try:") || stderr.contains("Make sure") || stderr.contains("Check"));

    assert!(
        suggestions_found,
        "Directory error should contain actionable suggestions. Got: {stderr}"
    );

    // Test 2: No results error (search in valid directory)
    let temp_dir = tempfile::TempDir::new().unwrap();
    let output = Command::new("./target/debug/semisearch-new")
        .args([
            "impossible_query_xyz_123",
            temp_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // For no results, should provide search-specific suggestions
    if stdout.contains("No matches found") || stderr.contains("No matches found") {
        let combined_output = format!("{stdout}{stderr}");
        let suggestions_found = combined_output.contains("semisearch")
            && (combined_output.contains("--fuzzy") || combined_output.contains("Try:"));

        assert!(
            suggestions_found,
            "No results should contain search suggestions. Got stdout: {stdout}, stderr: {stderr}"
        );
    }

    // Should not contain generic unhelpful messages
    assert!(!stderr.contains("something went wrong"));
    assert!(!stderr.contains("an error occurred"));
    assert!(!stderr.contains("please try again"));
}
