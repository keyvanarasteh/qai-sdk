---
description: How to connect to MCP servers and use the auto-tool bridge
---

# MCP Integration

## Prerequisites

Add the `mcp` feature to `Cargo.toml`:
```toml
qai-sdk = { version = "0.1", features = ["mcp"] }
```

## Steps

### Option A: Auto-Tool Bridge (Recommended)

1. Import modules:
```rust
use qai_sdk::mcp::client::McpClient;
use qai_sdk::mcp::agent::run_mcp_agent;
```

2. Connect to an MCP server:
```rust
// Stdio transport
let client = McpClient::from_stdio("npx", &["-y", "@modelcontextprotocol/server-filesystem", "."]).await?;

// Or SSE transport
let client = McpClient::from_sse("http://localhost:3000/sse").await?;
```

3. Run the auto-tool bridge:
```rust
let model = provider.chat("gpt-4o");
let answer = run_mcp_agent(&model, &client, "List all Rust files").await?;
println!("{answer}");
```

### Option B: Manual Tool/Resource Operations

```rust
// List tools
let tools = client.list_tools().await?;

// Call a tool
let result = client.call_tool("read_file", serde_json::json!({"path": "src/main.rs"})).await?;

// List prompts
let prompts = client.list_prompts().await?;

// Get a prompt
let prompt = client.get_prompt("code_review", serde_json::json!({"lang": "rust"})).await?;

// List resources
let resources = client.list_resources().await?;

// Read a resource
let content = client.read_resource("file:///README.md").await?;

// Subscribe to resource updates
let mut rx = client.resource_updates();
while let Ok(uri) = rx.recv().await {
    println!("Updated: {uri}");
}
```

## Compatible MCP Servers

| Server | Command |
|---|---|
| Filesystem | `npx -y @modelcontextprotocol/server-filesystem .` |
| Everything (demo) | `npx -y @modelcontextprotocol/server-everything` |
| GitHub | `npx -y @modelcontextprotocol/server-github` |
| PostgreSQL | `npx -y @modelcontextprotocol/server-postgres` |
| Brave Search | `npx -y @modelcontextprotocol/server-brave-search` |

## Notes
- `run_mcp_agent` auto-discovers tools, injects them into the model, and handles the full tool-call round-trip
- Pagination is handled automatically for large tool/resource lists
- Resource subscriptions use `tokio::broadcast` channels
