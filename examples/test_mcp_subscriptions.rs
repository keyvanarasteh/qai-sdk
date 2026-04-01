use qai_sdk::mcp::{McpClient, McpTransport};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to dummy MCP server...");
    let transport = McpTransport::stdio("node", vec!["examples/dummy_mcp.js"]);
    let mcp_client = McpClient::connect(transport).await?;

    let uri = "file:///logs/app.log";
    
    // Create the broadcast channel stream
    let mut rx = mcp_client.resource_updates();

    println!("Subscribing to {}", uri);
    mcp_client.subscribe_resource(uri).await?;

    println!("Waiting for live updates...");
    
    // We should receive a few updates within 1.5 seconds
    for _ in 0..3 {
        match rx.recv().await {
            Ok(updated_uri) => {
                println!("\n✅ Received Notification: Resource updated -> {}", updated_uri);
                
                // Read the actual new content
                let content = mcp_client.read_resource(&updated_uri).await?;
                if let Some(c) = content.first() {
                    println!("=> New Content: {}", c.text.clone().unwrap_or_default());
                }
            }
            Err(e) => {
                println!("❌ Error receiving update: {:?}", e);
            }
        }
    }

    println!("\nUnsubscribing from {}", uri);
    mcp_client.unsubscribe_resource(uri).await?;

    // Wait a bit to ensure no more messages arrive
    println!("Waiting 1.5 seconds to verify stream is closed...");
    sleep(Duration::from_millis(1500)).await;
    
    if let Ok(late_msg) = rx.try_recv() {
        println!("❌ Unexpected late message: {}", late_msg);
    } else {
        println!("✅ Successfully halted subscription! No new messages arrived.");
    }

    Ok(())
}
