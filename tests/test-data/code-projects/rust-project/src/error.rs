use std::path::PathBuf;
use std::io;

/// Custom error type for the library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Path not found
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),
    
    /// Not a file
    #[error("Not a file: {0}")]
    NotAFile(PathBuf),
    
    /// Not a directory
    #[error("Not a directory: {0}")]
    NotADirectory(PathBuf),
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    /// Invalid query
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    
    /// Runtime error
    #[error("Runtime error: {0}")]
    Runtime(#[from] tokio::runtime::Error),
    
    /// Other error
    #[error("Error: {0}")]
    Other(String),
}

impl Error {
    /// Create a new error from a string
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::Other(msg.into())
    }
    
    /// Check if the error is a "not found" error
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::PathNotFound(_))
    }
    
    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::PathNotFound(path) => format!("Could not find the path: {}", path.display()),
            Self::NotAFile(path) => format!("Not a file: {}", path.display()),
            Self::NotADirectory(path) => format!("Not a directory: {}", path.display()),
            Self::Io(err) => format!("IO error: {}", err),
            Self::InvalidQuery(query) => format!("Invalid query: {}", query),
            Self::Runtime(err) => format!("Runtime error: {}", err),
            Self::Other(msg) => format!("Error: {}", msg),
        }
    }
} 