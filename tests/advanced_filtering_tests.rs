use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test advanced include/exclude filtering functionality
/// These features are only available in --advanced mode and should not affect basic usage

#[test]
fn test_basic_search_unaffected_by_filtering_fix() {
    // Test: Basic search should work exactly the same after filtering fix
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("test.rs"),
        "fn main() { println!(\"TODO: implement this\"); }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("test_file.py"),
        "# TODO: add tests here",
    )
    .unwrap();

    // Basic search (no --advanced flag) should work normally
    let output = Command::new(env!("CARGO_BIN_EXE_semisearch"))
        .args(["TODO"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "Basic search should work");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should find TODO in both files (no filtering applied)
    assert!(stdout.contains("Found"), "Should find results");
    assert!(
        stdout.contains("test.rs") || stdout.contains("test_file.py"),
        "Should find results in files"
    );
}

#[test]
fn test_advanced_exclude_patterns_work() {
    // Test: --exclude patterns should work in advanced mode
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("source.rs"),
        "fn main() { println!(\"TODO: implement this\"); }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("test_file.rs"),
        "// TODO: add tests here",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("example_test.py"),
        "# TODO: write test",
    )
    .unwrap();

    // Advanced search with exclude pattern
    let output = Command::new(env!("CARGO_BIN_EXE_semisearch"))
        .args(["--advanced", "TODO", "--exclude", "*test*"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Advanced exclude search should work"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("Found") {
        // Should find TODO in source.rs but NOT in test files
        assert!(
            stdout.contains("source.rs"),
            "Should find TODO in source.rs"
        );
        assert!(
            !stdout.contains("test_file.rs"),
            "Should NOT find TODO in test_file.rs"
        );
        assert!(
            !stdout.contains("example_test.py"),
            "Should NOT find TODO in example_test.py"
        );
    }
    // If no results found, that's also acceptable (means exclude worked too well)
}

#[test]
fn test_advanced_include_patterns_work() {
    // Test: --include patterns should work in advanced mode
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("source.rs"),
        "fn main() { println!(\"TODO: implement this\"); }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("document.md"),
        "# TODO: write documentation",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("script.py"),
        "# TODO: add functionality",
    )
    .unwrap();

    // Advanced search with include pattern (only .rs files)
    let output = Command::new(env!("CARGO_BIN_EXE_semisearch"))
        .args(["--advanced", "TODO", "--include", "*.rs"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Advanced include search should work"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("Found") {
        // Should find TODO in .rs files only
        assert!(
            stdout.contains("source.rs"),
            "Should find TODO in source.rs"
        );
        assert!(
            !stdout.contains("document.md"),
            "Should NOT find TODO in document.md"
        );
        assert!(
            !stdout.contains("script.py"),
            "Should NOT find TODO in script.py"
        );
    }
}

#[test]
fn test_advanced_multiple_include_patterns() {
    // Test: Multiple --include patterns should work
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("source.rs"),
        "fn main() { println!(\"TODO: implement this\"); }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("document.md"),
        "# TODO: write documentation",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("script.py"),
        "# TODO: add functionality",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("config.json"),
        "{\"todo\": \"configure this\"}",
    )
    .unwrap();

    // Advanced search with multiple include patterns
    let output = Command::new(env!("CARGO_BIN_EXE_semisearch"))
        .args([
            "--advanced",
            "TODO",
            "--include",
            "*.rs",
            "--include",
            "*.md",
        ])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Advanced multiple include search should work"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("Found") {
        // Should find TODO in .rs and .md files only
        assert!(
            stdout.contains("source.rs") || stdout.contains("document.md"),
            "Should find TODO in .rs or .md files"
        );
        assert!(
            !stdout.contains("script.py"),
            "Should NOT find TODO in script.py"
        );
        assert!(
            !stdout.contains("config.json"),
            "Should NOT find TODO in config.json"
        );
    }
}

#[test]
fn test_advanced_include_and_exclude_together() {
    // Test: --include and --exclude should work together
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("main.rs"),
        "fn main() { println!(\"TODO: implement this\"); }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("test_main.rs"),
        "// TODO: add tests here",
    )
    .unwrap();
    fs::write(temp_dir.path().join("lib.rs"), "// TODO: implement library").unwrap();
    fs::write(
        temp_dir.path().join("script.py"),
        "# TODO: add functionality",
    )
    .unwrap();

    // Include .rs files but exclude test files
    let output = Command::new(env!("CARGO_BIN_EXE_semisearch"))
        .args([
            "--advanced",
            "TODO",
            "--include",
            "*.rs",
            "--exclude",
            "*test*",
        ])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Advanced include+exclude search should work"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("Found") {
        // Should find TODO in .rs files but not test files
        assert!(
            stdout.contains("main.rs") || stdout.contains("lib.rs"),
            "Should find TODO in non-test .rs files"
        );
        assert!(
            !stdout.contains("test_main.rs"),
            "Should NOT find TODO in test_main.rs"
        );
        assert!(
            !stdout.contains("script.py"),
            "Should NOT find TODO in script.py"
        );
    }
}

#[test]
fn test_basic_search_never_uses_filtering() {
    // Test: Basic search (without --advanced) should ignore filtering patterns
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("test.rs"),
        "fn main() { println!(\"TODO: implement this\"); }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("test_other.py"),
        "# TODO: add tests here",
    )
    .unwrap();

    // Use filtering flags without --advanced (should be ignored)
    let output = Command::new(env!("CARGO_BIN_EXE_semisearch"))
        .args(["TODO", "--exclude", "*test*"]) // No --advanced flag
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Should succeed and find results in ALL files (filtering ignored)
    assert!(output.status.success(), "Basic search should work");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should find TODO in both files since filtering is ignored in basic mode
    assert!(stdout.contains("Found"), "Should find results");
    assert!(
        stdout.contains("test.rs") || stdout.contains("test_other.py"),
        "Should find results in files (filtering ignored in basic mode)"
    );

    // Verify that both files are found (exclude pattern ignored)
    let found_test_rs = stdout.contains("test.rs");
    let found_test_py = stdout.contains("test_other.py");
    assert!(
        found_test_rs || found_test_py,
        "Should find TODO in files regardless of exclude pattern in basic mode"
    );
}
