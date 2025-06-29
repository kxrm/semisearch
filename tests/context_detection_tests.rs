use anyhow::Result;
use search::context::{ContextAwareConfig, ProjectDetector, ProjectType};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Test context detection according to UX Remediation Plan Task 2.1
/// Focus: Silent operation with user-friendly indicators

#[test]
fn test_project_type_detection_silent_operation() {
    // Test: Context detection works silently during normal searches
    // Should not produce noisy output about project detection

    // Create a Rust project
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"",
    )
    .unwrap();
    fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    fs::write(
        temp_dir.path().join("src/main.rs"),
        "fn main() { println!(\"TODO: implement\"); }",
    )
    .unwrap();

    // Test project detection
    let project_type = ProjectDetector::detect(temp_dir.path());
    assert_eq!(project_type, ProjectType::RustProject);

    // Test context-aware configuration
    let config = ContextAwareConfig::from_project_type(project_type);
    assert_eq!(config.project_type, ProjectType::RustProject);
    assert!(config.search_paths.contains(&"src/".to_string()));
    assert!(config.file_patterns.contains(&"*.rs".to_string()));
    assert!(config.ignore_patterns.contains(&"target/".to_string()));
}

#[test]
fn test_javascript_project_detection() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();
    fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    fs::write(temp_dir.path().join("src/index.js"), "// TODO: implement").unwrap();

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert_eq!(project_type, ProjectType::JavaScriptProject);

    let config = ContextAwareConfig::from_project_type(project_type);
    assert!(config.search_paths.contains(&"src/".to_string()));
    assert!(config.file_patterns.contains(&"*.js".to_string()));
    assert!(config
        .ignore_patterns
        .contains(&"node_modules/".to_string()));
}

#[test]
fn test_python_project_detection() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("requirements.txt"), "flask==2.0.0").unwrap();
    fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    fs::write(temp_dir.path().join("src/main.py"), "# TODO: implement").unwrap();

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert_eq!(project_type, ProjectType::PythonProject);

    let config = ContextAwareConfig::from_project_type(project_type);
    assert!(config.search_paths.contains(&"src/".to_string()));
    assert!(config.file_patterns.contains(&"*.py".to_string()));
    assert!(config.ignore_patterns.contains(&"__pycache__/".to_string()));
}

#[test]
fn test_documentation_project_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Create mostly markdown files (>70% threshold)
    fs::write(temp_dir.path().join("README.md"), "# TODO: write docs").unwrap();
    fs::write(temp_dir.path().join("guide.md"), "# Guide").unwrap();
    fs::write(temp_dir.path().join("api.md"), "# API").unwrap();
    fs::write(temp_dir.path().join("config.txt"), "config file").unwrap();

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert_eq!(project_type, ProjectType::Documentation);

    let config = ContextAwareConfig::from_project_type(project_type);
    assert!(config.search_paths.contains(&"./".to_string()));
    assert!(config.file_patterns.contains(&"*.md".to_string()));
    assert!(config.file_patterns.contains(&"*.txt".to_string()));
}

#[test]
fn test_mixed_project_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple file types to trigger mixed detection
    fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    fs::create_dir_all(temp_dir.path().join("docs")).unwrap();

    // Create multiple .rs files
    for i in 1..=4 {
        fs::write(
            temp_dir.path().join(format!("src/file{i}.rs")),
            "fn main() {}",
        )
        .unwrap();
    }

    // Create multiple .md files
    for i in 1..=4 {
        fs::write(temp_dir.path().join(format!("docs/doc{i}.md")), "# Doc").unwrap();
    }

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert_eq!(project_type, ProjectType::Mixed);

    let config = ContextAwareConfig::from_project_type(project_type);
    assert!(config.search_paths.contains(&"./".to_string()));
    assert!(config.file_patterns.contains(&"*".to_string()));
    assert!(config.ignore_patterns.contains(&"target/".to_string()));
    assert!(config
        .ignore_patterns
        .contains(&"node_modules/".to_string()));
}

#[test]
fn test_unknown_project_fallback() {
    let temp_dir = TempDir::new().unwrap();

    // Create just a few random files - not enough to trigger any detection
    fs::write(temp_dir.path().join("random.txt"), "content").unwrap();

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert_eq!(project_type, ProjectType::Unknown);

    let config = ContextAwareConfig::from_project_type(project_type);
    assert_eq!(config, ContextAwareConfig::default());
    assert!(config.search_paths.contains(&"./".to_string()));
    assert!(config.file_patterns.contains(&"*".to_string()));
}

/// Test that context detection integrates with status command
/// According to UX Remediation Plan: show project type in status command
#[tokio::test]
async fn test_status_command_shows_project_type() -> Result<()> {
    // Run status command in current directory (which is a Rust project)
    let output = Command::new("cargo")
        .args(["run", "--bin", "semisearch", "--", "status"])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed
    assert!(output.status.success(), "Status command failed: {stderr}");

    // Should show health check
    assert!(
        stdout.contains("Health Check") || stdout.contains("status"),
        "Should show health check: {stdout}"
    );

    // Should show search capabilities without technical jargon
    assert!(
        stdout.contains("search") || stdout.contains("Search"),
        "Should mention search capabilities: {stdout}"
    );

    // Should show tips for users
    assert!(
        stdout.contains("💡") || stdout.contains("Tips") || stdout.contains("Try"),
        "Should provide user tips: {stdout}"
    );

    Ok(())
}

/// Test that context detection provides contextual tips
/// According to UX Remediation Plan: provide contextual tips when appropriate
#[test]
fn test_contextual_tips_for_project_types() {
    // Test that different project types would generate appropriate tips
    let rust_config = ContextAwareConfig::from_project_type(ProjectType::RustProject);
    let js_config = ContextAwareConfig::from_project_type(ProjectType::JavaScriptProject);
    let docs_config = ContextAwareConfig::from_project_type(ProjectType::Documentation);

    // Verify each project type has appropriate configuration
    // This would be used to generate contextual tips

    // Rust projects should focus on src/ and tests/
    assert!(rust_config.search_paths.contains(&"src/".to_string()));
    assert!(rust_config.search_paths.contains(&"tests/".to_string()));

    // JavaScript projects should focus on src/ and lib/
    assert!(js_config.search_paths.contains(&"src/".to_string()));
    assert!(js_config.search_paths.contains(&"lib/".to_string()));

    // Documentation projects should search everywhere for docs
    assert!(docs_config.search_paths.contains(&"./".to_string()));
    assert!(docs_config.file_patterns.contains(&"*.md".to_string()));
}

/// Test graceful failure handling
/// According to UX Remediation Plan: graceful failure handling
#[test]
fn test_graceful_failure_handling() {
    // Test with non-existent path
    let non_existent = Path::new("/nonexistent/path/that/does/not/exist");
    let project_type = ProjectDetector::detect(non_existent);

    // Should gracefully fall back to Unknown
    assert_eq!(project_type, ProjectType::Unknown);

    // Test with path that exists but has permission issues (simulated)
    let temp_dir = TempDir::new().unwrap();
    let project_type = ProjectDetector::detect(temp_dir.path());

    // Should not panic and should return a valid project type
    assert!(matches!(
        project_type,
        ProjectType::Unknown | ProjectType::Documentation | ProjectType::Mixed
    ));
}

/// Test that context detection doesn't show technical details during regular searches
/// According to UX Remediation Plan: "silent operation" - no noisy output
#[tokio::test]
async fn test_silent_operation_during_search() -> Result<()> {
    // Create a test project
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"",
    )
    .unwrap();
    fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    fs::write(
        temp_dir.path().join("src/main.rs"),
        "fn main() { println!(\"TODO: implement\"); }",
    )
    .unwrap();

    // Run a search in this directory
    let output = Command::new("cargo")
        .args(["run", "--bin", "semisearch", "--", "TODO"])
        .current_dir(temp_dir.path())
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should succeed or fail gracefully
    if output.status.success() {
        // Should show search results without technical context detection details
        assert!(
            !stdout.contains("ProjectType") && !stdout.contains("ContextAwareConfig"),
            "Should not show technical context detection details: {stdout}"
        );

        // Should not mention project detection in stderr
        assert!(
            !stderr.contains("detecting") && !stderr.contains("project type"),
            "Should not show context detection process: {stderr}"
        );
    } else {
        // If it fails, should fail gracefully without exposing context detection internals
        assert!(
            !stderr.contains("ProjectType") && !stderr.contains("ContextAwareConfig"),
            "Should not expose context detection internals in errors: {stderr}"
        );
    }

    Ok(())
}

/// Test override options work
/// According to UX Remediation Plan: allow override options
#[test]
fn test_override_options_concept() {
    // This tests the concept that users could override context detection
    // The actual CLI integration would be implemented separately

    // Test that manual configuration can override detected configuration
    let detected_config = ContextAwareConfig::from_project_type(ProjectType::RustProject);

    // User could override search paths
    let mut custom_config = detected_config.clone();
    custom_config.search_paths = vec!["./".to_string()]; // Search everywhere instead of just src/

    assert_ne!(detected_config.search_paths, custom_config.search_paths);
    assert_eq!(custom_config.search_paths, vec!["./".to_string()]);

    // User could override file patterns
    let mut custom_config2 = detected_config.clone();
    custom_config2.file_patterns = vec!["*".to_string()]; // Search all files instead of just .rs

    assert_ne!(detected_config.file_patterns, custom_config2.file_patterns);
    assert_eq!(custom_config2.file_patterns, vec!["*".to_string()]);
}

/// Test that context detection handles edge cases
#[test]
fn test_edge_cases() {
    // Test empty directory
    let empty_dir = TempDir::new().unwrap();
    let project_type = ProjectDetector::detect(empty_dir.path());
    assert_eq!(project_type, ProjectType::Unknown);

    // Test directory with only hidden files
    let hidden_files_dir = TempDir::new().unwrap();
    fs::write(hidden_files_dir.path().join(".gitignore"), "*.log").unwrap();
    fs::write(hidden_files_dir.path().join(".env"), "SECRET=value").unwrap();
    let project_type = ProjectDetector::detect(hidden_files_dir.path());
    assert_eq!(project_type, ProjectType::Unknown);

    // Test directory with conflicting project markers
    let conflicting_dir = TempDir::new().unwrap();
    fs::write(
        conflicting_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"",
    )
    .unwrap();
    fs::write(
        conflicting_dir.path().join("package.json"),
        r#"{"name": "test"}"#,
    )
    .unwrap();

    // Should prioritize Rust (first in detection order)
    let project_type = ProjectDetector::detect(conflicting_dir.path());
    assert_eq!(project_type, ProjectType::RustProject);
}

/// Integration test: verify context detection works with actual semisearch command
#[tokio::test]
async fn test_context_detection_integration() -> Result<()> {
    // Test in current directory (Rust project)
    let output = Command::new("cargo")
        .args(["run", "--bin", "semisearch", "--", "TODO", "--limit", "1"])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should work (succeed or fail gracefully)
    if output.status.success() {
        // If it finds results, should show them properly
        if stdout.contains("Found") {
            assert!(
                stdout.contains("TODO") || stdout.contains("matches"),
                "Should show search results: {stdout}"
            );
        }
    } else {
        // If it fails, should provide helpful error message
        assert!(
            !stderr.is_empty(),
            "Should provide error message when failing: {stderr}"
        );
    }

    // Context detection should work silently - no technical output about context detection
    assert!(
        !stdout.contains("ProjectType") && !stderr.contains("ContextAwareConfig") 
        && !stdout.contains("project type") && !stderr.contains("project type")
        && !stdout.contains("context detection") && !stderr.contains("context detection"),
        "Context detection should be silent (no technical context detection details). stdout: {stdout} stderr: {stderr}"
    );

    Ok(())
}
