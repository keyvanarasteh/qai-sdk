use qai_sdk::core::types::{ProviderSettings, TranscriptionOptions};
use qai_sdk::core::TranscriptionModel;
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

    // Connect to Groq's high-speed Whisper model
    let stt_model = provider.transcription("whisper-large-v3-turbo");

    let dummy_audio_path = "test_audio.wav";
    if !std::path::Path::new(dummy_audio_path).exists() {
        println!("No test_audio.wav found. Please provide an audio file to run the STT example.");
        return Ok(());
    }

    let audio_bytes = fs::read(dummy_audio_path)?;

    println!("Sending audio to GroqCloud Whisper...");
    let result = stt_model
        .transcribe(TranscriptionOptions {
            model_id: "whisper-large-v3-turbo".into(),
            audio: audio_bytes,
            language: Some("en".into()), // Optional override
            prompt: None,
            temperature: None,
        })
        .await?;

    println!("\n--- Transcription ---");
    println!("{}", result.text);

    if let Some(dur) = result.duration {
        println!("Audio Duration: {:.2}s", dur);
    }

    Ok(())
}
