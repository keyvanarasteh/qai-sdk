//! # DeepSeek Reasoner Example
//!
//! Demonstrates the native Thinking Mode capabilities of `deepseek-reasoner`.
//! The `qai-sdk` provides a dedicated `ReasoningDelta` stream part so you can cleanly
//! separate the model's intermediate thoughts from its final text response.

use futures_util::StreamExt;
use qai_sdk::{
    core::types::{Content, GenerateOptions, Message, Prompt, Role, StreamPart},
    DeepSeekModel, LanguageModel, Result,
};
use std::io::{self, Write};

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

    let model = DeepSeekModel::new(api_key);

    println!("============================================================");
    println!(" DeepSeek Reasoner (Thinking Mode) Example");
    println!("============================================================\n");

    let prompt_text = "How many 'r's are in the word strawberry? Think through it carefully.";
    println!("User: {}\n", prompt_text);

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: prompt_text.to_string(),
            }],
        }],
    };

    let mut stream = model
        .generate_stream(
            prompt,
            GenerateOptions {
                model_id: "deepseek-reasoner".to_string(),
                ..Default::default()
            },
        )
        .await?;

    let mut is_thinking = false;
    let mut finished_thinking = false;

    while let Some(part) = stream.next().await {
        match part {
            StreamPart::ReasoningDelta { delta } => {
                if !is_thinking {
                    println!("\n\x1b[90m[Thinking...]\x1b[0m");
                    is_thinking = true;
                }
                // Print reasoning in gray
                print!("\x1b[90m{}\x1b[0m", delta);
                io::stdout().flush().unwrap();
            }
            StreamPart::TextDelta { delta } => {
                if is_thinking && !finished_thinking {
                    println!("\n\n\x1b[32m[Final Answer]\x1b[0m");
                    finished_thinking = true;
                }
                print!("{}", delta);
                io::stdout().flush().unwrap();
            }
            StreamPart::Usage { usage } => {
                println!("\n\n--- Usage ---");
                println!("Prompt tokens: {}", usage.prompt_tokens);
                println!("Completion tokens: {}", usage.completion_tokens);
                if let Some(hit) = usage.cache_hit_tokens {
                    println!("Cache hit tokens: {}", hit);
                }
            }
            StreamPart::Finish { finish_reason } => {
                println!("\n--- Finished ({}) ---", finish_reason);
            }
            StreamPart::ToolCallDelta { .. } => {}
            StreamPart::ExecutedTool { .. } => {}
            StreamPart::Error { message } => {
                println!("\nError: {}", message);
            }
        }
    }

    Ok(())
}
