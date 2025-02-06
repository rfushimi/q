use std::sync::Arc;
use futures::StreamExt;
use crate::api::LLMApi;
use super::{CoreError, CoreResult};

/// Handle a streaming response from the LLM API
/// Currently disabled - will be implemented in a future update
pub async fn handle_streaming_response(client: Arc<dyn LLMApi>, prompt: &str) -> CoreResult<String> {
    // For now, just use the regular query method
    client.send_query(prompt).await.map_err(CoreError::Api)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ApiError;
    use async_trait::async_trait;

    #[tokio::test]
    async fn test_streaming_response() {
        struct TestApi;

        #[async_trait::async_trait]
        impl LLMApi for TestApi {
            async fn send_query(&self, _prompt: &str) -> Result<String, ApiError> {
                Ok("Hello, world!".to_string())
            }

            async fn send_streaming_query(
                &self,
                _prompt: &str,
            ) -> Result<crate::api::StreamingResponse, ApiError> {
                unimplemented!()
            }

            async fn validate_key(&self) -> Result<(), ApiError> {
                Ok(())
            }
        }

        let api = Arc::new(TestApi);
        let result = handle_streaming_response(api, "test").await.unwrap();
        assert_eq!(result, "Hello, world!");
    }
}
