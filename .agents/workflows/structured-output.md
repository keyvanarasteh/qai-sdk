---
description: How to force models to return structured JSON using generate_object
---

# Structured Output

## Steps

1. Import the structured module:
```rust
use qai_sdk::core::structured::*;
```

2. Define your JSON Schema:
```rust
let schema = serde_json::json!({
    "type": "object",
    "properties": {
        "name": { "type": "string" },
        "age": { "type": "integer" },
        "skills": { "type": "array", "items": { "type": "string" } }
    },
    "required": ["name", "age"]
});
```

3. Choose an OutputMode:
- `OutputMode::Json` — Injects schema into system prompt, model returns raw JSON (best for OpenAI, Gemini)
- `OutputMode::Tool` — Wraps schema as a fake tool definition (best for Anthropic, fallback)

4. Call `generate_object`:
```rust
let result = generate_object(
    &model,
    "Generate a profile for Jane Doe, software engineer, age 28",
    ObjectGenerateOptions {
        model_id: "gpt-4o".into(),
        schema: schema,
        mode: OutputMode::Json,
        ..Default::default()
    },
).await?;
```

5. Access the parsed JSON:
```rust
println!("{}", result.object);          // serde_json::Value
println!("Raw: {}", result.raw_text);   // Original model output
```

## Notes
- The SDK auto-strips markdown code fences (```json ... ```) from model output
- `generate_object` works with any `LanguageModel` implementation
- For providers without native JSON mode, use `OutputMode::Tool` as fallback
- The result `.object` is a `serde_json::Value` — deserialize to your struct with `serde_json::from_value`
