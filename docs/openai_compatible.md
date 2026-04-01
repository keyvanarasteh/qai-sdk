<p align="center">
  <img src="../assets/openaic_cover.png" alt="OpenAI Compatible Module Banner" width="100%"/>
</p>

# OpenAI Compatible Provider (`qai_sdk::openai_compatible`)

The universal escape hatch. Since the OpenAI REST API format (`/chat/completions`) has become an industry standard, dozens of local and cloud inference engines implement it. This module lets you connect to **any** of them.

---

## Implemented Traits

| Trait | Support |
|---|---|
| `LanguageModel` | Chat, streaming, tool calling (if server supports it) |

---

## Supported Backends

| Backend | Typical Base URL | Notes |
|---|---|---|
| **Ollama** | `http://localhost:11434/v1` | Local models |
| **LM Studio** | `http://localhost:1234/v1` | Desktop GUI |
| **vLLM** | `http://localhost:8000/v1` | Production serving |
| **TensorRT-LLM** | `http://localhost:8000/v1` | NVIDIA optimized |
| **Azure OpenAI** | `https://NAME.openai.azure.com/openai/deployments/MODEL` | Enterprise |
| **Llama.cpp** | `http://localhost:8080/v1` | C++ inference |
| **Together AI** | `https://api.together.xyz/v1` | Cloud hosted |
| **Groq** | `https://api.groq.com/openai/v1` | Ultra-fast inference |
| **Fireworks AI** | `https://api.fireworks.ai/inference/v1` | Cloud hosted |

---

## Initialization

```rust
use qai_sdk::prelude::*;

// Local Ollama
let provider = create_openai_compatible(ProviderSettings {
    base_url: Some("http://localhost:11434/v1".into()),
    api_key: Some("ollama".into()),  // Ollama doesn't need real auth
    ..Default::default()
});
let model = provider.chat("llama3.2");

// LM Studio
let provider = create_openai_compatible(ProviderSettings {
    base_url: Some("http://localhost:1234/v1".into()),
    api_key: Some("not-needed".into()),
    ..Default::default()
});
let model = provider.chat("local-model");

// Together AI (cloud)
let provider = create_openai_compatible(ProviderSettings {
    base_url: Some("https://api.together.xyz/v1".into()),
    api_key: Some(std::env::var("TOGETHER_API_KEY").unwrap()),
    ..Default::default()
});
let model = provider.chat("meta-llama/Llama-3-70b-chat-hf");

// Groq
let provider = create_openai_compatible(ProviderSettings {
    base_url: Some("https://api.groq.com/openai/v1".into()),
    api_key: Some(std::env::var("GROQ_API_KEY").unwrap()),
    ..Default::default()
});
let model = provider.chat("llama-3.3-70b-versatile");
```

---

## Chat Generation

```rust
let result = model.generate(
    Prompt {
        messages: vec![
            Message { role: Role::User, content: vec![Content::Text { text: "Hello from local Llama!".into() }] },
        ],
    },
    GenerateOptions {
        model_id: "llama3.2".into(),
        max_tokens: Some(500),
        temperature: Some(0.7),
        ..Default::default()
    },
).await?;

println!("{}", result.text);
```

---

## Streaming

```rust
use futures::StreamExt;

let mut stream = model.generate_stream(prompt, options).await?;
while let Some(part) = stream.next().await {
    match part {
        StreamPart::TextDelta { delta } => print!("{delta}"),
        StreamPart::Finish { finish_reason } => println!("\n[{finish_reason}]"),
        _ => {}
    }
}
```

---

## Tool Calling

Works with backends that support OpenAI-compatible function calling (vLLM, Together AI, Groq, etc.):

```rust
let result = model.generate(
    prompt,
    GenerateOptions {
        model_id: "llama3.2".into(),
        tools: Some(vec![my_tool_definition]),
        ..Default::default()
    },
).await?;
```

---

## Feature Support Matrix

Feature availability depends on the target backend:

| Feature | Ollama | LM Studio | vLLM | Together | Groq |
|---|:---:|:---:|:---:|:---:|:---:|
| Chat | ✅ | ✅ | ✅ | ✅ | ✅ |
| Streaming | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tool Calling | ⚠️ | ⚠️ | ✅ | ✅ | ✅ |
| Vision | Model-dependent | Model-dependent | Model-dependent | ✅ | ✅ |
| JSON Mode | ⚠️ | ⚠️ | ✅ | ✅ | ✅ |

*⚠️ = depends on model and server version*
