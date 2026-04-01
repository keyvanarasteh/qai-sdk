//! Anthropic server-defined tools.
//!
//! These tools are Anthropic-specific capabilities that can be passed alongside
//! user-defined tools in API requests. Each function returns a `serde_json::Value`
//! representing the tool configuration.

use serde_json::{json, Value};

/// Creates a bash tool (version 2024-10-22) for executing shell commands.
#[must_use]
pub fn bash_20241022() -> Value {
    json!({ "type": "bash_20241022", "name": "bash" })
}

/// Creates a bash tool (version 2025-01-24) with enhanced capabilities.
#[must_use]
pub fn bash_20250124() -> Value {
    json!({ "type": "bash_20250124", "name": "bash" })
}

/// Creates a code execution tool (version 2025-05-22).
#[must_use]
pub fn code_execution_20250522() -> Value {
    json!({ "type": "code_execution_20250522", "name": "code_execution" })
}

/// Creates a code execution tool (version 2025-08-25) with Python and Bash support.
#[must_use]
pub fn code_execution_20250825() -> Value {
    json!({ "type": "code_execution_20250825", "name": "code_execution" })
}

/// Creates a code execution tool (version 2026-01-20), recommended version.
#[must_use]
pub fn code_execution_20260120() -> Value {
    json!({ "type": "code_execution_20260120", "name": "code_execution" })
}

/// Creates a computer use tool (version 2024-10-22).
#[must_use]
pub fn computer_20241022(
    display_width_px: u32,
    display_height_px: u32,
    display_number: Option<u32>,
) -> Value {
    let mut tool = json!({
        "type": "computer_20241022",
        "name": "computer",
        "display_width_px": display_width_px,
        "display_height_px": display_height_px,
    });
    if let Some(num) = display_number {
        tool["display_number"] = json!(num);
    }
    tool
}

/// Creates a computer use tool (version 2025-01-24).
#[must_use]
pub fn computer_20250124(
    display_width_px: u32,
    display_height_px: u32,
    display_number: Option<u32>,
) -> Value {
    let mut tool = json!({
        "type": "computer_20250124",
        "name": "computer",
        "display_width_px": display_width_px,
        "display_height_px": display_height_px,
    });
    if let Some(num) = display_number {
        tool["display_number"] = json!(num);
    }
    tool
}

/// Creates a computer use tool (version 2025-11-24) with zoom support.
#[must_use]
pub fn computer_20251124(
    display_width_px: u32,
    display_height_px: u32,
    display_number: Option<u32>,
    enable_zoom: Option<bool>,
) -> Value {
    let mut tool = json!({
        "type": "computer_20251124",
        "name": "computer",
        "display_width_px": display_width_px,
        "display_height_px": display_height_px,
    });
    if let Some(num) = display_number {
        tool["display_number"] = json!(num);
    }
    if let Some(zoom) = enable_zoom {
        tool["enable_zoom"] = json!(zoom);
    }
    tool
}

/// Creates a memory tool (version 2025-08-18) for persistent storage.
#[must_use]
pub fn memory_20250818() -> Value {
    json!({ "type": "memory_20250818", "name": "memory" })
}

/// Creates a text editor tool (version 2024-10-22).
#[must_use]
pub fn text_editor_20241022() -> Value {
    json!({ "type": "text_editor_20241022", "name": "str_replace_editor" })
}

/// Creates a text editor tool (version 2025-01-24).
#[must_use]
pub fn text_editor_20250124() -> Value {
    json!({ "type": "text_editor_20250124", "name": "str_replace_editor" })
}

/// Creates a text editor tool (version 2025-04-29).
#[must_use]
pub fn text_editor_20250429() -> Value {
    json!({ "type": "text_editor_20250429", "name": "str_replace_editor" })
}

/// Creates a text editor tool (version 2025-07-28) with optional `max_characters`.
#[must_use]
pub fn text_editor_20250728(max_characters: Option<u32>) -> Value {
    let mut tool = json!({
        "type": "text_editor_20250728",
        "name": "str_replace_editor",
    });
    if let Some(mc) = max_characters {
        tool["max_characters"] = json!(mc);
    }
    tool
}

/// Creates a web search tool (version 2025-03-05).
#[must_use]
pub fn web_search_20250305(
    max_uses: Option<u32>,
    allowed_domains: Option<Vec<String>>,
    blocked_domains: Option<Vec<String>>,
) -> Value {
    let mut tool = json!({ "type": "web_search_20250305", "name": "web_search" });
    if let Some(mu) = max_uses {
        tool["max_uses"] = json!(mu);
    }
    if let Some(ad) = allowed_domains {
        tool["allowed_domains"] = json!(ad);
    }
    if let Some(bd) = blocked_domains {
        tool["blocked_domains"] = json!(bd);
    }
    tool
}

/// Creates a web search tool (version 2026-02-09).
#[must_use]
pub fn web_search_20260209(
    max_uses: Option<u32>,
    allowed_domains: Option<Vec<String>>,
    blocked_domains: Option<Vec<String>>,
) -> Value {
    let mut tool = json!({ "type": "web_search_20260209", "name": "web_search" });
    if let Some(mu) = max_uses {
        tool["max_uses"] = json!(mu);
    }
    if let Some(ad) = allowed_domains {
        tool["allowed_domains"] = json!(ad);
    }
    if let Some(bd) = blocked_domains {
        tool["blocked_domains"] = json!(bd);
    }
    tool
}

/// Creates a web fetch tool (version 2025-09-10).
#[must_use]
pub fn web_fetch_20250910(
    max_uses: Option<u32>,
    allowed_domains: Option<Vec<String>>,
    blocked_domains: Option<Vec<String>>,
) -> Value {
    let mut tool = json!({ "type": "web_fetch_20250910", "name": "web_fetch" });
    if let Some(mu) = max_uses {
        tool["max_uses"] = json!(mu);
    }
    if let Some(ad) = allowed_domains {
        tool["allowed_domains"] = json!(ad);
    }
    if let Some(bd) = blocked_domains {
        tool["blocked_domains"] = json!(bd);
    }
    tool
}

/// Creates a web fetch tool (version 2026-02-09).
#[must_use]
pub fn web_fetch_20260209(
    max_uses: Option<u32>,
    allowed_domains: Option<Vec<String>>,
    blocked_domains: Option<Vec<String>>,
) -> Value {
    let mut tool = json!({ "type": "web_fetch_20260209", "name": "web_fetch" });
    if let Some(mu) = max_uses {
        tool["max_uses"] = json!(mu);
    }
    if let Some(ad) = allowed_domains {
        tool["allowed_domains"] = json!(ad);
    }
    if let Some(bd) = blocked_domains {
        tool["blocked_domains"] = json!(bd);
    }
    tool
}

/// Creates a tool search tool using regex (version 2025-11-19).
#[must_use]
pub fn tool_search_regex_20251119() -> Value {
    json!({ "type": "tool_search_regex_20251119", "name": "tool_search" })
}

/// Creates a tool search tool using BM25 (version 2025-11-19).
#[must_use]
pub fn tool_search_bm25_20251119() -> Value {
    json!({ "type": "tool_search_bm25_20251119", "name": "tool_search" })
}
