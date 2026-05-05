<p align="center">
  <img src="../assets/xai_cover.png" alt="xAI Module Banner" width="100%"/>
</p>

# xAI Provider (`qai_sdk::xai`)

Integration with xAI's Grok models via their OpenAI-compatible API endpoint.

---

## Implemented Traits

| Trait | Models |
|---|---|
| `LanguageModel` | `grok-2`, `grok-4.3` (Reasoning), `grok-4-1-fast` (Reasoning), `grok-4.20-multi-agent` |
| `ImageModel` | `grok-imagine-image` |

---

## Initialization

```rust
use qai_sdk::prelude::*;

let provider = create_xai(ProviderSettings {
    api_key: Some(std::env::var("XAI_API_KEY").unwrap()),
    ..Default::default()
});

let model = provider.chat("grok-2");
```

### Direct Instantiation

```rust
use qai_sdk::XAIModel;
let model = XAIModel::new(api_key);
```

---

## Chat Generation

```rust
let result = model.generate(
    Prompt {
        messages: vec![
            Message { role: Role::System, content: vec![Content::Text { text: "You are Grok.".into() }] },
            Message { role: Role::User, content: vec![Content::Text { text: "Tell me something witty.".into() }] },
        ],
    },
    GenerateOptions {
        model_id: "grok-2".into(),
        max_tokens: Some(500),
        temperature: Some(0.9),
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

```rust
let result = model.generate(
    prompt,
    GenerateOptions {
        model_id: "grok-2".into(),
        tools: Some(vec![my_tool]),
        ..Default::default()
    },
).await?;

for tc in &result.tool_calls {
    println!("Grok tool call: {} -> {}", tc.name, tc.arguments);
}
```

---

## Configuration

| Parameter | Value |
|---|---|
| Base URL | `https://api.x.ai/v1` |
| Auth Header | `Authorization: Bearer <API_KEY>` |
| Protocol | OpenAI-compatible `chat/completions` |
| Streaming | Standard SSE format |
| System Prompts | Fully supported via `Role::System` |

## Vision (Multimodal)

xAI supports vision capabilities through the `grok-2-vision-1212` model. You can pass images as either base64-encoded strings or direct URLs using the standard `Content::Image` structure.

```rust
let image_url = "https://example.com/image.jpg";

let prompt = Prompt {
    messages: vec![Message {
        role: Role::User,
        content: vec![
            Content::Text { text: "What's in this image?".into() },
            Content::Image { source: ImageSource::Url { url: image_url.into() } },
        ],
    }],
};

let result = model.generate(prompt, GenerateOptions {
    model_id: "grok-2-vision-1212".into(),
    ..Default::default()
}).await?;
```

**Example:** [`xai_vision.rs`](../examples/xai_vision.rs)

---

## Image Generation

xAI provides image generation via the `grok-imagine-image` model. The SDK supports this natively through the `ImageModel` interface.

```rust
let image_model = provider.image("grok-imagine-image");

let result = image_model.generate(ImageGenerateOptions {
    model_id: "grok-imagine-image".into(),
    prompt: "A futuristic cyberpunk city at night".into(),
    n: Some(1),
    size: Some("1024x1024".into()),
    response_format: Some("url".into()), // Can also be "b64_json"
}).await?;

if let Some(url) = result.images.first() {
    println!("Generated Image URL: {}", url);
}
```

**Example:** [`xai_image_generation.rs`](../examples/xai_image_generation.rs)

---

## Reasoning / Thinking

Grok reasoning models think through problems step-by-step before delivering an answer. Reasoning tokens are exposed in usage metrics and the reasoning trace is available via the `reasoning` field.

### Model Behavior

| Model | Reasoning | Configuration |
|---|---|---|
| `grok-4.3` | ✅ Automatic | No `reasoning_effort` — errors if set |
| `grok-4-1-fast` | ✅ Automatic | No `reasoning_effort` — errors if set |
| `grok-4.20-multi-agent` | ✅ Multi-agent | `reasoning_effort` controls agent count (low=4, high/xhigh=16) |
| `grok-2` | ❌ | Standard chat |

### Basic Usage (grok-4.3)

Grok-4.3 reasons automatically — no special parameters needed:

```rust
let model = provider.chat("grok-4.3");

let result = model.generate(prompt, GenerateOptions {
    model_id: "grok-4.3".into(),
    max_tokens: Some(2048),
    // Do NOT set reasoning_effort — grok-4.3 will error
    ..Default::default()
}).await?;

// Reasoning trace is available in result.reasoning
if let Some(reasoning) = &result.reasoning {
    println!("Reasoning: {}", reasoning);
}
println!("Answer: {}", result.text);
```

### Streaming with Reasoning Deltas

Reasoning content arrives as `StreamPart::ReasoningDelta` events before the final response:

```rust
let mut stream = model.generate_stream(prompt, options).await?;

while let Some(part) = stream.next().await {
    match part {
        StreamPart::ReasoningDelta { delta } => {
            print!("{}", delta); // Reasoning tokens stream first
        }
        StreamPart::TextDelta { delta } => {
            print!("{}", delta); // Then the final answer
        }
        _ => {}
    }
}
```

### Responses API

The Responses API also supports reasoning, with `response.reasoning_summary_text.delta` events in streaming mode:

```rust
let responses_model = provider.responses("grok-4.3");
let result = responses_model.generate(prompt, options).await?;
```

### Important Caveats

- **`grok-4.3` and `grok-4-1-fast`**: Do NOT set `reasoning_effort` — it will return an error
- **`presencePenalty`, `frequencyPenalty`, `stop`**: Not supported by reasoning models
- **Reasoning tokens are billed** as part of total consumption
- **Timeouts**: Reasoning models may take longer; consider increasing HTTP timeouts
- **Encrypted reasoning**: Available via `include: ["reasoning.encrypted_content"]` in the Responses API for multi-turn context

### Example

- [`xai_reasoning.rs`](../examples/xai_reasoning.rs) — Auto-reasoning, streaming deltas, Responses API

---

## Audio & Voice

xAI supports high-quality text-to-speech (TTS), speech-to-text (STT), and conversational voice agents.

### Text-to-Speech (TTS)

```rust
let speech_model = provider.speech("grok-voice-tts");
let result = speech_model.generate(SpeechGenerateOptions {
    text: "Hello, I am Grok.".into(),
    voice: Some("grok-standard".into()),
    ..Default::default()
}).await?;
```

### Speech-to-Text (STT)

```rust
let trans_model = provider.transcription("grok-voice-stt");
let result = trans_model.generate(TranscriptionGenerateOptions {
    audio: audio_bytes,
    ..Default::default()
}).await?;
```

---

## Advanced Agentic Tools

xAI models (e.g., `grok-2`) support advanced server-side tools that don't require client-side implementation.

### Web Search & Code Execution

```rust
let options = GenerateOptions {
    server_tools: Some(vec![
        ServerTool { tool_type: "web_search".into(), ..Default::default() },
        ServerTool { tool_type: "code_execution".into(), ..Default::default() },
    ]),
    include_citations: Some(true),
    ..Default::default()
};

let result = model.generate(prompt, options).await?;

// View citations
for citation in result.citations {
    println!("Source: {} -> {}", citation.source, citation.snippet.unwrap_or_default());
}
```

### Collections Search (RAG)

Search through specific sets of URIs or documents provided in the request.

```rust
let options = GenerateOptions {
    server_tools: Some(vec![
        ServerTool { 
            tool_type: "collections_search".into(), 
            collection_uris: Some(vec!["https://docs.x.ai".into()]),
            ..Default::default() 
        },
    ]),
    ..Default::default()
};
```

---

## Prompt Caching

xAI automatically caches the prefix of your message history. When subsequent requests share the same prefix, cached tokens are served at reduced cost. **No code changes are needed** for basic caching — it happens transparently.

### How It Works

1. **First request** — full prompt is processed and cached server-side
2. **Subsequent requests** — if messages at the start match a previous request exactly, the matching prefix is served from cache
3. **Billing** — cached tokens are billed at a substantially lower rate

### Maximizing Cache Hits with `x-grok-conv-id`

By default, requests may be routed to different servers. Use the `x-grok-conv-id` header to route all requests in the same conversation to the same server, maximizing cache hit rates:

```rust
use std::collections::HashMap;

let mut headers = HashMap::new();
headers.insert("x-grok-conv-id".to_string(), "my-conversation-id".to_string());

let result = model.generate(prompt, GenerateOptions {
    model_id: "grok-3-fast".into(),
    max_tokens: Some(1024),
    extra_headers: Some(headers),
    ..Default::default()
}).await?;

// Check cache metrics
if let Some(cached) = result.usage.cache_hit_tokens {
    println!("Cached tokens: {} (saved on billing)", cached);
}
```

### What Breaks Caching

- **Editing earlier messages** — any modification to messages in the cached prefix invalidates the cache from that point
- **Removing messages** — deleting messages from the conversation history
- **Reordering messages** — changing the order of messages
- **Changing system prompt** — even minor edits to the system prompt break the cache

### Best Practices

1. **Keep system prompts static** — put long, stable context in the system prompt
2. **Only append new messages** — never edit or remove earlier messages
3. **Use `x-grok-conv-id`** via `extra_headers` for multi-turn conversations
4. **Reuse conversation IDs** — use the same `x-grok-conv-id` across a session

### Example

- [`xai_prompt_caching.rs`](../examples/xai_prompt_caching.rs) — Multi-turn conversation with cache metrics
