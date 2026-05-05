//! # GroqCloud MCP Integration Example
//!
//! Demonstrates remote MCP (Model Context Protocol) server integration
//! with GroqCloud. Groq acts as the MCP client, handling tool discovery
//! and execution server-side.
//!
//! ## Requirements
//! - `GROQ_API_KEY` environment variable
//! - Access to an MCP server endpoint (HTTPS)
//!
//! ## Run
//! ```bash
//! cargo run --example groqcloud_mcp --features groqcloud
//! ```

use qai_sdk::groqcloud::tools::{GroqMcpAllowedTool, GroqTool};
use qai_sdk::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let provider = create_groqcloud(ProviderSettings::default());

    // ===================================================================
    // 1. MCP Tool Configuration Examples
    // ===================================================================
    println!("=== GroqCloud Remote MCP Integration ===\n");

    // Simple MCP tool (no auth required)
    let public_mcp = GroqTool::mcp(
        "deepseek-docs",
        "https://mcp.deepseek.example.com/sse",
    );
    println!(
        "Public MCP config:\n{}\n",
        serde_json::to_string_pretty(&public_mcp).unwrap_or_default()
    );

    // MCP tool with authentication
    let mut auth_headers = HashMap::new();
    auth_headers.insert(
        "Authorization".to_string(),
        format!(
            "Bearer {}",
            std::env::var("HF_TOKEN").unwrap_or_else(|_| "hf_example_token".to_string())
        ),
    );

    let hf_mcp = GroqTool::Mcp {
        server_label: "huggingface".to_string(),
        server_url: "https://huggingface.co/mcp".to_string(),
        headers: Some(auth_headers),
        server_description: Some(
            "Search HuggingFace for models, datasets, and papers".to_string(),
        ),
        require_approval: Some("never".to_string()),
        allowed_tools: Some(vec![
            GroqMcpAllowedTool {
                name: "search_models".to_string(),
            },
            GroqMcpAllowedTool {
                name: "search_datasets".to_string(),
            },
        ]),
    };

    println!(
        "HuggingFace MCP config:\n{}\n",
        serde_json::to_string_pretty(&hf_mcp).unwrap_or_default()
    );

    // ===================================================================
    // 2. Using MCP tools via Chat Completions (compound model)
    // ===================================================================
    println!("=== MCP via Compound Model ===\n");

    let compound = provider.compound("groq/compound");

    // Note: This example shows the tool configuration serialization.
    // Actual MCP server execution requires a live MCP server endpoint.
    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "Search for the best Rust ML models on HuggingFace.".into(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "groq/compound".to_string(),
        max_tokens: Some(1024),
        temperature: Some(0.3),
        ..Default::default()
    };

    // In a real scenario, the MCP tool JSON would be passed via the raw API.
    // The SDK demonstrates the serialization format:
    println!("Would send MCP tool configuration to Groq API:");
    println!(
        "{}",
        serde_json::to_string_pretty(&hf_mcp).unwrap_or_default()
    );

    match compound.generate(prompt, options).await {
        Ok(result) => {
            println!("\n📝 Response:\n{}", result.text);
        }
        Err(e) => eprintln!("\nExpected error (no live MCP server): {e}"),
    }

    // ===================================================================
    // 3. MCP via Responses API
    // ===================================================================
    println!("\n\n=== MCP via Responses API ===\n");

    let responses_model = provider.responses("groq/compound");

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "What are the top trending AI research papers this week?".into(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "groq/compound".to_string(),
        max_tokens: Some(1024),
        ..Default::default()
    };

    match responses_model.generate(prompt, options).await {
        Ok(result) => {
            println!("📝 Response:\n{}", result.text);
            println!(
                "\n📊 Tokens: {} in, {} out",
                result.usage.prompt_tokens, result.usage.completion_tokens
            );
        }
        Err(e) => eprintln!("Expected error (requires API access): {e}"),
    }

    Ok(())
}
