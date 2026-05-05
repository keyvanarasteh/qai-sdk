//! # DeepSeek List Models Example
//!
//! Demonstrates how to fetch the list of available models from the DeepSeek provider.

use qai_sdk::{core::types::ProviderSettings, deepseek::create_deepseek, Result};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| {
        println!("Warning: DEEPSEEK_API_KEY not set.");
        String::new()
    });

    if api_key.is_empty() {
        println!("Please set DEEPSEEK_API_KEY to run this example.");
        return Ok(());
    }

    let provider = create_deepseek(ProviderSettings {
        api_key: Some(api_key),
        ..Default::default()
    });

    println!("Fetching available DeepSeek models...\n");

    let models_response = provider.list_models().await?;

    println!("Response Object: {}", models_response.object);
    println!("Total Models: {}", models_response.data.len());
    
    for model in models_response.data {
        println!("---");
        println!("ID:       {}", model.id);
        println!("Object:   {}", model.object);
        println!("Owned By: {}", model.owned_by);
    }

    Ok(())
}
