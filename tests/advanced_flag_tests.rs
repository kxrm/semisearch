use anyhow::Result;
use std::process::Command;

/// Test that simple mode hides advanced options by default
#[tokio::test]
async fn test_simple_mode_hides_advanced_options() -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--no-default-features",
            "--",
            "--help",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    // Should succeed
    assert!(output.status.success(), "Help command should work");

    // Should show simple interface (≤10 options)
    let option_count = stdout.matches("--").count();
    assert!(
        option_count <= 10,
        "Simple mode should show ≤10 options, found: {option_count}"
    );

    // Should not show advanced options
    assert!(
        !stdout.contains("--mode"),
        "Should not show --mode in simple mode"
    );
    assert!(
        !stdout.contains("--semantic-threshold"),
        "Should not show --semantic-threshold in simple mode"
    );
    assert!(
        !stdout.contains("--format"),
        "Should not show --format in simple mode"
    );
    assert!(
        !stdout.contains("--files-only"),
        "Should not show --files-only in simple mode"
    );

    // Should show basic options
    assert!(
        stdout.contains("--fuzzy") || stdout.contains("fuzzy"),
        "Should show fuzzy option"
    );
    assert!(
        stdout.contains("--exact") || stdout.contains("exact"),
        "Should show exact option"
    );

    Ok(())
}

/// Test that search subcommand shows simple interface by default
#[tokio::test]
async fn test_search_subcommand_simple_by_default() -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--no-default-features",
            "--",
            "search",
            "--help",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    // Should succeed
    assert!(output.status.success(), "Search help should work");

    // Should show simple search interface
    let option_count = stdout.matches("--").count();
    assert!(
        option_count <= 12,
        "Simple search should show ≤12 options, found: {option_count}"
    );

    // Should not show advanced options by default
    assert!(
        !stdout.contains("--mode"),
        "Should not show --mode in simple search"
    );
    assert!(
        !stdout.contains("--semantic-threshold"),
        "Should not show --semantic-threshold"
    );
    assert!(!stdout.contains("--context"), "Should not show --context");

    // Should show basic search options
    assert!(
        stdout.contains("--fuzzy") || stdout.contains("fuzzy"),
        "Should show fuzzy option"
    );
    assert!(
        stdout.contains("--exact") || stdout.contains("exact"),
        "Should show exact option"
    );
    assert!(
        stdout.contains("--path") || stdout.contains("path"),
        "Should show path option"
    );

    Ok(())
}

/// Test that --advanced flag reveals all options
#[tokio::test]
async fn test_advanced_flag_shows_all_options() -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--no-default-features",
            "--",
            "--advanced",
            "--help",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    // Should succeed
    assert!(output.status.success(), "Advanced help should work");

    // Should show more options than simple mode
    let option_count = stdout.matches("--").count();
    assert!(
        option_count > 10,
        "Advanced mode should show >10 options, found: {option_count}"
    );

    Ok(())
}

/// Test that advanced search subcommand shows all options
#[tokio::test]
async fn test_advanced_search_shows_all_options() -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--no-default-features",
            "--",
            "--advanced",
            "search",
            "--help",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    // Should succeed
    assert!(output.status.success(), "Advanced search help should work");

    // Should show advanced search options
    let option_count = stdout.matches("--").count();
    assert!(
        option_count > 10,
        "Advanced search should show >10 options, found: {option_count}"
    );

    Ok(())
}

/// Test that basic search functionality works in simple mode
#[tokio::test]
async fn test_simple_mode_search_functionality() -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--no-default-features",
            "--",
            "search",
            "TODO",
            "--path",
            ".",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed
    assert!(
        output.status.success(),
        "Simple search should work: {stderr}"
    );

    // Should show results or "No matches found"
    assert!(
        stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("matches"),
        "Should show search results: {stdout}"
    );

    Ok(())
}

/// Test that advanced flags work when advanced mode is enabled
#[tokio::test]
async fn test_advanced_flags_work_in_advanced_mode() -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--no-default-features",
            "--",
            "--advanced",
            "search",
            "TODO",
            "--path",
            ".",
            "--case-sensitive",
            "--typo-tolerance",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed
    assert!(
        output.status.success(),
        "Advanced search with flags should work: {stderr}"
    );

    // Should show results or "No matches found"
    assert!(
        stdout.contains("Found") || stdout.contains("No matches") || stdout.contains("matches"),
        "Should show search results: {stdout}"
    );

    Ok(())
}

/// Test that mode-specific advanced flags work
#[tokio::test]
async fn test_mode_specific_advanced_flags() -> Result<()> {
    // This test validates that advanced flags like --format, --semantic, etc. work
    // Test basic advanced functionality by running a command with advanced flags

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--no-default-features",
            "--",
            "--advanced",
            "search",
            "TODO",
            "--case-sensitive",
        ])
        .output()?;

    // Should succeed - this validates advanced flag parsing is working
    assert!(
        output.status.success(),
        "Advanced flags should be parsed correctly"
    );

    Ok(())
}

/// Test that help system respects advanced flag
#[tokio::test]
async fn test_help_system_respects_advanced_flag() -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--no-default-features",
            "--",
            "--help",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    // Simple help should not overwhelm users
    assert!(output.status.success(), "Help should work");
    assert!(stdout.contains("Usage"), "Should show usage information");

    // Should hint at advanced mode
    assert!(
        stdout.contains("advanced") || stdout.contains("Advanced"),
        "Should mention advanced mode"
    );

    Ok(())
}

/// Test version information
#[tokio::test]
async fn test_version_information() -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--no-default-features",
            "--",
            "--version",
        ])
        .output()?;

    // Should succeed and show version
    assert!(output.status.success(), "Version command should work");

    Ok(())
}

/// Test advanced version information
#[tokio::test]
async fn test_advanced_version_information() -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "semisearch-new",
            "--no-default-features",
            "--",
            "--advanced",
            "--version",
        ])
        .output()?;

    // Should succeed and show version
    assert!(
        output.status.success(),
        "Advanced version command should work"
    );

    Ok(())
}
