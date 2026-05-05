use qai_sdk::prelude::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize the xAI provider
    let api_key = std::env::var("XAI_API_KEY").expect("XAI_API_KEY not set");
    let provider = create_xai(ProviderSettings {
        api_key: Some(api_key),
        ..Default::default()
    });

    let model = provider.chat("grok-2");

    // 2. Prepare tools (Web Search)
    let server_tools = vec![ServerTool {
        tool_type: "web_search".to_string(),
        config: serde_json::json!({}),
    }];

    // 3. Generate a response with web search and citations
    let prompt = Prompt::from_user("What are the latest news about SpaceX Starship as of today?");
    
    println!("Asking Grok with Web Search...");
    
    let result = model.generate(
        prompt,
        GenerateOptions {
            model_id: "grok-2".to_string(),
            server_tools: Some(server_tools),
            include_citations: Some(true),
            ..Default::default()
        },
    ).await?;

    println!("\nResponse:\n{}", result.text);

    if !result.citations.is_empty() {
        println!("\nCitations:");
        for citation in &result.citations {
            println!("[{}] {}: {}", citation.index, citation.source, citation.uri.as_deref().unwrap_or("No URL"));
        }
    }

    Ok(())
}
