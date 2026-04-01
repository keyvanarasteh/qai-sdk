//! OpenAI server-defined tools.
//!
//! These tools are OpenAI-specific capabilities available through the Responses API.
//! Each function returns a `serde_json::Value` representing the tool configuration.

use serde_json::{json, Value};

/// Creates a code interpreter tool for running Python in a sandbox.
pub fn code_interpreter(container: Option<String>) -> Value {
    let mut tool = json!({ "type": "code_interpreter" });
    if let Some(c) = container {
        tool["container"] = json!(c);
    }
    tool
}

/// Creates a file search tool for vector store retrieval.
pub fn file_search(vector_store_ids: Vec<String>, max_num_results: Option<u32>) -> Value {
    let mut tool = json!({
        "type": "file_search",
        "vector_store_ids": vector_store_ids,
    });
    if let Some(m) = max_num_results {
        tool["max_num_results"] = json!(m);
    }
    tool
}

/// Creates a web search preview tool.
pub fn web_search_preview(search_context_size: Option<String>) -> Value {
    let mut tool = json!({ "type": "web_search_preview" });
    if let Some(s) = search_context_size {
        tool["search_context_size"] = json!(s);
    }
    tool
}

/// Creates a web search tool with filters and location support.
pub fn web_search(
    search_context_size: Option<String>,
    user_location: Option<Value>,
    filters: Option<Value>,
) -> Value {
    let mut tool = json!({ "type": "web_search" });
    if let Some(s) = search_context_size {
        tool["search_context_size"] = json!(s);
    }
    if let Some(ul) = user_location {
        tool["user_location"] = ul;
    }
    if let Some(f) = filters {
        tool["filters"] = f;
    }
    tool
}

/// Creates an image generation tool.
pub fn image_generation(
    model: Option<String>,
    quality: Option<String>,
    size: Option<String>,
    background: Option<String>,
    output_format: Option<String>,
) -> Value {
    let mut tool = json!({ "type": "image_generation" });
    if let Some(m) = model {
        tool["model"] = json!(m);
    }
    if let Some(q) = quality {
        tool["quality"] = json!(q);
    }
    if let Some(s) = size {
        tool["size"] = json!(s);
    }
    if let Some(b) = background {
        tool["background"] = json!(b);
    }
    if let Some(f) = output_format {
        tool["output_format"] = json!(f);
    }
    tool
}

/// Creates a shell tool for GPT-5.1 command execution.
pub fn shell() -> Value {
    json!({ "type": "shell" })
}

/// Creates a local shell tool for GPT-5 Codex.
pub fn local_shell() -> Value {
    json!({ "type": "local_shell" })
}

/// Creates an apply_patch tool for structured code diffs.
pub fn apply_patch() -> Value {
    json!({ "type": "apply_patch" })
}

/// Creates a custom tool with grammar constraints.
pub fn custom_tool(name: String, description: Option<String>, format: Option<Value>) -> Value {
    let mut tool = json!({
        "type": "custom_tool",
        "name": name,
    });
    if let Some(d) = description {
        tool["description"] = json!(d);
    }
    if let Some(f) = format {
        tool["format"] = f;
    }
    tool
}

/// Creates an MCP (Model Context Protocol) tool.
pub fn mcp(
    server_label: String,
    server_url: Option<String>,
    allowed_tools: Option<Value>,
    headers: Option<Value>,
) -> Value {
    let mut tool = json!({
        "type": "mcp",
        "server_label": server_label,
    });
    if let Some(u) = server_url {
        tool["server_url"] = json!(u);
    }
    if let Some(at) = allowed_tools {
        tool["allowed_tools"] = at;
    }
    if let Some(h) = headers {
        tool["headers"] = h;
    }
    tool
}

/// Creates a tool search tool for deferred tool loading.
pub fn tool_search() -> Value {
    json!({ "type": "tool_search" })
}
