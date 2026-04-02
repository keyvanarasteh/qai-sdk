use futures::StreamExt;
use qai_sdk::types::{Content, GenerateOptions, Message, Prompt, Role};
use qai_sdk::GoogleModel;

use qai_sdk::LanguageModel;
#[tokio::main]
async fn main() -> qai_sdk::Result<()> {
    dotenvy::dotenv().ok();
    let api_key = std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY not set");
    let model = GoogleModel::new(api_key);

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "Explain how a tokamak works in one sentence.".to_string(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "gemini-1.5-flash".to_string(),
        max_tokens: Some(100),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None,
        response_format: None,
    };

    println!("--- Generating (Non-Streaming) ---");
    let result = model.generate(prompt.clone(), options.clone()).await?;
    println!("Response: {}", result.text);
    println!(
        "Usage: {} tokens in, {} tokens out",
        result.usage.prompt_tokens, result.usage.completion_tokens
    );

    println!("\n--- Generating (Streaming) ---");
    let mut stream = model.generate_stream(prompt, options).await?;
    while let Some(part) = stream.next().await {
        match part {
            qai_sdk::types::StreamPart::TextDelta { delta } => print!("{}", delta),
            qai_sdk::types::StreamPart::Usage { usage } => println!(
                "\nUsage: {}/{}",
                usage.prompt_tokens, usage.completion_tokens
            ),
            _ => {}
        }
    }
    println!();

    Ok(())
}
