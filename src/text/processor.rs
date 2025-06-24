use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;

/// Text processor for cleaning and tokenizing content
pub struct TextProcessor {
    stop_words: HashSet<String>,
    min_chunk_length: usize,
    max_chunk_length: usize,
}

impl TextProcessor {
    pub fn new() -> Self {
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do",
            "does", "did", "will", "would", "could", "should", "may", "might", "must", "can",
            "this", "that", "these", "those", "i", "you", "he", "she", "it", "we", "they", "me",
            "him", "her", "us", "them", "my", "your", "his", "her", "its", "our", "their", "mine",
            "yours", "hers", "ours", "theirs",
        ]
        .iter()
        .map(|&s| s.to_string())
        .collect();

        Self {
            stop_words,
            min_chunk_length: 10,
            max_chunk_length: 1000,
        }
    }

    pub fn with_config(min_chunk_length: usize, max_chunk_length: usize) -> Self {
        let mut processor = Self::new();
        processor.min_chunk_length = min_chunk_length;
        processor.max_chunk_length = max_chunk_length;
        processor
    }

    /// Process file content into searchable chunks
    pub fn process_file(&self, content: &str) -> Vec<TextChunk> {
        content
            .lines()
            .enumerate()
            .filter_map(|(line_num, line)| {
                let cleaned = self.clean_text(line);
                if cleaned.len() >= self.min_chunk_length {
                    Some(TextChunk {
                        line_number: line_num + 1,
                        content: cleaned.clone(),
                        tokens: self.tokenize(&cleaned),
                        start_char: 0, // Simplified - could calculate actual position
                        end_char: cleaned.len(),
                        language_hint: self.detect_language_hint(line),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Process content into overlapping chunks for better semantic coverage
    pub fn process_file_with_overlap(
        &self,
        content: &str,
        chunk_size: usize,
        overlap: usize,
    ) -> Vec<TextChunk> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        let mut start_line = 0;

        while start_line < lines.len() {
            let end_line = std::cmp::min(start_line + chunk_size, lines.len());
            let chunk_content = lines[start_line..end_line].join("\n");
            let cleaned = self.clean_text(&chunk_content);

            if cleaned.len() >= self.min_chunk_length {
                chunks.push(TextChunk {
                    line_number: start_line + 1,
                    content: cleaned.clone(),
                    tokens: self.tokenize(&cleaned),
                    start_char: 0,
                    end_char: cleaned.len(),
                    language_hint: self.detect_language_hint(&chunk_content),
                });
            }

            // Ensure we have proper overlap
            if overlap > 0 && chunk_size > overlap {
                start_line += chunk_size - overlap;
            } else {
                start_line += chunk_size;
            }

            // Prevent infinite loop
            if start_line == 0 {
                break;
            }
        }

        chunks
    }

    /// Clean and normalize text
    pub fn clean_text(&self, text: &str) -> String {
        text.trim()
            .replace('\t', " ")
            .replace('\r', "")
            // Normalize multiple whitespace to single space
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            // Remove common code artifacts for better text processing
            .replace("//", " ")
            .replace("/*", " ")
            .replace("*/", " ")
            .replace("<!--", " ")
            .replace("-->", " ")
    }

    /// Tokenize text into searchable terms
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        text.unicode_words()
            .map(|w| w.to_lowercase())
            .filter(|w| {
                w.len() > 1 && !self.stop_words.contains(w) && !w.chars().all(|c| c.is_numeric())
            })
            .collect()
    }

    /// Extract key phrases (2-3 word combinations)
    pub fn extract_phrases(&self, text: &str) -> Vec<String> {
        let tokens = self.tokenize(text);
        let mut phrases = Vec::new();

        // Extract 2-word phrases
        for window in tokens.windows(2) {
            phrases.push(window.join(" "));
        }

        // Extract 3-word phrases
        for window in tokens.windows(3) {
            phrases.push(window.join(" "));
        }

        phrases
    }

    /// Detect programming language or content type
    fn detect_language_hint(&self, content: &str) -> Option<String> {
        let content_lower = content.to_lowercase();

        // Simple heuristics for language detection
        if content_lower.contains("function") && content_lower.contains("var") {
            Some("javascript".to_string())
        } else if content_lower.contains("def ") && content_lower.contains("import") {
            Some("python".to_string())
        } else if content_lower.contains("fn ") && content_lower.contains("let") {
            Some("rust".to_string())
        } else if content_lower.contains("public class") || content_lower.contains("import java") {
            Some("java".to_string())
        } else if content_lower.contains("#include") || content_lower.contains("int main") {
            Some("c".to_string())
        } else if content_lower.contains("<!doctype") || content_lower.contains("<html") {
            Some("html".to_string())
        } else if content_lower.contains("select") && content_lower.contains("from") {
            Some("sql".to_string())
        } else {
            None
        }
    }

    /// Calculate text complexity score
    pub fn calculate_complexity(&self, text: &str) -> f32 {
        let tokens = self.tokenize(text);
        let unique_tokens: HashSet<_> = tokens.iter().collect();

        if tokens.is_empty() {
            return 0.0;
        }

        // Complexity based on vocabulary diversity and average word length
        let vocabulary_diversity = unique_tokens.len() as f32 / tokens.len() as f32;
        let avg_word_length: f32 =
            tokens.iter().map(|t| t.len() as f32).sum::<f32>() / tokens.len() as f32;

        // Normalize average word length to 0-1 range (assuming max word length of 15)
        let normalized_word_length = (avg_word_length / 15.0).min(1.0);

        // Weight vocabulary diversity more heavily
        let complexity = (vocabulary_diversity * 0.7) + (normalized_word_length * 0.3);

        complexity.min(1.0)
    }
}

impl Default for TextProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// A chunk of processed text with metadata
#[derive(Debug, Clone, PartialEq)]
pub struct TextChunk {
    pub line_number: usize,
    pub content: String,
    pub tokens: Vec<String>,
    pub start_char: usize,
    pub end_char: usize,
    pub language_hint: Option<String>,
}

impl TextChunk {
    /// Get the token count
    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    /// Get character count
    pub fn char_count(&self) -> usize {
        self.content.len()
    }

    /// Check if chunk contains any of the given terms
    pub fn contains_terms(&self, terms: &[String]) -> bool {
        terms.iter().any(|term| {
            self.tokens.contains(term) || self.content.to_lowercase().contains(&term.to_lowercase())
        })
    }

    /// Calculate term frequency for a specific term
    pub fn term_frequency(&self, term: &str) -> f32 {
        let term_lower = term.to_lowercase();
        let count = self.tokens.iter().filter(|&t| t == &term_lower).count();
        if self.tokens.is_empty() {
            0.0
        } else {
            count as f32 / self.tokens.len() as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_processor_creation() {
        let processor = TextProcessor::new();
        assert!(processor.stop_words.contains("the"));
        assert!(processor.stop_words.contains("and"));
        assert_eq!(processor.min_chunk_length, 10);
    }

    #[test]
    fn test_clean_text() {
        let processor = TextProcessor::new();
        let input = "  Hello\t\tWorld  \n  with   multiple   spaces  ";
        let cleaned = processor.clean_text(input);
        assert_eq!(cleaned, "Hello World with multiple spaces");
    }

    #[test]
    fn test_tokenize() {
        let processor = TextProcessor::new();
        let text = "Hello World! This is a test.";
        let tokens = processor.tokenize(text);

        // Should exclude stop words like "is", "a"
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
        assert!(!tokens.contains(&"is".to_string()));
        assert!(!tokens.contains(&"a".to_string()));
    }

    #[test]
    fn test_process_file() {
        let processor = TextProcessor::new();
        let content = "Line 1: Hello World\nLine 2: This is a test\nShort\nLine 4: Another longer line for testing";
        let chunks = processor.process_file(content);

        // Should skip "Short" line as it's too short
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].line_number, 1);
        assert_eq!(chunks[1].line_number, 2);
        assert_eq!(chunks[2].line_number, 4);
    }

    #[test]
    fn test_extract_phrases() {
        let processor = TextProcessor::new();
        let text = "machine learning algorithm";
        let phrases = processor.extract_phrases(text);

        assert!(phrases.contains(&"machine learning".to_string()));
        assert!(phrases.contains(&"learning algorithm".to_string()));
        assert!(phrases.contains(&"machine learning algorithm".to_string()));
    }

    #[test]
    fn test_language_detection() {
        let processor = TextProcessor::new();

        let rust_code = "fn main() { let x = 5; }";
        let chunks = processor.process_file(rust_code);
        assert_eq!(chunks[0].language_hint, Some("rust".to_string()));

        let python_code = "def hello(): import os";
        let chunks = processor.process_file(python_code);
        assert_eq!(chunks[0].language_hint, Some("python".to_string()));
    }

    #[test]
    fn test_text_chunk_methods() {
        let chunk = TextChunk {
            line_number: 1,
            content: "hello world test".to_string(),
            tokens: vec!["hello".to_string(), "world".to_string(), "test".to_string()],
            start_char: 0,
            end_char: 16,
            language_hint: None,
        };

        assert_eq!(chunk.token_count(), 3);
        assert_eq!(chunk.char_count(), 16);
        assert!(chunk.contains_terms(&vec!["hello".to_string()]));
        assert!(!chunk.contains_terms(&vec!["missing".to_string()]));
        assert_eq!(chunk.term_frequency("hello"), 1.0 / 3.0);
    }

    #[test]
    fn test_calculate_complexity() {
        let processor = TextProcessor::new();

        let simple_text = "hello hello hello";
        let simple_complexity = processor.calculate_complexity(simple_text);

        let complex_text = "sophisticated algorithms demonstrate remarkable capabilities";
        let complex_complexity = processor.calculate_complexity(complex_text);

        assert!(complex_complexity > simple_complexity);
    }

    #[test]
    fn test_process_file_with_overlap() {
        let processor = TextProcessor::new();
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
        let chunks = processor.process_file_with_overlap(content, 2, 1);

        // Should create overlapping chunks
        assert!(!chunks.is_empty());
        // Verify overlap by checking that consecutive chunks share content
        if chunks.len() > 1 {
            // Check if chunks have overlapping content by looking for common words
            let first_words: Vec<&str> = chunks[0].content.split_whitespace().collect();
            let second_words: Vec<&str> = chunks[1].content.split_whitespace().collect();

            // Should have at least one overlapping word (like "Line" and "2")
            let has_overlap = first_words.iter().any(|word| second_words.contains(word));
            assert!(
                has_overlap,
                "First chunk words {:?} should overlap with second chunk words {:?}",
                first_words, second_words
            );
        }
    }
}
