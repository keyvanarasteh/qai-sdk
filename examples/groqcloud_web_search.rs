//! # GroqCloud Web Search Example
//!
//! Demonstrates built-in web search using Groq's compound models.
//! These models automatically execute web searches server-side with
//! inline citation support.
//!
//! ## Requirements
//! - `GROQ_API_KEY` environment variable
//!
//! ## Run
//! ```bash
//! cargo run --example groqcloud_web_search --features groqcloud
//! ```

use qai_sdk::groqcloud::tools::GroqTool;
use qai_sdk::*;

use qai_sdk::LanguageModel;
#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let provider = create_groqcloud(ProviderSettings::default());

    // ===================================================================
    // 1. Basic Web Search with Compound Model
    // ===================================================================
    println!("=== GroqCloud Web Search (Compound Model) ===\n");

    let compound_model = provider.compound("groq/compound");

    // The compound model + web_search_preview tool triggers server-side
    // web searches automatically when the model determines it's needed.
    let web_search_tool = GroqTool::builtin_web_search(Some(5));
    let tool_json = serde_json::to_value(&web_search_tool).unwrap_or_default();

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "What are the latest developments in Rust programming language in 2025?"
                    .into(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "groq/compound".to_string(),
        max_tokens: Some(2048),
        temperature: Some(0.3),
        // Pass the Groq tool as raw JSON in response_format for compound model orchestration
        response_format: None,
        tools: Some(vec![ToolDefinition {
            name: "web_search_preview".to_string(),
            description: "Built-in web search".to_string(),
            parameters: tool_json,
        }]),
        ..Default::default()
    };

    match compound_model.generate(prompt, options).await {
        Ok(result) => {
            println!("📝 Response:\n{}", result.text);
            println!(
                "\n📊 Tokens: {} in, {} out",
                result.usage.prompt_tokens, result.usage.completion_tokens
            );
            if !result.executed_tools.is_empty() {
                println!("\n🔧 Executed tools:");
                for tool in &result.executed_tools {
                    println!("  - {} ({})", tool.name, tool.tool_type);
                }
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    // ===================================================================
    // 2. Web Search with Location Context
    // ===================================================================
    println!("\n\n=== Web Search with Location ===\n");

    use qai_sdk::groqcloud::tools::GroqUserLocation;
    let localized_tool = GroqTool::builtin_web_search_with_config(
        Some(3),
        Some("high".to_string()),
        Some(GroqUserLocation::full("Istanbul", "Istanbul", "TR")),
    );
    println!(
        "Localized tool config:\n{}\n",
        serde_json::to_string_pretty(&localized_tool).unwrap_or_default()
    );

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "What's the weather like today? Any local events happening?".into(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "groq/compound-mini".to_string(),
        max_tokens: Some(1024),
        ..Default::default()
    };

    match compound_model.generate(prompt, options).await {
        Ok(result) => {
            println!("📝 Response:\n{}", result.text);
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    // ===================================================================
    // 3. Visit Website Tool
    // ===================================================================
    println!("\n\n=== Visit Website Tool ===\n");

    let visit_tool = GroqTool::builtin_visit_website();
    println!(
        "Visit tool config: {}",
        serde_json::to_string(&visit_tool).unwrap_or_default()
    );

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "Visit https://docs.rs/qai-sdk and summarize what the qai-sdk crate does."
                    .into(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "groq/compound".to_string(),
        max_tokens: Some(1024),
        ..Default::default()
    };

    match compound_model.generate(prompt, options).await {
        Ok(result) => {
            println!("📝 Response:\n{}", result.text);
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    Ok(())
}
