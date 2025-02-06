# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.html).

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

### Changed
- Updated main.rs to support async operations
- Enhanced CLI to handle basic LLM queries
- Added context flags to CLI arguments
- Improved error handling with context-specific errors
- Enhanced prompt formatting with context injection

### Deprecated
- None

### Removed
- None

### Fixed
- None

### Security
- API keys stored in separate files outside of git
- Basic API key validation before usage
- Secure error handling to prevent key exposure
- Safe handling of file paths in context providers
- Size limits for context data
