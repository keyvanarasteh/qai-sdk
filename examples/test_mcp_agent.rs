use qai_sdk::mcp::{run_mcp_agent, McpClient, McpTransport};
use qai_sdk::*;
use std::env;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize MCP Client
    println!("Connecting to dummy MCP server...");
    let transport = McpTransport::stdio("node", vec!["examples/dummy_mcp.js"]);
    let mcp_client = McpClient::connect(transport).await?;

    // 2. Setup QAI-SDK Provider (OpenAI Compatible or DeepSeek)
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| "dummy_key".to_string());
    let provider = create_openai(ProviderSettings {
        api_key: Some(api_key),
        ..Default::default()
    });
    
    // For this example, if the API key is "dummy_key" we won't actually call the LLM
    if env::var("OPENAI_API_KEY").is_err() {
        println!("No OPENAI_API_KEY set. Cannot run full agent loop.");
        return Ok(());
    }

    let model = provider.chat("gpt-4o");

    println!("Starting autonomous LLM Tool loop via McpAgent:");
    let initial_messages = vec![Message {
        role: Role::User,
        content: vec![Content::Text {
            text: "What is the weather in Tokyo?".to_string(),
        }],
    }];

    let options = GenerateOptions {
        model_id: "gpt-4o".to_string(),
        max_tokens: None,
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None, // Tools are automatically populated by the loop
        response_format: None,
    };

    // 3. Just one line to run the entire loop!
    let final_answer = run_mcp_agent(
        &model,
        &mcp_client,
        initial_messages,
        options,
        5 // max turns
    ).await?;

    println!("Final Answer: {:?}", final_answer);

    Ok(())
}
