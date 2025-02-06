use futures::StreamExt;
use q::api::{LLMApi, ApiError, ModelConfig};
use q::api::openai::OpenAIClient;
use std::sync::Arc;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_openai_query() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{
                "message": {
                    "content": "Test response"
                }
            }]
        })))
        .mount(&mock_server)
        .await;

    let client = OpenAIClient::builder("test_key".to_string())
        .with_api_url(format!("{}/v1/chat/completions", mock_server.uri()))
        .with_config(ModelConfig::default())
        .build();

    let result = client.send_query("test prompt").await;
    assert!(result.is_ok(), "Query failed: {}", result.unwrap_err());
    assert_eq!(result.unwrap(), "Test response");
}

#[tokio::test]
async fn test_openai_streaming() {
    let mock_server = MockServer::start().await;

    // Each chunk is sent as a separate SSE message
    let response_body = "\
        data: {\"choices\":[{\"delta\":{\"role\":\"assistant\"}}]}\n\n\
        data: {\"choices\":[{\"delta\":{\"content\":\"Test\"}}]}\n\n\
        data: {\"choices\":[{\"delta\":{\"content\":\" \"}}]}\n\n\
        data: {\"choices\":[{\"delta\":{\"content\":\"response\"}}]}\n\n\
        data: [DONE]\n\n";

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(response_body)
            .append_header("content-type", "text/event-stream")
            .append_header("transfer-encoding", "chunked"))
        .mount(&mock_server)
        .await;

    let client = Arc::new(OpenAIClient::builder("test_key".to_string())
        .with_api_url(format!("{}/v1/chat/completions", mock_server.uri()))
        .with_config(ModelConfig::default())
        .build());

    let result = client.send_streaming_query("test prompt").await;
    assert!(result.is_ok(), "Streaming query failed to start");

    let mut stream = result.unwrap();
    let mut response = String::new();

    // Process each chunk
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(text) => {
                println!("Received chunk: {:?}", text);
                response.push_str(&text);
            }
            Err(e) => panic!("Stream error: {}", e),
        }
    }

    assert_eq!(response, "Test response");
}

#[tokio::test]
async fn test_invalid_key() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {
                "message": "Invalid API key"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = OpenAIClient::builder("invalid_key".to_string())
        .with_api_url(format!("{}/v1/chat/completions", mock_server.uri()))
        .with_config(ModelConfig::default())
        .build();

    let result = client.send_query("test prompt").await;
    assert!(matches!(result, Err(ApiError::InvalidKey)));
}

#[tokio::test]
async fn test_rate_limit() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "error": {
                "message": "Rate limit exceeded"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = OpenAIClient::builder("test_key".to_string())
        .with_api_url(format!("{}/v1/chat/completions", mock_server.uri()))
        .with_config(ModelConfig::default())
        .build();

    let result = client.send_query("test prompt").await;
    assert!(matches!(result, Err(ApiError::RateLimit)));
}

#[tokio::test]
async fn test_streaming_error() {
    let mock_server = MockServer::start().await;

    // Send an error response in the middle of streaming
    let response_body = "\
        data: {\"choices\":[{\"delta\":{\"role\":\"assistant\"}}]}\n\n\
        data: {\"choices\":[{\"delta\":{\"content\":\"Test\"}}]}\n\n\
        data: {\"error\":{\"message\":\"Stream error\"}}\n\n";

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(response_body)
            .append_header("content-type", "text/event-stream")
            .append_header("transfer-encoding", "chunked"))
        .mount(&mock_server)
        .await;

    let client = Arc::new(OpenAIClient::builder("test_key".to_string())
        .with_api_url(format!("{}/v1/chat/completions", mock_server.uri()))
        .with_config(ModelConfig::default())
        .build());

    let result = client.send_streaming_query("test prompt").await;
    assert!(result.is_ok(), "Streaming query failed to start");

    let mut stream = result.unwrap();
    let mut response = String::new();

    // Process each chunk until error
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(text) => {
                println!("Received chunk: {:?}", text);
                response.push_str(&text);
            }
            Err(e) => {
                println!("Expected error: {}", e);
                assert!(e.to_string().contains("Stream error"));
                return;
            }
        }
    }

    panic!("Expected stream error but got complete response: {}", response);
}
