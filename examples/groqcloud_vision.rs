use qai_sdk::core::types::{
    Content, GenerateOptions, ImageSource, Message, Prompt, ProviderSettings, Role,
};
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

    // Connect to Llama 4 Scout Vision Model
    let vision_model = provider.chat("meta-llama/llama-4-scout-17b-16e-instruct");

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![
                Content::Text {
                    text: "What is in this image? Please describe it.".into(),
                },
                Content::Image {
                    source: ImageSource::Url {
                        url: "https://upload.wikimedia.org/wikipedia/commons/f/f2/LPU-v1-die.jpg"
                            .into(),
                    },
                },
            ],
        }],
    };

    println!("Sending image to GroqCloud (Llama 4 Scout)...");
    let result = vision_model
        .generate(
            prompt,
            GenerateOptions {
                model_id: "meta-llama/llama-4-scout-17b-16e-instruct".into(),
                max_tokens: Some(1024),
                temperature: Some(0.5),
                ..Default::default()
            },
        )
        .await?;

    println!("\n--- Groq Vision Response ---\n{}", result.text);

    println!(
        "\n[Usage] Prompt: {}, Completion: {}",
        result.usage.prompt_tokens, result.usage.completion_tokens
    );

    Ok(())
}
