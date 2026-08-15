use qai_sdk::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let api_key = env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set");

    let provider = create_anthropic(ProviderSettings {
        api_key: Some(api_key),
        ..Default::default()
    });

    let model = provider.chat("claude-3-7-sonnet-20250219");

    println!("🧠 Testing Anthropic Extended Thinking...");

    let prompt =
        Prompt::from_user("Explain the Riemann Hypothesis and how it relates to prime numbers.");

    let mut options = GenerateOptions::default();
    options.model_id = "claude-3-7-sonnet-20250219".to_string();
    options.reasoning_effort = Some("high".to_string());
    options.reasoning_format = Some("raw".to_string());
    options.max_tokens = Some(4096);

    let result = model.generate(prompt, options).await?;

    println!("--- Reasoning ---");
    // In actual response, reasoning might be part of content parts
    // Our generate() merges text, but let's see the result.
    println!("{}", result.text);

    Ok(())
}
