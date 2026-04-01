//! # Multimodal Chat Example
//!
//! Demonstrates sending images and files alongside text to vision-capable models.

use qai_sdk::types::ImageSource;
use qai_sdk::*;

use qai_sdk::LanguageModel;
#[tokio::main]
async fn main() -> qai_sdk::Result<()> {
    dotenvy::dotenv().ok();

    // ===================================================================
    // 1. Image URL input (OpenAI GPT-4o)
    // ===================================================================
    println!("=== Image URL Input (OpenAI) ===");
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    let model = qai_sdk::openai::OpenAIModel::new(api_key);

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![
                Content::Text {
                    text: "What do you see in this image? Be brief.".to_string(),
                },
                Content::Image {
                    source: ImageSource::Url {
                        url: "https://upload.wikimedia.org/wikipedia/commons/thumb/d/dd/Gfp-wisconsin-madison-the-702-702-702-702-702-702-702-702.jpg/1280px-Gfp-wisconsin-madison-the-nature-702.jpg".to_string(),
                    },
                },
            ],
        }],
    };

    let options = GenerateOptions {
        model_id: "gpt-4o-mini".to_string(),
        max_tokens: Some(200),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None,
    };

    let result = model.generate(prompt, options).await?;
    println!("Vision response: {}\n", result.text);

    // ===================================================================
    // 2. Base64 image input (Anthropic Claude)
    // ===================================================================
    println!("=== Base64 Image Input (Anthropic) ===");
    let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    let model = qai_sdk::anthropic::AnthropicModel::new(api_key);

    // Example: 1x1 red PNG pixel as base64
    let tiny_png_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==";

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![
                Content::Text {
                    text: "Describe this image.".to_string(),
                },
                Content::Image {
                    source: ImageSource::Base64 {
                        media_type: "image/png".to_string(),
                        data: tiny_png_base64.to_string(),
                    },
                },
            ],
        }],
    };

    let options = GenerateOptions {
        model_id: "claude-3-haiku-20240307".to_string(),
        max_tokens: Some(100),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None,
    };

    let result = model.generate(prompt, options).await?;
    println!("Vision response: {}\n", result.text);

    // ===================================================================
    // 3. Multi-turn conversation with images (Google Gemini)
    // ===================================================================
    println!("=== Multi-turn with Image (Google) ===");
    let api_key = std::env::var("GOOGLE_GENERATIVE_AI_API_KEY").unwrap_or_default();
    let model = qai_sdk::google::GoogleModel::new(api_key);

    let prompt = Prompt {
        messages: vec![
            Message {
                role: Role::User,
                content: vec![
                    Content::Text {
                        text: "Remember this image.".to_string(),
                    },
                    Content::Image {
                        source: ImageSource::Url {
                            url: "https://upload.wikimedia.org/wikipedia/commons/thumb/1/15/Cat_August_2010-4.jpg/1200px-Cat_August_2010-4.jpg".to_string(),
                        },
                    },
                ],
            },
            Message {
                role: Role::Assistant,
                content: vec![Content::Text {
                    text: "I see a cat in the image.".to_string(),
                }],
            },
            Message {
                role: Role::User,
                content: vec![Content::Text {
                    text: "What breed might it be?".to_string(),
                }],
            },
        ],
    };

    let options = GenerateOptions {
        model_id: "gemini-1.5-flash".to_string(),
        max_tokens: Some(200),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None,
    };

    let result = model.generate(prompt, options).await?;
    println!("Multi-turn vision response: {}", result.text);

    Ok(())
}
