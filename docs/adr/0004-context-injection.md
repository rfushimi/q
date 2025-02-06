# ADR 0004: Context Injection

## Status
Accepted

## Context
The CLI tool needs to support context injection to provide more relevant responses by including shell history, directory listings, and file contents in LLM queries. We need to design a flexible and extensible system for gathering and injecting different types of context.

## Decision
1. Create a dedicated context module with trait-based abstraction:
   - Define a `ContextProvider` trait that all context types must implement
   - Support different context types through enum variants
   - Allow multiple contexts to be combined

2. Implement three core context providers:
   - Shell history provider (`@hist`)
   - Directory listing provider (`@here`)
   - File content provider (`@file`)

3. Context handling strategy:
   - Parse context flags from command line arguments
   - Gather context data asynchronously
   - Format context data appropriately for LLM prompts
   - Support size limits and truncation

4. Security measures:
   - Validate file paths and prevent directory traversal
   - Handle permissions securely
   - Implement size limits for context data
   - Sanitize context data before sending to LLM

## Consequences

### Positive
- Clean abstraction allows easy addition of new context types
- Consistent handling across different context providers
- Secure handling of sensitive data
- Flexible combination of multiple contexts
- Clear separation of concerns

### Negative
- Additional complexity from trait-based design
- Need to handle large context data carefully
- Must manage memory usage with large files/directories
- Potential performance impact from gathering context

## Implementation Notes
- Use walkdir for efficient directory traversal
- Implement async context gathering where beneficial
- Cache context data when appropriate
- Support configurable size limits
- Handle different file encodings properly

## Alternatives Considered
1. Command-line parsing for each context type
   - Rejected due to lack of extensibility
2. Synchronous context gathering
   - Rejected due to potential performance impact
3. Single context provider
   - Rejected in favor of modular approach
4. Global context cache
   - Deferred for future consideration

## Updates
None yet
