//! # QAI Ollama Types
//!
//! Types for the native Ollama management API endpoints.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The default local host for an Ollama server.
pub const DEFAULT_OLLAMA_LOCAL_URL: &str = "http://localhost:11434/v1";

/// The default remote host for Ollama Cloud (when an API key is provided).
pub const DEFAULT_OLLAMA_CLOUD_URL: &str = "https://api.ollama.cloud/v1";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaModelDetails {
    pub parent_model: Option<String>,
    pub format: Option<String>,
    pub family: Option<String>,
    pub families: Option<Vec<String>>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaModelInfo {
    pub name: String,
    pub model: String,
    pub modified_at: Option<String>,
    pub size: Option<u64>,
    pub digest: Option<String>,
    pub details: Option<OllamaModelDetails>,
    // Fields specific to the /api/ps endpoint
    pub expires_at: Option<String>,
    pub size_vram: Option<u64>,
    pub context_length: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaListResponse {
    pub models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaPsResponse {
    pub models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaShowRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbose: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaShowResponse {
    pub modelfile: Option<String>,
    pub parameters: Option<String>,
    pub template: Option<String>,
    pub details: Option<OllamaModelDetails>,
    pub model_info: Option<HashMap<String, serde_json::Value>>,
    pub modified_at: Option<String>,
    pub license: Option<String>,
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaCopyRequest {
    pub source: String,
    pub destination: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaDeleteRequest {
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaPullRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaPullResponse {
    pub status: String,
    pub digest: Option<String>,
    pub total: Option<u64>,
    pub completed: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaPushRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaCreateRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modelfile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaVersionResponse {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebSearchRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebSearchResponse {
    pub results: Vec<WebSearchResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebFetchRequest {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebFetchResponse {
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub links: Vec<String>,
}
