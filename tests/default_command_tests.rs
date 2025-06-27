use anyhow::Result;
use std::process::Command;

/// Test that direct query works without subcommand
#[tokio::test]
async fn test_direct_query_without_subcommand() -> Result<()> {
    // Test: semisearch "TODO" should work (no explicit "search" subcommand)
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed and find results
    assert!(output.status.success(), "Direct query failed: {}", stderr);
    assert!(
        stdout.contains("Found") || stdout.contains("matches") || stdout.contains("TODO"),
        "Should find TODO results: {}",
        stdout
    );

    Ok(())
}

/// Test that direct query with flags works
#[tokio::test]
async fn test_direct_query_with_flags() -> Result<()> {
    // Test: semisearch "TODO" --fuzzy should work
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO",
            "--fuzzy",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed with fuzzy search
    assert!(
        output.status.success(),
        "Direct query with flags failed: {}",
        stderr
    );
    assert!(
        stdout.contains("Found") || stdout.contains("matches") || stdout.contains("TODO"),
        "Should find TODO results with fuzzy: {}",
        stdout
    );

    Ok(())
}

/// Test that explicit search subcommand still works
#[tokio::test]
async fn test_explicit_search_subcommand_still_works() -> Result<()> {
    // Test: semisearch search "TODO" should still work
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
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed
    assert!(
        output.status.success(),
        "Explicit search subcommand failed: {}",
        stderr
    );
    assert!(
        stdout.contains("Found") || stdout.contains("matches") || stdout.contains("TODO"),
        "Should find TODO results with explicit search: {}",
        stdout
    );

    Ok(())
}

/// Test that non-search commands are not affected
#[tokio::test]
async fn test_non_search_commands_unaffected() -> Result<()> {
    // Test: semisearch status should still work
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "status",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed and show status
    assert!(output.status.success(), "Status command failed: {}", stderr);
    assert!(
        stdout.contains("Health Check") || stdout.contains("Ready") || stdout.contains("Available"),
        "Should show status information: {}",
        stdout
    );

    Ok(())
}

/// Test that help commands are not affected
#[tokio::test]
async fn test_help_commands_unaffected() -> Result<()> {
    // Test: semisearch help-me should still work
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "help-me",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed and show help
    assert!(
        output.status.success(),
        "Help-me command failed: {}",
        stderr
    );
    assert!(
        stdout.contains("Welcome") || stdout.contains("Usage") || stdout.contains("Examples"),
        "Should show help information: {}",
        stdout
    );

    Ok(())
}

/// Test that quoted queries work correctly
#[tokio::test]
async fn test_quoted_queries() -> Result<()> {
    // Test: semisearch "error handling" should work
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "error handling",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed (even if no results found)
    assert!(output.status.success(), "Quoted query failed: {}", stderr);
    // Should either find results or show "No matches found"
    assert!(
        stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("matches"),
        "Should handle quoted query: {}",
        stdout
    );

    Ok(())
}

/// Test that queries with special characters work
#[tokio::test]
async fn test_special_character_queries() -> Result<()> {
    // Test: semisearch "function()" should work
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "function()",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed
    assert!(
        output.status.success(),
        "Special character query failed: {}",
        stderr
    );
    assert!(
        stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("matches"),
        "Should handle special characters: {}",
        stdout
    );

    Ok(())
}

/// Test that path specification works with direct queries
#[tokio::test]
async fn test_direct_query_with_path() -> Result<()> {
    // Test: semisearch "TODO" --path src/ should work
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

    // Should succeed
    assert!(
        output.status.success(),
        "Direct query with path failed: {}",
        stderr
    );
    assert!(
        stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("matches"),
        "Should handle direct query with path: {}",
        stdout
    );

    Ok(())
}

/// Test that advanced mode works with direct queries
#[tokio::test]
async fn test_direct_query_with_advanced_mode() -> Result<()> {
    // Test: semisearch --advanced "TODO" --format json should work
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
            "--format",
            "json",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed
    assert!(
        output.status.success(),
        "Advanced direct query failed: {}",
        stderr
    );
    // Should output JSON or show no results message
    assert!(
        stdout.contains("[") || stdout.contains("No matches") || stdout.contains("Found"),
        "Should handle advanced direct query: {}",
        stdout
    );

    Ok(())
}

/// Test edge cases and error handling
#[tokio::test]
async fn test_edge_cases() -> Result<()> {
    // Test empty query handling - it should work and return all matches
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Empty query should succeed and return matches (matches everything)
    assert!(
        output.status.success(),
        "Empty query should succeed: {}",
        stderr
    );
    assert!(
        stdout.contains("Found") || stdout.contains("matches"),
        "Empty query should find matches: {}",
        stdout
    );

    Ok(())
}

/// Test that conflicting command detection works
#[tokio::test]
async fn test_command_conflict_detection() -> Result<()> {
    // This tests the logic that determines if first arg is a query or command

    // Known commands should be treated as commands
    let commands = ["search", "status", "help-me", "index", "config", "doctor"];

    for command in &commands {
        let output = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "semisearch-new",
                "--features",
                "neural-embeddings",
                "--",
                command,
                "--help",
            ])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        // Should succeed and show command-specific help
        assert!(
            output.status.success(),
            "Command {} --help failed: {}",
            command,
            stderr
        );
        assert!(
            stdout.contains("Usage") || stdout.contains("Options") || stdout.contains("help"),
            "Should show help for command {}: {}",
            command,
            stdout
        );
    }

    Ok(())
}

/// Test that the default behavior preserves all functionality
#[tokio::test]
async fn test_functionality_preservation() -> Result<()> {
    // Test that all search functionality still works with direct queries

    // Test basic search
    let basic = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO",
        ])
        .output()?;
    assert!(basic.status.success(), "Basic direct search should work");

    // Test fuzzy search
    let fuzzy = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "databse",
            "--fuzzy",
        ])
        .output()?;
    assert!(fuzzy.status.success(), "Fuzzy direct search should work");

    // Test exact search
    let exact = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO:",
            "--exact",
        ])
        .output()?;
    assert!(exact.status.success(), "Exact direct search should work");

    // Test with score threshold
    let score = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO",
            "--score",
            "0.5",
        ])
        .output()?;
    assert!(
        score.status.success(),
        "Direct search with score should work"
    );

    // Test with limit
    let limit = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--features",
            "neural-embeddings",
            "--",
            "TODO",
            "--limit",
            "5",
        ])
        .output()?;
    assert!(
        limit.status.success(),
        "Direct search with limit should work"
    );

    Ok(())
}
