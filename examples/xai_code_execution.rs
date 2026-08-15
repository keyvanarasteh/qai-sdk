use qai_sdk::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize the xAI provider
    let api_key = std::env::var("XAI_API_KEY").expect("XAI_API_KEY not set");
    let provider = create_xai(ProviderSettings {
        api_key: Some(api_key),
        ..Default::default()
    });

    let model = provider.chat("grok-2");

    // 2. Prepare tools (Code Execution)
    let server_tools = vec![ServerTool {
        tool_type: "code_execution".to_string(),
        config: serde_json::json!({}),
    }];

    // 3. Generate a response with code execution
    let prompt = Prompt::from_user("Calculate the first 10 prime numbers using Python.");

    println!("Asking Grok with Code Execution...");

    let result = model
        .generate(
            prompt,
            GenerateOptions {
                model_id: "grok-2".to_string(),
                server_tools: Some(server_tools),
                ..Default::default()
            },
        )
        .await?;

    println!("\nResponse:\n{}", result.text);

    // Note: Built-in tool outputs for code execution currently appear within the text response
    // or as executed_tools metadata if identifying patterns are matched.

    Ok(())
}
