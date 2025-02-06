# Milestone 3: Basic Query to LLM Implementation Plan

## Overview
This milestone implements the core LLM query functionality, starting with OpenAI's API integration. The goal is to enable basic text queries that return LLM responses.

## Goals
1. Create the api module structure with common traits
2. Implement OpenAI API integration
3. Add streaming response handling
4. Implement basic error handling and retries
5. Add comprehensive testing

## Implementation Details

### 1. New Dependencies
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "stream"] }  # HTTP client with streaming
tokio = { version = "1.0", features = ["rt", "macros"] }      # Async runtime
serde_json = "1.0"                                            # JSON handling
futures = "0.3"                                               # Async utilities
thiserror = "1.0"                                            # Error handling
```

### 2. Project Structure Updates
```
src/
 ├── api/
 │    ├── mod.rs       // Common traits and types
 │    └── openai.rs    // OpenAI implementation
 └── tests/
      └── api_tests.rs // API integration tests
```

### 3. API Interface
```rust
// api/mod.rs
pub trait LLMApi {
    async fn send_query(&self, prompt: &str) -> Result<String, ApiError>;
    async fn send_streaming_query(
        &self,
        prompt: &str,
    ) -> Result<impl Stream<Item = Result<String, ApiError>>, ApiError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("Invalid API key")]
    InvalidKey,
    #[error("API error: {0}")]
    Other(String),
}
```

### 4. OpenAI Implementation
```rust
// api/openai.rs
pub struct OpenAIClient {
    client: reqwest::Client,
    api_key: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::new();
        Self { client, api_key }
    }
}

impl LLMApi for OpenAIClient {
    async fn send_query(&self, prompt: &str) -> Result<String, ApiError> {
        // Implementation
    }

    async fn send_streaming_query(
        &self,
        prompt: &str,
    ) -> Result<impl Stream<Item = Result<String, ApiError>>, ApiError> {
        // Implementation
    }
}
```

### 5. Key Features
1. **Basic Query Support**
   - Send text prompts to OpenAI
   - Parse and return responses
   - Handle API errors gracefully

2. **Streaming Support**
   - Stream responses token by token
   - Handle connection drops
   - Support cancellation

3. **Error Handling**
   - Network errors
   - Rate limiting
   - Invalid API keys
   - Malformed responses

4. **Integration with Config**
   - Use stored API keys
   - Support key validation
   - Handle missing keys

### 6. Testing Strategy

#### Unit Tests
- Test request building
- Test response parsing
- Test error handling
- Test streaming functionality

#### Integration Tests
- Test actual API calls
- Test rate limiting handling
- Test streaming responses
- Test invalid key scenarios

## Success Criteria
1. Basic query `q "Hello?"` returns valid response
2. Errors are handled gracefully with clear messages
3. API keys are properly used from config
4. All tests pass

## Security Considerations
1. API keys are never logged
2. Secure handling of responses
3. Rate limit adherence
4. Proper error masking in production

## Testing Commands
```bash
# These commands should work after implementation
q "What is Rust?"              # Basic query
q "Tell me a joke"             # Another simple query
q --debug "Test query"         # Show additional debug info
```

## Future Considerations
1. Gemini API integration
2. Response caching
3. Prompt templates
4. Model selection
5. Advanced parameter tuning
