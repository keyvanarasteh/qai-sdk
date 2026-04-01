<p align="center">
  <img src="../assets/xai_cover.png" alt="xAI Module Banner" width="100%"/>
</p>

# xAI Provider (`qai_sdk::xai`)

Integration with xAI's Grok models via their OpenAI-compatible API endpoint.

---

## Implemented Traits

| Trait | Models |
|---|---|
| `LanguageModel` | grok-beta, grok-2, grok-2-mini, grok-vision-beta |

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
