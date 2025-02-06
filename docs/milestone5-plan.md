# Milestone 5: Command Suggestions Implementation Plan

## Overview
This milestone implements the `@cmd` functionality to suggest command-line tools based on user queries. The system will analyze the user's question and recommend appropriate tools with explanations.

## Goals
1. Implement command suggestion logic
2. Create a command database
3. Add pattern matching for tool queries
4. Implement colored output for suggestions
5. Add comprehensive testing

## Implementation Details

### 1. New Dependencies
```toml
[dependencies]
regex = "1.10"        # For pattern matching
lazy_static = "1.4"   # For static command database
```

### 2. Project Structure Updates
```
src/
 ├── commands/
 │    ├── mod.rs       // Command module interface
 │    ├── database.rs  // Command database
 │    ├── matcher.rs   // Pattern matching logic
 │    └── suggest.rs   // Suggestion formatting
 └── tests/
      └── command_tests.rs  // Command module tests
```

### 3. Command Database Structure
```rust
// commands/database.rs
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub category: Category,
    pub examples: Vec<String>,
    pub keywords: Vec<String>,
}

pub enum Category {
    System,
    Network,
    FileSystem,
    Process,
    Performance,
    Development,
    Other,
}

pub struct CommandDatabase {
    commands: HashMap<String, CommandInfo>,
}
```

### 4. Key Features

1. **Command Database**
   - Comprehensive tool information
   - Categories and keywords
   - Usage examples
   - Common patterns

2. **Pattern Matching**
   - Keyword extraction
   - Intent recognition
   - Category matching
   - Fuzzy matching

3. **Suggestion Logic**
   - Relevance scoring
   - Multiple suggestions
   - Alternative tools
   - Usage hints

4. **Output Formatting**
   - Colored tool names
   - Example usage
   - Brief explanations
   - Alternative suggestions

### 5. Testing Strategy

#### Unit Tests
- Test pattern matching
- Test database lookups
- Test suggestion scoring
- Test output formatting

#### Integration Tests
- Test `@cmd` flag
- Test various query patterns
- Test edge cases
- Test colored output

## Success Criteria
1. `q @cmd "tool to profile execution time"` suggests "hyperfine"
2. Suggestions are relevant to queries
3. Output is properly colored
4. All tests pass

## Security Considerations
1. Safe command execution
2. Input validation
3. No arbitrary command execution
4. Safe pattern matching

## Testing Commands
```bash
# These commands should work after implementation
q @cmd "tool to profile execution time"
q @cmd "how to monitor disk usage"
q @cmd "find duplicate files"
q @cmd "process monitoring tool"
```

## Future Considerations
1. User-defined command database
2. Learning from user feedback
3. Command chaining suggestions
4. Platform-specific suggestions
5. Integration with package managers
