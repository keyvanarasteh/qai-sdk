//! # GroqCloud Local Tool Calling Example
//!
//! Demonstrates local function calling with Groq models using the
//! universal `Agent` framework for automatic tool-call loop execution.
//!
//! ## Requirements
//! - `GROQ_API_KEY` environment variable
//!
//! ## Run
//! ```bash
//! cargo run --example groqcloud_tool_calling --features groqcloud
//! ```

use qai_sdk::core::agent::Agent;
use qai_sdk::*;
use serde_json::json;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let provider = create_groqcloud(ProviderSettings::default());

    // ===================================================================
    // 1. Simple Tool Calling (single-turn)
    // ===================================================================
    println!("=== GroqCloud Local Tool Calling (Single-Turn) ===\n");

    let model = provider.chat("llama-3.3-70b-versatile");

    let weather_tool = ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get the current weather for a city.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name, e.g. 'Istanbul, TR'"
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

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "What's the weather like in Istanbul?".into(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "llama-3.3-70b-versatile".to_string(),
        max_tokens: Some(1024),
        temperature: Some(0.0), // Low temperature recommended for tool calling
        tools: Some(vec![weather_tool.clone()]),
        tool_choice: Some(json!("auto")), // Let the model decide
        ..Default::default()
    };

    use qai_sdk::LanguageModel;
    let result = model.generate(prompt, options).await?;

    println!("Finish reason: {}", result.finish_reason);
    if !result.tool_calls.is_empty() {
        println!("🔧 Tool calls:");
        for tc in &result.tool_calls {
            println!(
                "  - {}({})",
                tc.name,
                serde_json::to_string(&tc.arguments).unwrap_or_default()
            );
        }
    } else {
        println!("📝 Response: {}", result.text);
    }

    // ===================================================================
    // 2. Agent-Based Tool Loop (multi-turn)
    // ===================================================================
    println!("\n\n=== GroqCloud Agent Tool Loop ===\n");

    let calculator_tool = ToolDefinition {
        name: "calculate".to_string(),
        description: "Evaluate a mathematical expression.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "The math expression to evaluate"
                }
            },
            "required": ["expression"]
        }),
    };

    let agent = Agent::builder()
        .model(Box::new(provider.chat("llama-3.3-70b-versatile")))
        .model_id("llama-3.3-70b-versatile")
        .system("You are a helpful assistant with access to weather and calculator tools. Use the tools when appropriate.")
        .tools(vec![weather_tool, calculator_tool])
        .tool_handler(|name, args| async move {
            match name.as_str() {
                "get_weather" => {
                    let location = args.get("location")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    // Simulated weather response
                    Ok(json!({
                        "location": location,
                        "temperature": 22,
                        "unit": "celsius",
                        "condition": "Partly cloudy",
                        "humidity": 65
                    }))
                }
                "calculate" => {
                    let expr = args.get("expression")
                        .and_then(|v| v.as_str())
                        .unwrap_or("0");
                    // Simple evaluator for demo purposes
                    Ok(json!({
                        "expression": expr,
                        "result": "714",
                        "note": "Simulated calculation"
                    }))
                }
                _ => Err(anyhow::anyhow!("Unknown tool: {name}")),
            }
        })
        .temperature(0.0) // Low temperature for reliable tool calling
        .max_tokens(1024)
        .max_steps(5)
        .build()?;

    let result = agent
        .run("What's the weather in Istanbul and what is 42 * 17?")
        .await?;

    println!("📝 Final response:\n{}", result.text);
    println!("\n📊 Steps taken: {}", result.total_steps);

    for step in &result.steps {
        println!("\n  Step {}:", step.step);
        if !step.text.is_empty() {
            println!("    Text: {}", &step.text[..step.text.len().min(100)]);
        }
        for tc in &step.tool_calls {
            println!("    🔧 {} → {:?}", tc.name, tc.result);
        }
    }

    // ===================================================================
    // 3. Forced Tool Choice
    // ===================================================================
    println!("\n\n=== Forced Tool Choice ===\n");

    let model = provider.chat("llama-3.3-70b-versatile");

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "Tell me about Istanbul".into(), // Vague request
            }],
        }],
    };

    // Force the model to use the get_weather tool specifically
    let options = GenerateOptions {
        model_id: "llama-3.3-70b-versatile".to_string(),
        max_tokens: Some(512),
        temperature: Some(0.0),
        tools: Some(vec![ToolDefinition {
            name: "get_weather".to_string(),
            description: "Get current weather for a city".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string" }
                },
                "required": ["location"]
            }),
        }]),
        // Force this specific tool
        tool_choice: Some(json!({
            "type": "function",
            "function": { "name": "get_weather" }
        })),
        ..Default::default()
    };

    let result = model.generate(prompt, options).await?;
    println!("Finish reason: {}", result.finish_reason);
    if !result.tool_calls.is_empty() {
        println!("✅ Model was forced to call get_weather:");
        for tc in &result.tool_calls {
            println!(
                "   {}({})",
                tc.name,
                serde_json::to_string(&tc.arguments).unwrap_or_default()
            );
        }
    }

    Ok(())
}
