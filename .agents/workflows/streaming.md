---
description: How to stream responses token-by-token from any provider
---

# Streaming Responses

## Steps

1. Import required dependencies:
```rust
use qai_sdk::prelude::*;
use futures::StreamExt;
```

2. Call `generate_stream` instead of `generate`:
```rust
let model = provider.chat("gpt-4o");
let mut stream = model.generate_stream(prompt, options).await?;
```

3. Process the stream with pattern matching:
```rust
while let Some(part) = stream.next().await {
    match part {
        StreamPart::TextDelta { delta } => print!("{delta}"),
        StreamPart::ToolCallDelta { index, id, name, arguments_delta } => {
            if let Some(n) = name { print!("[tool: {n}] "); }
            if let Some(a) = arguments_delta { print!("{a}"); }
        }
        StreamPart::Usage { usage } => {
            println!("\n[{} tokens]", usage.prompt_tokens + usage.completion_tokens);
        }
        StreamPart::Finish { finish_reason } => println!("\n[Done: {finish_reason}]"),
        StreamPart::Error { message } => eprintln!("\n[Error: {message}]"),
    }
}
```

## StreamPart Variants

| Variant | Fields | When Emitted |
|---|---|---|
| `TextDelta` | `delta: String` | Each text chunk |
| `ToolCallDelta` | `index`, `id`, `name`, `arguments_delta` | Tool call streaming |
| `Usage` | `usage: Usage` | Final token counts |
| `Finish` | `finish_reason: String` | Stream complete |
| `Error` | `message: String` | Error occurred |

## Notes
- All providers normalize their SSE events to the same `StreamPart` enum
- Anthropic uses `message_start`/`content_block_delta`, Gemini streams JSON arrays — the SDK handles all translation transparently
- Ensure `futures` is in your `Cargo.toml` dependencies
