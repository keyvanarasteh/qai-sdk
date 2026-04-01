use qai_sdk::mcp::{McpClient, McpTransport};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to dummy MCP server...");
    
    // Use the Node.js dummy server
    let transport = McpTransport::stdio("node", vec!["examples/dummy_mcp.js"]);
    let mcp_client = McpClient::connect(transport).await?;

    println!("\n--- Fetching Prompts ---");
    let (prompts, next_cursor) = mcp_client.list_prompts(None).await?;
    
    println!("Fetched {} prompts from MCP server.", prompts.len());
    for p in &prompts {
        println!("- {} ({} args)", p.name, p.arguments.as_ref().map_or(0, |a| a.len()));
        if let Some(desc) = &p.description {
            println!("  Desc: {}", desc);
        }
    }

    if let Some(first) = prompts.first() {
        println!("\n--- Fetching Details for Prompt '{}' ---", first.name);
        
        let mut args = HashMap::new();
        args.insert("code".to_string(), "fn main() { println!(\"Hello World!\"); }".to_string());
        
        let (description, messages) = mcp_client.get_prompt(&first.name, Some(args)).await?;
        
        println!("Prompt Description: {}", description);
        println!("Generated Messages:");
        for msg in messages {
            println!("  Role: {:?}", msg.role);
            for content in msg.content {
                println!("  Content: {:?}", content);
            }
        }
    }

    Ok(())
}
