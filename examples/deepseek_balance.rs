//! # DeepSeek Balance Check Example
//!
//! Demonstrates how to fetch the user's available balance and top-up info
//! using the DeepSeek provider's custom API endpoint.

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

    println!("Fetching DeepSeek account balance...\n");

    let balance_response = provider.get_balance().await?;

    println!("Account Available: {}", balance_response.is_available);
    
    for info in balance_response.balance_infos {
        println!("---");
        println!("Currency: {}", info.currency);
        println!("Total Balance: {}", info.total_balance);
        println!("Granted Balance: {}", info.granted_balance);
        println!("Topped-up Balance: {}", info.topped_up_balance);
    }

    Ok(())
}
