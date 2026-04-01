use qai_sdk::mcp::{McpClient, McpTransport};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to MCP SQLite server via npx...");
    
    // Spawn standard npx sqlite server for stdio MCP
    let transport = McpTransport::stdio(
        "node", 
        vec!["examples/dummy_mcp.js"]
    );
    
    let client = McpClient::connect(transport).await?;
    println!("Successfully connected and initialized!");

    // Fetch tool list
    let (tools, _) = client.get_tools(None).await?;
    println!("Available Tools:");
    for tool in tools {
        println!(" - {}: {}", tool.name, tool.description);
        println!("   Params: {}", tool.parameters);
    }

    Ok(())
}
