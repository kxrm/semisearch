use std::collections::HashSet;

/// Centralized pattern definitions to eliminate code duplication
pub struct PatternDefinitions;

impl PatternDefinitions {
    /// Common programming terms used across the codebase
    pub fn programming_terms() -> &'static HashSet<&'static str> {
        &PROGRAMMING_TERMS
    }

    /// Common file extensions
    pub fn file_extensions() -> &'static HashSet<&'static str> {
        &FILE_EXTENSIONS
    }

    /// Common noise words for query simplification
    pub fn noise_words() -> &'static HashSet<&'static str> {
        &NOISE_WORDS
    }

    /// Code-specific keywords for pattern detection
    pub fn code_keywords() -> &'static HashSet<&'static str> {
        &CODE_KEYWORDS
    }

    /// Semantic analysis words with weights
    pub fn semantic_words() -> &'static [(&'static str, u16)] {
        SEMANTIC_WORDS
    }

    /// Common bigram patterns that suggest coherent queries
    pub fn coherent_bigrams() -> &'static [((&'static str, &'static str), u16)] {
        COHERENT_BIGRAMS
    }

    /// Typo patterns for detection
    pub fn typo_patterns() -> &'static [&'static str] {
        TYPO_PATTERNS.as_slice()
    }
}

lazy_static::lazy_static! {
    static ref PROGRAMMING_TERMS: HashSet<&'static str> = {
        [
            "function", "class", "method", "async", "await", "const", "let", "var",
            "public", "private", "protected", "static", "final", "abstract", "interface",
            "type", "enum", "struct", "trait", "impl", "mod", "import", "export",
            "require", "include", "using", "namespace", "try", "catch", "throw",
            "throws", "error", "exception", "handler", "validate", "validation",
            "check", "verify", "test", "testing", "config", "configuration",
            "setup", "initialize", "init", "db", "query", "sql", "api", "endpoint",
            "route", "controller", "service", "repository", "model", "view",
            "component", "utils", "utility", "helper", "fn", "pub", "def", "return",
        ].iter().cloned().collect()
    };

    static ref FILE_EXTENSIONS: HashSet<&'static str> = {
        [
            ".rs", ".py", ".js", ".ts", ".md", ".txt", ".json", ".toml", ".yaml",
            ".yml", ".xml", ".html", ".css", ".scss", ".sql", ".sh", ".bash",
            ".exe", ".dll", ".so", ".dylib", ".bin", ".obj", ".o", ".a", ".lib",
            ".zip", ".tar", ".gz", ".bz2", ".7z", ".rar", ".jpg", ".jpeg", ".png",
            ".gif", ".bmp", ".tiff", ".svg", ".mp3", ".mp4", ".avi", ".mov",
            ".wav", ".flac", ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx",
        ].iter().cloned().collect()
    };

    static ref NOISE_WORDS: HashSet<&'static str> = {
        [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "can", "this", "that", "these", "those",
        ].iter().cloned().collect()
    };

    static ref CODE_KEYWORDS: HashSet<&'static str> = {
        [
            "function", "class", "TODO", "FIXME", "import", "export", "async", "await",
            "Function", "Class", "todo", "fixme", "Import", "Export", "Async", "Await",
            "fn", "pub", "mod", "struct", "enum", "trait", "impl", "let", "const",
            "var", "def", "method", "constructor", "abstract", "static", "final",
            "public", "private", "protected", "virtual", "override", "extends", "implements",
        ].iter().cloned().collect()
    };

    static ref TYPO_PATTERNS: [&'static str; 11] = [
        "databse", "functoin", "recieve", "seperate", "occurance", "accomodate",
        "arguement", "begining", "definately", "existance", "independant",
    ];
}

const SEMANTIC_WORDS: &[(&str, u16)] = &[
    // Conceptual terms
    ("relationship", 200),
    ("concept", 190),
    ("theory", 185),
    ("analysis", 180),
    ("structure", 175),
    ("pattern", 170),
    ("framework", 180),
    ("model", 175),
    ("system", 170),
    ("process", 165),
    ("method", 160),
    ("approach", 165),
    // Abstract terms
    ("understanding", 180),
    ("meaning", 175),
    ("context", 170),
    ("interpretation", 185),
    ("significance", 180),
    ("implication", 175),
    // Technical concepts
    ("algorithm", 190),
    ("implementation", 185),
    ("architecture", 180),
    ("optimization", 185),
    ("evaluation", 175),
    ("performance", 170),
    ("memory", 165),
    ("cache", 160),
    ("database", 170),
    ("network", 165),
    ("security", 170),
    ("authentication", 175),
    ("authorization", 175),
    ("encryption", 170),
    ("protocol", 165),
    ("interface", 160),
    ("inheritance", 170),
    ("polymorphism", 180),
    ("abstraction", 175),
    ("management", 170),
    ("handling", 165),
    ("processing", 160),
    ("execution", 165),
    ("operation", 160),
    ("transaction", 165),
    ("synchronization", 180),
    ("coordination", 175),
    ("integration", 170),
    ("complexity", 175),
    ("scalability", 180),
    ("reliability", 175),
    ("availability", 170),
    ("consistency", 175),
    ("concurrency", 180),
    ("latency", 170),
    ("throughput", 165),
    ("bottleneck", 170),
    ("design", 170),
    ("principle", 175),
    ("practice", 165),
    ("strategy", 170),
    ("technique", 165),
    ("methodology", 175),
    ("paradigm", 180),
    ("philosophy", 175),
];

const COHERENT_BIGRAMS: &[((&str, &str), u16)] = &[
    (("object", "oriented"), 220),
    (("data", "structure"), 215),
    (("machine", "learning"), 220),
    (("neural", "network"), 225),
    (("natural", "language"), 220),
    (("user", "interface"), 210),
    (("error", "handling"), 205),
    (("memory", "management"), 210),
    (("file", "system"), 200),
    (("operating", "system"), 205),
    (("design", "pattern"), 215),
    (("best", "practice"), 200),
    (("use", "case"), 195),
    (("edge", "case"), 190),
    (("high", "level"), 185),
    (("low", "level"), 185),
    (("open", "source"), 190),
    (("real", "time"), 195),
    (("time", "complexity"), 200),
    (("space", "complexity"), 200),
];

/// Utility functions for pattern matching
pub mod utils {
    use super::*;
    use regex::Regex;

    /// Check if query contains code-related keywords
    pub fn contains_code_keywords(query: &str) -> bool {
        let query_lower = query.to_lowercase();
        let words: Vec<&str> = query_lower.split_whitespace().collect();

        if words.len() > 2 {
            // For multi-word queries, be more conservative
            let code_specific_patterns = [
                "function", "class", "todo", "fixme", "import", "export", "async", "await", "fn",
                "pub", "mod", "struct", "enum", "trait", "impl", "def", "method",
            ];
            words
                .iter()
                .any(|word| code_specific_patterns.contains(word))
        } else {
            // For shorter queries, be more permissive
            words
                .iter()
                .any(|word| PatternDefinitions::code_keywords().contains(word))
        }
    }

    /// Check if query contains file extensions
    pub fn contains_file_extensions(query: &str) -> bool {
        PatternDefinitions::file_extensions()
            .iter()
            .any(|ext| query.contains(ext))
    }

    /// Check if query looks like a regex pattern
    pub fn looks_like_regex(query: &str) -> bool {
        let regex_metacharacters = [
            ".*", "\\d+", "\\w+", "\\s+", "\\b", "\\B", "\\A", "\\Z", "\\z", "[", "]", "(", ")",
            "{", "}", "|", "^", "$", "?", "*", "+",
        ];

        regex_metacharacters
            .iter()
            .any(|&meta| query.contains(meta))
    }

    /// Simplify query by removing noise words and programming terms
    pub fn simplify_query(query: &str) -> String {
        let re = Regex::new(r"[ \t\n\r\.\:\(\)\{\}\[\],;]+")
            .expect("Invalid regex for query simplification");

        let tokens: Vec<&str> = re
            .split(query)
            .filter(|token| {
                let token_lower = token.to_lowercase();
                token.len() > 2
                    && !PatternDefinitions::programming_terms().contains(token_lower.as_str())
                    && !PatternDefinitions::file_extensions().contains(token)
                    && !PatternDefinitions::noise_words().contains(token_lower.as_str())
            })
            .collect();

        if tokens.is_empty() {
            "search term".to_string()
        } else {
            tokens.into_iter().take(3).collect::<Vec<_>>().join(" ")
        }
    }

    /// Analyze query pattern for usage tracking
    pub fn analyze_query_pattern(query: &str) -> QueryPattern {
        // Check for regex patterns
        if query.contains(".*")
            || query.contains("\\d")
            || query.contains("[")
            || query.contains("(")
        {
            return QueryPattern::RegexLike;
        }

        // Check for file extension filtering
        if contains_file_extensions(query) {
            return QueryPattern::FileFiltering;
        }

        // Check for potential typos
        if PatternDefinitions::typo_patterns()
            .iter()
            .any(|&typo| query.contains(typo))
        {
            return QueryPattern::PotentialTypo;
        }

        // Check for conceptual queries (multi-word, descriptive)
        if query.split_whitespace().count() > 3 {
            return QueryPattern::Conceptual;
        }

        QueryPattern::Simple
    }
}

/// Query pattern types for usage analysis
#[derive(Debug, Clone, PartialEq)]
pub enum QueryPattern {
    Simple,        // "TODO"
    RegexLike,     // "TODO.*Fix"
    PotentialTypo, // "databse"
    Conceptual,    // "error handling patterns"
    FileFiltering, // "TODO .py files"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_definitions_consistency() {
        // Ensure all pattern sets are non-empty
        assert!(!PatternDefinitions::programming_terms().is_empty());
        assert!(!PatternDefinitions::file_extensions().is_empty());
        assert!(!PatternDefinitions::noise_words().is_empty());
        assert!(!PatternDefinitions::code_keywords().is_empty());
    }

    #[test]
    fn test_code_keyword_detection() {
        assert!(utils::contains_code_keywords("function test"));
        assert!(utils::contains_code_keywords("TODO fix this"));
        assert!(!utils::contains_code_keywords("simple search"));
    }

    #[test]
    fn test_query_simplification() {
        assert_eq!(
            utils::simplify_query("function validateUserInput"),
            "validateUserInput"
        );
        assert_eq!(
            utils::simplify_query("the quick brown fox"),
            "quick brown fox"
        );
        assert_eq!(
            utils::simplify_query("async await authentication"),
            "authentication"
        );
    }
}
