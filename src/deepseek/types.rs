use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekRequest {
    pub model: String,
    pub messages: Vec<DeepSeekMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum DeepSeekMessage {
    System {
        content: String,
    },
    User {
        content: String,
    },
    Assistant {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reasoning_content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<DeepSeekToolCall>>,
    },
    Tool {
        content: String,
        tool_call_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String, // "function"
    pub function: DeepSeekFunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekResponse {
    pub id: String,
    pub choices: Vec<DeepSeekChoice>,
    pub usage: DeepSeekUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekChoice {
    pub message: DeepSeekResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekResponseMessage {
    pub content: Option<String>,
    pub reasoning_content: Option<String>,
    pub tool_calls: Option<Vec<DeepSeekToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

// --- DeepSeek-Specific Configuration ---

/// Known `DeepSeek` chat model IDs.
pub const DEEPSEEK_CHAT: &str = "deepseek-chat";
pub const DEEPSEEK_REASONER: &str = "deepseek-reasoner";

/// Thinking configuration for `DeepSeek` reasoning models.
/// When `thinking_type` is `enabled`, the model will produce reasoning content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekThinkingConfig {
    #[serde(rename = "type")]
    pub thinking_type: DeepSeekThinkingType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeepSeekThinkingType {
    Enabled,
    Disabled,
}

impl Default for DeepSeekThinkingConfig {
    fn default() -> Self {
        Self {
            thinking_type: DeepSeekThinkingType::Enabled,
        }
    }
}
