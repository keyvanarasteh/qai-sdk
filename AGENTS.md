# AGENTS.md

This file provides context for AI coding assistants (Cursor, GitHub Copilot, Claude Code, Gemini, etc.) working with the QAI SDK.

## Project Overview

**QAI SDK** is a monolithic Rust crate providing a unified interface for 6+ AI providers (OpenAI, Anthropic, Google Gemini, DeepSeek, xAI, and any OpenAI-compatible endpoint). It includes structured output, a provider registry, composable middleware, a universal agent framework, and full MCP integration.

- **Repository**: https://github.com/keyvanarasteh/qai-sdk
- **Crate**: https://crates.io/crates/qai-sdk
- **License**: MIT OR Apache-2.0

## Repository Structure

```
qai-sdk/
├── src/
│   ├── lib.rs                  # Crate root, feature gates, public re-exports
│   ├── core/
│   │   ├── mod.rs              # Core module exports
│   │   ├── types.rs            # Prompt, Message, Role, Content, GenerateOptions, etc.
│   │   ├── error.rs            # ProviderError enum
│   │   ├── structured.rs       # generate_object(), ObjectGenerateOptions
│   │   ├── registry.rs         # ProviderRegistry, Provider trait
│   │   ├── middleware.rs       # LanguageModelMiddleware, wrap_language_model
│   │   └── agent.rs            # Agent builder, AgentResult, tool loop
│   ├── openai/                 # OpenAI provider (GPT, DALL-E, Whisper, TTS)
│   ├── anthropic/              # Anthropic provider (Claude)
│   ├── google/                 # Google Gemini provider
│   ├── deepseek/               # DeepSeek provider
│   ├── xai/                    # xAI / Grok provider
│   ├── openai_compatible/      # Generic OpenAI-compatible endpoint
│   └── mcp/                    # Model Context Protocol client & agent
├── examples/                   # Runnable examples
├── docs/                       # Module documentation with Mermaid diagrams
├── assets/                     # Cover images and visual assets
├── playground.html             # Interactive web playground
├── .github/workflows/          # CI and release pipelines
└── .agents/workflows/          # Agent workflows (this directory)
```

## Core Traits

| Trait | Purpose | Implemented By |
|---|---|---|
| `LanguageModel` | Chat generation & streaming | All 6 providers |
| `EmbeddingModel` | Text vector embeddings | OpenAI, Google |
| `ImageModel` | Image generation | OpenAI (DALL-E) |
| `SpeechModel` | Text-to-speech | OpenAI (TTS) |
| `TranscriptionModel` | Speech-to-text | OpenAI (Whisper) |
| `CompletionModel` | Legacy text completion | OpenAI |

## Feature Flags

```toml
[dependencies]
qai-sdk = { version = "0.1", features = ["openai", "anthropic", "google", "deepseek", "xai", "openai-compatible", "mcp"] }
```

| Feature | What it enables |
|---|---|
| `openai` | OpenAI GPT, DALL-E, Whisper, TTS |
| `anthropic` | Anthropic Claude |
| `google` | Google Gemini |
| `deepseek` | DeepSeek Chat/Coder |
| `xai` | xAI Grok |
| `openai-compatible` | Any OpenAI-compatible endpoint |
| `mcp` | Model Context Protocol client |

## Key Patterns

### Provider Initialization
```rust
use qai_sdk::prelude::*;
let provider = create_openai(ProviderSettings {
    api_key: Some(api_key),
    ..Default::default()
});
let model = provider.chat("gpt-4o");
```

### Error Handling
All SDK operations return `Result<T, ProviderError>`. Match on specific variants:
- `ProviderError::RateLimit` — 429 responses
- `ProviderError::Unauthorized` — auth failures
- `ProviderError::ContextLengthExceeded` — token limit
- `ProviderError::InvalidResponse` — malformed API response
- `ProviderError::NotSupported` — unsupported feature
- `ProviderError::Network` — HTTP errors
- `ProviderError::Configuration` — invalid config

## Development Commands

```bash
# Build
cargo build
cargo build --all-features

# Check
cargo check
cargo clippy --workspace --all-targets -- -D warnings

# Run tests
cargo test
cargo test --all-features

# Run examples
cargo run --example chat_basic
cargo run --example test_mcp --features mcp

# Release
cargo check && git tag v0.x.y && git push --tags
```

## Import Patterns

| What | Import |
|---|---|
| All common types | `use qai_sdk::prelude::*;` |
| Provider factory | `use qai_sdk::create_openai;` |
| Direct model | `use qai_sdk::OpenAIModel;` |
| Structured output | `use qai_sdk::core::structured::*;` |
| Registry | `use qai_sdk::core::registry::ProviderRegistry;` |
| Middleware | `use qai_sdk::core::middleware::*;` |
| Agent | `use qai_sdk::core::agent::Agent;` |
| MCP client | `use qai_sdk::mcp::client::McpClient;` |
| MCP agent | `use qai_sdk::mcp::agent::run_mcp_agent;` |
| Error types | `use qai_sdk::core::error::ProviderError;` |

## File Naming

- Source files: `snake_case.rs`
- Examples: `snake_case.rs` in `examples/`
- Docs: `snake_case.md` in `docs/`

## Do Not

- Use `unwrap()` in library code (use `?` or proper error handling)
- Add dependencies without updating `Cargo.toml` features appropriately
- Modify public API without updating docs
- Use `println!` in library code (use logging or return values)
