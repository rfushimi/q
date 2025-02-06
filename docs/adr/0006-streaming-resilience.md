# ADR 0006: Streaming & Resilience

## Status
Accepted

## Context
The CLI tool needs to handle large responses efficiently and be resilient to network failures. We need to design a system that can:
- Stream responses token by token
- Retry failed requests
- Cache frequent queries
- Show progress to users

## Decision
1. Create a dedicated core module with:
   - Streaming response handler
   - Retry logic with backoff
   - Query cache
   - Progress indicators

2. Streaming Design:
   - Use reqwest streaming support
   - Process tokens incrementally
   - Support cancellation
   - Handle partial responses

3. Retry Strategy:
   - Implement exponential backoff
   - Classify errors for retry decisions
   - Set maximum retry attempts
   - Add jitter to prevent thundering herd

4. Caching Approach:
   - Use in-memory cache
   - Implement TTL-based expiration
   - Set size limits
   - Support cache invalidation

5. Progress Indication:
   - Show spinners for queries
   - Display progress bars for streaming
   - Indicate retry attempts
   - Show cache hits/misses

## Consequences

### Positive
- Better user experience with streaming
- More reliable operation with retries
- Faster responses for cached queries
- Clear progress indication
- Robust error handling

### Negative
- Increased complexity
- Memory usage for cache
- Need to manage cache invalidation
- Progress indicators may clutter output
- Retry logic may delay final failure

## Implementation Notes
- Use backoff crate for retry logic
- Use cached crate for query caching
- Use indicatif for progress bars
- Implement proper cleanup on cancellation
- Handle all error cases gracefully

## Alternatives Considered
1. Synchronous response handling
   - Rejected for poor UX with large responses
2. File-based caching
   - Rejected for simplicity and performance
3. Fixed retry intervals
   - Rejected in favor of exponential backoff
4. No progress indication
   - Rejected for poor user feedback

## Updates
None yet
