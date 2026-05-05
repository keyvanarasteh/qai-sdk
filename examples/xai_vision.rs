//! # xAI Vision Example
//!
//! Demonstrates how to use xAI's vision capabilities with the `grok-2-vision-1212` model.
//! You can pass images as base64 strings or URLs alongside text in the prompt.
//!
//! ## Usage
//!
//! ```bash
//! export XAI_API_KEY=xai-...
//! cargo run --example xai_vision --features xai
//! ```

use qai_sdk::core::types::{Content, GenerateOptions, ImageSource, Message, Prompt, Role};
use qai_sdk::core::LanguageModel;
use qai_sdk::xai::create_xai;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("XAI_API_KEY").expect("XAI_API_KEY must be set");

    let provider = create_xai(qai_sdk::core::types::ProviderSettings {
        api_key: Some(api_key),
        ..Default::default()
    });

    let model = provider.chat("grok-2-vision-1212");

    // We can use a public image URL or base64 data.
    // Here we use a sample image URL.
    let image_url = "https://upload.wikimedia.org/wikipedia/commons/thumb/d/dd/Gfp-wisconsin-madison-the-nature-boardwalk.jpg/2560px-Gfp-wisconsin-madison-the-nature-boardwalk.jpg";

    println!("Analyzing image from: {}", image_url);
    println!("Model: grok-2-vision-1212\n");

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![
                Content::Text {
                    text: "What's in this image? Please describe the scene in detail.".to_string(),
                },
                Content::Image {
                    source: ImageSource::Url {
                        url: image_url.to_string(),
                    },
                },
            ],
        }],
    };

    let result = model
        .generate(
            prompt,
            GenerateOptions {
                model_id: "grok-2-vision-1212".to_string(),
                max_tokens: Some(500),
                ..Default::default()
            },
        )
        .await?;

    println!("Response:\n{}", result.text);
    println!(
        "\nUsage: prompt={}, completion={}",
        result.usage.prompt_tokens, result.usage.completion_tokens
    );

    Ok(())
}
