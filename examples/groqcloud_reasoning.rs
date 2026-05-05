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

    // Connect to Qwen 3 32B Reasoning Model
    let reasoning_model = provider.chat("qwen/qwen3-32b");

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "How do airplanes fly? Be concise.".into(),
            }],
        }],
    };

    println!("Sending request to GroqCloud (Qwen 3 32B Reasoning)...");
    let result = reasoning_model
        .generate(
            prompt,
            GenerateOptions {
                model_id: "qwen/qwen3-32b".into(),
                max_tokens: Some(1024),
                reasoning_format: Some("parsed".into()),
                reasoning_effort: Some("high".into()),
                ..Default::default()
            },
        )
        .await?;

    if let Some(reasoning) = &result.reasoning {
        println!("\n--- Groq Reasoning Trace ---\n{}", reasoning);
    }

    println!("\n--- Groq Final Answer ---\n{}", result.text);

    if let Some(usage) = result.usage {
        println!(
            "\n[Usage] Prompt: {}, Completion: {}",
            usage.prompt_tokens, usage.completion_tokens
        );
    }

    Ok(())
}
