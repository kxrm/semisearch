use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test automatic typo correction and user-friendly error messages
/// According to UX Remediation Plan Tasks 1.2 and 1.3

#[test]
fn test_typo_correction_suggestions() {
    // Test: When no results found, should suggest fuzzy search
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("test.txt"),
        "database connection error",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_semisearch"))
        .args(["xyz123impossible"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should suggest fuzzy search when no results found
    assert!(
        stderr.contains("--fuzzy") || stderr.contains("Check spelling"),
        "Should suggest fuzzy search for typos. stderr: {stderr}"
    );
}

#[test]
fn test_automatic_typo_correction_with_suggestions() {
    // Test: Should provide helpful suggestions when automatic correction works
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("test.txt"),
        "database connection\nfunction call",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_semisearch"))
        .args(["databse", "--limit", "1"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should find "database" via fuzzy matching
    assert!(
        stdout.contains("database") || stdout.contains("Found"),
        "Should find database via fuzzy matching. stdout: {stdout}"
    );
}

#[test]
fn test_helpful_error_messages_for_no_results() {
    // Test: Error messages should be user-friendly, not technical
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "some content here").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_semisearch"))
        .args(["nonexistentterm123"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Should provide helpful suggestions, not technical errors
    assert!(
        !combined_output.contains("anyhow") && !combined_output.contains("Error:"),
        "Should not show technical error details. output: {combined_output}"
    );

    // Should suggest actionable steps
    assert!(
        combined_output.contains("Try:")
            || combined_output.contains("💡")
            || combined_output.contains("Tip:"),
        "Should provide actionable suggestions. output: {combined_output}"
    );
}

#[test]
fn test_typo_correction_preserves_context() {
    // Test: Typo correction should work while preserving search context
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("code.rs"),
        "fn process_data() { println!(\"processing\"); }",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_semisearch"))
        .args(["processs", "--limit", "1"]) // Extra 's' typo
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should find "process" and show the context
    assert!(
        stdout.contains("process") && (stdout.contains("fn") || stdout.contains("processing")),
        "Should find process with context. stdout: {stdout}"
    );
}

#[test]
fn test_progressive_typo_suggestions() {
    // Test: Should provide increasingly helpful suggestions for difficult cases
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "authentication system").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_semisearch"))
        .args(["authenitcation"]) // Multiple typos
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Should either find it via fuzzy or suggest simpler terms
    assert!(
        combined_output.contains("authentication")
            || combined_output.contains("simpler terms")
            || combined_output.contains("Try:"),
        "Should handle multiple typos gracefully. output: {combined_output}"
    );
}

#[test]
fn test_context_aware_typo_suggestions() {
    // Test: Suggestions should be context-aware (code vs docs)
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"",
    )
    .unwrap();
    fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    fs::write(
        temp_dir.path().join("src/main.rs"),
        "fn main() { println!(\"function\"); }",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_semisearch"))
        .args(["fuction", "--limit", "1"]) // Typo for "function"
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should find "function" in Rust context
    assert!(
        stdout.contains("function") || stdout.contains("fn"),
        "Should find function-related content. stdout: {stdout}"
    );
}

#[test]
fn test_no_technical_jargon_in_errors() {
    // Test: Error messages should not contain technical implementation details
    let temp_dir = TempDir::new().unwrap();
    // Create empty directory to trigger "no results" scenario

    let output = Command::new(env!("CARGO_BIN_EXE_semisearch"))
        .args(["impossiblequery123"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Should not contain technical terms
    let technical_terms = ["anyhow", "Result", "Error", "panic", "unwrap", "expect"];
    for term in &technical_terms {
        assert!(
            !combined_output.contains(term),
            "Should not contain technical term '{term}'. output: {combined_output}"
        );
    }
}

#[test]
fn test_fuzzy_search_quality() {
    // Test: Fuzzy search should find reasonable matches for common typos
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("test.txt"),
        "configuration settings\ndatabase connection\nuser authentication\nfunction definition",
    )
    .unwrap();

    let test_cases = [
        ("confguration", "configuration"),
        ("databse", "database"),
        ("authentcation", "authentication"),
        ("fuction", "function"),
    ];

    for (typo, expected) in &test_cases {
        let output = Command::new(env!("CARGO_BIN_EXE_semisearch"))
            .args([typo, "--limit", "1"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            stdout.contains(expected) || stdout.contains("Found"),
            "Typo '{typo}' should find '{expected}'. stdout: {stdout}"
        );
    }
}
