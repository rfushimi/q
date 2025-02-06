use thiserror::Error;

#[derive(Error, Debug)]
pub enum QError {
    #[error("CLI error: {0}")]
    Cli(#[from] clap::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Context error: {0}")]
    Context(String),

    #[error("Usage error: {0}")]
    Usage(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

// Implement conversion from string types for convenience
impl From<String> for QError {
    fn from(err: String) -> QError {
        QError::Unknown(err)
    }
}

impl From<&str> for QError {
    fn from(err: &str) -> QError {
        QError::Unknown(err.to_string())
    }
}
