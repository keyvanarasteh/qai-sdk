//! # Gemini Search Grounding Example
//!
//! Demonstrates how to use Google Gemini's native Search Grounding feature
//! to fetch live information from the web.
//!
//! ## Usage
//!
//! ```bash
//! export GOOGLE_API_KEY=AIzaSy...
//! cargo run --example gemini_search_grounding --features google
//! ```

use qai_sdk::core::types::{Content, GenerateOptions, Message, Prompt, Role, ToolDefinition};
use qai_sdk::core::LanguageModel;
use qai_sdk::google::create_google;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY must be set");

    let provider = create_google(qai_sdk::core::types::ProviderSettings {
        api_key: Some(api_key),
        ..Default::default()
    });

    let model = provider.chat("gemini-2.5-flash");

    // The query about something recent that requires a web search
    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "What is the exact current stock price of Google (GOOGL)?".to_string(),
            }],
        }],
    };

    println!("Asking Gemini about live stock prices using Google Search Grounding...\n");

    let result = model
        .generate(
            prompt,
            GenerateOptions {
                model_id: "gemini-2.5-flash".to_string(),
                // To enable Search Grounding, we pass a ToolDefinition named exactly "google_search_retrieval"
                tools: Some(vec![ToolDefinition {
                    name: "google_search_retrieval".to_string(),
                    description: "Native Google Search Grounding".to_string(),
                    parameters: serde_json::json!({}),
                }]),
                ..Default::default()
            },
        )
        .await?;

    println!("Response: {}\n", result.text);

    // If grounding was successful, it will be surfaced in `executed_tools`
    if let Some(tool) = result.executed_tools.first() {
        println!("Executed Tool: {}", tool.name);
        println!("Type: {}", tool.tool_type);
        if let Some(metadata) = &tool.output {
            println!("\nGrounding Metadata (Search Results):");
            println!("{}", serde_json::to_string_pretty(metadata)?);
        }
    } else {
        println!("No grounding metadata was returned (model may not have needed to search).");
    }

    Ok(())
}
