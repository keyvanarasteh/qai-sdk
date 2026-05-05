<p align="center">
  <img src="../assets/deepseek_cover.png" alt="DeepSeek Module Banner" width="100%"/>
</p>

# DeepSeek Provider (`qai_sdk::deepseek`)

Integration with DeepSeek's fast, open-weights LLMs via their dedicated API. Built on the OpenAI-compatible protocol with DeepSeek-specific optimizations.

---

## Implemented Traits

| Trait | Models |
|---|---|
| `LanguageModel` | deepseek-chat, deepseek-coder, deepseek-reasoner |

---

## Initialization

```rust
use qai_sdk::prelude::*;

let provider = create_deepseek(ProviderSettings {
    api_key: Some(std::env::var("DEEPSEEK_API_KEY").unwrap()),
    ..Default::default()
});

let model = provider.chat("deepseek-chat");
```

### Direct Instantiation

```rust
use qai_sdk::DeepSeekModel;
let model = DeepSeekModel::new(api_key);
```

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
        model_id: "deepseek-coder".into(),
        max_tokens: Some(2048),
        temperature: Some(0.3),
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
        model_id: "deepseek-chat".into(),
        tools: Some(vec![my_tool]),
        ..Default::default()
    },
).await?;

for tc in &result.tool_calls {
    println!("DeepSeek tool call: {} -> {}", tc.name, tc.arguments);
}
```

---

## DeepSeek-R1 Reasoning Mode

For reasoning models like `deepseek-reasoner`, the SDK provides native, zero-middleware support for separating "Thinking" from the final answer.

```rust
// Standard Generation
let result = model.generate(prompt, options).await?;
if let Some(reasoning) = result.reasoning {
    println!("Thinking Process: {}", reasoning);
}
println!("Final Answer: {}", result.text);

// Streaming Generation
let mut stream = model.generate_stream(prompt, options).await?;
while let Some(part) = stream.next().await {
    match part {
        StreamPart::ReasoningDelta { delta } => print!("{}", delta), // The thought process
        StreamPart::TextDelta { delta } => print!("{}", delta),      // The actual answer
        _ => {}
    }
}
```

---

## Context Caching (KV Cache)

DeepSeek's Context Caching is natively tracked via the `Usage` struct returned in both `generate` and `generate_stream`:

```rust
let result = model.generate(prompt, options).await?;

if let Some(usage) = result.usage {
    println!("Cache Hit Tokens: {}", usage.cache_hit_tokens.unwrap_or(0));
    println!("Cache Miss Tokens: {}", usage.cache_miss_tokens.unwrap_or(0));
}
```

---

## Account Management

The `DeepSeekProvider` includes native API wrappers for DeepSeek-specific endpoints:

```rust
// Fetch your available account balance and top-up info
let balance = provider.get_balance().await?;
println!("Available: {}", balance.is_available);

// List all models available to your account
let models = provider.list_models().await?;
println!("Total Models: {}", models.data.len());
```

---

## Configuration

| Parameter | Value |
|---|---|
| Base URL | `https://api.deepseek.com/v1` |
| Auth Header | `Authorization: Bearer <API_KEY>` |
| Protocol | OpenAI-compatible `chat/completions` |
| Streaming | Standard SSE format |
