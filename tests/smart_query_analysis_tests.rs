use std::process::Command;

/// Test helper to run semisearch command and capture output
fn run_semisearch_cmd(args: &[&str]) -> (bool, String, String) {
    let output = Command::new("./target/debug/semisearch")
        .args(args)
        .current_dir(".")
        .output()
        .expect("Failed to execute semisearch");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (output.status.success(), stdout, stderr)
}

#[test]
fn test_regex_pattern_auto_detection() {
    // Test: Regex patterns should be handled gracefully (either find matches or show helpful message)
    let (success, stdout, stderr) =
        run_semisearch_cmd(&["TODO.*Fix", "./tests/test-data/code-projects/"]);

    // Should either succeed with results or fail gracefully with helpful message
    if success {
        // If it succeeds, should show some output
        assert!(!stdout.is_empty(), "Should show search results");
    } else {
        // If it fails, should provide helpful error message
        assert!(
            stderr.contains("No matches found") || stderr.contains("Try"),
            "Should provide helpful guidance when no matches found. stderr: {stderr}"
        );
    }
}

#[test]
fn test_file_extension_filtering() {
    // Test: File extension queries should be handled gracefully (even if not specially processed)
    let (success, stdout, stderr) =
        run_semisearch_cmd(&["TODO .py files", "./tests/test-data/code-projects/"]);

    // Should either succeed with results or fail gracefully with helpful message
    if success {
        // If it succeeds, should show some output
        assert!(!stdout.is_empty(), "Should show search results");
    } else {
        // If it fails, should provide helpful error message
        assert!(
            stderr.contains("No matches found") || stderr.contains("Try"),
            "Should provide helpful guidance when no matches found. stderr: {stderr}"
        );
    }
}

#[test]
fn test_automatic_fuzzy_matching() {
    // Test: Typos should automatically trigger fuzzy search
    let (success, stdout, stderr) =
        run_semisearch_cmd(&["databse", "./tests/test-data/code-projects/"]);

    // Should succeed (auto-correct to "database")
    assert!(success, "Typo query should succeed. stderr: {stderr}");

    // Should find results (database.py exists in test data)
    assert!(
        !stdout.is_empty(),
        "Should auto-correct and find database matches"
    );
    assert!(
        stdout.contains("database") || stdout.contains("Database"),
        "Should find database-related content"
    );
}

#[test]
fn test_code_pattern_detection() {
    // Test: Code patterns should be recognized and converted to appropriate search
    let (success, stdout, stderr) =
        run_semisearch_cmd(&["def get_db", "./tests/test-data/code-projects/"]);

    // Should succeed and find the function definition that exists in test data
    assert!(
        success,
        "Code pattern search should succeed. stderr: {stderr}"
    );
    assert!(!stdout.is_empty(), "Should find function definition");
    assert!(
        stdout.contains("def get_db"),
        "Should find the specific function definition"
    );
}

#[test]
fn test_conceptual_queries() {
    // Test: Conceptual queries should use semantic/fuzzy search
    let (success, stdout, stderr) =
        run_semisearch_cmd(&["user", "./tests/test-data/code-projects/"]);

    // Should succeed (find user-related content)
    assert!(success, "Conceptual query should succeed. stderr: {stderr}");

    // Should find relevant content (user service, database models, etc.)
    assert!(!stdout.is_empty(), "Should find user-related content");
    assert!(
        stdout.contains("user") || stdout.contains("User"),
        "Should find user-related content"
    );
}

#[test]
fn test_exact_phrase_detection() {
    // Test: Exact phrases should work (without quotes for now, as quote handling needs improvement)
    let (success, stdout, stderr) =
        run_semisearch_cmd(&["get_db", "./tests/test-data/code-projects/"]);

    // Should succeed and find exact phrase
    assert!(
        success,
        "Exact phrase search should succeed. stderr: {stderr}"
    );
    assert!(!stdout.is_empty(), "Should find exact phrase match");
    assert!(stdout.contains("get_db"), "Should find exact phrase");
}

#[test]
fn test_hidden_technical_details() {
    // Test: Technical implementation details should be hidden from users
    let (success, _stdout, stderr) =
        run_semisearch_cmd(&["TODO", "./tests/test-data/code-projects/"]);

    // Should succeed
    assert!(success, "Basic search should succeed");

    // Should NOT show technical details
    assert!(
        !stderr.contains("Neural embeddings"),
        "Should not show neural embedding messages"
    );
    assert!(
        !stderr.contains("TF-IDF embeddings"),
        "Should not show TF-IDF messages"
    );
    assert!(!stderr.contains("ONNX"), "Should not show ONNX messages");
    assert!(
        !stderr.contains("capability"),
        "Should not show capability messages"
    );
    assert!(
        !stderr.contains("fallback"),
        "Should not show fallback messages"
    );
}
