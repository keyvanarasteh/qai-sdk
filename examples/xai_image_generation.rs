//! # xAI Image Generation Example
//!
//! Demonstrates how to use xAI's image generation capabilities with the `grok-imagine-image` model.
//!
//! ## Usage
//!
//! ```bash
//! export XAI_API_KEY=xai-...
//! cargo run --example xai_image_generation --features xai
//! ```

use qai_sdk::core::types::ImageGenerateOptions;
use qai_sdk::core::ImageModel;
use qai_sdk::xai::create_xai;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("XAI_API_KEY").expect("XAI_API_KEY must be set");

    let provider = create_xai(qai_sdk::core::types::ProviderSettings {
        api_key: Some(api_key),
        ..Default::default()
    });

    let model = provider.image("grok-imagine-image");

    let prompt = "A futuristic cyberpunk city with neon lights, flying cars, and holographic billboards reflecting in wet streets, highly detailed, 8k resolution, cinematic lighting";

    println!("Generating image with prompt: {}", prompt);
    println!("Model: grok-imagine-image\n");

    let result = model
        .generate(ImageGenerateOptions {
            model_id: "grok-imagine-image".to_string(),
            prompt: prompt.to_string(),
            n: Some(1),
            size: Some("1024x1024".to_string()),
            quality: None,
            response_format: Some("url".to_string()),
        })
        .await?;

    if let Some(url) = result.images.first() {
        println!("Image generated successfully!");
        println!("URL: {}", url);
    } else {
        println!("Failed to generate image (no URL returned).");
    }

    Ok(())
}
