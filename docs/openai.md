# OpenAI Provider (`qai_sdk::openai`)

The OpenAI module provides a comprehensive implementation of the OpenAI API surface, fully covering GPT models, DALL-E, Whisper, TTS, and the specialized Responses API.

## Implemented Traits
- `LanguageModel` (GPT-4o, GPT-4, GPT-3.5)
- `EmbeddingModel` (text-embedding-3-large, text-embedding-3-small)
- `ImageModel` (DALL-E 2, DALL-E 3)
- `TranscriptionModel` (Whisper)
- `SpeechModel` (TTS)

## Initialization

```rust
use qai_sdk::prelude::*;
use qai_sdk::openai::OpenAIModel; // Optional direct import

// Using the universal provider factory
let provider = create_openai(ProviderSettings {
    api_key: Some(std::env::var("OPENAI_API_KEY").unwrap()),
    ..Default::default()
});

// Or construct specifically
let model = OpenAIModel::new(api_key);
```

## Supported Features
- **Multimodal (Vision)**: Supported. You can pass base64 encoded images in the `Content::Image` prompt block, and `gpt-4o` will read them.
- **Tool Calling**: Natively supported through JSON Schemas.
- **Streaming**: Full SSE implementation parsing text deltas and tool call deltas seamlessly.

## Specialized: Responses API
OpenAI recently introduced a specialized Responses API. This is supported natively via:
```rust
let responses_client = provider.responses("gpt-4o");
let result = responses_client.create(request).await?;
```
