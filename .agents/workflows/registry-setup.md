---
description: How to set up and use the ProviderRegistry for multi-provider applications
---

# Provider Registry

## Steps

1. Import the registry module:
```rust
use qai_sdk::core::registry::{ProviderRegistry, Provider};
```

2. Create providers:
```rust
let openai = create_openai(ProviderSettings { api_key: Some(openai_key), ..Default::default() });
let anthropic = create_anthropic(ProviderSettings { api_key: Some(anthropic_key), ..Default::default() });
let google = create_google(ProviderSettings { api_key: Some(google_key), ..Default::default() });
```

3. Register them:
```rust
let registry = ProviderRegistry::new()
    .register("openai", openai)
    .register("anthropic", anthropic)
    .register("google", google);
```

4. Resolve models by string:
```rust
let model = registry.language_model("openai:gpt-4o")?;
let embedder = registry.embedding_model("openai:text-embedding-3-small")?;
let imager = registry.image_model("openai:dall-e-3")?;
```

5. Use resolved models normally:
```rust
let result = model.generate(prompt, options).await?;
```

## Custom Separator

```rust
// Use '/' instead of ':'
let registry = ProviderRegistry::with_separator('/')
    .register("openai", provider);
let model = registry.language_model("openai/gpt-4o")?;
```

## Implementing the Provider Trait

To register custom providers, implement the `Provider` trait:
```rust
impl Provider for MyProvider {
    fn language_model(&self, model_id: &str) -> Option<Box<dyn LanguageModel>> {
        Some(Box::new(MyModel::new(model_id, &self.settings)))
    }
    // Optional: embedding_model, image_model
}
```

## Notes
- String format is always `"provider_id:model_id"` (or custom separator)
- If provider not found, returns `ProviderError::InvalidResponse`
- Registry is `Send + Sync` and can be shared across threads with `Arc`
