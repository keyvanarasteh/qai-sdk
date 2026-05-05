//! # Ollama Basic Chat Example
//!
//! Demonstrates how to connect to a local Ollama instance.
//!
//! Ensure you have Ollama running locally (`ollama run llama3.2`) before executing.

use qai_sdk::{
    core::types::{Content, GenerateOptions, Message, Prompt, ProviderSettings, Role},
    ollama::create_ollama,
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // By default, create_ollama connects to http://localhost:11434/v1
    let provider = create_ollama(ProviderSettings::default());

    // Connect to the llama3.2 model (or whichever model you have pulled in Ollama)
    let model = provider.chat("llama3.2");

    let prompt = Prompt {
        messages: vec![
            Message {
                role: Role::System,
                content: vec![Content::Text {
                    text: "You are a helpful local AI assistant.".to_string(),
                }],
            },
            Message {
                role: Role::User,
                content: vec![Content::Text {
                    text: "Write a short haiku about open source models.".to_string(),
                }],
            },
        ],
    };

    println!("Sending request to local Ollama (llama3.2)...\n");

    let options = GenerateOptions {
        model_id: "llama3.2".to_string(),
        temperature: Some(0.7),
        ..Default::default()
    };

    let result = model.generate(prompt, options).await?;

    println!("Response:\n{}", result.text);

    if let Some(usage) = result.usage {
        println!("\nTokens -> Prompt: {}, Completion: {}", usage.prompt_tokens, usage.completion_tokens);
    }

    Ok(())
}
