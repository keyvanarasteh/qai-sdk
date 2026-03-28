use qai_openai::OpenAIModel;
use qai_core::types::{Prompt, Message, Role, Content, GenerateOptions};
use qai_core::LanguageModel;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let model = OpenAIModel::new(api_key);

    let prompt = Prompt {
        messages: vec![
            Message {
                role: Role::User,
                content: vec![Content::Text { text: "Explain the Fermi Paradox in one sentence.".to_string() }],
            },
        ],
    };

    let options = GenerateOptions {
        model_id: "gpt-4o".to_string(),
        max_tokens: Some(100),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None,
    };

    println!("--- Generating (Non-Streaming) ---");
    let result = model.generate(prompt.clone(), options.clone()).await?;
    println!("Response: {}", result.text);
    println!("Usage: {} tokens in, {} tokens out", result.usage.prompt_tokens, result.usage.completion_tokens);

    println!("\n--- Generating (Streaming) ---");
    let mut stream = model.generate_stream(prompt, options).await?;
    while let Some(part) = stream.next().await {
        match part {
            qai_core::types::StreamPart::TextDelta { delta } => print!("{}", delta),
            qai_core::types::StreamPart::Usage { usage } => println!("\nUsage: {}/{}", usage.prompt_tokens, usage.completion_tokens),
            _ => {}
        }
    }
    println!();

    Ok(())
}
