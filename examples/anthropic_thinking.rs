//! # Anthropic Claude Extended / Adaptive Thinking Example
//!
//! Demonstrates Claude's extended thinking and adaptive thinking capabilities.
//! Extended thinking lets Claude reason through complex problems step-by-step,
//! with thought summaries exposed in the response.
//!
//! ## Thinking Modes
//!
//! - **Adaptive** (recommended for Claude 4.6+): Claude decides when and how much to think.
//! - **Manual** (legacy): Fixed token budget for thinking.
//! - **Disabled**: No thinking.
//!
//! ## Requirements
//! - `ANTHROPIC_API_KEY` environment variable
//!
//! ## Run
//! ```bash
//! cargo run --example anthropic_thinking --features anthropic
//! ```

use futures::StreamExt;
use qai_sdk::*;

use qai_sdk::LanguageModel;
#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let provider = create_anthropic(ProviderSettings {
        api_key: Some(std::env::var("ANTHROPIC_API_KEY").unwrap_or_default()),
        ..Default::default()
    });

    // ===================================================================
    // 1. Adaptive Thinking (recommended for Claude 4.6+)
    // ===================================================================
    println!("=== Claude Adaptive Thinking ===\n");

    let model = provider.chat("claude-sonnet-4-6");

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "What is the greatest common divisor of 1071 and 462? Show your work."
                    .into(),
            }],
        }],
    };

    // reasoning_effort = "adaptive" → thinking: {type: "adaptive"}
    // reasoning_format = "parsed" → display: "summarized" (shows thought summaries)
    let options = GenerateOptions {
        model_id: "claude-sonnet-4-6".to_string(),
        max_tokens: Some(16000),
        reasoning_format: Some("parsed".to_string()),
        reasoning_effort: Some("adaptive".to_string()),
        ..Default::default()
    };

    match model.generate(prompt.clone(), options).await {
        Ok(result) => {
            if let Some(reasoning) = &result.reasoning {
                println!("🧠 Thinking Summary:\n{}\n", reasoning);
            }
            println!("📝 Answer:\n{}", result.text);
            println!(
                "\n📊 Tokens: {} in, {} out",
                result.usage.prompt_tokens, result.usage.completion_tokens
            );
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    // ===================================================================
    // 2. Streaming with Thinking Deltas
    // ===================================================================
    println!("\n\n=== Streaming with Thinking Deltas ===\n");

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "A snail is at the bottom of a 30-foot well. Each day it climbs up 3 feet, \
                       but each night it slides back 2 feet. How many days does it take to reach \
                       the top?"
                    .into(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "claude-sonnet-4-6".to_string(),
        max_tokens: Some(16000),
        reasoning_format: Some("parsed".to_string()),
        reasoning_effort: Some("high".to_string()),
        ..Default::default()
    };

    match model.generate_stream(prompt, options).await {
        Ok(mut stream) => {
            let mut in_reasoning = false;
            while let Some(part) = stream.next().await {
                match part {
                    StreamPart::ReasoningDelta { delta } => {
                        if !in_reasoning {
                            println!("🧠 Thinking:");
                            in_reasoning = true;
                        }
                        print!("{}", delta);
                    }
                    StreamPart::TextDelta { delta } => {
                        if in_reasoning {
                            println!("\n\n📝 Answer:");
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
    // 3. Manual Thinking Budget (legacy, for older models)
    // ===================================================================
    println!("\n\n=== Manual Thinking Budget (Legacy) ===\n");

    let model_legacy = provider.chat("claude-sonnet-4-5-20250514");

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "Write a haiku about recursion.".into(),
            }],
        }],
    };

    // Numeric reasoning_effort = manual budget_tokens
    let options = GenerateOptions {
        model_id: "claude-sonnet-4-5-20250514".to_string(),
        max_tokens: Some(16000),
        reasoning_format: Some("parsed".to_string()),
        reasoning_effort: Some("10000".to_string()), // budget_tokens = 10000
        ..Default::default()
    };

    match model_legacy.generate(prompt, options).await {
        Ok(result) => {
            if let Some(reasoning) = &result.reasoning {
                println!("🧠 Thinking:\n{}\n", reasoning);
            }
            println!("📝 Answer:\n{}", result.text);
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    Ok(())
}
