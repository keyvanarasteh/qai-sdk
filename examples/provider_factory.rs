//! # Provider Factory Example
//!
//! Demonstrates creating providers with `ProviderSettings` and accessing
//! all model types through the factory pattern.

use qai_sdk::*;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // ===================================================================
    // 1. OpenAI Provider — full model suite
    // ===================================================================
    let openai = create_openai(ProviderSettings {
        api_key: Some(std::env::var("OPENAI_API_KEY").unwrap_or_default()),
        base_url: None, // defaults to https://api.openai.com/v1
        headers: None,
    });

    let _chat = openai.chat("gpt-4o");
    let _embedding = openai.embedding("text-embedding-3-small");
    let _image = openai.image("dall-e-3");
    let _completion = openai.completion("gpt-3.5-turbo-instruct");
    let _speech = openai.speech("tts-1");
    let _transcription = openai.transcription("whisper-1");
    let _responses = openai.responses("gpt-4o");
    println!("✅ OpenAI provider created with 7 model types");

    // ===================================================================
    // 2. Anthropic Provider — chat only
    // ===================================================================
    let anthropic = create_anthropic(ProviderSettings {
        api_key: Some(std::env::var("ANTHROPIC_API_KEY").unwrap_or_default()),
        base_url: None,
        headers: None,
    });

    let _chat = anthropic.chat("claude-3-haiku-20240307");
    println!("✅ Anthropic provider created");

    // ===================================================================
    // 3. Google Provider — chat + embedding + image
    // ===================================================================
    let google = create_google(ProviderSettings {
        api_key: Some(std::env::var("GOOGLE_GENERATIVE_AI_API_KEY").unwrap_or_default()),
        base_url: None,
        headers: None,
    });

    let _chat = google.chat("gemini-1.5-flash");
    let _embedding = google.embedding("text-embedding-004");
    let _image = google.image("imagen-3.0-generate-001");
    println!("✅ Google provider created with 3 model types");

    // ===================================================================
    // 4. DeepSeek Provider — chat (OpenAI-compatible)
    // ===================================================================
    let deepseek = create_deepseek(ProviderSettings {
        api_key: Some(std::env::var("DEEPSEEK_API_KEY").unwrap_or_default()),
        base_url: None, // defaults to https://api.deepseek.com
        headers: None,
    });

    let _chat = deepseek.chat("deepseek-chat");
    let _reasoner = deepseek.chat("deepseek-reasoner");
    println!("✅ DeepSeek provider created");

    // ===================================================================
    // 5. xAI Provider — chat + image + responses
    // ===================================================================
    let xai = create_xai(ProviderSettings {
        api_key: Some(std::env::var("XAI_API_KEY").unwrap_or_default()),
        base_url: None, // defaults to https://api.x.ai/v1
        headers: None,
    });

    let _chat = xai.chat("grok-3");
    let _image = xai.image("grok-imagine-image");
    let _responses = xai.responses("grok-3");
    println!("✅ xAI provider created with 3 model types");

    // ===================================================================
    // 6. OpenAI-Compatible Provider — any endpoint
    // ===================================================================
    use qai_sdk::openai_compatible::OpenAICompatibleProviderSettings;

    let compatible = create_openai_compatible(OpenAICompatibleProviderSettings {
        base_url: "https://api.together.xyz/v1".to_string(),
        name: "together".to_string(),
        api_key: Some(std::env::var("TOGETHER_API_KEY").unwrap_or_default()),
        headers: None,
    });

    let _chat = compatible.chat("meta-llama/Llama-3-70b-chat-hf");
    let _embedding = compatible.embedding("togethercomputer/m2-bert-80M-8k-retrieval");
    let _image = compatible.image("stabilityai/stable-diffusion-xl-base-1.0");
    let _completion = compatible.completion("meta-llama/Llama-3-70b-chat-hf");
    println!("✅ OpenAI-Compatible provider created with 4 model types");

    // ===================================================================
    // Using the prelude for convenience
    // ===================================================================
    println!("\n--- Provider Factory Summary ---");
    println!("OpenAI:      chat, embedding, image, completion, speech, transcription, responses");
    println!("Anthropic:   chat");
    println!("Google:      chat, embedding, image");
    println!("DeepSeek:    chat");
    println!("xAI:         chat, image, responses");
    println!("Compatible:  chat, embedding, image, completion");

    Ok(())
}
