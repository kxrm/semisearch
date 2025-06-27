use anyhow::Result;
use std::process::Command;

/// Test that natural path syntax works: semisearch "query" path/
#[tokio::test]
async fn test_natural_path_syntax() -> Result<()> {
    // Test: semisearch "TODO" src/ should work
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO",
            "src/",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed and search in src/ directory
    assert!(
        output.status.success(),
        "Natural path syntax failed: {}",
        stderr
    );
    assert!(
        stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("matches"),
        "Should handle natural path syntax: {}",
        stdout
    );

    Ok(())
}

/// Test that natural path works with current directory
#[tokio::test]
async fn test_natural_path_current_directory() -> Result<()> {
    // Test: semisearch "TODO" . should work
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO",
            ".",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed and search in current directory
    assert!(
        output.status.success(),
        "Current directory path failed: {}",
        stderr
    );
    assert!(
        stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("matches"),
        "Should handle current directory: {}",
        stdout
    );

    Ok(())
}

/// Test that natural path works with flags
#[tokio::test]
async fn test_natural_path_with_flags() -> Result<()> {
    // Test: semisearch "TODO" src/ --fuzzy should work
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO",
            "src/",
            "--fuzzy",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed with fuzzy search in src/
    assert!(
        output.status.success(),
        "Natural path with flags failed: {}",
        stderr
    );
    assert!(
        stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("matches"),
        "Should handle natural path with flags: {}",
        stdout
    );

    Ok(())
}

/// Test that --path flag still works for backward compatibility
#[tokio::test]
async fn test_path_flag_backward_compatibility() -> Result<()> {
    // Test: semisearch "TODO" --path src/ should still work
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO",
            "--path",
            "src/",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed with --path flag
    assert!(output.status.success(), "--path flag failed: {}", stderr);
    assert!(
        stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("matches"),
        "Should handle --path flag: {}",
        stdout
    );

    Ok(())
}

/// Test that explicit search subcommand works with natural path
#[tokio::test]
async fn test_explicit_search_with_natural_path() -> Result<()> {
    // Test: semisearch search "TODO" src/ should work
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "search",
            "TODO",
            "src/",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed with explicit search command
    assert!(
        output.status.success(),
        "Explicit search with natural path failed: {}",
        stderr
    );
    assert!(
        stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("matches"),
        "Should handle explicit search with natural path: {}",
        stdout
    );

    Ok(())
}

/// Test that multi-word queries work with natural path
#[tokio::test]
async fn test_multi_word_query_with_natural_path() -> Result<()> {
    // Test: semisearch "error handling" src/ should work
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "error handling",
            "src/",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed with multi-word query
    assert!(
        output.status.success(),
        "Multi-word query with natural path failed: {}",
        stderr
    );
    assert!(
        stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("matches"),
        "Should handle multi-word query with natural path: {}",
        stdout
    );

    Ok(())
}

/// Test that the UX examples from the plan work
#[tokio::test]
async fn test_ux_plan_examples() -> Result<()> {
    // Test the exact examples from Task 1.4.1
    let examples = [
        ("TODO", "src/"),
        ("error handling", "."),
        ("function", "tests/"),
    ];

    for (query, path) in examples {
        let output = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "semisearch-new",
                "--features",
                "neural-embeddings",
                "--",
                query,
                path,
            ])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        // Should succeed for all UX plan examples
        assert!(
            output.status.success(),
            "UX example '{} {}' failed: {}",
            query,
            path,
            stderr
        );
        assert!(
            stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("matches"),
            "Should handle UX example '{} {}': {}",
            query,
            path,
            stdout
        );
    }

    Ok(())
}

/// Test that help text reflects natural path syntax
#[tokio::test]
async fn test_help_text_includes_natural_path() -> Result<()> {
    // Test: semisearch search --help should mention natural path syntax
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "search",
            "--help",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    // Should mention natural path syntax in help
    assert!(output.status.success(), "Help command should work");
    assert!(
        stdout.contains("QUERY") && stdout.contains("PATH"),
        "Help should mention QUERY and PATH arguments: {}",
        stdout
    );

    Ok(())
}

/// Test that the natural path syntax is more intuitive than --path flag
#[tokio::test]
async fn test_natural_path_intuitiveness() -> Result<()> {
    // Compare natural path vs --path flag usage
    let natural_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO",
            "src/",
        ])
        .output()?;

    let flag_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO",
            "--path",
            "src/",
        ])
        .output()?;

    // Both should work identically
    assert!(natural_output.status.success(), "Natural path should work");
    assert!(flag_output.status.success(), "--path flag should work");

    // Results should be similar (allowing for timing differences)
    let natural_stdout = String::from_utf8(natural_output.stdout)?;
    let flag_stdout = String::from_utf8(flag_output.stdout)?;

    assert!(
        natural_stdout.contains("Found") || natural_stdout.contains("No matches"),
        "Natural path should produce results"
    );
    assert!(
        flag_stdout.contains("Found") || flag_stdout.contains("No matches"),
        "--path flag should produce results"
    );

    Ok(())
}
