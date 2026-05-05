use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub messages: Vec<Message>,
}

impl Prompt {
    #[must_use]
    pub fn new(messages: Vec<Message>) -> Self {
        Self { messages }
    }

    /// Creates a prompt from a single user message.
    #[must_use]
    pub fn from_user(text: impl Into<String>) -> Self {
        Self {
            messages: vec![Message {
                role: Role::User,
                content: vec![Content::Text {
                    text: text.into(),
                }],
            }],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<Content>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Content {
    Text {
        text: String,
    },
    Image {
        source: ImageSource,
    },
    /// Spatial reasoning data (e.g. bounding boxes for robotics/vision).
    Spatial {
        boxes: Vec<BoundingBox>,
    },
    File {
        source: FileSource,
    },
    ToolCall {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    ToolResult {
        id: String,
        result: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FileSource {
    Base64 { media_type: String, data: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ImageSource {
    Base64 { media_type: String, data: String },
    Url { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerateOptions {
    pub model_id: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub response_format: Option<serde_json::Value>,
    /// Specifies the format of the reasoning output (e.g., "raw", "parsed", "hidden").
    pub reasoning_format: Option<String>,
    /// Specifies the level of effort for reasoning (e.g., "low", "medium", "high").
    pub reasoning_effort: Option<String>,
    /// Controls which (if any) tool the model should call.
    /// Accepts `"auto"`, `"required"`, `"none"`, or `{"type":"function","function":{"name":"..."}}`.
    pub tool_choice: Option<serde_json::Value>,
    /// When true, enables parallel tool call execution by the model.
    pub parallel_tool_calls: Option<bool>,
    /// Extra HTTP headers to include in the API request.
    /// Useful for provider-specific features like xAI's `x-grok-conv-id` for prompt caching
    /// or custom proxy headers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_headers: Option<std::collections::HashMap<String, String>>,
    /// Provider-native tools that are executed on the server side (e.g., Code Interpreter, Web Search).
    pub server_tools: Option<Vec<ServerTool>>,
    /// When true, the model will include citations in its response if available.
    pub include_citations: Option<bool>,
    /// When true, enables real-time streaming of tool outputs (for observability).
    pub include_tool_outputs: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// A tool call returned by the model (native function calling).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResult {
    pub text: String,
    pub usage: Usage,
    pub finish_reason: String,
    /// Native tool calls returned by the model (e.g., Gemini `FunctionCall`, `OpenAI` `tool_calls`).
    /// Empty if the model returned only text.
    #[serde(default)]
    pub tool_calls: Vec<ToolCallResult>,
    /// Intermediate reasoning (e.g., "thinking") produced by the model before its final text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    /// Metadata about tools that were executed server-side (e.g., built-in web search, MCP calls).
    /// Empty for standard local tool calling.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub executed_tools: Vec<ExecutedTool>,
    /// Source citations provided by the model for its claims.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<Citation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// The source name or identifier (e.g., "Wikipedia", "example.com").
    pub source: String,
    /// The specific text snippet from the source, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
    /// The 1-based index of the citation as it appears in the text.
    pub index: u32,
    /// The URI or URL to the source, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

/// Metadata about a tool that was executed server-side by the provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutedTool {
    /// The tool name (e.g., "web_search", "visit_website", MCP tool name).
    pub name: String,
    /// The type of tool execution ("web_search", "visit_website", "mcp_call").
    pub tool_type: String,
    /// The arguments passed to the tool, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
    /// The output/result of the tool execution, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    /// The server/label that executed this tool (for MCP tools).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_hit_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_miss_tokens: Option<u32>,
}

impl Usage {
    #[must_use]
    pub fn from_headers(headers: &reqwest::header::HeaderMap) -> Option<Self> {
        let mut prompt_tokens = None;
        let mut completion_tokens = None;

        // Common Header Names (OpenAI, Anthropic, and various proxies)
        let prompt_header_keys = [
            "x-openai-usage-prompt-tokens",
            "x-anthropic-usage-input-tokens",
            "x-usage-prompt-tokens",
            "usage-prompt-tokens",
            "x-proxy-prompt-tokens",
        ];

        let completion_header_keys = [
            "x-openai-usage-completion-tokens",
            "x-anthropic-usage-output-tokens",
            "x-usage-completion-tokens",
            "usage-completion-tokens",
            "x-proxy-completion-tokens",
        ];

        for key in prompt_header_keys {
            if let Some(val) = headers
                .get(key)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u32>().ok())
            {
                prompt_tokens = Some(val);
                break;
            }
        }

        for key in completion_header_keys {
            if let Some(val) = headers
                .get(key)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u32>().ok())
            {
                completion_tokens = Some(val);
                break;
            }
        }

        // Check for composite JSON header (e.g., anthropic-usage)
        if prompt_tokens.is_none() || completion_tokens.is_none() {
            if let Some(val) = headers
                .get("anthropic-usage")
                .or_else(|| headers.get("x-ai-usage"))
                .and_then(|v| v.to_str().ok())
            {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(val) {
                    if let Some(p) = json
                        .get("input_tokens")
                        .or_else(|| json.get("prompt_tokens"))
                        .and_then(serde_json::Value::as_u64)
                    {
                        prompt_tokens = Some(p as u32);
                    }
                    if let Some(c) = json
                        .get("output_tokens")
                        .or_else(|| json.get("completion_tokens"))
                        .and_then(serde_json::Value::as_u64)
                    {
                        completion_tokens = Some(c as u32);
                    }
                }
            }
        }

        if let (Some(p), Some(c)) = (prompt_tokens, completion_tokens) {
            Some(Usage {
                prompt_tokens: p,
                completion_tokens: c,
                cache_hit_tokens: None,
                cache_miss_tokens: None,
            })
        } else {
            None
        }
    }
}

// --- Streaming Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StreamPart {
    TextDelta {
        delta: String,
    },
    ReasoningDelta {
        delta: String,
    },
    ToolCallDelta {
        index: u32,
        id: Option<String>,
        name: Option<String>,
        arguments_delta: Option<String>,
    },
    Usage {
        usage: Usage,
    },
    Finish {
        finish_reason: String,
    },
    Error {
        message: String,
    },
    ExecutedTool {
        tool: ExecutedTool,
    },
    Citation {
        citation: Citation,
    },
}

// --- Provider Settings ---

/// Common provider settings for configuring API access.
#[derive(Debug, Clone, Default)]
pub struct ProviderSettings {
    /// Base URL for API calls (overrides default).
    pub base_url: Option<String>,
    /// API key for authentication.
    pub api_key: Option<String>,
    /// Custom headers to include in requests.
    pub headers: Option<std::collections::HashMap<String, String>>,
}

// --- Embedding Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingOptions {
    pub model_id: String,
    pub dimensions: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResult {
    pub embeddings: Vec<Vec<f32>>,
    pub usage: Option<EmbeddingUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    pub prompt_tokens: u32,
    pub total_tokens: Option<u32>,
}

// --- Image Generation Types ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageGenerateOptions {
    pub model_id: String,
    pub prompt: String,
    pub n: Option<u32>,
    pub size: Option<String>,
    pub quality: Option<String>,
    pub response_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerateResult {
    /// Base64-encoded images or URLs, depending on `response_format`.
    pub images: Vec<String>,
    pub revised_prompt: Option<String>,
}

// --- Completion Types ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletionOptions {
    pub model_id: String,
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub suffix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResult {
    pub text: String,
    pub usage: Usage,
    pub finish_reason: String,
}

// --- Speech Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechOptions {
    pub model_id: String,
    pub input: String,
    pub voice: String,
    pub response_format: Option<String>,
    pub speed: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechResult {
    /// Raw audio bytes.
    pub audio: Vec<u8>,
}

// --- Transcription Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionOptions {
    pub model_id: String,
    /// Raw audio bytes to transcribe.
    pub audio: Vec<u8>,
    pub language: Option<String>,
    pub prompt: Option<String>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
    pub duration: Option<f64>,
}

// --- Video Types ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VideoGenerateOptions {
    pub model_id: String,
    pub prompt: String,
    /// Optional negative prompt.
    pub negative_prompt: Option<String>,
    /// Number of frames or duration in seconds.
    pub duration: Option<f32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// Seed for reproducible generation.
    pub seed: Option<u64>,
    pub n: Option<u32>,
    pub size: Option<String>,
    pub fps: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoGenerateResult {
    /// URL of the generated video.
    pub url: Option<String>,
    /// Raw video bytes (if available).
    pub data: Option<Vec<u8>>,
    /// Revision or internal ID of the generation.
    pub revision: Option<String>,
}

/// Options for generating music.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MusicGenerateOptions {
    pub model_id: String,
    pub prompt: String,
    pub n: Option<u32>,
    pub duration: Option<u32>,
}

/// Result of music generation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MusicGenerateResult {
    /// List of base64-encoded audio bytes.
    pub audio: Vec<String>,
}

// --- Realtime Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RealtimeEvent {
    Text { text: String },
    Audio { data: Vec<u8> },
    ToolCall { id: String, name: String, arguments: serde_json::Value },
    Error { message: String },
    SessionStarted,
    SessionEnded,
}

// --- Server-Defined Tool Types ---

/// A server-defined tool that can be passed to a provider alongside user-defined tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTool {
    /// The tool type as the provider expects it (e.g. "`computer_20241022`", "`code_interpreter`").
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Provider-specific configuration for this tool, serialized as JSON.
    #[serde(flatten)]
    pub config: serde_json::Value,
}

/// A bounding box for spatial reasoning.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BoundingBox {
    pub y_min: f32,
    pub x_min: f32,
    pub y_max: f32,
    pub x_max: f32,
    pub label: Option<String>,
}
