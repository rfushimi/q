pub mod cache;
pub mod retry;

use std::sync::Arc;
use std::time::Duration;
use indicatif::ProgressBar;

use crate::api::LLMApi;
use crate::cli::args::Verbosity;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("API error: {0}")]
    Api(#[from] crate::api::ApiError),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Retry error: {0}")]
    Retry(String),

    #[error("Other error: {0}")]
    Other(String),
}

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Clone)]
pub struct QueryConfig {
    pub max_retries: u32,
    pub show_progress: bool,
    pub cache_ttl: Duration,
    pub max_cache_size: usize,
    pub retry_delay: Duration,
    pub max_retry_delay: Duration,
    pub verbosity: Verbosity,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            show_progress: true,
            cache_ttl: Duration::from_secs(3600),
            max_cache_size: 1000,
            retry_delay: Duration::from_secs(1),
            max_retry_delay: Duration::from_secs(30),
            verbosity: Verbosity::default(),
        }
    }
}

pub struct QueryEngine {
    client: Arc<dyn LLMApi>,
    config: QueryConfig,
    progress: Option<ProgressBar>,
}

impl QueryEngine {
    pub fn new(client: Arc<dyn LLMApi>, config: QueryConfig) -> Self {
        Self {
            client,
            config,
            progress: None,
        }
    }

    pub async fn query(&mut self, prompt: &str) -> CoreResult<String> {
        let progress = self.create_progress_bar();
        progress.set_message("Generating...");

        let response = self.client.send_query(prompt)
            .await
            .map_err(CoreError::Api)?;

        progress.finish_and_clear();
        Ok(response)
    }

    fn create_progress_bar(&self) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb
    }
}
