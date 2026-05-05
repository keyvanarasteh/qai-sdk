//! # Google Gemini Thinking/Reasoning Example
//!
//! Demonstrates the thinking/reasoning capabilities of Gemini 3 and 2.5 models.
//! Gemini's thinking feature lets the model reason through complex problems,
//! with optional thought summaries exposed in the response.
//!
//! ## Requirements
//! - `GOOGLE_GENERATIVE_AI_API_KEY` environment variable
//!
//! ## Run
//! ```bash
//! cargo run --example gemini_thinking --features google
//! ```

use futures::StreamExt;
use qai_sdk::*;

use qai_sdk::LanguageModel;
#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let provider = create_google(ProviderSettings {
        api_key: Some(std::env::var("GOOGLE_GENERATIVE_AI_API_KEY").unwrap_or_default()),
        ..Default::default()
    });

    // ===================================================================
    // 1. Basic Thinking (Gemini 3 with thought summaries)
    // ===================================================================
    println!("=== Gemini Thinking with Thought Summaries ===\n");

    let model = provider.chat("gemini-3-flash-preview");

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "What is the sum of the first 50 prime numbers? Show your reasoning."
                    .into(),
            }],
        }],
    };

    // Enable thought summaries via reasoning_format = "parsed"
    // Control thinking level via reasoning_effort
    let options = GenerateOptions {
        model_id: "gemini-3-flash-preview".to_string(),
        max_tokens: Some(4096),
        reasoning_format: Some("parsed".to_string()),
        reasoning_effort: Some("high".to_string()),
        ..Default::default()
    };

    match model.generate(prompt.clone(), options).await {
        Ok(result) => {
            if let Some(reasoning) = &result.reasoning {
                println!("🧠 Thought Summary:\n{}\n", reasoning);
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
    // 2. Streaming with Thought Deltas
    // ===================================================================
    println!("\n\n=== Streaming with Thought Deltas ===\n");

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "Alice, Bob, and Carol each live in a different colored house. \
                       The red house owner has a cat. Bob doesn't live in the green house. \
                       Carol owns a dog. Who lives where?"
                    .into(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "gemini-3-flash-preview".to_string(),
        max_tokens: Some(2048),
        reasoning_format: Some("parsed".to_string()),
        reasoning_effort: Some("medium".to_string()),
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
    // 3. Gemini 2.5 with Thinking Budget
    // ===================================================================
    println!("\n\n=== Gemini 2.5 with Thinking Budget ===\n");

    let model_25 = provider.chat("gemini-2.5-flash-preview-05-20");

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "Write a haiku about recursion.".into(),
            }],
        }],
    };

    // Use a numeric budget (e.g., "1024") or "dynamic" for Gemini 2.5
    let options = GenerateOptions {
        model_id: "gemini-2.5-flash-preview-05-20".to_string(),
        max_tokens: Some(512),
        reasoning_format: Some("parsed".to_string()),
        reasoning_effort: Some("1024".to_string()), // thinking_budget = 1024 tokens
        ..Default::default()
    };

    match model_25.generate(prompt, options).await {
        Ok(result) => {
            if let Some(reasoning) = &result.reasoning {
                println!("🧠 Thought Summary:\n{}\n", reasoning);
            }
            println!("📝 Answer:\n{}", result.text);
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    Ok(())
}
