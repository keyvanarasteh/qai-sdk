use qai_sdk::mcp::{McpClient, McpTransport};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to dummy MCP server...");
    
    // Use the Node.js dummy server
    let transport = McpTransport::stdio("node", vec!["examples/dummy_mcp.js"]);
    let mcp_client = McpClient::connect(transport).await?;

    println!("\n--- Fetching Resources ---");
    let (resources, next_cursor) = mcp_client.list_resources(None).await?;
    println!("Fetched {} resources from MCP server.", resources.len());
    for r in &resources {
        println!("- {} ({}): {}", r.name, r.uri, r.description.as_deref().unwrap_or(""));
    }

    println!("\n--- Fetching Resource Templates ---");
    let (templates, next_cursor_tmpl) = mcp_client.list_resource_templates(None).await?;
    println!("Fetched {} templates from MCP server.", templates.len());
    for t in &templates {
        println!("- {} ({})", t.name, t.uri_template);
    }

    if let Some(first) = resources.first() {
        println!("\n--- Reading Resource '{}' ---", first.uri);
        
        let contents = mcp_client.read_resource(&first.uri).await?;
        
        for content in contents {
            println!("URI: {}", content.uri);
            if let Some(mime) = content.mime_type {
                println!("MIME: {}", mime);
            }
            if let Some(text) = content.text {
                println!("Text Content:\n{}", text);
            }
            if let Some(blob) = content.blob {
                println!("Blob Content (Base64): {}", blob);
            }
        }
    }

    Ok(())
}
