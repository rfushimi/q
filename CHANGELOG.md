# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0/).

## [Unreleased]

### Added
- Initial project setup
- Basic CLI structure using clap
- Implementation plan for milestone 1
- Architecture Decision Record (ADR) for CLI structure
- Implementation plan for milestone 2 (Config & Key Management)
- Architecture Decision Record (ADR) for configuration management
- Implementation plan for milestone 3 (Basic Query to LLM)
- Architecture Decision Record (ADR) for LLM API integration
- API module with OpenAI integration
- Async support with tokio
- Streaming response capability
- Integration tests for API functionality
- Secure API key handling from external files
- Implementation plan for milestone 4 (Context Injection)
- Architecture Decision Record (ADR) for context injection
- Context module with providers for history, directory, and file
- Shell history reading support (@hist)
- Directory listing support (@here)
- File content reading support (@file)
- Context size limits and validation
- Integration tests for context providers
- Implementation plan for milestone 5 (Command Suggestions)
- Architecture Decision Record (ADR) for command suggestions
- Commands module with database and matcher
- Command suggestion support (@cmd)
- Colored output for suggestions
- Pattern matching for command queries
- Comprehensive command database
- Integration tests for command suggestions
- Implementation plan for milestone 6 (Streaming & Resilience)
- Architecture Decision Record (ADR) for streaming and resilience
- Core module with streaming, retry, and cache support
- Exponential backoff retry logic
- Query response caching
- Progress indicators
- Integration tests for resilience features
- Robust streaming implementation with MultiProgress
- Comprehensive streaming error handling
- Chunked response processing for streaming
- Unit tests for streaming functionality
- Optional streaming mode with --stream flag
- Progress display for non-streaming mode

### Changed
- Updated main.rs to support async operations
- Enhanced CLI to handle basic LLM queries
- Added context flags to CLI arguments
- Improved error handling with context-specific errors
- Enhanced prompt formatting with context injection
- Added command suggestion flag
- Improved error handling with command-specific errors
- Enhanced output formatting with colored suggestions
- Added streaming and caching flags
- Improved error handling with retry logic
- Enhanced progress indication
- Added debug mode support
- Redesigned streaming output with MultiProgress for better UX:
  - Added three-line display with spinner, response text, and status
  - Improved color coding (dark gray for status, green for response, blue for completion)
  - Enhanced error presentation with red error messages
  - Eliminated cursor jumping issues
  - Added proper cleanup on completion or error
- Improved streaming response handling:
  - Added robust chunk processing
  - Enhanced error detection in stream
  - Added support for SSE message parsing
  - Improved stream state management
- Changed streaming to opt-in:
  - Made non-streaming mode the default
  - Added --stream flag to enable streaming output
  - Added progress display for non-streaming mode
  - Updated CLI help documentation
- Enhanced non-streaming mode:
  - Added progress spinner during response generation
  - Added colored response output
  - Added completion status indicator
  - Improved error presentation

### Deprecated
- None

### Removed
- Old cursor manipulation-based streaming implementation
- Manual ANSI cursor positioning code
- Default streaming behavior (now opt-in with --stream)

### Fixed
- Streaming output stability and readability
- Terminal cursor positioning issues
- Progress bar flickering during streaming
- Error message clarity in streaming mode
- Stream error handling and propagation
- Chunked response processing
- SSE message parsing reliability
- Progress indication in non-streaming mode
- Response formatting consistency

### Security
- API keys stored in separate files outside of git
- Basic API key validation before usage
- Secure error handling to prevent key exposure
- Safe handling of file paths in context providers
- Size limits for context data
- Safe pattern matching for command suggestions
- Safe cache size limits
- Secure retry logic
- Safe stream handling
- Secure streaming response processing
