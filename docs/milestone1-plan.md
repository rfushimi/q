# Milestone 1: CLI Scaffolding Implementation Plan

## Overview
This milestone focuses on setting up the basic CLI structure using `clap`, establishing the foundational architecture, and implementing the help command functionality. This is the first step in building the `q` CLI tool.

## Goals
1. Set up the basic project structure following the design doc
2. Implement basic CLI argument parsing with `clap`
3. Establish help command functionality
4. Set up initial testing framework

## Implementation Details

### 1. Project Structure
```
src/
 ├── main.rs              // Entry point, basic error handling
 ├── cli/
 │    ├── mod.rs         // CLI module public interface
 │    └── args.rs        // Clap argument definitions
 ├── utils/
 │    ├── mod.rs         // Utils module public interface
 │    └── errors.rs      // Custom error types
 └── tests/
      └── cli_tests.rs   // CLI integration tests
```

### 2. Dependencies
```toml
[dependencies]
clap = { version = "4.4", features = ["derive"] }
thiserror = "1.0"        # For error handling
colored = "2.0"          # For colored output

[dev-dependencies]
assert_cmd = "2.0"       # For CLI testing
predicates = "3.0"       # For test assertions
```

### 3. CLI Structure
```rust
// Planned structure for args.rs
#[derive(Parser)]
#[command(name = "q")]
#[command(about = "CLI tool for querying LLMs")]
pub struct Cli {
    #[arg(help = "The prompt to send to the LLM")]
    prompt: Option<String>,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Set API key")]
    SetKey {
        #[arg(help = "The API key to set")]
        key: String,
    },
    // More commands will be added in future milestones
}
```

### 4. Error Handling
- Create custom error types in `utils/errors.rs`
- Implement `Display` and `Error` traits
- Use `thiserror` for derive macros

### 5. Testing Strategy

#### Unit Tests
- Test argument parsing
- Test help text formatting
- Test error handling

#### Integration Tests
- Test help command (`q --help`)
- Test invalid arguments
- Test version display

## Success Criteria
1. Running `q --help` displays correct usage information
2. Basic argument structure is validated
3. All tests pass
4. Error messages are clear and helpful

## Future Considerations
1. Prepare for config module integration (milestone 2)
2. Design for extensibility (future commands)
3. Consider shell completion groundwork

## Testing Commands
```bash
# These commands should work after implementation
q --help              # Display help
q --version          # Display version
q "test prompt"      # Error (API not implemented yet)
q invalid-command    # Error with helpful message
```

## Notes
- Following repository pattern for future data access
- Implementing proper error handling from the start
- Setting up modular structure for easy extension
