use std::pin::Pin;
use futures::Stream;
use async_trait::async_trait;
use thiserror::Error;

pub mod openai;
pub mod gemini;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Invalid API key")]
    InvalidKey,
    
    #[error("API error: {0}")]
    Other(String),
}

impl ApiError {
    pub fn is_retryable(&self) -> bool {
        match self {
            ApiError::Network(_) => true,
            ApiError::RateLimit => true,
            ApiError::InvalidKey => false,
            ApiError::Other(_) => false,
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
pub type StreamingResponse = Pin<Box<dyn Stream<Item = ApiResult<String>> + Send>>;

#[async_trait]
pub trait LLMApi: Send + Sync {
    /// Sends a query to the LLM and returns the complete response
    async fn send_query(&self, prompt: &str) -> ApiResult<String>;

    /// Sends a query to the LLM and returns a stream of response tokens
    async fn send_streaming_query(&self, prompt: &str) -> ApiResult<StreamingResponse>;

    /// Validates the API key format and connectivity
    async fn validate_key(&self) -> ApiResult<()>;

    /// Returns the model name being used
    fn model(&self) -> &str;
}

/// Common configuration for LLM models
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: None,
        }
    }
}

/// Helper function to read API key from file
pub fn read_api_key(path: &str) -> std::io::Result<String> {
    std::fs::read_to_string(path)
        .map(|s| s.trim().to_string())
}
