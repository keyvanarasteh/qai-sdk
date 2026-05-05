//! # xAI Prompt Caching Example
//!
//! Demonstrates how to use xAI's automatic prompt caching with the SDK.
//! xAI caches the prefix of your conversation automatically. Using the
//! `x-grok-conv-id` header routes requests to the same server for maximum
//! cache hit rates.
//!
//! ## Usage
//!
//! ```bash
//! export XAI_API_KEY=xai-...
//! cargo run --example xai_prompt_caching --features xai
//! ```

use qai_sdk::core::types::{Content, GenerateOptions, Message, Prompt, Role};
use qai_sdk::core::LanguageModel;
use qai_sdk::xai::create_xai;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("XAI_API_KEY").expect("XAI_API_KEY must be set");

    let provider = create_xai(qai_sdk::core::types::ProviderSettings {
        api_key: Some(api_key),
        ..Default::default()
    });

    let model = provider.chat("grok-3-fast");

    // A long system prompt that benefits from caching
    let system_prompt = r#"You are an expert Rust programming assistant. You have deep knowledge of:
- The Rust programming language, including ownership, borrowing, lifetimes, and trait systems
- Async/await patterns with tokio and futures
- Serde serialization/deserialization
- Error handling with anyhow and thiserror
- HTTP clients like reqwest and hyper
- Web frameworks like axum and actix-web
- Database libraries like sqlx and diesel
- Testing patterns and best practices
Always provide idiomatic Rust code with proper error handling."#;

    // Create extra_headers with a conversation ID for sticky routing
    let mut headers = HashMap::new();
    headers.insert(
        "x-grok-conv-id".to_string(),
        "qai-sdk-example-conv-001".to_string(),
    );

    // --- Turn 1: First request establishes the cache ---
    println!("=== Turn 1: First request (cache priming) ===\n");

    let result = model
        .generate(
            Prompt {
                messages: vec![
                    Message {
                        role: Role::System,
                        content: vec![Content::Text {
                            text: system_prompt.to_string(),
                        }],
                    },
                    Message {
                        role: Role::User,
                        content: vec![Content::Text {
                            text: "Explain Rust's ownership model in one sentence.".to_string(),
                        }],
                    },
                ],
            },
            GenerateOptions {
                model_id: "grok-3-fast".to_string(),
                max_tokens: Some(200),
                extra_headers: Some(headers.clone()),
                ..Default::default()
            },
        )
        .await?;

    println!("Response: {}", result.text);
    println!(
        "\nUsage: prompt={}, completion={}",
        result.usage.prompt_tokens, result.usage.completion_tokens
    );
    println!(
        "Cache hit tokens: {} (expected 0 on first request)",
        result
            .usage
            .cache_hit_tokens
            .map_or("none".to_string(), |t| t.to_string())
    );

    // --- Turn 2: Same prefix → cache hit ---
    println!("\n=== Turn 2: Cache hit expected ===\n");

    let result2 = model
        .generate(
            Prompt {
                messages: vec![
                    Message {
                        role: Role::System,
                        content: vec![Content::Text {
                            text: system_prompt.to_string(),
                        }],
                    },
                    Message {
                        role: Role::User,
                        content: vec![Content::Text {
                            text: "Explain Rust's ownership model in one sentence.".to_string(),
                        }],
                    },
                    Message {
                        role: Role::Assistant,
                        content: vec![Content::Text {
                            text: result.text.clone(),
                        }],
                    },
                    Message {
                        role: Role::User,
                        content: vec![Content::Text {
                            text: "Now explain borrowing.".to_string(),
                        }],
                    },
                ],
            },
            GenerateOptions {
                model_id: "grok-3-fast".to_string(),
                max_tokens: Some(200),
                extra_headers: Some(headers.clone()),
                ..Default::default()
            },
        )
        .await?;

    println!("Response: {}", result2.text);
    println!(
        "\nUsage: prompt={}, completion={}",
        result2.usage.prompt_tokens, result2.usage.completion_tokens
    );
    println!(
        "Cache hit tokens: {} (should be > 0 if cache hit)",
        result2
            .usage
            .cache_hit_tokens
            .map_or("none".to_string(), |t| t.to_string())
    );

    // --- Turn 3: Continue multi-turn → growing cache ---
    println!("\n=== Turn 3: Growing cache ===\n");

    let result3 = model
        .generate(
            Prompt {
                messages: vec![
                    Message {
                        role: Role::System,
                        content: vec![Content::Text {
                            text: system_prompt.to_string(),
                        }],
                    },
                    Message {
                        role: Role::User,
                        content: vec![Content::Text {
                            text: "Explain Rust's ownership model in one sentence.".to_string(),
                        }],
                    },
                    Message {
                        role: Role::Assistant,
                        content: vec![Content::Text {
                            text: result.text.clone(),
                        }],
                    },
                    Message {
                        role: Role::User,
                        content: vec![Content::Text {
                            text: "Now explain borrowing.".to_string(),
                        }],
                    },
                    Message {
                        role: Role::Assistant,
                        content: vec![Content::Text {
                            text: result2.text.clone(),
                        }],
                    },
                    Message {
                        role: Role::User,
                        content: vec![Content::Text {
                            text: "What about lifetimes?".to_string(),
                        }],
                    },
                ],
            },
            GenerateOptions {
                model_id: "grok-3-fast".to_string(),
                max_tokens: Some(200),
                extra_headers: Some(headers),
                ..Default::default()
            },
        )
        .await?;

    println!("Response: {}", result3.text);
    println!(
        "\nUsage: prompt={}, completion={}",
        result3.usage.prompt_tokens, result3.usage.completion_tokens
    );
    println!(
        "Cache hit tokens: {} (should be larger than turn 2)",
        result3
            .usage
            .cache_hit_tokens
            .map_or("none".to_string(), |t| t.to_string())
    );

    println!("\n=== Summary ===");
    println!("Prompt caching is automatic with xAI. Use extra_headers with");
    println!("'x-grok-conv-id' to route requests to the same server for");
    println!("maximum cache hit rates across multi-turn conversations.");

    Ok(())
}
