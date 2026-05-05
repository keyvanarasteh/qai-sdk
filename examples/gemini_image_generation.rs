use qai_sdk::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = env::var("GOOGLE_GENERATIVE_AI_API_KEY")
        .expect("GOOGLE_GENERATIVE_AI_API_KEY must be set");

    let provider = create_google(ProviderSettings {
        api_key: Some(api_key),
        ..Default::default()
    });

    // Imagen 3 model
    let model = provider.image("imagen-3.0-generate-001");

    println!("Generating image with Imagen 3...");
    let result = model
        .generate(ImageGenerateOptions {
            prompt: "A futuristic city with flying cars and neon lights, hyper-realistic, 8k".to_string(),
            n: Some(1),
            size: Some("1:1".to_string()),
            ..Default::default()
        })
        .await?;

    if let Some(image_base64) = result.images.first() {
        println!("Successfully generated image!");
        println!("Image base64 length: {}", image_base64.len());
        
        // Save to file
        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD.decode(image_base64)?;
        std::fs::write("gemini_image.png", bytes)?;
        println!("Saved to gemini_image.png");
    } else {
        println!("No images generated.");
    }

    Ok(())
}
