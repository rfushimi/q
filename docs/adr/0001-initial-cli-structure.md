# ADR 0001: Initial CLI Structure

## Status
Accepted

## Context
The `q` CLI tool needs a robust and extensible command-line interface that can handle various types of queries and commands. We need to choose an appropriate CLI argument parsing library and establish the initial command structure.

## Decision
1. Use `clap` (v4+) with derive macros for argument parsing because:
   - Built-in support for generating help text
   - Strong typing through derive macros
   - Future support for shell completions
   - Active maintenance and wide adoption

2. Implement a two-level command structure:
   - Root level for direct LLM queries
   - Subcommands for utility functions (e.g., setting API keys)

3. Use the repository pattern for future data access to maintain clean separation of concerns

## Consequences
### Positive
- Type-safe argument parsing
- Automatic help text generation
- Easy to extend with new subcommands
- Clear separation of concerns

### Negative
- Slightly larger binary size due to derive macros
- More initial boilerplate compared to simpler CLI parsers

## References
- [Clap Documentation](https://docs.rs/clap)
- [Repository Pattern in Rust](https://rust-lang.github.io/api-guidelines/)
