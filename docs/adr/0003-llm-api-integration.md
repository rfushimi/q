# ADR 0003: LLM API Integration

## Status
Accepted

## Context
The CLI tool needs to interact with LLM APIs (OpenAI/Gemini) to process user queries. We need to design a flexible and extensible API integration layer that can:
- Support multiple LLM providers
- Handle streaming responses
- Manage API keys securely
- Provide robust error handling
- Support future additions of new providers

## Decision
1. Create a trait-based abstraction for LLM APIs:
   - Define a common `LLMApi` trait that all providers must implement
   - Support both streaming and non-streaming query methods
   - Use async/await for all network operations

2. Start with OpenAI implementation:
   - Use reqwest for HTTP client with streaming support
   - Implement proper error handling with custom error types
   - Support API key loading from config or environment

3. Security measures:
   - Load API keys from separate files (~/keys/openai.key, ~/keys/gemini.key)
   - Never log or expose API keys in error messages
   - Ensure keys are not committed to git

4. Error handling strategy:
   - Create custom error types for different failure scenarios
   - Implement proper error conversion using thiserror
   - Provide clear error messages to users

## Consequences

### Positive
- Clean abstraction allows easy addition of new providers
- Consistent error handling across providers
- Secure key management
- Streaming support from the start
- Clear separation of concerns

### Negative
- Additional complexity from trait-based design
- Need to maintain compatibility across different provider APIs
- Must handle varying rate limits and quotas differently

## Implementation Notes
- Use reqwest with JSON and streaming features
- Implement retry logic for transient failures
- Support cancellation of streaming responses
- Keep provider-specific code isolated in separate modules

## Alternatives Considered
1. Direct API integration without abstraction
   - Rejected due to lack of flexibility
2. Synchronous API calls
   - Rejected due to blocking nature and poor UX
3. Storing keys in config file
   - Rejected in favor of separate key files for better security

## Updates
None yet
