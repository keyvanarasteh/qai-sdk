---
description: How to generate a basic chat response using any provider
---

# Chat Generation

## Steps

1. Import the prelude:
```rust
use qai_sdk::prelude::*;
```

2. Create a provider with the appropriate factory function:
```rust
let provider = create_openai(ProviderSettings {
    api_key: Some(std::env::var("OPENAI_API_KEY").unwrap()),
    ..Default::default()
});
```

Available factories: `create_openai`, `create_anthropic`, `create_google`, `create_deepseek`, `create_xai`, `create_openai_compatible`.

3. Get a chat model:
```rust
let model = provider.chat("gpt-4o");
```

4. Generate a response:
```rust
let result = model.generate(
    Prompt {
        messages: vec![
            Message { role: Role::System, content: vec![Content::Text { text: "You are helpful.".into() }] },
            Message { role: Role::User, content: vec![Content::Text { text: "Hello!".into() }] },
        ],
    },
    GenerateOptions {
        model_id: "gpt-4o".into(),
        max_tokens: Some(500),
        temperature: Some(0.7),
        top_p: None,
        stop_sequences: None,
        tools: None,
    },
).await?;
```

5. Access the result:
```rust
println!("{}", result.text);
println!("Tokens: {} + {}", result.usage.prompt_tokens, result.usage.completion_tokens);
```

## Environment Variables

| Provider | Variable |
|---|---|
| OpenAI | `OPENAI_API_KEY` |
| Anthropic | `ANTHROPIC_API_KEY` |
| Google | `GOOGLE_API_KEY` |
| DeepSeek | `DEEPSEEK_API_KEY` |
| xAI | `XAI_API_KEY` |

## Error Handling

```rust
use qai_sdk::core::error::ProviderError;

match model.generate(prompt, options).await {
    Ok(result) => println!("{}", result.text),
    Err(ProviderError::RateLimit(msg)) => eprintln!("Rate limited: {msg}"),
    Err(ProviderError::Unauthorized(msg)) => eprintln!("Auth failed: {msg}"),
    Err(ProviderError::ContextLengthExceeded(msg)) => eprintln!("Too many tokens: {msg}"),
    Err(e) => eprintln!("Error: {e}"),
}
```
