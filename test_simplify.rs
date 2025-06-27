use search::errors::user_errors::UserError;

fn main() {
    // Test cases for the enhanced simplify_query function
    let test_cases = vec![
        "function validateUserInput database query",
        "async await authentication handler",
        "TODO: implement error handling",
        "config.json setup initialization",
        "the quick brown fox",
        "user login authentication",
        "test validation check",
        "simple search term",
        "complex.function.name()",
        "file.rs extension test",
    ];

    println!("Enhanced simplify_query function test results:\n");
    
    for query in test_cases {
        let simplified = UserError::no_matches(query);
        println!("Original: '{}'", query);
        println!("Simplified suggestion: '{}'", simplified.suggestions[1].split(": ").nth(1).unwrap_or(""));
        println!("---");
    }
} 