use crate::core::indexer::IndexStats;
use std::io::{self, Write};

/// Handles progress reporting for indexing operations
/// This separates the progress reporting concern from the core indexing logic
pub trait ProgressReporter {
    fn start_indexing(&self, path: &str);
    fn start_file_processing(&self, file_path: &str);
    fn file_processed(&self, chunks_created: usize, was_updated: bool);
    fn file_error(&self, error: &str);
    fn start_embedding_generation(&self);
    fn embedding_progress(&self, current: usize, total: usize);
    fn embedding_complete(&self, total: usize);
    fn indexing_complete(&self, stats: &IndexStats);
}

/// Silent progress reporter for basic mode
pub struct SilentReporter;

impl ProgressReporter for SilentReporter {
    fn start_indexing(&self, path: &str) {
        println!("Indexing directory: {path}");
    }

    fn start_file_processing(&self, _file_path: &str) {
        // Silent in basic mode
    }

    fn file_processed(&self, _chunks_created: usize, _was_updated: bool) {
        // Silent in basic mode
    }

    fn file_error(&self, _error: &str) {
        // Silent in basic mode - errors are handled elsewhere
    }

    fn start_embedding_generation(&self) {
        // Silent in basic mode
    }

    fn embedding_progress(&self, _current: usize, _total: usize) {
        // Silent in basic mode
    }

    fn embedding_complete(&self, _total: usize) {
        // Silent in basic mode
    }

    fn indexing_complete(&self, stats: &IndexStats) {
        println!("Indexing complete:");
        println!("  Files processed: {}", stats.files_processed);
        println!("  Files updated: {}", stats.files_updated);
        println!("  Files skipped: {}", stats.files_skipped);
        println!("  Chunks created: {}", stats.chunks_created);
        let duration = stats.duration_seconds;
        println!("  Duration: {duration:.2}s");
        println!("  Errors: {}", stats.errors.len());
    }
}

/// Advanced progress reporter with emojis and detailed information
pub struct AdvancedReporter {
    has_embeddings: bool,
}

impl AdvancedReporter {
    pub fn new(has_embeddings: bool) -> Self {
        Self { has_embeddings }
    }
}

impl ProgressReporter for AdvancedReporter {
    fn start_indexing(&self, path: &str) {
        println!("🔍 Indexing directory: {path}");
    }

    fn start_file_processing(&self, file_path: &str) {
        print!("📄 Processing: {file_path} ");
        let _ = io::stdout().flush();
    }

    fn file_processed(&self, chunks_created: usize, was_updated: bool) {
        if was_updated {
            println!("✅ Updated ({chunks_created} chunks)");
        } else {
            println!("⏭️  Skipped (no changes)");
        }
    }

    fn file_error(&self, error: &str) {
        println!("❌ Error: {error}");
    }

    fn start_embedding_generation(&self) {
        if self.has_embeddings {
            print!("🧠 Generating embeddings: ");
            let _ = io::stdout().flush();
        }
    }

    fn embedding_progress(&self, current: usize, total: usize) {
        if self.has_embeddings && current % 10 == 0 && current > 0 {
            print!("{current}/{total} ");
            let _ = io::stdout().flush();
        }
    }

    fn embedding_complete(&self, total: usize) {
        if self.has_embeddings && total > 0 {
            println!("{total}/{total} ✅");
        }
    }

    fn indexing_complete(&self, stats: &IndexStats) {
        println!("🎯 Indexing complete:");
        println!("  📊 Files processed: {}", stats.files_processed);
        println!("  🔄 Files updated: {}", stats.files_updated);
        println!("  ⏭️  Files skipped: {}", stats.files_skipped);
        println!("  📝 Chunks created: {}", stats.chunks_created);
        let duration = stats.duration_seconds;
        println!("  ⏱️  Duration: {duration:.2}s");
        println!("  ❌ Errors: {}", stats.errors.len());

        if self.has_embeddings {
            println!(
                "  🧠 Embeddings: Generated for {} chunks",
                stats.chunks_created
            );
        }

        // Show performance metrics
        let files_per_second =
            (stats.files_processed + stats.files_updated) as f64 / stats.duration_seconds;
        let chunks_per_second = stats.chunks_created as f64 / stats.duration_seconds;
        println!(
            "  🚀 Performance: {files_per_second:.1} files/sec, {chunks_per_second:.1} chunks/sec"
        );
    }
}

/// Factory for creating appropriate progress reporters
pub struct ProgressReporterFactory;

impl ProgressReporterFactory {
    /// Create a progress reporter based on advanced mode and embeddings availability
    pub fn create(advanced_mode: bool, has_embeddings: bool) -> Box<dyn ProgressReporter> {
        if advanced_mode {
            Box::new(AdvancedReporter::new(has_embeddings))
        } else {
            Box::new(SilentReporter)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silent_reporter_factory() {
        let reporter = ProgressReporterFactory::create(false, false);

        // Test that it doesn't panic when called
        reporter.start_indexing("test");
        reporter.start_file_processing("test.txt");
        reporter.file_processed(5, true);
        reporter.file_error("test error");

        let stats = IndexStats {
            files_processed: 1,
            files_updated: 2,
            files_skipped: 0,
            chunks_created: 10,
            total_size_bytes: 1000,
            duration_seconds: 1.5,
            errors: vec![],
        };
        reporter.indexing_complete(&stats);
    }

    #[test]
    fn test_advanced_reporter_factory() {
        let reporter = ProgressReporterFactory::create(true, true);

        // Test that it doesn't panic when called
        reporter.start_indexing("test");
        reporter.start_file_processing("test.txt");
        reporter.file_processed(5, true);
        reporter.start_embedding_generation();
        reporter.embedding_progress(10, 100);
        reporter.embedding_complete(100);

        let stats = IndexStats {
            files_processed: 1,
            files_updated: 2,
            files_skipped: 0,
            chunks_created: 10,
            total_size_bytes: 1000,
            duration_seconds: 1.5,
            errors: vec![],
        };
        reporter.indexing_complete(&stats);
    }
}
