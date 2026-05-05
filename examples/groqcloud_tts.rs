use qai_sdk::core::types::{ProviderSettings, SpeechOptions};
use qai_sdk::core::SpeechModel;
use qai_sdk::groqcloud::create_groqcloud;
use std::env;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Requires GROQ_API_KEY environment variable.
    if env::var("GROQ_API_KEY").is_err() {
        println!("Please set the GROQ_API_KEY environment variable.");
        return Ok(());
    }

    let provider = create_groqcloud(ProviderSettings::default());

    // Connect to Groq's Orpheus TTS model
    let tts_model = provider.speech("canopylabs/orpheus-v1-english");

    let text_to_speak = "Welcome to Groq text-to-speech. [cheerful] This is an example of high-quality English audio generation with vocal directions support.";

    println!("Synthesizing speech via GroqCloud Orpheus...");
    let result = tts_model
        .synthesize(SpeechOptions {
            model_id: "canopylabs/orpheus-v1-english".into(),
            input: text_to_speak.into(),
            voice: "troy".into(), // Orpheus voices: "troy", "hannah", "austin"
            response_format: Some("wav".into()),
            speed: None,
        })
        .await?;

    let output_path = "orpheus_output.wav";
    fs::write(output_path, &result.audio)?;

    println!("Successfully generated audio file: {}", output_path);

    Ok(())
}
