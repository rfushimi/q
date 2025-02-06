use async_trait::async_trait;
use thiserror::Error;
use std::path::PathBuf;

pub mod directory;
pub mod file;
pub mod history;

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("History error: {0}")]
    History(String),

    #[error("Context too large: {0}")]
    TooLarge(String),

    #[error("Other error: {0}")]
    Other(String),
}

pub type ContextResult<T> = Result<T, ContextError>;

#[derive(Debug, Clone)]
pub enum ContextType {
    History,
    Directory,
    File(PathBuf),
}

#[derive(Debug)]
pub struct ContextData {
    pub context_type: ContextType,
    pub content: String,
}

#[async_trait]
pub trait ContextProvider: Send + Sync {
    /// Get the type of context this provider handles
    fn context_type(&self) -> ContextType;

    /// Get the context data
    async fn get_context(&self) -> ContextResult<ContextData>;
}

/// Configuration for context providers
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Maximum size in bytes for context data
    pub max_size: usize,
    /// Whether to include hidden files/directories
    pub include_hidden: bool,
    /// Maximum depth for directory traversal
    pub max_depth: Option<usize>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_size: 1024 * 1024, // 1MB
            include_hidden: false,
            max_depth: Some(3),
        }
    }
}

/// Helper function to check if a path should be included based on config
pub fn should_include_path(path: &PathBuf, config: &ContextConfig) -> bool {
    if !config.include_hidden {
        if let Some(file_name) = path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                if file_name_str.starts_with('.') {
                    return false;
                }
            }
        }
    }
    true
}

/// Helper function to validate context size
pub fn validate_size(size: usize, max_size: usize, context_type: &str) -> ContextResult<()> {
    if size > max_size {
        Err(ContextError::TooLarge(format!(
            "{} context size {} exceeds maximum {}",
            context_type, size, max_size
        )))
    } else {
        Ok(())
    }
}

/// Helper function to format file paths for display
pub fn format_path_for_display(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}
