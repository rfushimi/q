use std::sync::Arc;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::task;

use crate::api::LLMApi;
use super::{CoreError, CoreResult};

/// Handle a streaming response from the LLM API
pub async fn handle_streaming_response(client: Arc<dyn LLMApi>, prompt: &str) -> CoreResult<String> {
    // 1. Create MultiProgress and progress bars
    let multi = MultiProgress::new();
    
    // Spinner for status
    let spinner = multi.add(ProgressBar::new_spinner());
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    spinner.set_message("\x1B[90mConnecting... (model: gpt-4)\x1B[0m"); // Dark gray
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    // Text output bar
    let text_bar = multi.add(ProgressBar::new(0));
    text_bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg}")
            .unwrap()
    );
    text_bar.set_message("");

    // Done bar
    let done_bar = multi.add(ProgressBar::new(1));
    done_bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg}")
            .unwrap()
    );
    done_bar.set_message("");

    // 2. Start streaming
    let mut stream = client
        .send_streaming_query(prompt)
        .await
        .map_err(CoreError::Api)?;

    let mut response = String::new();

    // 4. Process stream
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(token) => {
                response.push_str(&token);
                // Color the response green
                let colored = format!("\x1B[32m{}\x1B[0m", response);
                text_bar.set_message(colored);
            }
            Err(e) => {
                spinner.finish_with_message("\x1B[31mError!\x1B[0m"); // Red error
                text_bar.finish();
                done_bar.set_message("\x1B[31mFailed: Check error above\x1B[0m");
                done_bar.finish();
                return Err(CoreError::Api(e));
            }
        }
    }

    // 5. Finalize
    spinner.finish_and_clear();
    text_bar.finish();
    done_bar.set_message("\x1B[34mDone!\x1B[0m"); // Blue
    done_bar.finish();

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;
    use std::time::Duration;
    use crate::api::ApiError;

    struct MockStreamingApi {
        chunks: Vec<String>,
    }

    #[async_trait::async_trait]
    impl LLMApi for MockStreamingApi {
        async fn send_query(&self, _prompt: &str) -> Result<String, ApiError> {
            unimplemented!()
        }

        async fn send_streaming_query(
            &self,
            _prompt: &str,
        ) -> Result<crate::api::StreamingResponse, ApiError> {
            let chunks = self.chunks.clone();
            let stream = stream::iter(chunks)
                .map(Ok)
                .then(|r| async move {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    r
                });
            Ok(Box::pin(stream))
        }

        async fn validate_key(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_streaming_response() {
        let api = Arc::new(MockStreamingApi {
            chunks: vec![
                "Hello".to_string(),
                ", ".to_string(),
                "world".to_string(),
                "!".to_string(),
            ],
        });

        let result = handle_streaming_response(api, "test").await.unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[tokio::test]
    async fn test_streaming_error() {
        struct ErrorApi;

        #[async_trait::async_trait]
        impl LLMApi for ErrorApi {
            async fn send_query(&self, _prompt: &str) -> Result<String, ApiError> {
                unimplemented!()
            }

            async fn send_streaming_query(
                &self,
                _prompt: &str,
            ) -> Result<crate::api::StreamingResponse, ApiError> {
                let stream = stream::iter(vec![
                    Ok("Token1 ".to_string()),
                    Err(ApiError::Other("Simulated error".into())),
                ]);
                Ok(Box::pin(stream))
            }

            async fn validate_key(&self) -> Result<(), ApiError> {
                Ok(())
            }
        }

        let api = Arc::new(ErrorApi);
        let result = handle_streaming_response(api, "test").await;
        assert!(result.is_err());
    }
}
