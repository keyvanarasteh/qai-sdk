use qai_sdk::core::types::{Content, GenerateOptions, Message, Prompt, ProviderSettings, Role};
use qai_sdk::core::LanguageModel;
use qai_sdk::groqcloud::create_groqcloud;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Requires GROQ_API_KEY environment variable.
    if env::var("GROQ_API_KEY").is_err() {
        println!("Please set the GROQ_API_KEY environment variable.");
        return Ok(());
    }

    let provider = create_groqcloud(ProviderSettings::default());

    // Connect to LLaMA-3.3 70B via Groq
    let model = provider.chat("llama-3.3-70b-versatile");

    let prompt = Prompt {
        messages: vec![
            Message {
                role: Role::System,
                content: vec![Content::Text {
                    text: "You are an expert systems programmer.".into(),
                }],
            },
            Message {
                role: Role::User,
                content: vec![Content::Text {
                    text: "Write a high-performance binary search in Rust.".into(),
                }],
            },
        ],
    };

    println!("Sending request to GroqCloud (LLaMA-3)...");
    let result = model
        .generate(
            prompt,
            GenerateOptions {
                model_id: "llama-3.3-70b-versatile".into(),
                max_tokens: Some(2048),
                temperature: Some(0.3),
                ..Default::default()
            },
        )
        .await?;

    println!("\n--- Groq Response ---\n{}", result.text);

    if let Some(usage) = result.usage {
        println!(
            "\n[Usage] Prompt: {}, Completion: {}",
            usage.prompt_tokens, usage.completion_tokens
        );
    }

    Ok(())
}
