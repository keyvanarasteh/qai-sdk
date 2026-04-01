//! # Speech & Transcription Example
//!
//! Demonstrates Text-to-Speech (TTS) and Speech-to-Text (STT) using
//! the `SpeechModel` and `TranscriptionModel` traits. Currently OpenAI-only.

use qai_sdk::types::{ProviderSettings, SpeechOptions, TranscriptionOptions};
use qai_sdk::{SpeechModel, TranscriptionModel};

use qai_sdk::LanguageModel;
#[tokio::main]
async fn main() -> qai_sdk::Result<()> {
    dotenvy::dotenv().ok();

    let provider = qai_sdk::openai::create_openai(ProviderSettings {
        api_key: Some(std::env::var("OPENAI_API_KEY").unwrap_or_default()),
        ..Default::default()
    });

    // ===================================================================
    // 1. Text-to-Speech (TTS)
    // ===================================================================
    println!("=== Text-to-Speech ===");
    let speech_model = provider.speech("tts-1");

    // Basic TTS with default settings
    let options = SpeechOptions {
        model_id: "tts-1".to_string(),
        input: "Hello! This is a demo of the QAI SDK's text-to-speech capabilities.".to_string(),
        voice: "alloy".to_string(), // alloy, echo, fable, onyx, nova, shimmer
        response_format: Some("mp3".to_string()), // mp3, opus, aac, flac, wav, pcm
        speed: None,                // 0.25 to 4.0, default 1.0
    };

    let result = speech_model.synthesize(options).await?;
    println!("Generated {} bytes of audio (MP3)", result.audio.len());

    // Save to file
    let path = "/tmp/qai_speech_demo.mp3";
    std::fs::write(path, &result.audio)?;
    println!("Saved to {}", path);

    // TTS with different voice and speed
    println!("\n--- Different voice + speed ---");
    let options = SpeechOptions {
        model_id: "tts-1-hd".to_string(), // HD model for higher quality
        input: "Rust is blazingly fast and memory efficient.".to_string(),
        voice: "nova".to_string(),
        response_format: Some("opus".to_string()),
        speed: Some(1.25), // 25% faster
    };

    let result = speech_model.synthesize(options).await?;
    println!(
        "Generated {} bytes of audio (Opus, 1.25x speed)",
        result.audio.len()
    );

    let path = "/tmp/qai_speech_fast.opus";
    std::fs::write(path, &result.audio)?;
    println!("Saved to {}", path);

    // All available voices demo
    println!("\n--- All voices ---");
    let voices = ["alloy", "echo", "fable", "onyx", "nova", "shimmer"];
    for voice in &voices {
        let options = SpeechOptions {
            model_id: "tts-1".to_string(),
            input: format!("Hi, I'm the {} voice.", voice),
            voice: voice.to_string(),
            response_format: Some("mp3".to_string()),
            speed: None,
        };
        let result = speech_model.synthesize(options).await?;
        println!("  Voice '{}': {} bytes", voice, result.audio.len());
    }

    // ===================================================================
    // 2. Speech-to-Text (Transcription)
    // ===================================================================
    println!("\n=== Speech-to-Text ===");
    let transcription_model = provider.transcription("whisper-1");

    // Transcribe the audio we just generated
    let audio_bytes = std::fs::read("/tmp/qai_speech_demo.mp3")?;
    println!("Loaded {} bytes of audio", audio_bytes.len());

    let options = TranscriptionOptions {
        model_id: "whisper-1".to_string(),
        audio: audio_bytes,
        language: Some("en".to_string()), // ISO 639-1 language code
        prompt: None,
        temperature: Some(0.0), // Lower = more deterministic
    };

    let result = transcription_model.transcribe(options).await?;
    println!("Transcription: \"{}\"", result.text);
    if let Some(lang) = &result.language {
        println!("Detected language: {}", lang);
    }
    if let Some(duration) = result.duration {
        println!("Duration: {:.2}s", duration);
    }

    // Transcribe with hint prompt
    println!("\n--- Transcription with context hint ---");
    let audio_bytes = std::fs::read("/tmp/qai_speech_fast.opus")?;

    let options = TranscriptionOptions {
        model_id: "whisper-1".to_string(),
        audio: audio_bytes,
        language: Some("en".to_string()),
        prompt: Some("This audio discusses the Rust programming language.".to_string()),
        temperature: Some(0.0),
    };

    let result = transcription_model.transcribe(options).await?;
    println!("Transcription: \"{}\"", result.text);

    Ok(())
}
