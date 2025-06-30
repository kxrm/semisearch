pub mod config;
pub mod processor;
pub mod error;
pub mod models;

/// Re-export common types
pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default search limit
pub const DEFAULT_LIMIT: usize = 10;

/// Test function for documentation
///
/// # Examples
///
/// ```
/// use testlib::search;
///
/// let results = search("test", ".").unwrap();
/// assert!(results.len() <= 10); // Default limit
/// ```
pub fn search(query: &str, path: &str) -> Result<Vec<models::SearchResult>> {
    let config = config::Config::default();
    let processor = processor::FileProcessor::new(config);
    
    // TODO: Implement async/await properly
    // For now, just use a blocking call to the async function
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(processor.process_directory(path.into(), query))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_search_with_defaults() {
        // FIXME: Create proper test fixtures
        let results = search("test", ".").unwrap();
        assert!(results.len() <= DEFAULT_LIMIT);
    }
    
    #[test]
    fn test_search_with_custom_config() {
        let config = config::Config::new()
            .with_limit(5)
            .with_verbose(true);
            
        let processor = processor::FileProcessor::new(config);
        
        // TODO: Implement proper async tests
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let results = runtime.block_on(processor.process_directory(PathBuf::from("."), "test")).unwrap();
        
        assert!(results.len() <= 5);
    }
} 