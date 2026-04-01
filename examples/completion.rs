//! # Completion API Example
//!
//! Demonstrates the legacy completion API (non-chat) using the `CompletionModel` trait.
//! This is used for code infill, text completion, and similar tasks.

use qai_core::types::{CompletionOptions, ProviderSettings};
use qai_core::CompletionModel;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // ===================================================================
    // 1. OpenAI Completion (GPT-3.5-Turbo-Instruct)
    // ===================================================================
    println!("=== OpenAI Completion ===");
    let provider = qai_sdk::openai::create_openai(ProviderSettings {
        api_key: Some(std::env::var("OPENAI_API_KEY").unwrap_or_default()),
        ..Default::default()
    });
    let model = provider.completion("gpt-3.5-turbo-instruct");

    // Basic text completion
    let options = CompletionOptions {
        model_id: "gpt-3.5-turbo-instruct".to_string(),
        prompt: "The capital of France is".to_string(),
        max_tokens: Some(20),
        temperature: Some(0.0),
        top_p: None,
        stop: None,
        suffix: None,
    };

    let result = model.complete(options).await?;
    println!("Completion: \"The capital of France is{}\"", result.text);
    println!(
        "Tokens: {} in, {} out",
        result.usage.prompt_tokens, result.usage.completion_tokens
    );

    // Completion with stop sequences
    println!("\n--- With stop sequences ---");
    let options = CompletionOptions {
        model_id: "gpt-3.5-turbo-instruct".to_string(),
        prompt: "List programming languages:\n1. Python\n2. Rust\n3.".to_string(),
        max_tokens: Some(50),
        temperature: Some(0.5),
        top_p: None,
        stop: Some(vec!["\n5.".to_string()]), // Stop after 4th item
        suffix: None,
    };

    let result = model.complete(options).await?;
    println!("Completion: {}", result.text);
    println!("Finish reason: {}", result.finish_reason);

    // Completion with suffix (fill-in-the-middle / infill)
    println!("\n--- Code infill with suffix ---");
    let options = CompletionOptions {
        model_id: "gpt-3.5-turbo-instruct".to_string(),
        prompt: "fn fibonacci(n: u32) -> u32 {\n    ".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.0),
        top_p: None,
        stop: None,
        suffix: Some("\n}\n".to_string()),
    };

    let result = model.complete(options).await?;
    println!(
        "Code infill:\nfn fibonacci(n: u32) -> u32 {{\n    {}\n}}",
        result.text.trim()
    );

    // ===================================================================
    // 2. OpenAI-Compatible Completion
    // ===================================================================
    println!("\n=== OpenAI-Compatible Completion ===");
    use qai_sdk::openai_compatible::OpenAICompatibleProviderSettings;

    let provider =
        qai_sdk::openai_compatible::create_openai_compatible(OpenAICompatibleProviderSettings {
            base_url: "https://api.together.xyz/v1".to_string(),
            name: "together".to_string(),
            api_key: Some(std::env::var("TOGETHER_API_KEY").unwrap_or_default()),
            headers: None,
        });
    let model = provider.completion("meta-llama/Llama-3-70b-chat-hf");

    let options = CompletionOptions {
        model_id: "meta-llama/Llama-3-70b-chat-hf".to_string(),
        prompt: "Explain quantum computing in one sentence:".to_string(),
        max_tokens: Some(50),
        temperature: Some(0.7),
        top_p: None,
        stop: None,
        suffix: None,
    };

    let result = model.complete(options).await?;
    println!("Completion: {}", result.text);

    Ok(())
}
