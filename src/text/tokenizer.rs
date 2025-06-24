use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;

/// Simple tokenizer for text processing
pub struct Tokenizer {
    stop_words: HashSet<String>,
}

impl Tokenizer {
    pub fn new() -> Self {
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
        ]
        .iter()
        .map(|&s| s.to_string())
        .collect();

        Self { stop_words }
    }

    /// Tokenize text into words
    pub fn tokenize(&self, text: &str) -> Vec<Token> {
        text.unicode_words()
            .enumerate()
            .map(|(index, word)| {
                let word_lower = word.to_lowercase();
                let token_type = self.classify_token(&word_lower);

                Token {
                    text: word_lower,
                    original: word.to_string(),
                    position: index,
                    token_type,
                }
            })
            .collect()
    }

    /// Classify token type
    fn classify_token(&self, word: &str) -> TokenType {
        if self.stop_words.contains(word) {
            TokenType::StopWord
        } else if word.chars().all(|c| c.is_numeric()) {
            TokenType::Number
        } else if word.chars().all(|c| c.is_alphabetic()) {
            TokenType::Word
        } else {
            TokenType::Mixed
        }
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a single token
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub text: String,
    pub original: String,
    pub position: usize,
    pub token_type: TokenType,
}

/// Classification of token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Word,
    Number,
    StopWord,
    Mixed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_creation() {
        let tokenizer = Tokenizer::new();
        assert!(tokenizer.stop_words.contains("the"));
        assert!(tokenizer.stop_words.contains("and"));
    }

    #[test]
    fn test_basic_tokenization() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("Hello world test");

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].original, "Hello");
        assert_eq!(tokens[0].position, 0);
        assert_eq!(tokens[0].token_type, TokenType::Word);
    }

    #[test]
    fn test_token_classification() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("The number 123 and mixed123");

        // "The" should be classified as stop word
        assert_eq!(tokens[0].token_type, TokenType::StopWord);

        // "number" should be a regular word
        assert_eq!(tokens[1].token_type, TokenType::Word);

        // "123" should be classified as number
        assert_eq!(tokens[2].token_type, TokenType::Number);

        // "and" should be stop word
        assert_eq!(tokens[3].token_type, TokenType::StopWord);

        // "mixed123" should be mixed
        assert_eq!(tokens[4].token_type, TokenType::Mixed);
    }

    #[test]
    fn test_empty_text() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("");

        assert!(tokens.is_empty());
    }

    #[test]
    fn test_unicode_handling() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("café naïve résumé");

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "café");
        assert_eq!(tokens[1].text, "naïve");
        assert_eq!(tokens[2].text, "résumé");
    }

    #[test]
    fn test_position_tracking() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("first second third");

        assert_eq!(tokens[0].position, 0);
        assert_eq!(tokens[1].position, 1);
        assert_eq!(tokens[2].position, 2);
    }
}
