use std::sync::Arc;
use futures::StreamExt;
use crate::api::LLMApi;
use super::{CoreError, CoreResult};

/// Handle a streaming response from the LLM API
pub async fn handle_streaming_response(client: Arc<dyn LLMApi>, prompt: &str) -> CoreResult<String> {
    let mut stream = client.send_streaming_query(prompt)
        .await
        .map_err(CoreError::Api)?;

    let mut full_response = String::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(text) => {
                print!("{}", text);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                full_response.push_str(&text);
            }
            Err(e) => return Err(CoreError::Api(e)),
        }
    }
    Ok(full_response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ApiError;
    use async_trait::async_trait;
    use futures::stream;

    struct TestApi;

    #[async_trait::async_trait]
    impl LLMApi for TestApi {
        fn model(&self) -> &str {
            "test-model"
        }

        async fn send_query(&self, _prompt: &str) -> Result<String, ApiError> {
            Ok("Hello, world!".to_string())
        }

        async fn send_streaming_query(
            &self,
            _prompt: &str,
        ) -> Result<crate::api::StreamingResponse, ApiError> {
            let chunks = vec!["Hello", ", ", "world", "!"];
            let stream = stream::iter(chunks)
                .map(|chunk| Ok(chunk.to_string()));
            Ok(Box::pin(stream))
        }

        async fn validate_key(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_streaming_response() {
        let api = Arc::new(TestApi);
        let result = handle_streaming_response(api, "test").await.unwrap();
        assert_eq!(result, "Hello, world!");
    }
}
