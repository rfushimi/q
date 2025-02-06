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

## Installation

```bash
cargo install q
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
  -D, --here           Include current directory listing
  -F, --file <FILE>    Include file content
  -C, --cmd            Get command suggestions
      --stream         Enable streaming output
      --no-cache      Disable response caching
      --retries <N>    Maximum retry attempts [default: 3]
      --debug          Show debug information
  -P, --provider <PROVIDER>  Select LLM provider [default: openai]
  -h, --help          Print help
  -V, --version       Print version
```

## Development

Requirements:
- Rust 1.70 or later
- OpenAI API key (or other supported provider)

Build from source:
```bash
git clone https://github.com/yourusername/q.git
cd q
cargo build --release
```

Run tests:
```bash
cargo test
```

## License

MIT License - see [LICENSE](LICENSE) for details.
