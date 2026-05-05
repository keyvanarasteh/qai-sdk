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

## Tool Calling & Structured Outputs

Because `qai-sdk` targets Ollama's OpenAI-compatible endpoint, Tool Calling works exactly as it does on OpenAI, provided your local model supports it (like `llama3.1` or `llama3.2`). The SDK also supports `response_format` for enforcing JSON Schemas natively via Ollama's structured output engine.

```rust
let result = model.generate(
    prompt,
    GenerateOptions {
        model_id: "llama3.2".into(),
        tools: Some(vec![my_tool]),
        // Enforce JSON outputs natively
        response_format: Some(json!({"type": "json_object"})),
        ..Default::default()
    },
).await?;
```

---

## Vision, Thinking & Embeddings

`OllamaModel` automatically parses and supports advanced multimodal capabilities:
- **Vision:** Pass images via `ImageSource::Base64` or `ImageSource::Url` and they will be routed accurately to vision models like `llava` or `gemma3`.
- **Embeddings:** Supports `model.embed(texts, options)` natively by wrapping the standard `/v1/embeddings` endpoint.
- **Thinking Mode:** DeepSeek-R1 style reasoning chunks emitted from Ollama natively bubble up as `StreamPart::ReasoningDelta`.

---

## Native Management API (Administrative)

Beyond standard chat endpoints, the `OllamaProvider` natively implements Ollama's proprietary `/api` lifecycle and administrative endpoints:

```rust
// Get a list of installed models
let tags = provider.list_models().await?;

// View currently running models and memory usage
let running = provider.list_running_models().await?;

// Get details about a specific model
let info = provider.show_model_info(OllamaShowRequest { model: "llama3.2".into(), verbose: None }).await?;

// Pull a model from the registry
provider.pull_model(OllamaPullRequest { model: "qwen3".into(), insecure: None, stream: None }).await?;

// Perform a Web Search via Ollama's new built-in tools
let search = provider.web_search(WebSearchRequest { query: "Ollama new engine".into(), max_results: Some(3) }).await?;
```

Supported management functions: `list_models`, `list_running_models`, `show_model_info`, `create_model`, `copy_model`, `delete_model`, `pull_model`, `push_model`, `get_version`, `web_search`, and `web_fetch`.

---

## Configuration Details

| Parameter | Default Local | Default Cloud |
|---|---|---|
| Base URL | `http://localhost:11434/v1` | `https://api.ollama.cloud/v1` |
| Protocol | `chat/completions` (OpenAI format) | `chat/completions` |
| Auth Header | None | `Authorization: Bearer <API_KEY>` |

---

## SDK Helpers (Configuration, Runner, & Modelfile)

Ollama relies heavily on server-side environment variables and custom JSON payloads. The `qai-sdk` provides specialized, strongly-typed builders to seamlessly configure and control these aspects from Rust:

### 1. `OllamaOptionsBuilder`
Allows passing strict Ollama-specific configuration options like `num_ctx` (context length) or `keep_alive` directly into the `custom` generation options:
```rust
use qai_sdk::ollama::OllamaOptionsBuilder;

let options = OllamaOptionsBuilder::new()
    .num_ctx(64000)
    .keep_alive("24h")
    .temperature(0.8)
    .build();
```

### 2. `ModelfileBuilder`
Programmatically build Dockerfile-like Modelfiles to import Safetensors/GGUFs or configure adapters:
```rust
use qai_sdk::ollama::ModelfileBuilder;

let modelfile = ModelfileBuilder::new()
    .from("llama3.2")
    .parameter("temperature", "0.8")
    .system("You are a helpful assistant.")
    .build();
```

### 3. `LocalOllamaRunner`
A wrapper around `std::process::Command` that lets you natively boot and configure the local Ollama server from Rust, managing GPU limits and parallel processing:
```rust
use qai_sdk::ollama::LocalOllamaRunner;

let child_process = LocalOllamaRunner::new()
    .host("0.0.0.0:11434")
    .context_length(64000)
    .max_loaded_models(3)
    .disable_cloud(true)
    .spawn()?;
```
