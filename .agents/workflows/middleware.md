---
description: How to create and apply composable middleware to language models
---

# Middleware

## Steps

1. Import the middleware module:
```rust
use qai_sdk::core::middleware::*;
```

2. Wrap a model with built-in middlewares:
```rust
let wrapped = wrap_language_model(
    model,
    vec![
        Box::new(DefaultSettingsMiddleware {
            temperature: Some(0.7),
            max_tokens: Some(4096),
            top_p: None,
        }),
        Box::new(ExtractReasoningMiddleware::default()),
    ],
);
```

3. Use the wrapped model exactly like the original:
```rust
let result = wrapped.generate(prompt, options).await?;
println!("{}", result.text); // Clean output, no <think> blocks
```

## Built-in Middlewares

| Middleware | Purpose |
|---|---|
| `DefaultSettingsMiddleware` | Injects default `temperature`, `max_tokens`, `top_p` when not explicitly set |
| `ExtractReasoningMiddleware` | Strips `<think>...</think>` blocks from output (for DeepSeek-R1, QwQ) |

## Creating Custom Middleware

```rust
use async_trait::async_trait;

struct LoggingMiddleware;

#[async_trait]
impl LanguageModelMiddleware for LoggingMiddleware {
    async fn transform_params(&self, options: GenerateOptions) -> Result<GenerateOptions> {
        println!("[LOG] model={} temp={:?}", options.model_id, options.temperature);
        Ok(options)
    }
    // Optional: override wrap_generate for response transformation
}
```

## Notes
- Middlewares are applied in order: first registered = first to transform params
- `wrap_language_model` returns a `Box<dyn LanguageModel>` — fully compatible with all SDK functions
- Chain multiple middlewares for composable behavior (logging → defaults → reasoning extraction)
