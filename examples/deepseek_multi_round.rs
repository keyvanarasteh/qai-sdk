//! # DeepSeek Multi-round Chat Example
//!
//! Demonstrates multi-round conversational interaction and the DeepSeek KV context cache.

use qai_sdk::*;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    // tracing_subscriber::fmt::init(); // Optional, commented to avoid noisy logs by default

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
    println!(" DeepSeek Multi-round Chat (with Context Caching)");
    println!(" Type 'quit' or 'exit' to stop.");
    println!("============================================================\n");

    // The context vector to hold our multi-round conversation
    let mut messages = vec![Message {
        role: Role::System,
        content: vec![Content::Text {
            text: "You are a helpful and concise assistant. Always respond in english.".to_string(),
        }],
    }];

    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            break;
        }
        if input.is_empty() {
            continue;
        }

        // Add user message to the context
        messages.push(Message {
            role: Role::User,
            content: vec![Content::Text {
                text: input.to_string(),
            }],
        });

        let prompt = Prompt {
            messages: messages.clone(),
        };

        // Generate the response statelessly
        let response = match model
            .generate(
                prompt,
                GenerateOptions {
                    model_id: "deepseek-chat".to_string(),
                    ..Default::default()
                },
            )
            .await
        {
            Ok(r) => r,
            Err(e) => {
                println!("\nError generating response: {}", e);
                // Remove the failed user message from history
                messages.pop();
                continue;
            }
        };

        println!("\nDeepSeek: {}", response.text.trim());

        // Print cache metrics
        println!("\n--- Usage Stats ---");
        println!("Prompt tokens: {}", response.usage.prompt_tokens);
        println!("Completion tokens: {}", response.usage.completion_tokens);

        if let Some(hit) = response.usage.cache_hit_tokens {
            println!("KV Cache Hit tokens: {}", hit);
        }
        if let Some(miss) = response.usage.cache_miss_tokens {
            println!("KV Cache Miss tokens: {}", miss);
        }
        println!("-------------------\n");

        // Add assistant's response to the context
        messages.push(Message {
            role: Role::Assistant,
            content: vec![Content::Text {
                text: response.text,
            }],
        });
    }

    Ok(())
}
