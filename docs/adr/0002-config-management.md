# ADR 0002: Configuration Management

## Status
Accepted

## Context
The `q` CLI tool needs to securely store and manage API keys for various LLM providers. We need to decide on:
1. Configuration file format and location
2. Key storage mechanism
3. Configuration structure that supports multiple providers

## Decision
1. Use TOML for configuration:
   - Human-readable and editable
   - Strong type safety with serde
   - Simple and widely used in Rust ecosystem

2. Use standard OS config directories:
   - Leverage `directories` crate for cross-platform support
   - Store in ~/.config/q/config.toml (Unix) or equivalent
   - Follow OS conventions for config storage

3. Implement modular config structure:
   - Separate API keys from general settings
   - Support multiple LLM providers
   - Allow for future extensibility

4. Security approach:
   - File permissions restricted to user
   - Basic key format validation
   - No key logging or exposure in errors
   - Environment variable override support

## Consequences
### Positive
- Standard location makes config easy to find and backup
- TOML format is human-readable and maintainable
- Modular structure allows easy addition of new providers
- Following OS conventions improves user experience

### Negative
- Plain text storage (future enhancement could add encryption)
- Manual backup needed (no cloud sync)
- Users need to set keys per machine

## References
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- [directories crate](https://crates.io/crates/directories)
- [TOML Format](https://toml.io/en/)
