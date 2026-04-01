# OpenAI Compatible Provider (`qai_sdk::openai_compatible`)

The OpenAI-Compatible Provider is the ultimate escape-hatch for AI engineering. Since the OpenAI REST API format `chat/completions` has essentially become an industry standard, many local inference engines mock it.

This module lets you easily target:
- Local **Ollama** servers
- **LM Studio**
- **vLLM** / **TensorRT-LLM**
- Azure OpenAI endpoints
- Llama.cpp servers

## Implemented Traits
- `LanguageModel`

## Initialization

You simply override the `base_url` within `ProviderSettings`:

```rust
use qai_sdk::prelude::*;

// Example: Pointing to a local LM Studio server running on port 1234
let provider = create_openai_compatible(ProviderSettings {
    base_url: Some("http://localhost:1234/v1".to_string()),
    api_key: Some("not-needed".to_string()), 
    ..Default::default()
});

let model = provider.chat("local-model-name-doesnt-matter");
```

## Supported Features
Everything supported by the underlying target server. If you point this at `vLLM` running `Llama-3`, you will instantly reap all features that server provides (streaming, function calling, logic) wrapped elegantly inside the `qai-sdk` `stream.next().await` standard abstraction.
