use std::time::Duration;
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{ApiError, ApiResult, LLMApi, ModelConfig, StreamingResponse};

const API_URL: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_MODEL: &str = "gpt-3.5-turbo";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct OpenAIClient {
    client: Client,
    api_key: String,
    model: String,
    config: ModelConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatStreamResponse {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: DeltaContent,
}

#[derive(Debug, Deserialize)]
struct DeltaContent {
    content: Option<String>,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self::new_with_config(api_key, ModelConfig::default())
    }

    pub fn new_with_config(api_key: String, config: ModelConfig) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", api_key))
                .expect("Invalid API key format"),
        );

        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            model: DEFAULT_MODEL.to_string(),
            config,
        }
    }

    fn build_request(&self, prompt: &str, stream: bool) -> ChatRequest {
        ChatRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream,
        }
    }
}

#[async_trait]
impl LLMApi for OpenAIClient {
    async fn send_query(&self, prompt: &str) -> ApiResult<String> {
        let request = self.build_request(prompt, false);
        
        let response = self.client
            .post(API_URL)
            .json(&request)
            .send()
            .await
            .map_err(ApiError::Network)?;

        if !response.status().is_success() {
            match response.status().as_u16() {
                401 => return Err(ApiError::InvalidKey),
                429 => return Err(ApiError::RateLimit),
                _ => {
                    let error_text = response.text().await.unwrap_or_default();
                    return Err(ApiError::Other(error_text));
                }
            }
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| ApiError::Other(format!("Failed to parse response: {}", e)))?;

        Ok(chat_response
            .choices
            .first()
            .ok_or_else(|| ApiError::Other("No response choices".to_string()))?
            .message
            .content
            .clone())
    }

    async fn send_streaming_query(&self, prompt: &str) -> ApiResult<StreamingResponse> {
        let request = self.build_request(prompt, true);
        
        let response = self.client
            .post(API_URL)
            .json(&request)
            .send()
            .await
            .map_err(ApiError::Network)?;

        if !response.status().is_success() {
            match response.status().as_u16() {
                401 => return Err(ApiError::InvalidKey),
                429 => return Err(ApiError::RateLimit),
                _ => {
                    let error_text = response.text().await.unwrap_or_default();
                    return Err(ApiError::Other(error_text));
                }
            }
        }

        let stream = response
            .bytes_stream()
            .map(|result| {
                result
                    .map_err(ApiError::Network)
                    .and_then(|bytes| {
                        let text = String::from_utf8_lossy(&bytes);
                        if text.starts_with("data: ") {
                            let json_str = text.strip_prefix("data: ").unwrap();
                            if json_str.trim() == "[DONE]" {
                                return Ok(String::new());
                            }
                            if let Ok(chunk) = serde_json::from_str::<ChatStreamResponse>(json_str) {
                                if let Some(choice) = chunk.choices.first() {
                                    if let Some(content) = &choice.delta.content {
                                        return Ok(content.clone());
                                    }
                                }
                            }
                        }
                        Ok(String::new())
                    })
            })
            .filter_map(|result| async move {
                match result {
                    Ok(text) if !text.is_empty() => Some(Ok(text)),
                    Ok(_) => None,
                    Err(e) => Some(Err(e)),
                }
            });

        Ok(Box::pin(stream))
    }

    async fn validate_key(&self) -> ApiResult<()> {
        // Send a minimal query to validate the key
        let request = json!({
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": "test"
            }],
            "max_tokens": 1
        });

        let response = self.client
            .post(API_URL)
            .json(&request)
            .send()
            .await
            .map_err(ApiError::Network)?;

        match response.status().as_u16() {
            200 => Ok(()),
            401 => Err(ApiError::InvalidKey),
            429 => Err(ApiError::RateLimit),
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(ApiError::Other(error_text))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_send_query_success() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Hello, world!"
                    }
                }]
            })))
            .mount(&mock_server)
            .await;

        let client = OpenAIClient::new("test_key".to_string());
        let response = client.send_query("Hi").await.unwrap();
        
        assert_eq!(response, "Hello, world!");
    }

    #[tokio::test]
    async fn test_invalid_api_key() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let client = OpenAIClient::new("invalid_key".to_string());
        let result = client.send_query("Hi").await;
        
        assert!(matches!(result, Err(ApiError::InvalidKey)));
    }
}
