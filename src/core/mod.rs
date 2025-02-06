pub mod stream;
pub mod retry;
pub mod cache;

use std::time::Duration;
use std::sync::Arc;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use thiserror::Error;

use crate::api::{ApiError, LLMApi};

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Retry error: {0}")]
    Retry(String),

    #[error("Stream error: {0}")]
    Stream(String),

    #[error("Other error: {0}")]
    Other(String),
}

pub type CoreResult<T> = Result<T, CoreError>;

pub struct QueryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial retry delay
    pub retry_delay: Duration,
    /// Maximum retry delay
    pub max_retry_delay: Duration,
    /// Cache TTL
    pub cache_ttl: Duration,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Whether to stream responses
    pub stream_responses: bool,
    /// Whether to show progress
    pub show_progress: bool,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            max_retry_delay: Duration::from_secs(30),
            cache_ttl: Duration::from_secs(3600), // 1 hour
            max_cache_size: 1000,
            stream_responses: false, // Changed to false as default
            show_progress: true,
        }
    }
}

pub struct QueryEngine {
    client: Arc<dyn LLMApi>,
    config: QueryConfig,
    cache: cache::QueryCache,
    progress: Option<ProgressBar>,
}

impl QueryEngine {
    pub fn new(client: Arc<dyn LLMApi>, config: QueryConfig) -> Self {
        let cache = cache::QueryCache::new(config.max_cache_size, config.cache_ttl);
        Self {
            client,
            config,
            cache,
            progress: None,
        }
    }

    pub async fn query(&mut self, prompt: &str) -> CoreResult<String> {
        // Check cache first
        if let Some(cached_response) = self.cache.get(prompt) {
            return Ok(cached_response);
        }

        // Setup progress display
        let multi = MultiProgress::new();
        let spinner = multi.add(ProgressBar::new_spinner());
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        spinner.set_message("\x1B[90mConnecting... (model: gpt-4)\x1B[0m"); // Dark gray
        spinner.enable_steady_tick(Duration::from_millis(100));

        // Create text bar for non-streaming mode
        let text_bar = if !self.config.stream_responses {
            let bar = multi.add(ProgressBar::new_spinner());
            bar.set_style(
                ProgressStyle::default_spinner()
                    .template("{msg}")
                    .unwrap()
            );
            bar.set_message("Waiting for response...");
            Some(bar)
        } else {
            None
        };

        // Create done bar
        let done_bar = multi.add(ProgressBar::new(1));
        done_bar.set_style(
            ProgressStyle::default_bar()
                .template("{msg}")
                .unwrap()
        );
        done_bar.set_message("");

        // Spawn blocking thread for MultiProgress
        let m = Arc::new(multi);
        let m2 = Arc::clone(&m);
        let handle = tokio::task::spawn_blocking(move || {
            loop {
                m2.draw().ok();
                std::thread::sleep(Duration::from_millis(50));
            }
        });

        // Send query with retry
        let client = self.client.clone();
        let stream_responses = self.config.stream_responses;

        let operation = move || {
            let client = client.clone();
            async move {
                let result = if stream_responses {
                    self::stream::handle_streaming_response(client, prompt).await
                } else {
                    client.send_query(prompt).await.map_err(CoreError::Api)
                };

                result
            }
        };

        let response = match retry::with_backoff(
            operation,
            self.config.max_retries,
            self.config.retry_delay,
            self.config.max_retry_delay,
        ).await {
            Ok(response) => {
                spinner.finish_and_clear();
                if let Some(text_bar) = text_bar {
                    text_bar.finish_with_message(format!("\x1B[32m{}\x1B[0m", response));
                }
                done_bar.finish_with_message("\x1B[34mDone!\x1B[0m");
                response
            }
            Err(e) => {
                spinner.finish_with_message("\x1B[31mError!\x1B[0m");
                if let Some(text_bar) = text_bar {
                    text_bar.finish_with_message("\x1B[31mFailed to get response\x1B[0m");
                }
                done_bar.finish_with_message("\x1B[31mFailed: Check error above\x1B[0m");
                return Err(e);
            }
        };

        // Stop the progress drawing thread
        handle.abort();

        // Cache successful response
        self.cache.insert(prompt.to_string(), response.clone());

        Ok(response)
    }

    fn create_progress_bar(&self) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::time::sleep;
    use futures::stream;

    struct MockLLMApi {
        response: String,
        fail_count: std::sync::atomic::AtomicU32,
    }

    #[async_trait::async_trait]
    impl LLMApi for MockLLMApi {
        async fn send_query(&self, _prompt: &str) -> Result<String, ApiError> {
            let fails = self.fail_count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            if fails > 0 {
                sleep(Duration::from_millis(100)).await;
                Err(ApiError::Other("mock error".into()))
            } else {
                Ok(self.response.clone())
            }
        }

        async fn send_streaming_query(
            &self,
            _prompt: &str,
        ) -> Result<crate::api::StreamingResponse, ApiError> {
            let response = self.response.clone();
            Ok(Box::pin(stream::once(async move {
                Ok(response)
            })))
        }

        async fn validate_key(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_query_with_retry() {
        let mock_api = Arc::new(MockLLMApi {
            response: "test response".to_string(),
            fail_count: std::sync::atomic::AtomicU32::new(2),
        });

        let config = QueryConfig {
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_millis(500),
            show_progress: false,
            ..Default::default()
        };

        let mut engine = QueryEngine::new(mock_api, config);
        let result = engine.query("test prompt").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test response");
    }

    #[tokio::test]
    async fn test_query_caching() {
        let mock_api = Arc::new(MockLLMApi {
            response: "test response".to_string(),
            fail_count: std::sync::atomic::AtomicU32::new(0),
        });

        let config = QueryConfig {
            show_progress: false,
            ..Default::default()
        };

        let mut engine = QueryEngine::new(mock_api, config);
        
        // First query should hit the API
        let result1 = engine.query("test prompt").await.unwrap();
        
        // Second query should hit the cache
        let result2 = engine.query("test prompt").await.unwrap();
        
        assert_eq!(result1, result2);
    }
}
