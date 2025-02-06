use std::path::PathBuf;
use q::api::{LLMApi, ApiError, ModelConfig, read_api_key};
use q::api::openai::OpenAIClient;

fn get_openai_key() -> String {
    let home = PathBuf::from(env!("HOME"));
    let key_path = home.join("keys").join("openai.key");
    read_api_key(key_path.to_str().unwrap())
        .expect("Failed to read OpenAI API key from ~/keys/openai.key")
}

#[tokio::test]
async fn test_openai_key_validation() {
    let client = OpenAIClient::new(get_openai_key());
    let result = client.validate_key().await;
    assert!(result.is_ok(), "API key validation failed: {:?}", result);
}

#[tokio::test]
async fn test_openai_basic_query() {
    let client = OpenAIClient::new(get_openai_key());
    let result = client.send_query("Say hello").await;
    assert!(result.is_ok(), "Query failed: {:?}", result);
    
    let response = result.unwrap();
    assert!(!response.is_empty(), "Response should not be empty");
}

#[tokio::test]
async fn test_openai_streaming_query() {
    use futures::StreamExt;
    
    let client = OpenAIClient::new(get_openai_key());
    let result = client.send_streaming_query("Count from 1 to 3").await;
    assert!(result.is_ok(), "Streaming query failed to start: {:?}", result);
    
    let mut stream = result.unwrap();
    let mut received_chunks = 0;
    
    while let Some(chunk) = stream.next().await {
        assert!(chunk.is_ok(), "Stream chunk error: {:?}", chunk);
        received_chunks += 1;
    }
    
    assert!(received_chunks > 0, "Should receive at least one chunk");
}

#[tokio::test]
async fn test_openai_invalid_key() {
    let client = OpenAIClient::new("invalid_key".to_string());
    let result = client.validate_key().await;
    assert!(matches!(result, Err(ApiError::InvalidKey)));
}

#[tokio::test]
async fn test_openai_with_config() {
    let config = ModelConfig {
        temperature: 0.0,  // Deterministic
        max_tokens: Some(10),  // Short response
    };
    
    let client = OpenAIClient::new_with_config(get_openai_key(), config);
    let result = client.send_query("Write a very long story").await;
    assert!(result.is_ok(), "Query failed: {:?}", result);
    
    let response = result.unwrap();
    assert!(!response.is_empty(), "Response should not be empty");
}
