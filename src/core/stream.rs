use std::sync::Arc;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};

use crate::api::LLMApi;
use super::{CoreError, CoreResult};

/// Handle a streaming response from the LLM API
pub async fn handle_streaming_response(client: Arc<dyn LLMApi>, prompt: &str) -> CoreResult<String> {
    let mut stream = client
        .send_streaming_query(prompt)
        .await
        .map_err(CoreError::Api)?;

    let progress = create_streaming_progress_bar();
    progress.set_message("Receiving response...");
    
    let mut response = String::new();

    // Create a new line for the response
    println!();
    
    // Move cursor up one line to keep progress bar at the bottom
    print!("\x1B[1A");
    std::io::Write::flush(&mut std::io::stdout())
        .map_err(|e| CoreError::Stream(e.to_string()))?;

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(token) => {
                // Save cursor position
                print!("\x1B[s");
                // Move cursor up one line
                print!("\x1B[1A");
                // Print token
                print!("{}", token);
                // Restore cursor position
                print!("\x1B[u");
                std::io::Write::flush(&mut std::io::stdout())
                    .map_err(|e| CoreError::Stream(e.to_string()))?;
                response.push_str(&token);
                progress.inc(1);
            }
            Err(e) => {
                progress.finish_with_message("Error!");
                return Err(CoreError::Api(e));
            }
        }
    }

    progress.finish_with_message("Done!");
    println!(); // Add newline after streaming
    Ok(response)
}

/// Create a progress bar for streaming responses
fn create_streaming_progress_bar() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;
    use std::time::Duration;

    struct MockStreamingApi {
        chunks: Vec<String>,
    }

    #[async_trait::async_trait]
    impl LLMApi for MockStreamingApi {
        async fn send_query(&self, _prompt: &str) -> Result<String, crate::api::ApiError> {
            unimplemented!()
        }

        async fn send_streaming_query(
            &self,
            _prompt: &str,
        ) -> Result<crate::api::StreamingResponse, crate::api::ApiError> {
            let chunks = self.chunks.clone();
            let stream = stream::iter(chunks)
                .map(Ok)
                .then(|r| async move {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    r
                });
            Ok(Box::pin(stream))
        }

        async fn validate_key(&self) -> Result<(), crate::api::ApiError> {
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
}
