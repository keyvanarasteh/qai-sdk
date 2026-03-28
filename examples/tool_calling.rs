//! # Tool Calling Example
//!
//! Demonstrates function tool calling with definition, invocation,
//! and result handling across OpenAI and Anthropic.

use qai_sdk::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // ===================================================================
    // Define tools
    // ===================================================================
    let weather_tool = qai_core::types::ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get the current weather for a given location.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name, e.g. 'San Francisco, CA'"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "Temperature unit"
                }
            },
            "required": ["location"]
        }),
    };

    let calculator_tool = qai_core::types::ToolDefinition {
        name: "calculate".to_string(),
        description: "Perform a mathematical calculation.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Mathematical expression to evaluate, e.g. '2 + 2'"
                }
            },
            "required": ["expression"]
        }),
    };

    let tools = vec![weather_tool, calculator_tool];

    // ===================================================================
    // OpenAI Tool Calling
    // ===================================================================
    println!("=== OpenAI Tool Calling ===");
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    let model = qai_sdk::openai::OpenAIModel::new(api_key);

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "What's the weather in Tokyo and what is 42 * 17?".to_string(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "gpt-4o-mini".to_string(),
        max_tokens: Some(500),
        temperature: Some(0.0),
        top_p: None,
        stop_sequences: None,
        tools: Some(tools.clone()),
    };

    let result = model.generate(prompt.clone(), options.clone()).await?;
    println!("Finish reason: {}", result.finish_reason);
    println!("Response text: {}", result.text);
    println!("(Tool calls would appear as [tool_call:id:name:args] in text)\n");

    // ===================================================================
    // Anthropic Tool Calling
    // ===================================================================
    println!("=== Anthropic Tool Calling ===");
    let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    let model = qai_sdk::anthropic::AnthropicModel::new(api_key);

    let options = GenerateOptions {
        model_id: "claude-3-haiku-20240307".to_string(),
        max_tokens: Some(500),
        temperature: Some(0.0),
        top_p: None,
        stop_sequences: None,
        tools: Some(tools.clone()),
    };

    let result = model.generate(prompt.clone(), options).await?;
    println!("Finish reason: {}", result.finish_reason);
    println!("Response text: {}\n", result.text);

    // ===================================================================
    // Multi-turn with tool results
    // ===================================================================
    println!("=== Multi-turn with Tool Results (OpenAI) ===");
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    let model = qai_sdk::openai::OpenAIModel::new(api_key);

    // Step 1: User asks, model calls tool
    // Step 2: Provide tool result, model synthesizes
    let prompt_with_result = Prompt {
        messages: vec![
            Message {
                role: Role::User,
                content: vec![Content::Text {
                    text: "What's the weather in London?".to_string(),
                }],
            },
            Message {
                role: Role::Assistant,
                content: vec![Content::ToolCall {
                    id: "call_123".to_string(),
                    name: "get_weather".to_string(),
                    arguments: json!({"location": "London", "unit": "celsius"}),
                }],
            },
            Message {
                role: Role::Tool,
                content: vec![Content::ToolResult {
                    id: "call_123".to_string(),
                    result: json!({"temperature": 15, "condition": "Partly cloudy", "unit": "celsius"}),
                }],
            },
        ],
    };

    let options = GenerateOptions {
        model_id: "gpt-4o-mini".to_string(),
        max_tokens: Some(200),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: Some(tools),
    };

    let result = model.generate(prompt_with_result, options).await?;
    println!("Final response: {}", result.text);

    Ok(())
}
