use futures::{stream, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio;

use q::api::{ApiError, LLMApi, StreamingResponse};
use q::core::stream::handle_streaming_response;

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
    ) -> Result<StreamingResponse, ApiError> {
        let chunks = self.chunks.clone();
        let stream = stream::iter(chunks)
            .then(|chunk| async move { Ok(chunk) })
            .boxed();
        Ok(stream)
    }

    async fn validate_key(&self) -> Result<(), ApiError> {
        Ok(())
    }
}

#[tokio::test]
async fn test_basic_streaming() {
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
        ) -> Result<StreamingResponse, ApiError> {
            let stream = stream::iter(vec![
                Ok("Token1 ".to_string()),
                Err(ApiError::Other("Simulated error".into())),
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
    if let Err(e) = result {
        assert!(e.to_string().contains("Simulated error"));
    }
}

#[tokio::test]
async fn test_empty_stream() {
    let api = Arc::new(MockStreamingApi {
        chunks: vec![],
    });

    let result = handle_streaming_response(api, "test").await.unwrap();
    assert_eq!(result, "");
}

#[tokio::test]
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
