//! OpenAI Responses API types.
//!
//! This module defines the comprehensive type system for the OpenAI Responses API,
//! which is the newer alternative to Chat Completions. It supports reasoning models,
//! tool execution (shell, apply_patch, code_interpreter, web_search, MCP), and
//! multi-turn conversations with `previous_response_id`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================================
// Input Types
// ============================================================================

/// A single input item in a Responses API request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponsesInputItem {
    /// System/developer message.
    #[serde(rename = "message")]
    Message {
        role: ResponsesRole,
        content: ResponsesMessageContent,
    },
    /// A function call from the model.
    FunctionCall {
        call_id: String,
        name: String,
        arguments: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    /// Output from a function call.
    FunctionCallOutput {
        call_id: String,
        output: Value,
    },
    /// Reasoning content.
    Reasoning {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        encrypted_content: Option<String>,
        summary: Vec<ResponsesReasoningSummary>,
    },
    /// Reference to a previous item by ID.
    ItemReference {
        id: String,
    },
    /// Custom tool call.
    CustomToolCall {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        call_id: String,
        name: String,
        input: String,
    },
    /// Custom tool call output.
    CustomToolCallOutput {
        call_id: String,
        output: Value,
    },
    /// MCP approval response.
    McpApprovalResponse {
        approval_request_id: String,
        approve: bool,
    },
    /// Local shell call.
    LocalShellCall {
        id: String,
        call_id: String,
        action: LocalShellAction,
    },
    /// Local shell call output.
    LocalShellCallOutput {
        call_id: String,
        output: String,
    },
    /// Shell call.
    ShellCall {
        id: String,
        call_id: String,
        status: String,
        action: ShellAction,
    },
    /// Shell call output.
    ShellCallOutput {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        call_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        output: Vec<ShellOutputEntry>,
    },
    /// Apply patch call.
    ApplyPatchCall {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        call_id: String,
        status: String,
        operation: ApplyPatchOperation,
    },
    /// Apply patch call output.
    ApplyPatchCallOutput {
        call_id: String,
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<String>,
    },
    /// Tool search call.
    ToolSearchCall {
        id: String,
        execution: String,
        call_id: Option<String>,
        status: String,
        arguments: Value,
    },
    /// Tool search output.
    ToolSearchOutput {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        execution: String,
        call_id: Option<String>,
        status: String,
        tools: Vec<Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponsesRole {
    System,
    Developer,
    User,
    Assistant,
}

/// Message content can be a string or structured parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponsesMessageContent {
    Text(String),
    Parts(Vec<ResponsesContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponsesContentPart {
    InputText { text: String },
    InputImage { image_url: String },
    InputFile { file_url: Option<String>, #[serde(skip_serializing_if = "Option::is_none")] file_id: Option<String> },
    OutputText { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsesReasoningSummary {
    #[serde(rename = "type")]
    pub summary_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalShellAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub command: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellAction {
    pub commands: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_length: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellOutputEntry {
    pub stdout: String,
    pub stderr: String,
    pub outcome: ShellOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ShellOutcome {
    Exit { exit_code: i32 },
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApplyPatchOperation {
    CreateFile { path: String, diff: String },
    DeleteFile { path: String },
    UpdateFile { path: String, diff: String },
}

// ============================================================================
// Tool Types
// ============================================================================

/// Tool that can be passed to the Responses API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponsesTool {
    Function {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        parameters: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
    WebSearch {
        #[serde(skip_serializing_if = "Option::is_none")]
        search_context_size: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_location: Option<Value>,
    },
    #[serde(rename = "web_search_preview")]
    WebSearchPreview {
        #[serde(skip_serializing_if = "Option::is_none")]
        search_context_size: Option<String>,
    },
    CodeInterpreter {
        #[serde(skip_serializing_if = "Option::is_none")]
        container: Option<Value>,
    },
    FileSearch {
        vector_store_ids: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_num_results: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filters: Option<Value>,
    },
    ImageGeneration {
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        quality: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        background: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        output_format: Option<String>,
    },
    Shell {
        #[serde(skip_serializing_if = "Option::is_none")]
        environment: Option<Value>,
    },
    LocalShell,
    ApplyPatch,
    Mcp {
        server_label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        server_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_tools: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        require_approval: Option<Value>,
    },
    Custom {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<Value>,
    },
    ToolSearch {
        #[serde(skip_serializing_if = "Option::is_none")]
        execution: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

// ============================================================================
// Request
// ============================================================================

/// Full request body for the Responses API.
#[derive(Debug, Clone, Serialize)]
pub struct ResponsesRequest {
    pub model: String,
    pub input: Vec<ResponsesInputItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ResponsesTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ResponsesReasoningConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsesReasoningConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

// ============================================================================
// Output Types
// ============================================================================

/// Full response from the Responses API (non-streaming).
#[derive(Debug, Clone, Deserialize)]
pub struct ResponsesResponse {
    pub id: String,
    #[serde(default)]
    pub output: Vec<ResponsesOutputItem>,
    #[serde(default)]
    pub usage: Option<ResponsesUsage>,
    #[serde(default)]
    pub error: Option<ResponsesError>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponsesError {
    pub message: String,
    #[serde(default)]
    pub code: Option<String>,
}

/// An output item from the Responses API.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponsesOutputItem {
    /// Text message from the model.
    Message {
        id: String,
        content: Vec<ResponsesOutputContent>,
        #[serde(default)]
        phase: Option<String>,
    },
    /// Function call from the model.
    FunctionCall {
        id: String,
        call_id: String,
        name: String,
        arguments: String,
    },
    /// Reasoning output.
    Reasoning {
        id: String,
        summary: Vec<ResponsesReasoningSummary>,
        #[serde(default)]
        encrypted_content: Option<String>,
    },
    /// Web search call.
    WebSearchCall {
        id: String,
        status: String,
        #[serde(default)]
        action: Option<Value>,
    },
    /// Code interpreter call.
    CodeInterpreterCall {
        id: String,
        #[serde(default)]
        code: Option<String>,
        #[serde(default)]
        container_id: Option<String>,
        #[serde(default)]
        outputs: Option<Vec<Value>>,
    },
    /// Image generation call.
    ImageGenerationCall {
        id: String,
        #[serde(default)]
        result: Option<String>,
    },
    /// File search call.
    FileSearchCall {
        id: String,
        #[serde(default)]
        queries: Option<Vec<String>>,
        #[serde(default)]
        results: Option<Vec<Value>>,
    },
    /// Shell call.
    ShellCall {
        id: String,
        call_id: String,
        status: String,
        action: ShellAction,
    },
    /// Shell call output.
    ShellCallOutput {
        #[serde(default)]
        id: Option<String>,
        call_id: String,
        output: Vec<ShellOutputEntry>,
    },
    /// Apply patch call.
    ApplyPatchCall {
        id: String,
        call_id: String,
        status: String,
        operation: ApplyPatchOperation,
    },
    /// Local shell call.
    LocalShellCall {
        id: String,
        call_id: String,
        action: LocalShellAction,
    },
    /// MCP call.
    McpCall {
        id: String,
        status: String,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        server_label: Option<String>,
        #[serde(default)]
        arguments: Option<String>,
        #[serde(default)]
        output: Option<String>,
        #[serde(default)]
        error: Option<String>,
    },
    /// MCP approval request.
    McpApprovalRequest {
        id: String,
    },
    /// Custom tool call.
    CustomToolCall {
        id: String,
        call_id: String,
        name: String,
        input: String,
    },
    /// Tool search call.
    ToolSearchCall {
        id: String,
        execution: String,
        #[serde(default)]
        call_id: Option<String>,
        status: String,
        #[serde(default)]
        arguments: Option<Value>,
    },
    /// Tool search output.
    ToolSearchOutput {
        #[serde(default)]
        id: Option<String>,
        execution: String,
        #[serde(default)]
        call_id: Option<String>,
        status: String,
        tools: Vec<Value>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponsesOutputContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
    #[serde(default)]
    pub annotations: Vec<Value>,
    #[serde(default)]
    pub logprobs: Option<Vec<ResponsesLogprob>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponsesLogprob {
    pub token: String,
    pub logprob: f64,
    #[serde(default)]
    pub top_logprobs: Vec<ResponsesTopLogprob>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponsesTopLogprob {
    pub token: String,
    pub logprob: f64,
}

// ============================================================================
// Usage
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ResponsesUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub input_tokens_details: Option<ResponsesTokenDetails>,
    #[serde(default)]
    pub output_tokens_details: Option<ResponsesOutputTokenDetails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponsesTokenDetails {
    #[serde(default)]
    pub cached_tokens: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponsesOutputTokenDetails {
    #[serde(default)]
    pub reasoning_tokens: Option<u32>,
}

// ============================================================================
// Streaming Events
// ============================================================================

/// SSE events for Responses API streaming.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponsesStreamEvent {
    /// Response created.
    #[serde(rename = "response.created")]
    ResponseCreated {
        response: ResponsesStreamCreatedData,
    },
    /// Response completed.
    #[serde(rename = "response.completed")]
    ResponseCompleted {
        response: ResponsesStreamCompletedData,
    },
    /// Response incomplete.
    #[serde(rename = "response.incomplete")]
    ResponseIncomplete {
        response: ResponsesStreamCompletedData,
    },
    /// Response failed.
    #[serde(rename = "response.failed")]
    ResponseFailed {
        response: ResponsesStreamFailedData,
    },
    /// Output item added.
    #[serde(rename = "response.output_item.added")]
    OutputItemAdded {
        output_index: u32,
        item: Value,
    },
    /// Output item done.
    #[serde(rename = "response.output_item.done")]
    OutputItemDone {
        output_index: u32,
        item: Value,
    },
    /// Text delta.
    #[serde(rename = "response.output_text.delta")]
    OutputTextDelta {
        item_id: String,
        delta: String,
        #[serde(default)]
        logprobs: Option<Vec<ResponsesLogprob>>,
    },
    /// Text done.
    #[serde(rename = "response.output_text.done")]
    OutputTextDone {
        item_id: String,
        text: String,
    },
    /// Function call arguments delta.
    #[serde(rename = "response.function_call_arguments.delta")]
    FunctionCallArgumentsDelta {
        item_id: String,
        call_id: String,
        delta: String,
    },
    /// Function call arguments done.
    #[serde(rename = "response.function_call_arguments.done")]
    FunctionCallArgumentsDone {
        item_id: String,
        call_id: String,
        arguments: String,
    },
    /// Reasoning summary text delta.
    #[serde(rename = "response.reasoning_summary_text.delta")]
    ReasoningSummaryTextDelta {
        item_id: String,
        delta: String,
    },
    /// Reasoning summary text done.
    #[serde(rename = "response.reasoning_summary_text.done")]
    ReasoningSummaryTextDone {
        item_id: String,
        text: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponsesStreamCreatedData {
    pub id: String,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponsesStreamCompletedData {
    #[serde(default)]
    pub usage: Option<ResponsesUsage>,
    #[serde(default)]
    pub service_tier: Option<String>,
    #[serde(default)]
    pub incomplete_details: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponsesStreamFailedData {
    #[serde(default)]
    pub error: Option<ResponsesError>,
    #[serde(default)]
    pub usage: Option<ResponsesUsage>,
}
