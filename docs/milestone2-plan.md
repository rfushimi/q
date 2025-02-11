g# Milestone 2: Config & Key Management Implementation Plan

## Overview
This milestone focuses on implementing configuration management and API key storage functionality. The goal is to securely store and manage API keys for LLM services using the standard OS config directory.

## Goals
1. Implement config file management using the `directories` crate
2. Create a secure key storage mechanism
3. Implement the `--set-key` command flow
4. Add configuration validation

## Implementation Details

### 1. New Dependencies
```toml
[dependencies]
directories = "5.0"     # For standard OS config paths
serde = { version = "1.0", features = ["derive"] }  # For config serialization
toml = "0.8"           # For TOML config format
```

### 2. Project Structure Updates
```
src/
 ├── config/
 │    ├── mod.rs       // Config module public interface
 │    ├── paths.rs     // Config file path management
 │    └── types.rs     // Config data structures
 └── tests/
      └── config_tests.rs  // Config module tests
```

### 3. Configuration Structure
```rust
// types.rs
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub api_keys: ApiKeys,
    pub settings: Settings,
}

#[derive(Serialize, Deserialize)]
pub struct ApiKeys {
    pub openai: Option<String>,
    pub gemini: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub default_model: String,
    pub temperature: f32,
}
```

### 4. Key Features
1. **Config File Management**
   - Use `directories` to get standard config path (~/.config/q/config.toml)
   - Create directory if it doesn't exist
   - Load/save TOML configuration

2. **Key Storage**
   - Store API keys in config file
   - Support multiple LLM providers
   - Basic key validation (format check)

3. **Command Integration**
   - Update CLI to handle `--set-key` with provider selection
   - Add key validation before storage
   - Provide feedback on successful key storage

### 5. Testing Strategy

#### Unit Tests
- Test config file creation
- Test key storage and retrieval
- Test config validation
- Test path resolution

#### Integration Tests
- Test `--set-key` command
- Test config persistence
- Test invalid key handling

## Success Criteria
1. `--set-key` command successfully stores API key
2. Config file is created in correct location
3. Stored keys persist between program runs
4. Invalid keys are rejected with helpful errors
5. All tests pass

## Security Considerations
1. File permissions set to user-only read/write
2. Keys are never logged or exposed in error messages
3. Basic key format validation before storage

## Testing Commands
```bash
# These commands should work after implementation
q set-key openai sk-...    # Set OpenAI API key
q set-key gemini ...       # Set Gemini API key
q set-key --provider openai --key sk-...  # Alternative syntax
```

## Future Considerations
1. Encryption of stored keys
2. Support for environment variable override
3. Multiple key management for same provider
4. Key rotation support
