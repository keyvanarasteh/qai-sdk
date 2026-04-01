---
description: How to build and run a universal agent with multi-step tool calling
---

# Agent Setup

## Steps

1. Import the agent module:
```rust
use qai_sdk::core::agent::Agent;
use qai_sdk::core::types::ToolDefinition;
```

2. Define your tools:
```rust
let tools = vec![
    ToolDefinition {
        name: "get_weather".into(),
        description: "Get weather for a city".into(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": { "city": { "type": "string" } },
            "required": ["city"]
        }),
    },
];
```

3. Build the agent:
```rust
let agent = Agent::builder()
    .model(model)
    .model_id("gpt-4o")
    .tools(tools)
    .tool_handler(|name, args| async move {
        match name.as_str() {
            "get_weather" => {
                let city = args["city"].as_str().unwrap_or("unknown");
                Ok(serde_json::json!({"temp": "22°C", "city": city}))
            }
            _ => Err(anyhow::anyhow!("Unknown tool: {name}")),
        }
    })
    .max_steps(10)
    .system("You are a helpful assistant with tool access.")
    .temperature(0.7)
    .build()
    .expect("agent build");
```

4. Run the agent:
```rust
let result = agent.run("What's the weather in Istanbul?").await?;
```

5. Inspect the result:
```rust
println!("Answer: {}", result.text);
println!("Steps: {}", result.total_steps);
println!("Finish: {}", result.finish_reason);

for step in &result.steps {
    println!("Step {}: {}", step.step, step.text);
    for tc in &step.tool_calls {
        println!("  Called: {} -> {}", tc.name, tc.result.as_deref().unwrap_or("N/A"));
    }
}
```

## Builder Options

| Method | Description | Default |
|---|---|---|
| `.model(m)` | Language model instance | Required |
| `.model_id(id)` | Model ID string | `""` |
| `.tools(vec)` | Available tool definitions | `[]` |
| `.tool_handler(fn)` | Async tool execution closure | Required |
| `.max_steps(n)` | Maximum tool-call loop iterations | `10` |
| `.system(s)` | System prompt | `None` |
| `.temperature(t)` | Generation temperature | `None` |
| `.max_tokens(n)` | Max tokens per generation | `None` |

## Notes
- The agent automatically loops: generate → tool call → execute → append result → generate again
- Loop terminates when model returns text without tool calls, or `max_steps` is reached
- Works with any `LanguageModel` — OpenAI, Anthropic, Google, etc.
