use search::{search_files, MatchType, SearchOptions};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_end_to_end_search() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    let movies_file = temp_dir.path().join("movies.txt");
    fs::write(
        &movies_file,
        "Ghostbusters is a classic comedy\n\
         Silence of the Lambs won many awards\n\
         Ace Ventura: Pet Detective stars Jim Carrey\n\
         It's a Wonderful Life is heartwarming",
    )
    .unwrap();

    let actors_file = temp_dir.path().join("actors.txt");
    fs::write(
        &actors_file,
        "Jim Carrey is a famous comedian\n\
         Anthony Hopkins starred in Silence of the Lambs\n\
         Bill Murray was in Ghostbusters",
    )
    .unwrap();

    // Test basic search
    let options = SearchOptions::default();
    let results = search_files("Jim Carrey", temp_dir.path().to_str().unwrap(), &options).unwrap();

    // Verify results
    assert!(!results.is_empty(), "Should find Jim Carrey references");
    assert_eq!(results.len(), 2, "Should find exactly 2 references");

    // Verify content matches
    let contents: Vec<&str> = results.iter().map(|r| r.content.as_str()).collect();
    assert!(contents.iter().any(|c| c.contains("Ace Ventura")));
    assert!(contents.iter().any(|c| c.contains("famous comedian")));
}

#[test]
fn test_fuzzy_search_integration() {
    let temp_dir = TempDir::new().unwrap();

    // Create test file with typos
    let file = temp_dir.path().join("test.txt");
    fs::write(
        &file,
        "TODO: Fix this bug\nTODO: Implement feature\nTODO task here",
    )
    .unwrap();

    let options = SearchOptions {
        fuzzy_matching: true,
        min_score: 0.3, // Lower threshold to catch more fuzzy matches
        ..Default::default()
    };

    let results = search_files("TODO", temp_dir.path().to_str().unwrap(), &options).unwrap();

    // Should find all 3 TODO matches
    assert!(
        results.len() >= 3,
        "Should find at least 3 matches, found {}",
        results.len()
    );

    // All should be exact matches since we're searching for "TODO" and all lines contain "TODO"
    for result in &results {
        if result.content.contains("TODO") {
            assert_eq!(
                result.match_type,
                Some(MatchType::Exact),
                "TODO lines should be exact matches"
            );
            assert_eq!(
                result.score,
                Some(1.0),
                "Exact matches should have score 1.0"
            );
        }
    }
}

#[test]
fn test_actual_fuzzy_matching() {
    let temp_dir = TempDir::new().unwrap();

    // Create test file with realistic fuzzy matches
    let file = temp_dir.path().join("test.txt");
    fs::write(
        &file,
        "function test() {\nfunction tset() {\nfunction best() {",
    )
    .unwrap();

    let options = SearchOptions {
        fuzzy_matching: true,
        min_score: 0.5,
        ..Default::default()
    };

    let results = search_files("test", temp_dir.path().to_str().unwrap(), &options).unwrap();

    // Should find both exact and fuzzy matches
    assert!(
        results.len() >= 2,
        "Should find at least 2 matches, found {}",
        results.len()
    );

    // Check that we have both exact and fuzzy matches
    let exact_matches: Vec<_> = results
        .iter()
        .filter(|r| r.match_type == Some(MatchType::Exact))
        .collect();
    let fuzzy_matches: Vec<_> = results
        .iter()
        .filter(|r| r.match_type == Some(MatchType::Fuzzy))
        .collect();

    assert!(
        !exact_matches.is_empty(),
        "Should have at least 1 exact match"
    );
    assert!(
        !fuzzy_matches.is_empty(),
        "Should have at least 1 fuzzy match"
    );
}

#[test]
fn test_regex_search_integration() {
    let temp_dir = TempDir::new().unwrap();

    let file = temp_dir.path().join("code.rs");
    fs::write(
        &file,
        "fn main() {\n\
         let x = 42;\n\
         let y = \"hello\";\n\
         println!(\"x = {}\", x);\n\
         }",
    )
    .unwrap();

    let options = SearchOptions {
        regex_mode: true,
        ..Default::default()
    };

    // Test regex pattern for variable declarations
    let results = search_files(r"let \w+ =", temp_dir.path().to_str().unwrap(), &options).unwrap();

    assert_eq!(results.len(), 2, "Should find 2 variable declarations");

    // Verify matches
    let contents: Vec<&str> = results.iter().map(|r| r.content.as_str()).collect();
    assert!(contents.iter().any(|c| c.contains("let x = 42")));
    assert!(contents.iter().any(|c| c.contains("let y = \"hello\"")));
}

#[test]
fn test_case_sensitive_search() {
    let temp_dir = TempDir::new().unwrap();

    let file = temp_dir.path().join("mixed_case.txt");
    fs::write(&file, "TODO: uppercase\ntodo: lowercase\nTodo: mixed case").unwrap();

    // Case insensitive (default)
    let options_insensitive = SearchOptions {
        case_sensitive: false,
        ..Default::default()
    };
    let results_insensitive = search_files(
        "todo",
        temp_dir.path().to_str().unwrap(),
        &options_insensitive,
    )
    .unwrap();
    assert_eq!(
        results_insensitive.len(),
        3,
        "Case insensitive should find all 3"
    );

    // Case sensitive
    let options_sensitive = SearchOptions {
        case_sensitive: true,
        ..Default::default()
    };
    let results_sensitive = search_files(
        "todo",
        temp_dir.path().to_str().unwrap(),
        &options_sensitive,
    )
    .unwrap();
    assert_eq!(
        results_sensitive.len(),
        1,
        "Case sensitive should find only 1"
    );

    // Verify the exact match
    assert!(results_sensitive[0].content.contains("todo: lowercase"));
}

#[test]
fn test_output_formats() {
    let temp_dir = TempDir::new().unwrap();

    let file = temp_dir.path().join("test.txt");
    fs::write(&file, "Sample content for testing").unwrap();

    let options = SearchOptions::default();
    let results = search_files("content", temp_dir.path().to_str().unwrap(), &options).unwrap();

    // Test JSON serialization
    let json_output = serde_json::to_string_pretty(&results).unwrap();
    assert!(json_output.contains("\"file_path\""));
    assert!(json_output.contains("\"line_number\""));
    assert!(json_output.contains("\"content\""));

    // Test that we got results
    assert!(!results.is_empty());
    assert!(results[0].file_path.contains("test.txt"));
    assert!(results[0].content.contains("Sample content"));
}

#[test]
fn test_large_directory_handling() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple nested directories with files
    for i in 0..5 {
        let sub_dir = temp_dir.path().join(format!("dir_{i}"));
        fs::create_dir(&sub_dir).unwrap();

        for j in 0..3 {
            let file = sub_dir.join(format!("file_{j}.txt"));
            fs::write(&file, format!("Content {j} in directory {i}")).unwrap();
        }
    }

    let options = SearchOptions {
        max_results: 20,
        ..Default::default()
    };

    let results = search_files("Content", temp_dir.path().to_str().unwrap(), &options).unwrap();

    // Should find all files (5 dirs * 3 files = 15 files)
    assert_eq!(results.len(), 15, "Should find all 15 files");

    // Verify we get results from different directories
    let dir_names: std::collections::HashSet<_> = results
        .iter()
        .map(|r| {
            std::path::Path::new(&r.file_path)
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        })
        .collect();

    assert_eq!(
        dir_names.len(),
        5,
        "Should have results from all 5 directories"
    );
}

#[test]
fn test_performance_with_limits() {
    let temp_dir = TempDir::new().unwrap();

    // Create many files with matches
    for i in 0..50 {
        let file = temp_dir.path().join(format!("file_{i}.txt"));
        fs::write(&file, format!("This is file number {i} with a match")).unwrap();
    }

    let options = SearchOptions {
        max_results: 10,
        ..Default::default()
    };

    let results = search_files("match", temp_dir.path().to_str().unwrap(), &options).unwrap();

    // Should respect the limit
    assert_eq!(results.len(), 10, "Should respect max_results limit");

    // Results should be sorted by score (descending)
    for i in 1..results.len() {
        let prev_score = results[i - 1].score.unwrap_or(0.0);
        let curr_score = results[i].score.unwrap_or(0.0);
        assert!(
            prev_score >= curr_score,
            "Results should be sorted by score"
        );
    }
}
