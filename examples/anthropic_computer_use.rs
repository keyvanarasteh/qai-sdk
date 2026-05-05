//! # Anthropic Computer Use Example
//!
//! Demonstrates how to define and use Anthropic's Computer Use Beta tools.
//! The SDK natively maps `computer_20241022`, `bash_20241022`, and `text_editor_20241022`
//! tools to their corresponding Anthropic beta structures.
//!
//! ## Usage
//!
//! ```bash
//! export ANTHROPIC_API_KEY=sk-...
//! cargo run --example anthropic_computer_use --features anthropic
//! ```

use qai_sdk::core::types::{GenerateOptions, Prompt, ToolDefinition};
use qai_sdk::core::LanguageModel;
use qai_sdk::anthropic::create_anthropic;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");

    let provider = create_anthropic(qai_sdk::core::types::ProviderSettings {
        api_key: Some(api_key),
        ..Default::default()
    });

    let model = provider.chat("claude-3-5-sonnet-20241022");

    let prompt = Prompt::user("Please look at my screen and tell me what you see, then run 'ls' in bash.");

    // Define the beta tools
    let computer_tool = ToolDefinition {
        name: "computer_20241022".to_string(),
        description: "Use the computer".to_string(),
        parameters: json!({
            "display_width_px": 1920,
            "display_height_px": 1080,
            "display_number": 1
        }),
    };

    let bash_tool = ToolDefinition {
        name: "bash_20241022".to_string(),
        description: "Run bash commands".to_string(),
        parameters: json!({}),
    };

    println!("Sending request to Claude with Computer Use tools...");

    let result = model
        .generate(
            prompt,
            GenerateOptions {
                tools: Some(vec![computer_tool, bash_tool]),
                ..Default::default()
            },
        )
        .await?;

    println!("\nResponse:");
    println!("{}", result.text);

    if !result.tool_calls.is_empty() {
        println!("\nTool Calls Requested:");
        for call in result.tool_calls {
            println!("- {}: {}", call.name, call.arguments);
        }
    }

    Ok(())
}
