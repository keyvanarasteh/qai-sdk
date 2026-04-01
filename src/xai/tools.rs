//! xAI server-defined tools.
//!
//! These tools are xAI-specific capabilities available through the responses API.
//! Each function returns a `serde_json::Value` representing the tool configuration.

use serde_json::{json, Value};

/// Creates a web search tool for xAI.
#[must_use]
pub fn web_search(search_context_size: Option<String>) -> Value {
    let mut tool = json!({ "type": "web_search" });
    if let Some(s) = search_context_size {
        tool["search_context_size"] = json!(s);
    }
    tool
}

/// Creates a code execution tool for xAI.
#[must_use]
pub fn code_execution() -> Value {
    json!({ "type": "code_execution" })
}

/// Creates a file search tool for xAI.
#[must_use]
pub fn file_search(vector_store_ids: Vec<String>) -> Value {
    json!({
        "type": "file_search",
        "vector_store_ids": vector_store_ids,
    })
}

/// Creates an MCP server tool for xAI.
#[must_use]
pub fn mcp_server(server_label: String, server_url: Option<String>) -> Value {
    let mut tool = json!({
        "type": "mcp",
        "server_label": server_label,
    });
    if let Some(u) = server_url {
        tool["server_url"] = json!(u);
    }
    tool
}

/// Creates a view image tool for xAI.
#[must_use]
pub fn view_image() -> Value {
    json!({ "type": "view_image" })
}

/// Creates a view X video tool for xAI.
#[must_use]
pub fn view_x_video() -> Value {
    json!({ "type": "view_x_video" })
}

/// Creates an X search tool for xAI.
#[must_use]
pub fn x_search() -> Value {
    json!({ "type": "x_search" })
}
