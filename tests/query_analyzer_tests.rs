use search::query::analyzer::{QueryAnalyzer, QueryType};

#[test]
fn test_exact_phrase_detection() {
    // Test queries with quotes should be detected as exact phrases
    assert!(matches!(
        QueryAnalyzer::analyze("\"specific function name\""),
        QueryType::ExactPhrase
    ));

    assert!(matches!(
        QueryAnalyzer::analyze("\"TODO: implement feature\""),
        QueryType::ExactPhrase
    ));

    assert!(matches!(
        QueryAnalyzer::analyze("\"error handling patterns\""),
        QueryType::ExactPhrase
    ));
}

#[test]
fn test_code_pattern_detection() {
    // Test queries containing code keywords
    let code_keywords = [
        "function", "class", "TODO", "FIXME", "import", "export", "async", "await", "Function",
        "Class", "todo", "fixme", "Import", "Export", "Async", "Await",
    ];

    for keyword in &code_keywords {
        assert!(
            matches!(QueryAnalyzer::analyze(keyword), QueryType::CodePattern),
            "Keyword {keyword} should be detected as CodePattern"
        );
    }

    // Test compound code patterns
    assert!(matches!(
        QueryAnalyzer::analyze("async function"),
        QueryType::CodePattern
    ));

    assert!(matches!(
        QueryAnalyzer::analyze("export class"),
        QueryType::CodePattern
    ));

    assert!(matches!(
        QueryAnalyzer::analyze("import React"),
        QueryType::CodePattern
    ));
}

#[test]
fn test_file_extension_detection() {
    // Test queries mentioning file extensions
    let file_extensions = [".rs", ".py", ".js", ".ts", ".md", ".txt", ".json", ".toml"];

    for ext in &file_extensions {
        assert!(
            matches!(QueryAnalyzer::analyze(ext), QueryType::FileExtension),
            "Extension {ext} should be detected as FileExtension"
        );
    }

    // Test compound file extension patterns
    assert!(matches!(
        QueryAnalyzer::analyze("rust files .rs"),
        QueryType::FileExtension
    ));

    assert!(matches!(
        QueryAnalyzer::analyze("python code .py"),
        QueryType::FileExtension
    ));
}

#[test]
fn test_regex_like_detection() {
    // Test queries containing regex metacharacters
    let regex_patterns = [
        ".*pattern",
        "regex.*",
        "\\d+",
        "\\w+",
        "[a-z]",
        "(group)",
        "\\s+",
        ".*",
        "\\bword\\b",
        "\\d{3}",
        "[0-9]+",
        "(pattern)",
    ];

    for pattern in &regex_patterns {
        assert!(
            matches!(QueryAnalyzer::analyze(pattern), QueryType::RegexLike),
            "Pattern {pattern} should be detected as RegexLike"
        );
    }
}

#[test]
fn test_conceptual_detection() {
    // Test multi-word queries that should be detected as conceptual
    let conceptual_queries = [
        "error handling patterns",
        "authentication and authorization",
        "database connection management",
        "user interface design principles",
        "machine learning algorithms",
        "web development best practices",
    ];

    for query in &conceptual_queries {
        assert!(
            matches!(QueryAnalyzer::analyze(query), QueryType::Conceptual),
            "Query {query} should be detected as Conceptual"
        );
    }
}

#[test]
fn test_single_word_defaults() {
    // Test single words that aren't code keywords should default to ExactPhrase
    let simple_queries = ["hello", "world", "test", "example", "data", "file"];

    for query in &simple_queries {
        assert!(
            matches!(QueryAnalyzer::analyze(query), QueryType::ExactPhrase),
            "Query {query} should default to ExactPhrase"
        );
    }
}

#[test]
fn test_two_word_defaults() {
    // Test two-word queries that aren't code patterns should default to ExactPhrase
    let two_word_queries = ["hello world", "test data", "file system", "user input"];

    for query in &two_word_queries {
        assert!(
            matches!(QueryAnalyzer::analyze(query), QueryType::ExactPhrase),
            "Query {query} should default to ExactPhrase"
        );
    }
}

#[test]
fn test_priority_order() {
    // Test that more specific patterns take priority over general ones

    // Exact phrase should take priority over code pattern
    assert!(matches!(
        QueryAnalyzer::analyze("\"function name\""),
        QueryType::ExactPhrase
    ));

    // Code pattern should take priority over conceptual
    assert!(matches!(
        QueryAnalyzer::analyze("function name implementation"),
        QueryType::CodePattern
    ));

    // File extension should take priority over conceptual
    assert!(matches!(
        QueryAnalyzer::analyze("rust code .rs files"),
        QueryType::FileExtension
    ));

    // Regex should take priority over conceptual
    assert!(matches!(
        QueryAnalyzer::analyze("pattern.*matching"),
        QueryType::RegexLike
    ));
}
