---
name: Provider Implementation
description: How to implement a new AI provider for qai-sdk
---

# Provider Implementation Skill

This skill covers how to add a new AI provider to the `qai-sdk` crate.

## Directory Structure

Every provider follows the same layout under `src/<provider_name>/`:

```
src/my_provider/
├── mod.rs          # Module root: re-exports, factory function, ProviderSettings
├── model.rs        # LanguageModel trait implementation
├── types.rs        # Provider-specific request/response types (serde)
└── tests.rs        # Unit tests (optional)
```

## Step 1: Create the Module

### `src/my_provider/mod.rs`
```rust
mod model;
pub(crate) mod types;

pub use model::MyProviderModel;

use crate::core::types::ProviderSettings;

/// Factory function — the public entry point
pub fn create_my_provider(settings: ProviderSettings) -> MyProviderModel {
    MyProviderModel::new(
        settings.api_key.expect("MY_PROVIDER_API_KEY required"),
    )
}
```

## Step 2: Implement `LanguageModel`

### `src/my_provider/model.rs`
```rust
use async_trait::async_trait;
use futures::stream::BoxStream;
use crate::core::types::*;
use crate::core::error::ProviderError;

pub struct MyProviderModel {
    api_key: String,
    client: reqwest::Client,
}

impl MyProviderModel {
    pub fn new(api_key: String) -> Self {
        Self { api_key, client: reqwest::Client::new() }
    }
}

#[async_trait]
impl LanguageModel for MyProviderModel {
    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> Result<GenerateResult, ProviderError> {
        // 1. Convert Prompt → provider-specific request body
        // 2. POST to the provider's API
        // 3. Parse response → GenerateResult
        todo!()
    }

    async fn generate_stream(&self, prompt: Prompt, options: GenerateOptions) -> Result<BoxStream<'static, StreamPart>, ProviderError> {
        // 1. Convert Prompt → provider request
        // 2. POST with streaming enabled
        // 3. Parse SSE events → StreamPart variants
        todo!()
    }
}
```

## Step 3: Define Provider Types

### `src/my_provider/types.rs`
```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct MyProviderRequest {
    pub model: String,
    pub messages: Vec<MyProviderMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Serialize)]
pub(crate) struct MyProviderMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub(crate) struct MyProviderResponse {
    pub choices: Vec<MyProviderChoice>,
    pub usage: Option<MyProviderUsage>,
}

#[derive(Deserialize)]
pub(crate) struct MyProviderChoice {
    pub message: MyProviderResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct MyProviderResponseMessage {
    pub content: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct MyProviderUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}
```

## Step 4: Register in `src/lib.rs`

```rust
#[cfg(feature = "my-provider")]
pub mod my_provider;

#[cfg(feature = "my-provider")]
pub use crate::my_provider::{create_my_provider, MyProviderModel};
```

## Step 5: Add Feature Flag to `Cargo.toml`

```toml
[features]
my-provider = []
```

## Conversion Patterns

### Prompt → Provider Request
```rust
fn convert_messages(prompt: &Prompt) -> Vec<MyProviderMessage> {
    prompt.messages.iter().map(|msg| {
        let role = match msg.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        };
        let text = msg.content.iter()
            .filter_map(|c| match c {
                Content::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");
        MyProviderMessage { role: role.into(), content: text }
    }).collect()
}
```

### Provider Response → GenerateResult
```rust
fn to_generate_result(resp: MyProviderResponse) -> GenerateResult {
    let choice = &resp.choices[0];
    GenerateResult {
        text: choice.message.content.clone().unwrap_or_default(),
        usage: Usage {
            prompt_tokens: resp.usage.as_ref().map(|u| u.prompt_tokens).unwrap_or(0),
            completion_tokens: resp.usage.as_ref().map(|u| u.completion_tokens).unwrap_or(0),
        },
        finish_reason: choice.finish_reason.clone().unwrap_or_else(|| "stop".into()),
        tool_calls: vec![],
    }
}
```

## Checklist

- [ ] Factory function `create_my_provider()` in `mod.rs`
- [ ] `LanguageModel` trait in `model.rs` with `generate` and `generate_stream`
- [ ] Request/response types in `types.rs` with proper serde attributes
- [ ] Feature gate in `Cargo.toml`
- [ ] Re-export in `src/lib.rs`
- [ ] Add doc page in `docs/my_provider.md`
- [ ] Add example in `examples/`
- [ ] Update `README.md` feature table
