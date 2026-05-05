<p align="center">
  <img src="../assets/ollama_cover.png" alt="Ollama Module Banner" width="100%"/>
</p>

# Ollama Provider (`qai_sdk::ollama`)

Integration with [Ollama](https://ollama.com) for running large language models natively locally or via Ollama Cloud. This provider intelligently uses Ollama's highly-recommended OpenAI-compatible `/v1` compatibility layer to guarantee robust support for Tool Calling, Structured Outputs, and Streaming.

---

## Implemented Traits

| Trait | Models |
|---|---|
| `LanguageModel` | Any local model (e.g. `llama3`, `mistral`, `gemma`) |
| `EmbeddingModel` | Any local embedding model (e.g. `nomic-embed-text`) |

---

## Initialization

The provider auto-detects if you are connecting locally or to the cloud based on your API Key and Base URL inputs.

### 1. Local Server (Default)
If you provide an empty configuration, the SDK automatically connects to `http://localhost:11434/v1`:

```rust
use qai_sdk::prelude::*;

// Connects to local Ollama on port 11434
let provider = create_ollama(ProviderSettings::default());
let model = provider.chat("llama3.2");
```

### 2. Ollama Cloud / Remote Server
If you provide an `api_key` or `OLLAMA_API_KEY` environment variable, the SDK automatically connects to `https://api.ollama.cloud/v1` using Bearer authentication:

```rust
use qai_sdk::prelude::*;

let provider = create_ollama(ProviderSettings {
    api_key: Some(std::env::var("OLLAMA_API_KEY").unwrap()),
    ..Default::default()
});
let model = provider.chat("llama3.2");
```

*(You can also explicitly override `base_url` in settings to point to a custom remote IP address).*

---

## Chat Generation

```rust
let result = model.generate(
    Prompt {
        messages: vec![
            Message { role: Role::System, content: vec![Content::Text { text: "You are a coding assistant.".into() }] },
            Message { role: Role::User, content: vec![Content::Text { text: "Write a binary search in Rust.".into() }] },
        ],
    },
    GenerateOptions {
        model_id: "llama3.2".into(),
        max_tokens: Some(2048),
        temperature: Some(0.3),
        ..Default::default()
    },
).await?;

println!("{}", result.text);
```

---

## Tool Calling

Because `qai-sdk` targets Ollama's OpenAI-compatible endpoint, Tool Calling works exactly as it does on OpenAI, provided your local model supports it (like `llama3.1` or `llama3.2`):

```rust
let result = model.generate(
    prompt,
    GenerateOptions {
        model_id: "llama3.2".into(),
        tools: Some(vec![my_tool]),
        ..Default::default()
    },
).await?;

for tc in &result.tool_calls {
    println!("Ollama invoked tool: {} -> {}", tc.name, tc.arguments);
}
```

---

## Configuration Details

| Parameter | Default Local | Default Cloud |
|---|---|---|
| Base URL | `http://localhost:11434/v1` | `https://api.ollama.cloud/v1` |
| Protocol | `chat/completions` (OpenAI format) | `chat/completions` |
| Auth Header | None | `Authorization: Bearer <API_KEY>` |
