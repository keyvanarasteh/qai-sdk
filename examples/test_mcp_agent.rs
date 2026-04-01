use qai_sdk::mcp::{McpClient, McpTransport};
use qai_sdk::*;
use std::env;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize MCP Client
    println!("Connecting to dummy MCP server...");
    let transport = McpTransport::stdio("node", vec!["examples/dummy_mcp.js"]);
    let mcp_client = McpClient::connect(transport).await?;

    // 2. Fetch available tools from the MCP server
    let mcp_tools = mcp_client.get_tools().await?;
    println!("Fetched {} tools from MCP server.", mcp_tools.len());

    // 3. Setup QAI-SDK Provider (OpenAI Compatible or DeepSeek)
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| "dummy_key".to_string());
    let provider = create_openai(ProviderSettings {
        api_key: Some(api_key),
        ..Default::default()
    });
    
    // For this example, if the API key is "dummy_key" we won't actually call the LLM
    // but we demonstrate the exact structure.
    let model = provider.chat("gpt-4o");

    println!("Sending request to LLM with MCP tools attached...");
    let messages = vec![Message {
        role: Role::User,
        content: vec![Content::Text {
            text: "What is the weather in Tokyo?".to_string(),
        }],
    }];

    // Only run real LLM if API key is provided
    if env::var("OPENAI_API_KEY").is_err() {
        println!("No OPENAI_API_KEY set. Simulating LLM tool call response...");
        
        let simulated_tool_call = Content::ToolCall {
            id: "call_123".to_string(),
            name: "get_weather".to_string(),
            arguments: serde_json::json!({ "location": "Tokyo" }),
        };

        if let Content::ToolCall { name, arguments, .. } = simulated_tool_call {
            println!("Model simulates invoking MCP tool: {}", name);
            // Execute the MCP tool using the client
            let tool_result = mcp_client.call_tool(&name, arguments).await?;
            println!("Result from MCP Server: {:?}", tool_result);
        }
        return Ok(());
    }

    // 4. Real Request to LLM
    let result = model
        .generate(
            Prompt { messages: messages.clone() },
            GenerateOptions {
                tools: Some(mcp_tools),
                model_id: "gpt-4o".to_string(),
                max_tokens: None,
                temperature: None,
                top_p: None,
                stop_sequences: None,
            },
        )
        .await?;

    println!("LLM Initial Response: {:?}", result.text);

    // 5. Execute MCP Tool Calls
    let mut new_messages = messages.clone();
    
    // Add assistant's message with tool calls
    let mut assistant_content = vec![];
    assistant_content.push(Content::Text { text: result.text.clone() });
    
    let mut call_idx = 1;
    for tc in &result.tool_calls {
        assistant_content.push(Content::ToolCall {
            id: format!("call_{}", call_idx),
            name: tc.name.clone(),
            arguments: tc.arguments.clone(),
        });
        call_idx += 1;
    }
    
    new_messages.push(Message {
        role: Role::Assistant,
        content: assistant_content,
    });

    let mut tool_results = vec![];
    let mut resp_call_idx = 1;
    for tool_call in result.tool_calls {
        println!("LLM requested MCP tool: {}", tool_call.name);
        
        // Execute the tool against the connected MCP server
        let tool_res = mcp_client.call_tool(&tool_call.name, tool_call.arguments).await?;
        println!("MCP Server returned: {:?}", tool_res);
        
        tool_results.push(Content::ToolResult {
            id: format!("call_{}", resp_call_idx),
            result: tool_res,
        });
        resp_call_idx += 1;
    }

    if !tool_results.is_empty() {
        new_messages.push(Message {
            role: Role::Tool,
            content: tool_results,
        });

        // Query LLM again with the gathered results
        println!("Sending MCP results back to LLM...");
        let final_result = model.generate(
            Prompt { messages: new_messages },
            GenerateOptions {
                model_id: "gpt-4o".to_string(),
                max_tokens: None,
                temperature: None,
                top_p: None,
                stop_sequences: None,
                tools: None,
            },
        ).await?;
        println!("Final Answer: {:?}", final_result.text);
    }

    Ok(())
}
