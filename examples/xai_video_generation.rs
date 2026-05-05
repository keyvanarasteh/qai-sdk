use qai_sdk::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let api_key = env::var("XAI_API_KEY").expect("XAI_API_KEY not set");

    let provider = create_xai(ProviderSettings {
        api_key: Some(api_key),
        ..Default::default()
    });

    let video_model = provider.video("grok-imagine-video");

    println!("🚀 Launching xAI Video Generation...");
    
    let result = video_model.generate(VideoGenerateOptions {
        model_id: "grok-imagine-video".to_string(),
        prompt: "A futuristic cyberpunk city with neon lights and flying cars, cinematic 4k".to_string(),
        ..Default::default()
    }).await?;

    if let Some(url) = result.url {
        println!("✅ Video ready: {}", url);
        println!("🆔 Request ID: {:?}", result.revision);
    } else {
        println!("❌ Video generation failed or timed out.");
    }

    Ok(())
}
