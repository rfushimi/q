# Milestone 4: Context Injection Implementation Plan

## Overview
This milestone implements context injection capabilities, allowing users to include shell history, directory listings, and file contents in their LLM queries. This enhances the CLI's ability to provide context-aware responses.

## Goals
1. Implement shell history reading (`@hist`)
2. Implement directory listing (`@here`)
3. Implement file content reading (`@file`)
4. Add context parsing and injection
5. Update CLI to handle context flags

## Implementation Details

### 1. New Dependencies
```toml
[dependencies]
shellexpand = "3.1"     # For expanding shell paths
walkdir = "2.4"         # For recursive directory listing
```

### 2. Project Structure Updates
```
src/
 ├── context/
 │    ├── mod.rs       // Context module interface
 │    ├── history.rs   // Shell history handling
 │    ├── directory.rs // Directory listing
 │    └── file.rs      // File content reading
 └── tests/
      └── context_tests.rs  // Context module tests
```

### 3. Context Module Interface
```rust
// context/mod.rs
pub enum ContextType {
    History,
    Directory,
    File(String),  // Filename
}

pub trait ContextProvider {
    fn get_context(&self) -> Result<String, QError>;
}

pub struct Context {
    context_type: ContextType,
    // Additional fields as needed
}
```

### 4. Key Features

1. **Shell History Reading**
   - Read zsh history file
   - Parse and format history entries
   - Handle history file permissions
   - Support history size limits

2. **Directory Listing**
   - List files in current directory
   - Format listing for LLM context
   - Handle hidden files
   - Support recursive listing option

3. **File Content Reading**
   - Read specified files
   - Handle different file types
   - Support size limits
   - Handle encoding issues

4. **Context Injection**
   - Parse context flags from command line
   - Format context for LLM prompt
   - Support multiple contexts
   - Handle context size limits

### 5. Testing Strategy

#### Unit Tests
- Test history file parsing
- Test directory listing formatting
- Test file content reading
- Test context injection

#### Integration Tests
- Test `@hist` command
- Test `@here` command
- Test `@file` command
- Test multiple contexts

## Success Criteria
1. `q @hist "Refine this script"` returns relevant suggestions
2. `q @here "What are these files?"` provides accurate directory analysis
3. `q @file main.rs "Review code"` gives meaningful code review
4. All tests pass

## Security Considerations
1. Safe handling of file paths
2. Proper error handling for missing files
3. Size limits for context data
4. Secure handling of history data

## Testing Commands
```bash
# These commands should work after implementation
q @hist "Improve this command"
q @here "Explain these files"
q @file src/main.rs "Review this code"
q @hist @here "What was I working on?"
```

## Future Considerations
1. Support for more shell types
2. Caching of context data
3. Context size optimization
4. Custom context providers
5. Context templates
