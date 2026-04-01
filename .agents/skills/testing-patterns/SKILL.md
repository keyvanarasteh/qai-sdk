---
name: Testing Patterns
description: How to write and run tests for qai-sdk modules
---

# Testing Patterns Skill

This skill covers testing conventions, mock patterns, and verification strategies for `qai-sdk`.

## Test File Location

Tests live alongside source files:
```
src/core/
├── types.rs
├── types_test.rs       # Unit tests
src/openai/
├── model.rs
├── model_test.rs       # Provider-specific tests
```

Or as integration tests:
```
examples/
├── chat_basic.rs       # Example doubles as integration test
```

## Unit Test Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_prompt_construction() {
        let prompt = Prompt {
            messages: vec![
                Message {
                    role: Role::User,
                    content: vec![Content::Text { text: "hello".into() }],
                },
            ],
        };
        assert_eq!(prompt.messages.len(), 1);
        assert!(matches!(prompt.messages[0].role, Role::User));
    }

    #[test]
    fn test_tool_definition_serialization() {
        let tool = ToolDefinition {
            name: "test".into(),
            description: "A test tool".into(),
            parameters: json!({"type": "object"}),
        };
        let serialized = serde_json::to_string(&tool).unwrap();
        assert!(serialized.contains("test"));
    }
}
```

## Async Test Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_with_mock() {
        let model = MockLanguageModel::new();
        let result = model.generate(prompt, options).await.unwrap();
        assert!(!result.text.is_empty());
    }
}
```

## Mock Language Model

Use `src/test_utils/` for shared mocks:

```rust
use async_trait::async_trait;
use crate::core::types::*;

pub struct MockLanguageModel {
    pub response_text: String,
}

impl MockLanguageModel {
    pub fn new() -> Self {
        Self { response_text: "Mock response".into() }
    }

    pub fn with_response(text: &str) -> Self {
        Self { response_text: text.into() }
    }
}

#[async_trait]
impl LanguageModel for MockLanguageModel {
    async fn generate(&self, _prompt: Prompt, _options: GenerateOptions) -> Result<GenerateResult> {
        Ok(GenerateResult {
            text: self.response_text.clone(),
            usage: Usage { prompt_tokens: 10, completion_tokens: 20 },
            finish_reason: "stop".into(),
            tool_calls: vec![],
        })
    }

    async fn generate_stream(&self, _prompt: Prompt, _options: GenerateOptions) -> Result<BoxStream<'static, StreamPart>> {
        let text = self.response_text.clone();
        let stream = futures::stream::once(async move {
            StreamPart::TextDelta { delta: text }
        });
        Ok(Box::pin(stream))
    }
}
```

## Testing Structured Output

```rust
#[tokio::test]
async fn test_generate_object() {
    let model = MockLanguageModel::with_response(r#"{"name": "test", "age": 25}"#);
    let result = generate_object(
        &model,
        "Generate a profile",
        ObjectGenerateOptions {
            model_id: "mock".into(),
            schema: json!({"type": "object", "properties": {"name": {"type": "string"}}}),
            mode: OutputMode::Json,
            ..Default::default()
        },
    ).await.unwrap();
    assert_eq!(result.object["name"], "test");
}
```

## Testing Middleware

```rust
#[tokio::test]
async fn test_default_settings_middleware() {
    let model = MockLanguageModel::new();
    let wrapped = wrap_language_model(model, vec![
        Box::new(DefaultSettingsMiddleware {
            temperature: Some(0.5),
            max_tokens: Some(100),
            top_p: None,
        }),
    ]);
    // The wrapped model should inject defaults
    let result = wrapped.generate(prompt, GenerateOptions::default()).await.unwrap();
    assert!(!result.text.is_empty());
}
```

## Testing Agent

```rust
#[tokio::test]
async fn test_agent_no_tools() {
    let model = MockLanguageModel::new();
    let agent = Agent::builder()
        .model(Box::new(model))
        .model_id("mock")
        .build()
        .unwrap();
    let result = agent.run("Hello").await.unwrap();
    assert_eq!(result.total_steps, 1);
}
```

## Running Tests

```bash
# All tests
cargo test

# All tests with all features
cargo test --all-features

# Specific module
cargo test core::structured

# With output
cargo test -- --nocapture

# Single test
cargo test test_generate_object
```

## Conventions

- Always use `#[cfg(test)]` for test modules
- Use `#[tokio::test]` for async tests
- Mock external APIs — never hit real endpoints in unit tests
- Integration tests in `examples/` can use real APIs with env vars
- Test error paths: `RateLimit`, `Unauthorized`, `InvalidResponse`
- Use `assert!(matches!(result, Err(ProviderError::...)))` for error testing
