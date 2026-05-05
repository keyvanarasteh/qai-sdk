//! # DeepSeek FIM (Fill-In-the-Middle) Example
//!
//! Demonstrates how to use DeepSeek's `/beta/completions` API to perform
//! Fill-In-the-Middle (FIM) text completions, commonly used for code generation.
//!
//! ## Usage
//!
//! ```bash
//! export DEEPSEEK_API_KEY=sk-...
//! cargo run --example deepseek_fim --features deepseek
//! ```

use qai_sdk::core::types::CompletionOptions;
use qai_sdk::core::CompletionModel;
use qai_sdk::deepseek::create_deepseek;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY must be set");

    let provider = create_deepseek(qai_sdk::core::types::ProviderSettings {
        api_key: Some(api_key),
        ..Default::default()
    });

    let model = provider.completion("deepseek-coder");

    let prefix = "def calculate_fibonacci(n):\n    if n <= 1:\n        return n\n    ";
    let suffix = "\n\nprint(calculate_fibonacci(10))";

    println!("Performing Fill-In-the-Middle completion...\n");
    println!("\x1b[90m[Prefix]\x1b[0m\n{}", prefix);
    println!("\x1b[90m[Suffix]\x1b[0m{}", suffix);
    println!("\x1b[32m[Filling...]\x1b[0m\n");

    let result = model
        .complete(CompletionOptions {
            model_id: "deepseek-coder".to_string(),
            prompt: prefix.to_string(),
            suffix: Some(suffix.to_string()),
            max_tokens: Some(128),
            temperature: Some(0.1),
            ..Default::default()
        })
        .await?;

    println!("Result:\n");
    println!("{}{}{}", prefix, result.text, suffix);

    println!("\nUsage:");
    println!("- Prompt tokens: {}", result.usage.prompt_tokens);
    println!("- Completion tokens: {}", result.usage.completion_tokens);

    Ok(())
}
