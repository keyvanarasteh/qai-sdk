//! OpenAI-Compatible-specific types and model IDs.

use serde::{Deserialize, Serialize};

/// OpenAI-compatible chat model ID (any string, since providers vary).
pub type OpenAICompatibleChatModelId = String;

/// OpenAI-compatible image model ID (any string, since providers vary).
pub type OpenAICompatibleImageModelId = String;

/// OpenAI-compatible embedding model ID (any string, since providers vary).
pub type OpenAICompatibleEmbeddingModelId = String;

/// OpenAI-compatible completion model ID (any string, since providers vary).
pub type OpenAICompatibleCompletionModelId = String;

/// Configuration for an OpenAI-compatible provider instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICompatibleConfig {
    /// Base URL for the API endpoint.
    pub base_url: String,
    /// Provider name identifier.
    pub name: String,
    /// API key for authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}
