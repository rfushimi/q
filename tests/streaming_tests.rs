use futures::{stream, StreamExt};
use std::sync::Arc;
use tokio;

use q::api::{ApiError, LLMApi, StreamingResponse};
use q::core::stream::handle_streaming_response;

struct MockStreamingApi {
    chunks: Vec<String>,
}

#[async_trait::async_trait]
impl LLMApi for MockStreamingApi {
    fn model(&self) -> &str {
        "gpt-3.5-turbo"
    }

    async fn send_query(&self, _prompt: &str) -> Result<String, ApiError> {
        unimplemented!()
    }

    async fn send_streaming_query(
        &self,
        _prompt: &str,
    ) -> Result<StreamingResponse, ApiError> {
        let chunks = self.chunks.clone();
        let stream = stream::iter(chunks)
            .map(Ok)
            .then(|r| async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                r
            });
        Ok(Box::pin(stream))
    }

    async fn validate_key(&self) -> Result<(), ApiError> {
        Ok(())
    }
}

#[tokio::test]
#[ignore = "TODO: Fix test after implementing OpenAI SSE format"]
async fn test_basic_streaming() {
    let api = Arc::new(MockStreamingApi {
        chunks: vec![
            format!("data: {{\"choices\":[{{\"delta\":{{\"content\":\"Hello\"}}}}]}}\n\n"),
            format!("data: {{\"choices\":[{{\"delta\":{{\"content\":\", \"}}}}]}}\n\n"),
            format!("data: {{\"choices\":[{{\"delta\":{{\"content\":\"world\"}}}}]}}\n\n"),
            format!("data: {{\"choices\":[{{\"delta\":{{\"content\":\"!\"}}}}]}}\n\n"),
            format!("data: [DONE]\n\n"),
        ],
    });

    let result = handle_streaming_response(api, "test").await.unwrap();
    assert_eq!(result, "Hello, world!");
}

#[tokio::test]
#[ignore = "TODO: Fix test after implementing OpenAI SSE format"]
async fn test_streaming_error() {
    struct ErrorApi;

    #[async_trait::async_trait]
    impl LLMApi for ErrorApi {
        fn model(&self) -> &str {
            "gpt-3.5-turbo"
        }

        async fn send_query(&self, _prompt: &str) -> Result<String, ApiError> {
            unimplemented!()
        }

        async fn send_streaming_query(
            &self,
            _prompt: &str,
        ) -> Result<StreamingResponse, ApiError> {
            let stream = stream::iter(vec![
                Ok("data: {\"choices\":[{\"delta\":{\"content\":\"Token1 \"}}]}\n\n".to_string()),
                Ok("data: {\"error\":{\"message\":\"Simulated error\"}}\n\n".to_string()),
            ]).boxed();
            Ok(stream)
        }

        async fn validate_key(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }

    let api = Arc::new(ErrorApi);
    let result = handle_streaming_response(api, "test").await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore = "TODO: Fix test after implementing OpenAI SSE format"]
async fn test_empty_stream() {
    let api = Arc::new(MockStreamingApi {
        chunks: vec![],
    });

    let result = handle_streaming_response(api, "test").await.unwrap();
    assert_eq!(result, "");
}

#[tokio::test]
#[ignore = "TODO: Fix test after implementing OpenAI SSE format"]
async fn test_large_stream() {
    let chunks: Vec<String> = (0..1000)
        .map(|i| format!("Token{} ", i))
        .collect();
    
    let api = Arc::new(MockStreamingApi {
        chunks: chunks.clone(),
    });

    let result = handle_streaming_response(api, "test").await.unwrap();
    let expected = chunks.join("");
    assert_eq!(result, expected);
}

#[tokio::test]
#[ignore = "TODO: Fix test after implementing OpenAI SSE format"]
async fn test_special_characters() {
    let api = Arc::new(MockStreamingApi {
        chunks: vec![
            "Hello\n".to_string(),
            "世界\n".to_string(),
            "🌍\n".to_string(),
            "!".to_string(),
        ],
    });

    let result = handle_streaming_response(api, "test").await.unwrap();
    assert_eq!(result, "Hello\n世界\n🌍\n!");
}
