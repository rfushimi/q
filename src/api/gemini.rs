use std::time::Duration;
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{ApiError, ApiResult, LLMApi, ModelConfig, StreamingResponse};
use crate::cli::args::Verbosity;

const DEFAULT_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent";
const DEFAULT_MODEL: &str = "gemini-2.0-flash";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct GeminiClient {
    client: Client,
    api_key: String,
    api_url: String,
    model: String,
    config: ModelConfig,
    verbosity: Verbosity,
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Part {
    text: String,
}

impl Default for Part {
    fn default() -> Self {
        Self {
            text: String::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Debug, Deserialize)]
struct StreamResponse {
    candidates: Vec<StreamCandidate>,
}

#[derive(Debug, Deserialize)]
struct StreamCandidate {
    content: StreamContent,
}

#[derive(Debug, Deserialize)]
struct StreamContent {
    parts: Vec<Part>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ErrorDetail {
    message: String,
}

pub struct GeminiClientBuilder {
    api_key: String,
    api_url: String,
    model: String,
    config: ModelConfig,
    verbosity: Verbosity,
}

impl GeminiClientBuilder {
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

    pub fn build(self) -> GeminiClient {
        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .expect("Failed to create HTTP client");

        GeminiClient {
            client,
            api_key: self.api_key,
            api_url: self.api_url,
            model: self.model,
            config: self.config,
            verbosity: self.verbosity,
        }
    }
}

impl GeminiClient {
    pub fn builder(api_key: String) -> GeminiClientBuilder {
        GeminiClientBuilder::new(api_key)
    }

    fn get_system_prompt(&self) -> &str {
        match self.verbosity {
            Verbosity::Concise => "Be concise and to the point. Provide only essential information without unnecessary details or explanations.",
            Verbosity::Normal => "Provide balanced responses with moderate detail.",
            Verbosity::Detailed => "Provide detailed and comprehensive responses with thorough explanations and examples where appropriate.",
        }
    }

    fn build_request(&self, prompt: &str) -> GeminiRequest {
        let system_prompt = self.get_system_prompt();
        let combined_prompt = format!("{}\n\nUser request: {}", system_prompt, prompt);

        GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: combined_prompt,
                }],
            }],
            max_tokens: self.config.max_tokens,
        }
    }

    fn get_api_url(&self) -> String {
        self.api_url.clone()
    }

    fn process_stream_chunk(chunk: &[u8]) -> ApiResult<Option<String>> {
        let text = String::from_utf8_lossy(chunk);
        
        // Check for error response
        if let Ok(error) = serde_json::from_str::<ErrorResponse>(&text) {
            return Err(ApiError::Other(error.error.message));
        }

        // Try to parse as stream response
        if let Ok(response) = serde_json::from_str::<StreamResponse>(&text) {
            if let Some(candidate) = response.candidates.first() {
                let content = candidate.content.parts.iter()
                    .map(|part| part.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                if !content.is_empty() {
                    return Ok(Some(content));
                }
            }
        }

        Ok(None)
    }
}

#[async_trait]
impl LLMApi for GeminiClient {
    fn model(&self) -> &str {
        &self.model
    }

    async fn send_query(&self, prompt: &str) -> ApiResult<String> {
        let request = self.build_request(prompt);
        let url = self.get_api_url();
        
        let response = self.client
            .post(&url)
            .json(&request)
            .query(&[("key", self.api_key.clone())])
            .send()
            .await
            .map_err(ApiError::Network)?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            eprintln!("Gemini API error response: {}", error_text);
            return Err(ApiError::Other(error_text));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| ApiError::Other(format!("Failed to parse response: {}", e)))?;

        let content = gemini_response
            .candidates
            .first()
            .ok_or_else(|| ApiError::Other("No response candidates".to_string()))?
            .content
            .parts
            .iter()
            .map(|part| part.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        Ok(content)
    }

    async fn send_streaming_query(&self, prompt: &str) -> ApiResult<StreamingResponse> {
        let request = self.build_request(prompt);
        let url = self.get_api_url();

        let response = self.client
            .post(&url)
            .json(&request)
            .query(&[("key", self.api_key.clone())])
            .send()
            .await
            .map_err(ApiError::Network)?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            eprintln!("Gemini API error response (streaming): {}", error_text);
            return Err(ApiError::Other(error_text));
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
        let request = self.build_request("test");
        let url = self.get_api_url();
        
        let response = self.client
            .post(&url)
            .json(&request)
            .query(&[("key", self.api_key.clone())])
            .send()
            .await
            .map_err(ApiError::Network)?;

        match response.status().as_u16() {
            200 => Ok(()),
            401 => Err(ApiError::InvalidKey),
            429 => Err(ApiError::RateLimit),
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                eprintln!("Gemini API error response: {}", error_text);
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
    use serde_json::json;

    #[tokio::test]
    async fn test_send_query_success() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path(format!("/v1beta/models/gemini-pro:generateContent")))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "candidates": [{
                    "content": {
                        "parts": [{
                            "text": "Hello, world!"
                        }]
                    }
                }]
            })))
            .mount(&mock_server)
            .await;

        let client = GeminiClient::builder("test_key".to_string())
            .with_api_url(mock_server.uri())
            .build();

        let response = client.send_query("Hi").await.unwrap();
        assert_eq!(response, "Hello, world!");
    }

    #[tokio::test]
    async fn test_invalid_api_key() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path(format!("/v1beta/models/gemini-pro:generateContent")))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let client = GeminiClient::builder("invalid_key".to_string())
            .with_api_url(mock_server.uri())
            .build();

        let result = client.validate_key().await;
        assert!(matches!(result, Err(ApiError::InvalidKey)));
    }
}
