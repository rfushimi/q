use std::time::Duration;
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{ApiError, ApiResult, LLMApi, ModelConfig, StreamingResponse};
use crate::cli::args::Verbosity;

const DEFAULT_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_MODEL: &str = "gpt-3.5-turbo";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct OpenAIClient {
    client: Client,
    api_key: String,
    api_url: String,
    model: String,
    config: ModelConfig,
    verbosity: Verbosity,
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
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ErrorDetail {
    message: String,
}

pub struct OpenAIClientBuilder {
    api_key: String,
    api_url: String,
    model: String,
    config: ModelConfig,
    verbosity: Verbosity,
}

impl OpenAIClientBuilder {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            api_url: DEFAULT_API_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
            config: ModelConfig::default(),
            verbosity: Verbosity::default(),
        }
    }

    pub fn with_api_url(mut self, url: String) -> Self {
        self.api_url = url;
        self
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub fn with_config(mut self, config: ModelConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    pub fn build(self) -> OpenAIClient {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .expect("Invalid API key format"),
        );

        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        OpenAIClient {
            client,
            api_key: self.api_key,
            api_url: self.api_url,
            model: self.model,
            config: self.config,
            verbosity: self.verbosity,
        }
    }
}

impl OpenAIClient {
    pub fn builder(api_key: String) -> OpenAIClientBuilder {
        OpenAIClientBuilder::new(api_key)
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    fn get_system_prompt(&self) -> &str {
        match self.verbosity {
            Verbosity::Concise => "You are a helpful assistant. Be concise and to the point. Provide only essential information without unnecessary details or explanations.",
            Verbosity::Normal => "You are a helpful assistant. Provide balanced responses with moderate detail.",
            Verbosity::Detailed => "You are a helpful assistant. Provide detailed and comprehensive responses with thorough explanations and examples where appropriate.",
        }
    }

    fn build_request(&self, prompt: &str, stream: bool) -> ChatRequest {
        ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: self.get_system_prompt().to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream,
        }
    }

    fn process_stream_chunk(chunk: &[u8]) -> ApiResult<Option<String>> {
        let text = String::from_utf8_lossy(chunk);
        let mut content = String::new();

        for line in text.lines() {
            if !line.starts_with("data: ") {
                continue;
            }

            let data = &line["data: ".len()..];
            if data.trim() == "[DONE]" {
                continue;
            }

            // Check for error response
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(data) {
                return Err(ApiError::Other(error.error.message));
            }

            // Try to parse as stream response
            if let Ok(chunk) = serde_json::from_str::<ChatStreamResponse>(data) {
                if let Some(choice) = chunk.choices.first() {
                    if let Some(token) = &choice.delta.content {
                        content.push_str(token);
                    }
                }
            }
        }

        if content.is_empty() {
            Ok(None)
        } else {
            Ok(Some(content))
        }
    }
}

#[async_trait]
impl LLMApi for OpenAIClient {
    fn model(&self) -> &str {
        &self.model
    }

    async fn send_query(&self, prompt: &str) -> ApiResult<String> {
        let request = self.build_request(prompt, false);
        
        let response = self.client
            .post(&self.api_url)
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
            .post(&self.api_url)
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
                        Self::process_stream_chunk(&bytes)
                            .map(|opt_content| opt_content.unwrap_or_default())
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
            .post(&self.api_url)
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

        let client = OpenAIClient::builder("test_key".to_string())
            .with_api_url(mock_server.uri())
            .build();

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

        let client = OpenAIClient::builder("invalid_key".to_string())
            .with_api_url(mock_server.uri())
            .build();

        let result = client.send_query("Hi").await;
        assert!(matches!(result, Err(ApiError::InvalidKey)));
    }

    #[tokio::test]
    async fn test_process_stream_chunk() {
        // Test regular content
        let chunk = b"data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n";
        assert_eq!(OpenAIClient::process_stream_chunk(chunk).unwrap(), Some("Hello".to_string()));

        // Test role message
        let chunk = b"data: {\"choices\":[{\"delta\":{\"role\":\"assistant\"}}]}\n\n";
        assert_eq!(OpenAIClient::process_stream_chunk(chunk).unwrap(), None);

        // Test [DONE] message
        let chunk = b"data: [DONE]\n\n";
        assert_eq!(OpenAIClient::process_stream_chunk(chunk).unwrap(), None);

        // Test error message
        let chunk = b"data: {\"error\":{\"message\":\"Stream error\"}}\n\n";
        assert!(OpenAIClient::process_stream_chunk(chunk).is_err());
        assert_eq!(
            OpenAIClient::process_stream_chunk(chunk).unwrap_err().to_string(),
            "Stream error"
        );

        // Test multiple chunks in one message
        let chunk = b"data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\" World\"}}]}\n\n";
        assert_eq!(OpenAIClient::process_stream_chunk(chunk).unwrap(), Some("Hello World".to_string()));
    }
}
