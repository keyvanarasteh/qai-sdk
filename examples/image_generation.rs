//! # Image Generation Example
//!
//! Demonstrates image generation using the `ImageModel` trait
//! across OpenAI (DALL-E), Google (Imagen), and xAI (Grok Imagine).

use qai_core::types::{ImageGenerateOptions, ProviderSettings};
use qai_core::ImageModel;

#[tokio::main]
async fn main() -> qai_core::Result<()> {
    dotenvy::dotenv().ok();

    // ===================================================================
    // 1. OpenAI DALL-E Image Generation
    // ===================================================================
    println!("=== OpenAI DALL-E 3 ===");
    let provider = qai_sdk::openai::create_openai(ProviderSettings {
        api_key: Some(std::env::var("OPENAI_API_KEY").unwrap_or_default()),
        ..Default::default()
    });
    let model = provider.image("dall-e-3");

    let options = ImageGenerateOptions {
        model_id: "dall-e-3".to_string(),
        prompt: "A serene Japanese garden with cherry blossoms, watercolor style".to_string(),
        n: Some(1),
        size: Some("1024x1024".to_string()),
        quality: Some("standard".to_string()),
        response_format: Some("url".to_string()), // "url" or "b64_json"
    };

    let result = model.generate(options).await?;
    println!("Generated {} image(s)", result.images.len());
    for (i, img) in result.images.iter().enumerate() {
        println!("  Image {}: {}", i, &img[..100.min(img.len())]);
    }
    if let Some(revised) = &result.revised_prompt {
        println!("  Revised prompt: {}", revised);
    }

    // ===================================================================
    // 2. OpenAI DALL-E with base64 response
    // ===================================================================
    println!("\n=== OpenAI DALL-E 2 (base64) ===");
    let options = ImageGenerateOptions {
        model_id: "dall-e-2".to_string(),
        prompt: "A cute robot cat, digital art".to_string(),
        n: Some(2), // DALL-E 2 supports multiple images
        size: Some("256x256".to_string()),
        quality: None,
        response_format: Some("b64_json".to_string()),
    };

    let result = model.generate(options).await?;
    println!("Generated {} base64 image(s)", result.images.len());
    for (i, img) in result.images.iter().enumerate() {
        println!("  Image {}: {} bytes (base64)", i, img.len());
    }

    // ===================================================================
    // 3. Google Imagen Image Generation
    // ===================================================================
    println!("\n=== Google Imagen 3 ===");
    let provider = qai_sdk::google::create_google(ProviderSettings {
        api_key: Some(std::env::var("GOOGLE_GENERATIVE_AI_API_KEY").unwrap_or_default()),
        ..Default::default()
    });
    let model = provider.image("imagen-3.0-generate-001");

    let options = ImageGenerateOptions {
        model_id: "imagen-3.0-generate-001".to_string(),
        prompt: "A futuristic city skyline at sunset, photorealistic".to_string(),
        n: Some(1),
        size: None,
        quality: None,
        response_format: None,
    };

    let result = model.generate(options).await?;
    println!("Generated {} image(s)", result.images.len());

    // ===================================================================
    // 4. xAI Grok Imagine
    // ===================================================================
    println!("\n=== xAI Grok Imagine ===");
    let provider = qai_sdk::xai::create_xai(ProviderSettings {
        api_key: Some(std::env::var("XAI_API_KEY").unwrap_or_default()),
        ..Default::default()
    });
    let model = provider.image("grok-imagine-image");

    let options = ImageGenerateOptions {
        model_id: "grok-imagine-image".to_string(),
        prompt: "An astronaut riding a unicorn on Mars".to_string(),
        n: Some(1),
        size: Some("1024x1024".to_string()),
        quality: None,
        response_format: None,
    };

    let result = model.generate(options).await?;
    println!("Generated {} image(s)", result.images.len());

    Ok(())
}
