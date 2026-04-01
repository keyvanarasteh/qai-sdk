//! # Streaming Chat Example
//!
//! Demonstrates real-time streaming responses from multiple providers
//! using `generate_stream()` and handling all `StreamPart` variants.

use futures::StreamExt;
use qai_sdk::*;

use qai_sdk::LanguageModel;
#[tokio::main]
async fn main() -> qai_sdk::Result<()> {
    dotenvy::dotenv().ok();

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "Write a haiku about Rust programming.".to_string(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "gpt-4o-mini".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.9),
        top_p: None,
        stop_sequences: None,
        tools: None,
    };

    // ===================================================================
    // OpenAI Streaming
    // ===================================================================
    println!("=== OpenAI Streaming ===");
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    let model = qai_sdk::openai::OpenAIModel::new(api_key);

    let mut stream = model
        .generate_stream(prompt.clone(), options.clone())
        .await?;
    while let Some(part) = stream.next().await {
        match part {
            StreamPart::TextDelta { delta } => {
                print!("{}", delta); // Print each token as it arrives
            }
            StreamPart::ToolCallDelta {
                index,
                id,
                name,
                arguments_delta,
            } => {
                println!(
                    "\n[Tool Call #{} id={:?} name={:?}]: {:?}",
                    index, id, name, arguments_delta
                );
            }
            StreamPart::Usage { usage } => {
                println!(
                    "\n📊 Usage: {} prompt + {} completion tokens",
                    usage.prompt_tokens, usage.completion_tokens
                );
            }
            StreamPart::Finish { finish_reason } => {
                println!("\n✅ Finished: {}", finish_reason);
            }
            StreamPart::Error { message } => {
                eprintln!("\n❌ Error: {}", message);
            }
        }
    }

    // ===================================================================
    // Anthropic Streaming
    // ===================================================================
    println!("\n\n=== Anthropic Streaming ===");
    let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    let model = qai_sdk::anthropic::AnthropicModel::new(api_key);

    let mut opts = options.clone();
    opts.model_id = "claude-3-haiku-20240307".to_string();

    let mut stream = model.generate_stream(prompt.clone(), opts).await?;
    while let Some(part) = stream.next().await {
        match part {
            StreamPart::TextDelta { delta } => print!("{}", delta),
            StreamPart::Usage { usage } => {
                println!(
                    "\n📊 Usage: {} prompt + {} completion tokens",
                    usage.prompt_tokens, usage.completion_tokens
                );
            }
            StreamPart::Finish { finish_reason } => {
                println!("\n✅ Finished: {}", finish_reason);
            }
            _ => {}
        }
    }

    // ===================================================================
    // Google Gemini Streaming
    // ===================================================================
    println!("\n\n=== Google Gemini Streaming ===");
    let api_key = std::env::var("GOOGLE_GENERATIVE_AI_API_KEY").unwrap_or_default();
    let model = qai_sdk::google::GoogleModel::new(api_key);

    let mut opts = options.clone();
    opts.model_id = "gemini-1.5-flash".to_string();

    let mut stream = model.generate_stream(prompt.clone(), opts).await?;
    while let Some(part) = stream.next().await {
        match part {
            StreamPart::TextDelta { delta } => print!("{}", delta),
            StreamPart::Finish { finish_reason } => {
                println!("\n✅ Finished: {}", finish_reason);
            }
            _ => {}
        }
    }
    println!();

    Ok(())
}
