<p align="center">
  <img src="../assets/anthropic_cover.png" alt="Anthropic Module Banner" width="100%"/>
</p>

# Anthropic Provider (`qai_sdk::anthropic`)

Complete integration with the Anthropic Messages API for Claude models. Handles Anthropic's unique API structure — separated system prompts, content blocks, and tool_use patterns — transparently.

---

## Implemented Traits

| Trait | Models |
|---|---|
| `LanguageModel` | `claude-opus-4-7`, `claude-opus-4-6`, `claude-sonnet-4-6`, `claude-sonnet-4-5-20250514`, `claude-haiku-4-5-20250514` |

---

## Initialization

```rust
use qai_sdk::prelude::*;

let provider = create_anthropic(ProviderSettings {
    api_key: Some(std::env::var("ANTHROPIC_API_KEY").unwrap()),
    ..Default::default()
});

let model = provider.chat("claude-3-5-sonnet-20241022");
```

### Direct Instantiation

```rust
use qai_sdk::AnthropicModel;
let model = AnthropicModel::new(api_key);
```

### Custom API Version

```rust
let provider = create_anthropic(ProviderSettings {
    api_key: Some(api_key),
    version: Some("2023-06-01".into()), // Override API version
    ..Default::default()
});
```

---

## Chat Generation

```rust
let result = model.generate(
    Prompt {
        messages: vec![
            Message { role: Role::System, content: vec![Content::Text { text: "You are a Rust expert.".into() }] },
            Message { role: Role::User, content: vec![Content::Text { text: "Explain ownership.".into() }] },
        ],
    },
    GenerateOptions {
        model_id: "claude-3-5-sonnet-20241022".into(),
        max_tokens: Some(1024),
        temperature: Some(0.5),
        ..Default::default()
    },
).await?;

println!("{}", result.text);
```

> **Note**: System prompts are automatically extracted from `Role::System` messages and placed in the top-level `system` field to comply with Anthropic API requirements.

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

The SDK translates Anthropic's proprietary SSE events seamlessly:

| Anthropic Event | SDK StreamPart |
|---|---|
| `message_start` | *(internal)* |
| `content_block_start` | *(internal)* |
| `content_block_delta` → `text_delta` | `StreamPart::TextDelta` |
| `content_block_delta` → `input_json_delta` | `StreamPart::ToolCallDelta` |
| `content_block_delta` → `thinking_delta` | `StreamPart::ReasoningDelta` |
| `content_block_delta` → `signature_delta` | *(internal, for thinking continuity)* |
| `message_delta` | `StreamPart::Finish` |
| *(usage fields)* | `StreamPart::Usage` |

---

## Tool Calling

```rust
let calculator = ToolDefinition {
    name: "calculate".into(),
    description: "Perform arithmetic".into(),
    parameters: serde_json::json!({
        "type": "object",
        "properties": {
            "expression": { "type": "string" }
        },
        "required": ["expression"]
    }),
};

let result = model.generate(
    prompt,
    GenerateOptions {
        model_id: "claude-3-5-sonnet-20241022".into(),
        tools: Some(vec![calculator]),
        ..Default::default()
    },
).await?;

// Send tool result back
if !result.tool_calls.is_empty() {
    let tc = &result.tool_calls[0];
    let tool_result = execute_tool(&tc.name, &tc.arguments);
    // Append ToolResult to conversation and call generate again
}
```

---

## Vision (Multimodal)

```rust
let prompt = Prompt {
    messages: vec![Message {
        role: Role::User,
        content: vec![
            Content::Text { text: "Describe this diagram.".into() },
            Content::Image { source: ImageSource::Base64 {
                media_type: "image/jpeg".into(),
                data: base64_image,
            }},
        ],
    }],
};
// Claude 3 family natively supports image analysis
```

---

## Extended Thinking / Adaptive Thinking

Claude models support extended thinking — a mode where the model reasons through complex problems before answering, with summarized thought content exposed in the response.

### Thinking Modes

| Mode | `reasoning_effort` | Models | Description |
|---|---|---|---|
| **Adaptive** (recommended) | `"adaptive"`, `"high"`, `"medium"`, `"low"` | Claude 4.6+, Opus 4.7 | Claude decides when/how much to think |
| **Manual** (legacy) | Numeric (e.g. `"10000"`) | Claude 3.7 Sonnet, Claude 4.5 | Fixed token budget for thinking |
| **Disabled** | `"off"`, `"disabled"` | All | No thinking |

### Enabling Adaptive Thinking

```rust
let options = GenerateOptions {
    model_id: "claude-sonnet-4-6".into(),
    max_tokens: Some(16000),
    // "parsed" → display: "summarized" (shows thought summaries)
    // "omitted" → display: "omitted" (faster, no thinking text)
    reasoning_format: Some("parsed".to_string()),
    // "adaptive", "high", "medium", "low" → adaptive thinking
    // Numeric → manual budget_tokens
    // "off" → disabled
    reasoning_effort: Some("adaptive".to_string()),
    ..Default::default()
};

let result = model.generate(prompt, options).await?;

if let Some(reasoning) = &result.reasoning {
    println!("Thinking: {}", reasoning);
}
println!("Answer: {}", result.text);
```

### Streaming with Thinking Deltas

In streaming mode, thinking content arrives as `StreamPart::ReasoningDelta` events:

```rust
while let Some(part) = stream.next().await {
    match part {
        StreamPart::ReasoningDelta { delta } => print!("🧠 {}", delta),
        StreamPart::TextDelta { delta } => print!("{}", delta),
        _ => {}
    }
}
```

### Important Notes

- **Temperature**: When thinking is enabled, `temperature` and `top_p` are automatically set to `None` (Anthropic requires this).
- **Display modes**: `"summarized"` (default) returns thought summaries; `"omitted"` skips thinking text for lower latency.
- **Interleaved thinking**: With adaptive mode on Claude 4.6+, thinking happens between tool calls automatically.
- **Thought signatures**: Claude returns cryptographic signatures with thinking blocks for multi-turn continuity.

### Example

- [`anthropic_thinking.rs`](../examples/anthropic_thinking.rs) — Adaptive thinking, streaming, and manual budget

```mermaid
flowchart LR
    subgraph "QAI SDK Standard"
        A[Prompt with Role::System]
        B[Content::ToolCall]
        C[Content::ToolResult]
    end
    
    subgraph "Anthropic API"
        D["top-level 'system' field"]
        E["tool_use content block"]
        F["tool_result content block"]
    end
    
    A -->|auto-extracted| D
    B -->|mapped| E
    C -->|mapped| F
```
