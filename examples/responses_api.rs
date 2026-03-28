//! # Responses API Example
//!
//! Demonstrates the OpenAI Responses API — the newer alternative to Chat Completions
//! that supports reasoning models, multi-turn via `previous_response_id`, and
//! server-executed tools (web_search, code_interpreter, file_search).

use qai_sdk::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // ===================================================================
    // 1. OpenAI Responses API — Basic Generation
    // ===================================================================
    println!("=== OpenAI Responses API (Basic) ===");
    let provider = qai_sdk::openai::create_openai(ProviderSettings {
        api_key: Some(std::env::var("OPENAI_API_KEY").unwrap_or_default()),
        ..Default::default()
    });
    let model = provider.responses("gpt-4o");

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "What are the top 3 features of the Rust programming language?".to_string(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "gpt-4o".to_string(),
        max_tokens: Some(300),
        temperature: Some(0.7),
        top_p: None,
        stop_sequences: None,
        tools: None,
    };

    let result = model.generate(prompt, options).await?;
    println!("Response: {}", result.text);
    println!("Tokens: {} in, {} out", result.usage.prompt_tokens, result.usage.completion_tokens);
    println!("Finish: {}\n", result.finish_reason);

    // ===================================================================
    // 2. Responses API with System Instructions
    // ===================================================================
    println!("=== Responses API with System Instructions ===");
    let prompt = Prompt {
        messages: vec![
            Message {
                role: Role::System,
                content: vec![Content::Text {
                    text: "You are a Rust expert. Always include code examples.".to_string(),
                }],
            },
            Message {
                role: Role::User,
                content: vec![Content::Text {
                    text: "Show me a simple async function.".to_string(),
                }],
            },
        ],
    };

    let options = GenerateOptions {
        model_id: "gpt-4o-mini".to_string(),
        max_tokens: Some(500),
        temperature: Some(0.3),
        top_p: None,
        stop_sequences: None,
        tools: None,
    };

    let result = model.generate(prompt, options).await?;
    println!("Response:\n{}\n", result.text);

    // ===================================================================
    // 3. Responses API — Streaming
    // ===================================================================
    println!("=== Responses API Streaming ===");
    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "Count from 1 to 10 slowly.".to_string(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "gpt-4o-mini".to_string(),
        max_tokens: Some(200),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None,
    };

    let mut stream = model.generate_stream(prompt, options).await?;
    while let Some(part) = stream.next().await {
        match part {
            StreamPart::TextDelta { delta } => print!("{}", delta),
            StreamPart::Usage { usage } => {
                println!("\n📊 Tokens: {} in, {} out", usage.prompt_tokens, usage.completion_tokens);
            }
            StreamPart::Finish { finish_reason } => {
                println!("✅ Finished: {}", finish_reason);
            }
            StreamPart::Error { message } => {
                eprintln!("❌ Error: {}", message);
            }
            _ => {}
        }
    }

    // ===================================================================
    // 4. Responses API with Tool Calling
    // ===================================================================
    println!("\n\n=== Responses API with Tools ===");
    use serde_json::json;

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "What's 2^10?".to_string(),
            }],
        }],
    };

    let tools = vec![qai_core::types::ToolDefinition {
        name: "calculate".to_string(),
        description: "Evaluate a math expression.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "expression": { "type": "string" }
            },
            "required": ["expression"]
        }),
    }];

    let options = GenerateOptions {
        model_id: "gpt-4o-mini".to_string(),
        max_tokens: Some(200),
        temperature: Some(0.0),
        top_p: None,
        stop_sequences: None,
        tools: Some(tools),
    };

    let result = model.generate(prompt, options).await?;
    println!("Response: {}", result.text);
    println!("Finish reason: {}", result.finish_reason);

    // ===================================================================
    // 5. xAI Responses API
    // ===================================================================
    println!("\n=== xAI Responses API ===");
    let xai_provider = qai_sdk::xai::create_xai(ProviderSettings {
        api_key: Some(std::env::var("XAI_API_KEY").unwrap_or_default()),
        ..Default::default()
    });
    let xai_model = xai_provider.responses("grok-3");

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "What's the meaning of life? Answer in one sentence.".to_string(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "grok-3".to_string(),
        max_tokens: Some(100),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None,
    };

    let result = xai_model.generate(prompt, options).await?;
    println!("Grok says: {}", result.text);

    Ok(())
}
