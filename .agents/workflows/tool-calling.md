---
description: How to use function calling / tool calling with any provider
---

# Tool Calling

## Steps

1. Define tools using `ToolDefinition` with JSON Schema parameters:
```rust
let weather_tool = ToolDefinition {
    name: "get_weather".into(),
    description: "Get weather for a location".into(),
    parameters: serde_json::json!({
        "type": "object",
        "properties": {
            "city": { "type": "string", "description": "City name" },
            "unit": { "type": "string", "enum": ["celsius", "fahrenheit"] }
        },
        "required": ["city"]
    }),
};
```

2. Pass tools to `GenerateOptions`:
```rust
let result = model.generate(prompt, GenerateOptions {
    model_id: "gpt-4o".into(),
    tools: Some(vec![weather_tool]),
    ..Default::default()
}).await?;
```

3. Check for tool calls in the result:
```rust
if !result.tool_calls.is_empty() {
    for tc in &result.tool_calls {
        println!("Tool: {} | Args: {}", tc.name, tc.arguments);
        let tool_output = execute_tool(&tc.name, &tc.arguments);
        // Append tool result back to conversation
    }
}
```

4. Continue the conversation with tool results:
```rust
prompt.messages.push(Message {
    role: Role::Assistant,
    content: result.tool_calls.iter().map(|tc| Content::ToolCall {
        id: tc.id.clone(),
        name: tc.name.clone(),
        arguments: tc.arguments.clone(),
    }).collect(),
});
prompt.messages.push(Message {
    role: Role::Tool,
    content: vec![Content::ToolResult {
        id: tool_call_id,
        result: serde_json::to_string(&tool_output)?,
    }],
});
let final_result = model.generate(prompt, options).await?;
```

## Notes
- Tool calling works across OpenAI, Anthropic, Google, DeepSeek, xAI
- For OpenAI-compatible endpoints, support depends on the backend
- Tool schemas use standard JSON Schema format
- The SDK handles provider-specific serialization (Anthropic XML, Gemini function_declarations)
