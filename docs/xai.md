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

