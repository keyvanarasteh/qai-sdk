//! # xAI Reasoning Example
//!
//! Demonstrates reasoning/thinking capabilities with Grok models.
//! Grok-4.3 and Grok-4-1-fast reason automatically — no configuration needed.
//!
//! ## Requirements
//! - `XAI_API_KEY` environment variable
//!
//! ## Run
//! ```bash
//! cargo run --example xai_reasoning --features xai
//! ```

use futures::StreamExt;
use qai_sdk::*;

use qai_sdk::LanguageModel;
#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let provider = create_xai(ProviderSettings {
        api_key: Some(std::env::var("XAI_API_KEY").unwrap_or_default()),
        ..Default::default()
    });

    // ===================================================================
    // 1. Basic Reasoning (grok-4.3 reasons automatically)
    // ===================================================================
    println!("=== xAI Grok Reasoning (Auto) ===\n");

    let model = provider.chat("grok-4.3");

    let prompt = Prompt {
        messages: vec![
            Message {
                role: Role::System,
                content: vec![Content::Text {
                    text: "You are a highly intelligent AI assistant.".into(),
                }],
            },
            Message {
                role: Role::User,
                content: vec![Content::Text {
                    text: "A projectile is launched at 30 m/s at 37° above horizontal from a 45 m cliff. Find its speed on impact. (g=10 m/s²)".into(),
                }],
            },
        ],
    };

    // NOTE: grok-4.3 reasons automatically — do NOT set reasoning_effort
    // (it will return an error if you do)
    let options = GenerateOptions {
        model_id: "grok-4.3".to_string(),
        max_tokens: Some(2048),
        ..Default::default()
    };

    match model.generate(prompt.clone(), options).await {
        Ok(result) => {
            if let Some(reasoning) = &result.reasoning {
                println!("--- Reasoning Trace ---\n{}\n", reasoning);
            }
            println!("--- Answer ---\n{}", result.text);
            println!(
                "\n📊 Tokens: {} in, {} out",
                result.usage.prompt_tokens, result.usage.completion_tokens
            );
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    // ===================================================================
    // 2. Streaming with Reasoning Deltas
    // ===================================================================
    println!("\n\n=== Streaming with Reasoning ===\n");

    let options = GenerateOptions {
        model_id: "grok-4.3".to_string(),
        max_tokens: Some(1024),
        ..Default::default()
    };

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "What is 101 * 3? Think step by step.".into(),
            }],
        }],
    };

    match model.generate_stream(prompt, options).await {
        Ok(mut stream) => {
            let mut in_reasoning = false;
            while let Some(part) = stream.next().await {
                match part {
                    StreamPart::ReasoningDelta { delta } => {
                        if !in_reasoning {
                            println!("🧠 Reasoning:");
                            in_reasoning = true;
                        }
                        print!("{}", delta);
                    }
                    StreamPart::TextDelta { delta } => {
                        if in_reasoning {
                            println!("\n\n📝 Response:");
                            in_reasoning = false;
                        }
                        print!("{}", delta);
                    }
                    StreamPart::Usage { usage } => {
                        println!(
                            "\n\n📊 Tokens: {} in, {} out",
                            usage.prompt_tokens, usage.completion_tokens
                        );
                    }
                    StreamPart::Finish { finish_reason } => {
                        println!("✅ Finished: {}", finish_reason);
                    }
                    _ => {}
                }
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    // ===================================================================
    // 3. Responses API (with reasoning summary)
    // ===================================================================
    println!("\n\n=== Responses API Reasoning ===\n");

    let responses_model = provider.responses("grok-4.3");

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "Explain why the sky is blue in one sentence.".into(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "grok-4.3".to_string(),
        max_tokens: Some(512),
        ..Default::default()
    };

    match responses_model.generate(prompt, options).await {
        Ok(result) => {
            println!("📝 Response: {}", result.text);
            println!(
                "📊 Tokens: {} in, {} out",
                result.usage.prompt_tokens, result.usage.completion_tokens
            );
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    Ok(())
}
