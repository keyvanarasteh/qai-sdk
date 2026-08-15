//! # GroqCloud Tool Use Types
//!
//! Types for GroqCloud's three-tier tool ecosystem:
//! - **Built-in tools**: `web_search`, `visit_website` (server-side execution on compound models)
//! - **Remote MCP**: Third-party MCP server integration via server-side orchestration
//! - **Local tool calling**: Standard function calling (uses the existing `ToolDefinition`)
//!
//! # Example
//! ```rust,ignore
//! use qai_sdk::groqcloud::tools::*;
//!
//! // Built-in web search tool
//! let tools = vec![
//!     GroqTool::builtin_web_search(None),
//!     GroqTool::builtin_visit_website(),
//! ];
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A Groq-specific tool definition that can represent built-in tools,
/// MCP server tools, or standard function tools.
///
/// These are serialized into the `tools[]` array of the Chat Completions API request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GroqTool {
    /// Standard function tool (same as OpenAI function calling).
    Function { function: GroqFunctionDefinition },
    /// Built-in web search tool — executed server-side by Groq's compound models.
    /// Requires `groq/compound` or `groq/compound-mini`.
    #[serde(rename = "web_search_preview")]
    WebSearch {
        /// Maximum number of search results to return (default: 5).
        #[serde(skip_serializing_if = "Option::is_none")]
        max_results: Option<u32>,
        /// Search context size: "low", "medium", "high" (default: "medium").
        #[serde(skip_serializing_if = "Option::is_none")]
        search_context_size: Option<String>,
        /// User location for localized search results.
        #[serde(skip_serializing_if = "Option::is_none")]
        user_location: Option<GroqUserLocation>,
    },
    /// Built-in visit-website tool — executed server-side by Groq's compound models.
    /// Requires `groq/compound` or `groq/compound-mini`.
    #[serde(rename = "visit_website_preview")]
    VisitWebsite {},
    /// Remote MCP server tool — Groq handles tool discovery + execution server-side.
    #[serde(rename = "mcp")]
    Mcp {
        /// Human-readable label for this MCP server.
        server_label: String,
        /// HTTPS URL of the MCP server.
        server_url: String,
        /// Authentication headers to pass to the MCP server.
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
        /// Description of what this MCP server provides.
        #[serde(skip_serializing_if = "Option::is_none")]
        server_description: Option<String>,
        /// Approval policy: "never" (auto-approve) or "always".
        #[serde(skip_serializing_if = "Option::is_none")]
        require_approval: Option<String>,
        /// Restrict to specific tool names from this server.
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_tools: Option<Vec<GroqMcpAllowedTool>>,
    },
}

/// A function definition within a Groq function tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqFunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// User location for web search localization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqUserLocation {
    /// The type of location (always "approximate").
    #[serde(rename = "type")]
    pub location_type: String,
    /// City name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// State/region name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// Country name (e.g., "US", "TR").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

/// An allowed tool reference for MCP tool filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqMcpAllowedTool {
    /// The tool name as published by the MCP server.
    pub name: String,
}

// --- Convenience Constructors ---

impl GroqTool {
    /// Creates a built-in web search tool with default settings.
    ///
    /// **Requires**: `groq/compound` or `groq/compound-mini` model.
    ///
    /// # Arguments
    /// * `max_results` - Maximum search results (default: 5 when None)
    #[must_use]
    pub fn builtin_web_search(max_results: Option<u32>) -> Self {
        GroqTool::WebSearch {
            max_results,
            search_context_size: None,
            user_location: None,
        }
    }

    /// Creates a built-in web search tool with full configuration.
    #[must_use]
    pub fn builtin_web_search_with_config(
        max_results: Option<u32>,
        search_context_size: Option<String>,
        user_location: Option<GroqUserLocation>,
    ) -> Self {
        GroqTool::WebSearch {
            max_results,
            search_context_size,
            user_location,
        }
    }

    /// Creates a built-in visit-website tool.
    ///
    /// **Requires**: `groq/compound` or `groq/compound-mini` model.
    #[must_use]
    pub fn builtin_visit_website() -> Self {
        GroqTool::VisitWebsite {}
    }

    /// Creates a remote MCP server tool definition.
    ///
    /// # Arguments
    /// * `label` - Human-readable name for the MCP server
    /// * `url` - HTTPS URL of the MCP server
    #[must_use]
    pub fn mcp(label: impl Into<String>, url: impl Into<String>) -> Self {
        GroqTool::Mcp {
            server_label: label.into(),
            server_url: url.into(),
            headers: None,
            server_description: None,
            require_approval: None,
            allowed_tools: None,
        }
    }

    /// Creates a remote MCP server tool with authentication headers.
    #[must_use]
    pub fn mcp_with_auth(
        label: impl Into<String>,
        url: impl Into<String>,
        headers: HashMap<String, String>,
    ) -> Self {
        GroqTool::Mcp {
            server_label: label.into(),
            server_url: url.into(),
            headers: Some(headers),
            server_description: None,
            require_approval: None,
            allowed_tools: None,
        }
    }

    /// Creates a standard function tool from a `ToolDefinition`.
    #[must_use]
    pub fn function(def: crate::core::types::ToolDefinition) -> Self {
        GroqTool::Function {
            function: GroqFunctionDefinition {
                name: def.name,
                description: def.description,
                parameters: def.parameters,
            },
        }
    }
}

impl GroqUserLocation {
    /// Creates a new user location.
    #[must_use]
    pub fn new(country: impl Into<String>) -> Self {
        Self {
            location_type: "approximate".to_string(),
            city: None,
            region: None,
            country: Some(country.into()),
        }
    }

    /// Creates a user location with full details.
    #[must_use]
    pub fn full(
        city: impl Into<String>,
        region: impl Into<String>,
        country: impl Into<String>,
    ) -> Self {
        Self {
            location_type: "approximate".to_string(),
            city: Some(city.into()),
            region: Some(region.into()),
            country: Some(country.into()),
        }
    }
}

/// Supported models for Groq built-in tools (compound models).
pub const COMPOUND_MODELS: &[&str] = &["groq/compound", "groq/compound-mini"];

/// Supported models for Groq local tool calling.
pub const TOOL_CALLING_MODELS: &[&str] = &[
    "llama-3.3-70b-versatile",
    "llama-3.1-8b-instant",
    "meta-llama/llama-4-scout-17b-16e-instruct",
    "qwen/qwen3-32b",
    "mistral-saba-24b",
];
