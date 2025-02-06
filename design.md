**ONE-PAGER DESIGN DOC**

---

## 1. **Overview**

**Purpose:**  
A Rust-based CLI tool named **`q`** that queries LLM APIs (OpenAI/Gemini) from the command line. It supports:
- Simple queries (`q <prompt>`)
- Command suggestions (`@cmd`)
- Contextual queries (`@here`, `@file`, `@hist`)
- Smart Zsh autocompletion

**Goals:**  
1. Enable quick, ad-hoc LLM queries in the terminal.  
2. Provide context-aware prompts (shell history, file contents, directory listings).  
3. Keep the design modular, secure, and extensible.  
4. Offer robust error handling (retry, caching, rate-limiting).  

---

## 2. **Recommended Rust Libraries**

1. **HTTP / API Client**
   - **`reqwest`**  
     - Justification: Mature, async-ready, supports streaming (necessary for LLM tokens).  
     - Caveats: Must enable features like `json` and `stream` for streaming.  
     - Alternatives: `hyper` (lower-level), `surf` (simpler but fewer features).
2. **CLI Parsing**
   - **`clap`** (version 4+)  
     - Justification: Comprehensive feature set, built-in support for generating Zsh completions.  
     - Caveats: Can be verbose; prefer using derive macros for simpler usage.  
     - Alternatives: `argh` (simpler, but less robust).
3. **Async Runtime**
   - **`tokio`**  
     - Justification: Standard, widely supported, integrates seamlessly with `reqwest`.  
     - Caveats: Use minimal feature sets to keep binary size small.  
     - Alternatives: `async-std` or `smol` (less common for large ecosystem).
4. **Terminal UI / Color Output**
   - **`colored`** or **`ansi_term`**  
     - Justification: Straightforward, widely used for color output.  
     - Caveats: Minimal overhead; be mindful of Windows compatibility.  
     - Alternatives: `crossterm` if more complex TUI is needed.
5. **Configuration & Key Management**
   - **`directories`** + manual TOML parsing or **`config`** crate  
     - Justification: Manages standard OS config paths (`~/.config/q/`).  
     - Caveats: For small tool, you might just read a `.toml` file.  
     - Alternatives: **`rust-keyring`** for storing secrets if you want OS-level encryption.
6. **Autocompletion**  
   - **`clap`** can generate Zsh completion scripts on build/install.  
     - Justification: No separate library needed if using Clap’s built-in completion generator.
7. **Caching / Retry / Rate-Limiting**
   - **`reqwest_retry`** or custom exponential-backoff logic with `tokio::time`  
   - **`cached`** or a simple local file-based approach for caching frequent queries.

---

## 3. **High-Level Abstraction Design**

```
src/
 ├── main.rs
 ├── cli/
 │    └── mod.rs          // Clap-based argument parsing & subcommands
 ├── commands/
 │    └── mod.rs          // Handles logic for @cmd suggestions
 ├── context/
 │    └── mod.rs          // @here, @file, @hist logic (shell/directory/file context)
 ├── api/
 │    ├── mod.rs          // Common traits / interfaces for LLM
 │    ├── openai.rs       // OpenAI-specific logic
 │    └── gemini.rs       // Gemini-specific logic
 ├── completion/
 │    └── mod.rs          // Zsh autocompletion generation
 ├── config/
 │    └── mod.rs          // API key loading, saving, parsing config
 ├── core/
 │    └── mod.rs          // Core query engine, streaming response handling
 └── utils/
      └── mod.rs          // Shared helpers (logging, text formatting)
```

1. **`cli` Module:**  
   - Defines command-line structure using `clap`.  
   - Maps subcommands like `--set-key`, `@cmd`, etc. to corresponding handlers.
2. **`commands` Module:**  
   - Knows how to interpret `@cmd`, queries the LLM for tool suggestions, returns results in color.  
3. **`context` Module:**  
   - Gathers environment data: shell history (`@hist`), directory listings (`@here`), file content (`@file`).  
   - Merges this with user prompt before sending to the LLM.
4. **`api` Module:**  
   - Provides unified trait: `LLMApi { fn send_query(&self, prompt: &str) -> Result<...> }`.  
   - Implements separate structs for OpenAI, Gemini, etc.
5. **`completion` Module:**  
   - Generates Zsh completions via `clap` APIs.  
   - Possibly includes dynamic completions (e.g., scanning local directories).
6. **`config` Module:**  
   - Loads/stores user config from `~/.config/q/config.toml` or environment variables.  
   - Provides an interface for setting/getting API keys.
7. **`core` Module:**  
   - Orchestrates queries, handles streaming, handles fallback logic (retry, caching).
8. **`utils` Module:**  
   - Logging/tracing, formatting, small helpers.

---

## 4. **Implementation Plan & Milestones**

1. **CLI Scaffolding**  
   - **Action:** Use `clap` derive macros to define basic commands/subcommands.  
   - **Verification:** Running `q --help` shows correct usage and subcommands.
2. **Config & Key Management**  
   - **Action:** Implement `config` module to read/write `~/.config/q/config.toml`; add `--set-key` flow.  
   - **Verification:** Setting an API key works, re-launching the tool picks it up.
3. **Basic Query to LLM**  
   - **Action:** Implement `api::openai` or `api::gemini` to send queries; store response.  
   - **Verification:** `$ q "Hello?"` returns a valid LLM response.
4. **Context Injection**  
   - **Action:** Implement `@hist` (zsh history read), `@here` (directory listing), and optionally `@file`.  
   - **Verification:** `$ q @hist "Refine this script"`, `$ q @file main.rs "Review code"`.
5. **Command Suggestions**  
   - **Action:** Implement `@cmd` logic in `commands::mod`. Possibly parse the user’s question, look for "tool for X" patterns.  
   - **Verification:** `$ q @cmd "tool to profile execution time"` returns “hyperfine” in green.
6. **Streaming & Resilience**  
   - **Action:** Use `reqwest` streaming, add retry with backoff. Cache frequent queries in local file or memory.  
   - **Verification:** Large responses stream token by token without blocking; retried on network errors.
7. **Zsh Autocompletion**  
   - **Action:** Implement `completion::mod` with Clap’s `generate` function. Possibly add dynamic file completions.  
   - **Verification:** After installation, typing `q <tab>` suggests subcommands and/or file paths.
8. **Final Testing & Packaging**  
   - **Action:** Add integration tests, finalize binary distribution.  
   - **Verification:** End-to-end tests passing, `cargo install --path .` works, `q` is functional in a real environment.

---

**URLs (References):**  
- Clap: <https://github.com/clap-rs/clap>  
- Reqwest: <https://github.com/seanmonstar/reqwest>  
- Tokio: <https://github.com/tokio-rs/tokio>  
- Colored: <https://github.com/mackwic/colored>  
- Config: <https://github.com/mehcode/config-rs>  
- Directories: <https://github.com/dirs-dev/directories-rs>  