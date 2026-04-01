# QAI SDK

[![Crates.io](https://img.shields.io/crates/v/qai-sdk.svg)](https://crates.io/crates/qai-sdk)
[![Documentation](https://docs.rs/qai-sdk/badge.svg)](https://docs.rs/qai-sdk)
[![CI](https://github.com/keyvanarasteh/qai-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/keyvanarasteh/qai-sdk/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/qai-sdk.svg)](https://github.com/keyvanarasteh/qai-sdk#license)

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

Or pick individual providers:

```toml
[dependencies]
qai-core = "0.1"
qai-openai = "0.1"
qai-anthropic = "0.1"
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

## Architecture

```
qai-sdk (umbrella re-export crate)
├── qai-core           — Core traits: LanguageModel, EmbeddingModel, ImageModel, etc.
├── qai-openai         — OpenAI (GPT, DALL-E, Whisper, TTS, Responses API)
├── qai-anthropic      — Anthropic (Claude)
├── qai-google         — Google (Gemini)
├── qai-deepseek       — DeepSeek (via OpenAI-compatible)
├── qai-xai            — xAI (Grok, via OpenAI-compatible)
└── qai-openai-compatible — Any OpenAI-compatible endpoint
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
