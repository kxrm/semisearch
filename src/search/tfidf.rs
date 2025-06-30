use super::{MatchType, ResourceRequirements, SearchOptions, SearchResult, SearchStrategy};
use crate::text::{TextChunk, TextProcessor};
use anyhow::Result;
use rustc_hash::FxHashMap;

/// TF-IDF based search implementation
pub struct TfIdfSearch {
    text_processor: TextProcessor,
    document_frequency: FxHashMap<String, usize>,
    total_documents: usize,
    term_cache: std::sync::Mutex<FxHashMap<String, Vec<f32>>>,
}

impl TfIdfSearch {
    pub fn new() -> Self {
        Self {
            text_processor: TextProcessor::new(),
            document_frequency: FxHashMap::default(),
            total_documents: 0,
            term_cache: std::sync::Mutex::new(FxHashMap::default()),
        }
    }

    /// Build TF-IDF index from a collection of text chunks
    pub fn build_index(&mut self, chunks: &[TextChunk]) -> Result<()> {
        self.document_frequency.clear();
        self.total_documents = chunks.len();

        // Calculate document frequency for each term
        for chunk in chunks {
            let unique_terms: std::collections::HashSet<_> = chunk.tokens.iter().collect();

            for term in unique_terms {
                *self.document_frequency.entry(term.clone()).or_insert(0) += 1;
            }
        }

        // Clear cache when index is rebuilt
        self.term_cache.lock().unwrap().clear();

        Ok(())
    }

    /// Search within indexed chunks using TF-IDF scoring
    pub fn search_chunks(
        &self,
        query: &str,
        chunks: &[TextChunk],
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let query_tokens = self.text_processor.tokenize(query);
        if query_tokens.is_empty() {
            return Ok(vec![]);
        }

        if self.total_documents == 0 {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();

        for chunk in chunks {
            let score = self.calculate_tfidf_score(&query_tokens, chunk);

            if score >= options.min_score {
                let (start_char, end_char) =
                    self.find_best_match_position(&chunk.content, &query_tokens);

                results.push(SearchResult {
                    file_path: "memory".to_string(),
                    line_number: chunk.line_number,
                    content: chunk.content.clone(),
                    score,
                    match_type: MatchType::TfIdf,
                    start_char,
                    end_char,
                    context_before: None,
                    context_after: None,
                });
            }
        }

        // Sort by TF-IDF score (descending)
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(options.max_results);

        Ok(results)
    }

    /// Calculate TF-IDF score for a document against query terms
    fn calculate_tfidf_score(&self, query_tokens: &[String], chunk: &TextChunk) -> f32 {
        if query_tokens.is_empty() || chunk.tokens.is_empty() {
            return 0.0;
        }

        let mut score = 0.0;
        let document_length = chunk.tokens.len() as f32;

        // Calculate term frequency for the document
        let mut term_frequency = FxHashMap::default();
        for token in &chunk.tokens {
            *term_frequency.entry(token.clone()).or_insert(0) += 1;
        }

        for query_term in query_tokens {
            let tf = self.calculate_tf(query_term, &term_frequency, document_length);
            let idf = self.calculate_idf(query_term);
            let tfidf = tf * idf;

            score += tfidf;
        }

        // Normalize by query length
        let normalized_score = score / query_tokens.len() as f32;

        // Apply additional scoring factors
        let phrase_bonus = self.calculate_phrase_bonus(query_tokens, &chunk.tokens);
        let length_penalty = self.calculate_length_penalty(chunk.tokens.len());

        ((normalized_score + phrase_bonus) * length_penalty).min(1.0)
    }

    /// Calculate Term Frequency (TF) using log normalization
    fn calculate_tf(
        &self,
        term: &str,
        term_frequency: &FxHashMap<String, usize>,
        _document_length: f32,
    ) -> f32 {
        if let Some(&count) = term_frequency.get(term) {
            if count > 0 {
                // Log normalization: 1 + log(tf)
                1.0 + (count as f32).ln()
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Calculate Inverse Document Frequency (IDF)
    fn calculate_idf(&self, term: &str) -> f32 {
        let document_frequency = self.document_frequency.get(term).unwrap_or(&0);

        if *document_frequency == 0 {
            0.0
        } else {
            // IDF = log(N / df) where N is total documents, df is document frequency
            (self.total_documents as f32 / *document_frequency as f32).ln()
        }
    }

    /// Calculate bonus for phrase matches in TF-IDF context
    fn calculate_phrase_bonus(&self, query_tokens: &[String], document_tokens: &[String]) -> f32 {
        if query_tokens.len() < 2 {
            return 0.0;
        }

        let mut bonus: f32 = 0.0;

        // Look for consecutive query terms in document
        for window in document_tokens.windows(query_tokens.len()) {
            if window == query_tokens {
                bonus += 0.3; // Strong bonus for exact phrase match
            } else if self.is_partial_phrase_match(query_tokens, window) {
                bonus += 0.1; // Smaller bonus for partial phrase match
            }
        }

        bonus.min(0.5) // Cap the phrase bonus
    }

    /// Check if window contains partial phrase match
    #[allow(unknown_lints)]
    #[allow(clippy::manual_div_ceil)]
    fn is_partial_phrase_match(&self, query_tokens: &[String], window: &[String]) -> bool {
        let matches = query_tokens
            .iter()
            .zip(window.iter())
            .filter(|(q, w)| q == w)
            .count();

        matches >= (query_tokens.len() + 1) / 2 // At least half the terms match
    }

    /// Calculate penalty for very long or very short documents
    fn calculate_length_penalty(&self, document_length: usize) -> f32 {
        // Optimal document length is around 20-100 tokens
        let optimal_min = 20.0;
        let optimal_max = 100.0;
        let length = document_length as f32;

        if length < optimal_min {
            // Penalty for very short documents
            (length / optimal_min).max(0.5)
        } else if length > optimal_max {
            // Penalty for very long documents
            (optimal_max / length).max(0.7)
        } else {
            1.0 // No penalty for optimal length
        }
    }

    /// Find the best match position for highlighting
    fn find_best_match_position(&self, content: &str, query_tokens: &[String]) -> (usize, usize) {
        if query_tokens.is_empty() {
            return (0, content.len().min(50));
        }

        let content_lower = content.to_lowercase();
        let first_token = &query_tokens[0];

        // Try to find the first query term
        if let Some(start) = content_lower.find(first_token) {
            let mut end = start + first_token.len();

            // Try to extend to include more query terms
            for token in query_tokens.iter().skip(1) {
                if let Some(token_pos) = content_lower[end..].find(token) {
                    end = end + token_pos + token.len();
                } else {
                    break;
                }
            }

            (start, end)
        } else {
            (0, content.len().min(100))
        }
    }

    /// Get statistics about the TF-IDF index
    pub fn get_index_stats(&self) -> IndexStats {
        IndexStats {
            total_documents: self.total_documents,
            vocabulary_size: self.document_frequency.len(),
            most_common_terms: self.get_most_common_terms(10),
            least_common_terms: self.get_least_common_terms(10),
        }
    }

    fn get_most_common_terms(&self, limit: usize) -> Vec<(String, usize)> {
        let mut terms: Vec<_> = self.document_frequency.iter().collect();
        terms.sort_by(|a, b| b.1.cmp(a.1));
        terms
            .into_iter()
            .take(limit)
            .map(|(term, freq)| (term.clone(), *freq))
            .collect()
    }

    fn get_least_common_terms(&self, limit: usize) -> Vec<(String, usize)> {
        let mut terms: Vec<_> = self.document_frequency.iter().collect();
        terms.sort_by(|a, b| a.1.cmp(b.1));
        terms
            .into_iter()
            .take(limit)
            .map(|(term, freq)| (term.clone(), *freq))
            .collect()
    }
}

impl SearchStrategy for TfIdfSearch {
    fn name(&self) -> &str {
        "tfidf"
    }

    fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        // Simplified implementation for the trait
        let chunks = self.text_processor.process_file(query);
        self.search_chunks(query, &chunks, options)
    }

    fn required_resources(&self) -> ResourceRequirements {
        ResourceRequirements {
            min_memory_mb: 50,
            requires_ml: false,
            requires_index: true,
            cpu_intensive: true,
        }
    }
}

impl Default for TfIdfSearch {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the TF-IDF index
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total_documents: usize,
    pub vocabulary_size: usize,
    pub most_common_terms: Vec<(String, usize)>,
    pub least_common_terms: Vec<(String, usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::TextChunk;

    fn create_test_chunk(line_number: usize, content: &str) -> TextChunk {
        let processor = TextProcessor::new();
        TextChunk {
            line_number,
            content: content.to_string(),
            tokens: processor.tokenize(content),
            start_char: 0,
            end_char: content.len(),
            language_hint: None,
        }
    }

    #[test]
    fn test_tfidf_search_creation() {
        let search = TfIdfSearch::new();
        assert_eq!(search.name(), "tfidf");
        assert_eq!(search.total_documents, 0);

        let requirements = search.required_resources();
        assert_eq!(requirements.min_memory_mb, 50);
        assert!(!requirements.requires_ml);
        assert!(requirements.requires_index);
        assert!(requirements.cpu_intensive);
    }

    #[test]
    fn test_index_building() {
        let mut search = TfIdfSearch::new();
        let chunks = vec![
            create_test_chunk(1, "machine learning algorithm"),
            create_test_chunk(2, "deep learning networks"),
            create_test_chunk(3, "machine vision systems"),
        ];

        search.build_index(&chunks).unwrap();

        assert_eq!(search.total_documents, 3);
        assert!(search.document_frequency.contains_key("machine"));
        assert!(search.document_frequency.contains_key("learning"));
        assert_eq!(search.document_frequency.get("machine"), Some(&2)); // appears in 2 documents
        assert_eq!(search.document_frequency.get("learning"), Some(&2)); // appears in 2 documents
    }

    #[test]
    fn test_tf_calculation() {
        let search = TfIdfSearch::new();
        let mut term_frequency = FxHashMap::default();
        term_frequency.insert("test".to_string(), 3);
        term_frequency.insert("example".to_string(), 1);

        let tf_test = search.calculate_tf("test", &term_frequency, 10.0);
        let tf_example = search.calculate_tf("example", &term_frequency, 10.0);
        let tf_missing = search.calculate_tf("missing", &term_frequency, 10.0);

        assert!(tf_test > tf_example); // "test" appears more frequently
        assert_eq!(tf_missing, 0.0); // missing term has 0 TF
        assert!(tf_test > 1.0); // log normalization: 1 + ln(3)
    }

    #[test]
    fn test_idf_calculation() {
        let mut search = TfIdfSearch::new();
        let chunks = vec![
            create_test_chunk(1, "common word appears here"),
            create_test_chunk(2, "common word appears there"),
            create_test_chunk(3, "rare term only here"),
        ];

        search.build_index(&chunks).unwrap();

        let idf_common = search.calculate_idf("common");
        let idf_rare = search.calculate_idf("rare");
        let idf_missing = search.calculate_idf("missing");

        assert!(idf_rare > idf_common); // rare terms have higher IDF
        assert_eq!(idf_missing, 0.0); // missing terms have 0 IDF
    }

    #[test]
    fn test_tfidf_scoring() {
        let mut search = TfIdfSearch::new();
        let chunks = vec![
            create_test_chunk(1, "machine learning algorithm implementation"),
            create_test_chunk(2, "deep learning neural networks"),
            create_test_chunk(3, "computer vision image processing"),
        ];

        search.build_index(&chunks).unwrap();

        let results = search
            .search_chunks("machine learning", &chunks, &SearchOptions::default())
            .unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].line_number, 1); // Should rank highest
        assert_eq!(results[0].match_type, MatchType::TfIdf);
        assert!(results[0].score > 0.0);
    }

    #[test]
    fn test_phrase_bonus() {
        let search = TfIdfSearch::new();

        let query_tokens = vec!["machine".to_string(), "learning".to_string()];
        let doc1_tokens = vec![
            "machine".to_string(),
            "learning".to_string(),
            "algorithm".to_string(),
        ];
        let doc2_tokens = vec![
            "learning".to_string(),
            "machine".to_string(),
            "algorithm".to_string(),
        ];

        let bonus1 = search.calculate_phrase_bonus(&query_tokens, &doc1_tokens);
        let bonus2 = search.calculate_phrase_bonus(&query_tokens, &doc2_tokens);

        assert!(bonus1 > bonus2); // Exact phrase should get higher bonus
        assert!(bonus1 > 0.0);
    }

    #[test]
    fn test_length_penalty() {
        let search = TfIdfSearch::new();

        let penalty_short = search.calculate_length_penalty(5); // Very short
        let penalty_optimal = search.calculate_length_penalty(50); // Optimal
        let penalty_long = search.calculate_length_penalty(500); // Very long

        assert!(penalty_optimal > penalty_short);
        assert!(penalty_optimal > penalty_long);
        assert_eq!(penalty_optimal, 1.0);
    }

    #[test]
    fn test_find_best_match_position() {
        let search = TfIdfSearch::new();

        let content = "This is a machine learning example";
        let query_tokens = vec!["machine".to_string(), "learning".to_string()];

        let (start, end) = search.find_best_match_position(content, &query_tokens);

        assert!(start <= 10); // Should find "machine"
        assert!(end > start);
        assert!(end <= content.len());
    }

    #[test]
    fn test_index_stats() {
        let mut search = TfIdfSearch::new();
        let chunks = vec![
            create_test_chunk(1, "common word appears frequently"),
            create_test_chunk(2, "common word appears again"),
            create_test_chunk(3, "rare term only once"),
        ];

        search.build_index(&chunks).unwrap();
        let stats = search.get_index_stats();

        assert_eq!(stats.total_documents, 3);
        assert!(stats.vocabulary_size > 0);
        assert!(!stats.most_common_terms.is_empty());
        assert!(!stats.least_common_terms.is_empty());

        // "common" should be in most common terms
        assert!(stats
            .most_common_terms
            .iter()
            .any(|(term, _)| term == "common"));
    }

    #[test]
    fn test_empty_query_handling() {
        let mut search = TfIdfSearch::new();
        let chunks = vec![create_test_chunk(1, "some content")];

        search.build_index(&chunks).unwrap();
        let results = search
            .search_chunks("", &chunks, &SearchOptions::default())
            .unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_no_index_handling() {
        let search = TfIdfSearch::new();
        let chunks = vec![create_test_chunk(1, "some content")];

        // Search without building index
        let results = search
            .search_chunks("test", &chunks, &SearchOptions::default())
            .unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_score_normalization() {
        let mut search = TfIdfSearch::new();
        let chunks = vec![create_test_chunk(
            1,
            "test content with multiple test occurrences",
        )];

        search.build_index(&chunks).unwrap();
        let results = search
            .search_chunks("test", &chunks, &SearchOptions::default())
            .unwrap();

        if !results.is_empty() {
            assert!(results[0].score <= 1.0);
            assert!(results[0].score >= 0.0);
        }
    }

    #[test]
    fn test_partial_phrase_matching() {
        let search = TfIdfSearch::new();

        let query_tokens = vec![
            "machine".to_string(),
            "learning".to_string(),
            "algorithm".to_string(),
        ];
        let window1 = vec![
            "machine".to_string(),
            "learning".to_string(),
            "system".to_string(),
        ]; // 2/3 match
        let window2 = vec![
            "deep".to_string(),
            "learning".to_string(),
            "algorithm".to_string(),
        ]; // 2/3 match
        let window3 = vec![
            "computer".to_string(),
            "vision".to_string(),
            "system".to_string(),
        ]; // 0/3 match

        assert!(search.is_partial_phrase_match(&query_tokens, &window1));
        assert!(search.is_partial_phrase_match(&query_tokens, &window2));
        assert!(!search.is_partial_phrase_match(&query_tokens, &window3));
    }
}
