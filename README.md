# q - CLI Tool for Querying LLMs

A command-line interface tool for querying Large Language Models (LLMs) with advanced features like context injection, command suggestions, and streaming output.

## Features

- 🤖 Support for multiple LLM providers (OpenAI, etc.)
- 📝 Context injection from various sources:
  - Shell history (`--hist`)
  - Directory listings (`--here`)
  - File contents (`--file`)
- 💡 Command suggestions mode (`--cmd`)
- 🔄 Optional streaming output (`--stream`)
- 🎨 Beautiful progress display and colored output
- 💾 Response caching
- 🔁 Automatic retry with exponential backoff
- 🔒 Secure API key management

## Quick Install
Requirements:
- Git
- Rust and Cargo (will be automatically installed if missing)

### Using Homebrew

```bash
brew tap rfushimi/tap
brew install q
```

### Manual Installation

Alternatively, you can install with a single command:

```bash
curl -sSL https://raw.githubusercontent.com/rfushimi/q/refs/heads/main/install.sh | bash
```

This will:
1. Install Rust if needed
2. Clone the repository
3. Build from source
4. Install the binary to `~/.bin/` (default)
5. Add the directory to your PATH if needed

You can customize the installation directory by setting the `BIN_DIR` environment variable:

```bash
BIN_DIR=/usr/local/bin curl -sSL https://raw.githubusercontent.com/rfushimi/q/refs/heads/main/install.sh | bash
```

## Usage

Basic query:
```bash
q "What is Rust?"
```

With context:
```bash
# Include shell history context
q --hist "What did I do wrong in my last command?"

# Include current directory listing
q --here "What are the main source files?"

# Include file content
q --file src/main.rs "What does this code do?"
```

Command suggestions:
```bash
q --cmd "How do I find large files?"
```

Streaming output:
```bash
q --stream "Explain quantum computing"
```

## Configuration

API keys are stored in configuration files:
```bash
# Set OpenAI API key
q set-key openai YOUR_API_KEY
```

## Options

```
Options:
  -H, --hist           Include shell history context
  -P, --provider       Select LLM provider [default: gemini]
  -M, --model         Select model name (e.g., gemini-2.0-flash, gpt-3.5-turbo)
  -D, --here           Include current directory listing
  -F, --file <FILE>    Include file content
  -C, --cmd            Get command suggestions
      --stream         Enable streaming output
      --no-cache      Disable response caching
      --retries <N>    Maximum retry attempts [default: 3]
      --debug          Show debug information
  -h, --help          Print help
  -V, --version       Print version
```

## Development

Requirements:
- Rust 1.70 or later
- OpenAI API key (or other supported provider)

Build from source:
```bash
git clone https://github.com/rfushimi/q.git
cd q
cargo build --release

# install
cargo install --path . q
```

Run tests:
```bash
cargo test
```

## License

MIT License - see [LICENSE](LICENSE) for details.
