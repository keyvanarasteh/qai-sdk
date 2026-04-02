//! # Basic Chat Example
//!
//! Demonstrates non-streaming chat generation across all 6 providers
//! using the prelude for concise imports.

use qai_sdk::*;

use qai_sdk::LanguageModel;
#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let prompt = Prompt {
        messages: vec![
            Message {
                role: Role::System,
                content: vec![Content::Text {
                    text: "You are a concise assistant. Answer in one sentence.".to_string(),
                }],
            },
            Message {
                role: Role::User,
                content: vec![Content::Text {
                    text: "What is Rust's ownership model?".to_string(),
                }],
            },
        ],
    };

    let options = GenerateOptions {
        model_id: "gpt-4o-mini".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        top_p: None,
        stop_sequences: None,
        tools: None,
        response_format: None,
    };

    // --- OpenAI ---
    println!("=== OpenAI ===");
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    let model = OpenAIModel::new(api_key);
    let result = model.generate(prompt.clone(), options.clone()).await?;
    println!("Response: {}", result.text);
    println!(
        "Tokens: {} in, {} out",
        result.usage.prompt_tokens, result.usage.completion_tokens
    );
    println!("Finish: {}\n", result.finish_reason);

    // --- Anthropic ---
    println!("=== Anthropic ===");
    let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    let model = AnthropicModel::new(api_key);
    let mut opts = options.clone();
    opts.model_id = "claude-3-haiku-20240307".to_string();
    let result = model.generate(prompt.clone(), opts).await?;
    println!("Response: {}", result.text);
    println!(
        "Tokens: {} in, {} out\n",
        result.usage.prompt_tokens, result.usage.completion_tokens
    );

    // --- Google ---
    println!("=== Google Gemini ===");
    let api_key = std::env::var("GOOGLE_GENERATIVE_AI_API_KEY").unwrap_or_default();
    let model = GoogleModel::new(api_key);
    let mut opts = options.clone();
    opts.model_id = "gemini-1.5-flash".to_string();
    let result = model.generate(prompt.clone(), opts).await?;
    println!("Response: {}", result.text);
    println!(
        "Tokens: {} in, {} out\n",
        result.usage.prompt_tokens, result.usage.completion_tokens
    );

    // --- DeepSeek ---
    println!("=== DeepSeek ===");
    let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
    let model = DeepSeekModel::new(api_key);
    let mut opts = options.clone();
    opts.model_id = "deepseek-chat".to_string();
    let result = model.generate(prompt.clone(), opts).await?;
    println!("Response: {}", result.text);
    println!(
        "Tokens: {} in, {} out\n",
        result.usage.prompt_tokens, result.usage.completion_tokens
    );

    // --- xAI ---
    println!("=== xAI (Grok) ===");
    let api_key = std::env::var("XAI_API_KEY").unwrap_or_default();
    let model = XAIModel::new(api_key);
    let mut opts = options.clone();
    opts.model_id = "grok-2".to_string();
    let result = model.generate(prompt.clone(), opts).await?;
    println!("Response: {}", result.text);
    println!(
        "Tokens: {} in, {} out\n",
        result.usage.prompt_tokens, result.usage.completion_tokens
    );

    Ok(())
}
