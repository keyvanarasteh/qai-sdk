# QAI SDK

[![Crates.io](https://img.shields.io/crates/v/qai-sdk.svg)](https://crates.io/crates/qai-sdk)
[![Downloads](https://img.shields.io/crates/d/qai-sdk.svg)](https://crates.io/crates/qai-sdk)
[![Documentation](https://docs.rs/qai-sdk/badge.svg)](https://docs.rs/qai-sdk)
[![CI](https://github.com/keyvanarasteh/qai-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/keyvanarasteh/qai-sdk/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/qai-sdk.svg)](https://github.com/keyvanarasteh/qai-sdk#license)
[![GitHub stars](https://img.shields.io/github/stars/keyvanarasteh/qai-sdk.svg?style=social&label=Star)](https://github.com/keyvanarasteh/qai-sdk)

A modular, type-safe Rust SDK for AI providers. One unified API across **OpenAI**, **Anthropic Claude**, **Google Gemini**, **DeepSeek**, **xAI Grok**, and any **OpenAI-compatible** endpoint.

## Features

| Capability | OpenAI | Anthropic | Google | DeepSeek | xAI | Compatible |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| Chat / Language Model | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Streaming | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tool Calling | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Vision / Multimodal | ✅ | ✅ | ✅ | — | — | — |
| Embeddings | ✅ | — | ✅ | — | — | — |
| Image Generation | ✅ | — | ✅ | — | — | — |
| Speech (TTS) | ✅ | — | — | — | — | — |
| Transcription (STT) | ✅ | — | — | — | — | — |
| Text Completion | ✅ | — | — | — | — | — |
| Responses API | ✅ | — | — | — | — | — |

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
qai-sdk = "0.1"
tokio = { version = "1", features = ["full"] }
```

By default, all providers are enabled. To optimize compile times, disable default features and select only the providers you need:

```toml
[dependencies]
qai-sdk = { version = "0.1", default-features = false, features = ["openai", "anthropic"] }
```

### Basic Usage

```rust
use qai_sdk::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a provider
    let provider = create_openai(ProviderSettings {
        api_key: Some("sk-...".to_string()),
        ..Default::default()
    });

    // Get a chat model
    let model = provider.chat("gpt-4o");

    // Generate a response
    let result = model.generate(
        Prompt {
            messages: vec![Message {
                role: Role::User,
                content: vec![Content::Text {
                    text: "Hello, world!".to_string(),
                }],
            }],
        },
        GenerateOptions {
            model_id: "gpt-4o".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            top_p: None,
            stop_sequences: None,
            tools: None,
        },
    ).await?;

    println!("{}", result.text);
    Ok(())
}
```

### Streaming

```rust
use qai_sdk::prelude::*;
use futures::StreamExt;

let model = provider.chat("gpt-4o");
let mut stream = model.generate_stream(prompt, options).await?;

while let Some(part) = stream.next().await {
    match part {
        StreamPart::TextDelta { delta } => print!("{delta}"),
        StreamPart::Finish { finish_reason } => println!("\n[{finish_reason}]"),
        _ => {}
    }
}
```

### Switch Providers in One Line

```rust
// OpenAI
let provider = create_openai(settings.clone());
// Anthropic
let provider = create_anthropic(settings.clone());
// Google Gemini
let provider = create_google(settings.clone());
// DeepSeek
let provider = create_deepseek(settings.clone());
// xAI Grok
let provider = create_xai(settings.clone());
// Any OpenAI-compatible API
let provider = create_openai_compatible(settings);
```

## Documentation

Dive deep into specific provider features and initialization parameters in our comprehensive module docs:

- [Core Interoperability `qai_sdk::core`](docs/core.md)
- [OpenAI Provider `qai_sdk::openai`](docs/openai.md)
- [Anthropic Provider `qai_sdk::anthropic`](docs/anthropic.md)
- [Google Gemini Provider `qai_sdk::google`](docs/google.md)
- [DeepSeek Provider `qai_sdk::deepseek`](docs/deepseek.md)
- [xAI Grok Provider `qai_sdk::xai`](docs/xai.md)
- [OpenAI Compatible Provider `qai_sdk::openai_compatible`](docs/openai_compatible.md)

## Architecture

`qai-sdk` is a single, monolithic crate designed with zero-cost abstractions. Providers are organically separated via modular architecture and gated by Cargo features, keeping compile times fast when you only need specific integrations:

```
qai-sdk
├── core                — Core traits: LanguageModel, EmbeddingModel, ImageModel
├── openai              — OpenAI API (GPT, DALL-E, Whisper, TTS, Responses)
├── anthropic           — Anthropic API (Claude)
├── google              — Google API (Gemini)
├── deepseek            — DeepSeek API (via OpenAI-compatible pipeline)
├── xai                 — xAI API (Grok, via OpenAI-compatible pipeline)
└── openai_compatible   — Any OpenAI-compatible endpoint (Ollama, LM Studio)
```

## Examples

See the [`examples/`](examples/) directory for 17 comprehensive examples covering:

- Basic chat, streaming, and multimodal conversations
- Tool calling / function calling
- Embeddings, image generation, speech, and transcription
- OpenAI Responses API
- Error handling patterns
- Provider factory pattern
- OpenAI-compatible endpoints (Ollama, LM Studio, etc.)

Run an example:

```bash
cp .env.example .env
# Fill in your API keys
cargo run --example chat_basic
```

## Environment Variables

| Variable | Provider |
|---|---|
| `OPENAI_API_KEY` | OpenAI |
| `ANTHROPIC_API_KEY` | Anthropic |
| `GOOGLE_API_KEY` | Google Gemini |
| `DEEPSEEK_API_KEY` | DeepSeek |
| `XAI_API_KEY` | xAI |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

## Author

**Keyvan Arasteh** — [@keyvanarasteh](https://github.com/keyvanarasteh)
