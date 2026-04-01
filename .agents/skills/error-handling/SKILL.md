---
name: Error Handling
description: Comprehensive error handling patterns for qai-sdk
---

# Error Handling Skill

This skill covers robust error handling patterns across all `qai-sdk` operations.

## ProviderError Variants

```rust
use qai_sdk::core::error::ProviderError;

pub enum ProviderError {
    Configuration(String),         // Bad API key format, missing config
    RateLimit(String),             // HTTP 429
    ContextLengthExceeded(String), // Token limit exceeded
    Unauthorized(String),          // HTTP 401/403
    Network(String),               // Connection failures, timeouts
    InvalidResponse(String),       // Malformed JSON, unexpected schema
    NotSupported(String),          // Feature not available for provider
    Other(anyhow::Error),          // Catch-all
}
```

## Basic Error Handling

```rust
match model.generate(prompt, options).await {
    Ok(result) => println!("{}", result.text),
    Err(ProviderError::RateLimit(msg)) => {
        eprintln!("Rate limited: {msg}");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        // Retry...
    }
    Err(ProviderError::ContextLengthExceeded(msg)) => {
        eprintln!("Context too long: {msg}");
        // Truncate prompt and retry
    }
    Err(ProviderError::Unauthorized(msg)) => {
        eprintln!("Auth failed: {msg}");
        // Check API key
    }
    Err(ProviderError::Network(msg)) => {
        eprintln!("Network error: {msg}");
        // Retry with backoff
    }
    Err(ProviderError::NotSupported(msg)) => {
        eprintln!("Not supported: {msg}");
        // Fall back to different provider/model
    }
    Err(e) => eprintln!("Unexpected: {e}"),
}
```

## Retry Pattern with Exponential Backoff

```rust
async fn generate_with_retry(
    model: &dyn LanguageModel,
    prompt: Prompt,
    options: GenerateOptions,
    max_retries: u32,
) -> Result<GenerateResult, ProviderError> {
    let mut delay = std::time::Duration::from_millis(500);

    for attempt in 0..=max_retries {
        match model.generate(prompt.clone(), options.clone()).await {
            Ok(result) => return Ok(result),
            Err(ProviderError::RateLimit(_)) | Err(ProviderError::Network(_)) if attempt < max_retries => {
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

## Provider Fallback Pattern

```rust
async fn generate_with_fallback(
    primary: &dyn LanguageModel,
    fallback: &dyn LanguageModel,
    prompt: Prompt,
    options: GenerateOptions,
) -> Result<GenerateResult, ProviderError> {
    match primary.generate(prompt.clone(), options.clone()).await {
        Ok(result) => Ok(result),
        Err(ProviderError::RateLimit(_))
        | Err(ProviderError::Network(_))
        | Err(ProviderError::NotSupported(_)) => {
            fallback.generate(prompt, options).await
        }
        Err(e) => Err(e),
    }
}
```

## Automatic Error Conversions

The SDK auto-converts common error types:

| From | To |
|---|---|
| `reqwest::Error` | `ProviderError::Network` |
| `serde_json::Error` | `ProviderError::InvalidResponse` |
| `std::io::Error` | `ProviderError::Other` |

## Best Practices

- Never `unwrap()` in library code — use `?` operator
- Match specific error variants for actionable recovery
- Use `ProviderError::Other(anyhow::anyhow!("msg"))` for custom errors
- Log errors before retrying for observability
- Set reasonable retry limits (3-5 max) with exponential backoff
