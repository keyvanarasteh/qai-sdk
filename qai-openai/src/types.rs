use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<OpenAIToolChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITool {
    #[serde(rename = "type")]
    pub tool_type: String, // "function"
    pub function: OpenAIFunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIToolChoice {
    String(String), // "none", "auto", "required"
    Object {
        #[serde(rename = "type")]
        tool_type: String,
        function: OpenAIFunctionName,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionName {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum OpenAIMessage {
    System {
        content: String,
    },
    User {
        content: Vec<OpenAIContent>,
    },
    Assistant {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<OpenAIToolCall>>,
    },
    Tool {
        content: String,
        tool_call_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIContent {
    Text { text: String },
    ImageUrl { image_url: OpenAIImageUrl },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIImageUrl {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String, // "function"
    pub function: OpenAIFunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChoice {
    pub message: OpenAIResponseMessage,
    pub finish_reason: Option<String>,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// --- Streaming Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamChunk {
    pub id: String,
    pub choices: Vec<OpenAIStreamChoice>,
    pub usage: Option<OpenAIUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamChoice {
    pub delta: OpenAIStreamDelta,
    pub finish_reason: Option<String>,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<OpenAIStreamToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamToolCall {
    pub index: u32,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub call_type: Option<String>,
    pub function: Option<OpenAIStreamFunctionCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamFunctionCall {
    pub name: Option<String>,
    pub arguments: Option<String>,
}
