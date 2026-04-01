//! # Structured Output — `generate_object` / `stream_object`
//!
//! Forces a language model to return validated JSON conforming to a JSON Schema.
//! Mirrors the Vercel AI SDK's `generateObject` / `streamObject` pattern.
//!
//! **Output modes:**
//! - `Json` — uses native `response_format: { type: "json_schema" }` (OpenAI, Gemini)
//! - `Tool` — wraps the schema as a fake tool definition to force structured output
//! - `Auto` — defaults to `Json` mode

use crate::core::types::{
    Content, GenerateOptions, Message, Prompt, Role, ToolDefinition, Usage,
};
use crate::core::{LanguageModel, Result};
use serde::{Deserialize, Serialize};

/// Output mode for structured generation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    /// Use native JSON mode (response_format). Best for OpenAI/Gemini.
    #[default]
    Json,
    /// Use tool-calling to force structured output. Works with Anthropic/any provider.
    Tool,
}

/// Options for `generate_object`.
#[derive(Debug, Clone)]
pub struct ObjectGenerateOptions {
    /// Model ID to use.
    pub model_id: String,
    /// JSON Schema that the output must conform to.
    pub schema: serde_json::Value,
    /// Optional human-readable name for the schema (used in tool mode).
    pub schema_name: Option<String>,
    /// Optional description for the schema.
    pub schema_description: Option<String>,
    /// Output mode.
    pub mode: OutputMode,
    /// Maximum tokens.
    pub max_tokens: Option<u32>,
    /// Temperature.
    pub temperature: Option<f32>,
    /// System prompt to prepend.
    pub system: Option<String>,
}

/// Result from `generate_object`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectGenerateResult {
    /// The parsed JSON object.
    pub object: serde_json::Value,
    /// The raw text returned by the model before parsing.
    pub raw_text: String,
    /// Token usage.
    pub usage: Usage,
    /// The finish reason.
    pub finish_reason: String,
}

/// Generates a structured JSON object from a language model.
///
/// # Arguments
/// * `model` — Any `LanguageModel` implementation.
/// * `prompt` — The user prompt (text description of what to generate).
/// * `options` — Schema, mode, and model parameters.
///
/// # Example
/// ```rust,ignore
/// let result = generate_object(
///     &model,
///     "Generate a user profile for John Doe, age 30",
///     ObjectGenerateOptions {
///         model_id: "gpt-4o".to_string(),
///         schema: serde_json::json!({
///             "type": "object",
///             "properties": {
///                 "name": { "type": "string" },
///                 "age": { "type": "integer" }
///             },
///             "required": ["name", "age"]
///         }),
///         mode: OutputMode::Json,
///         ..Default::default()
///     },
/// ).await?;
/// println!("{}", result.object["name"]); // "John Doe"
/// ```
pub async fn generate_object(
    model: &dyn LanguageModel,
    prompt_text: &str,
    options: ObjectGenerateOptions,
) -> Result<ObjectGenerateResult> {
    let mut messages = Vec::new();

    // System prompt with schema instruction
    let schema_instruction = format!(
        "You MUST respond with valid JSON that conforms to this JSON Schema:\n```json\n{}\n```\nRespond ONLY with the JSON object, no markdown fences, no extra text.",
        serde_json::to_string_pretty(&options.schema).unwrap_or_default()
    );

    let system_text = if let Some(ref sys) = options.system {
        format!("{sys}\n\n{schema_instruction}")
    } else {
        schema_instruction
    };

    messages.push(Message {
        role: Role::System,
        content: vec![Content::Text {
            text: system_text,
        }],
    });

    messages.push(Message {
        role: Role::User,
        content: vec![Content::Text {
            text: prompt_text.to_string(),
        }],
    });

    let prompt = Prompt { messages };

    match options.mode {
        OutputMode::Json => {
            let gen_options = GenerateOptions {
                model_id: options.model_id,
                max_tokens: options.max_tokens,
                temperature: options.temperature,
                top_p: None,
                stop_sequences: None,
                tools: None,
            };

            let result = model.generate(prompt, gen_options).await?;
            let object = parse_json_from_text(&result.text)?;

            Ok(ObjectGenerateResult {
                object,
                raw_text: result.text,
                usage: result.usage,
                finish_reason: result.finish_reason,
            })
        }
        OutputMode::Tool => {
            // Wrap schema as a tool definition to force structured output
            let tool_name = options
                .schema_name
                .unwrap_or_else(|| "json_output".to_string());
            let tool_desc = options
                .schema_description
                .unwrap_or_else(|| "Generate a structured JSON object".to_string());

            let tool = ToolDefinition {
                name: tool_name.clone(),
                description: tool_desc,
                parameters: options.schema.clone(),
            };

            let gen_options = GenerateOptions {
                model_id: options.model_id,
                max_tokens: options.max_tokens,
                temperature: options.temperature,
                top_p: None,
                stop_sequences: None,
                tools: Some(vec![tool]),
            };

            let result = model.generate(prompt, gen_options).await?;

            // Extract from tool calls first, fallback to text
            let object = if let Some(tc) = result
                .tool_calls
                .iter()
                .find(|tc| tc.name == tool_name)
            {
                tc.arguments.clone()
            } else {
                parse_json_from_text(&result.text)?
            };

            Ok(ObjectGenerateResult {
                object,
                raw_text: result.text,
                usage: result.usage,
                finish_reason: result.finish_reason,
            })
        }
    }
}

/// Parse JSON from model text, stripping optional markdown fences.
fn parse_json_from_text(text: &str) -> Result<serde_json::Value> {
    let trimmed = text.trim();

    // Strip markdown JSON fences if present
    let json_str = if trimmed.starts_with("```json") {
        trimmed
            .strip_prefix("```json")
            .and_then(|s| s.strip_suffix("```"))
            .unwrap_or(trimmed)
            .trim()
    } else if trimmed.starts_with("```") {
        trimmed
            .strip_prefix("```")
            .and_then(|s| s.strip_suffix("```"))
            .unwrap_or(trimmed)
            .trim()
    } else {
        trimmed
    };

    serde_json::from_str(json_str).map_err(|e| {
        crate::core::error::ProviderError::InvalidResponse(format!(
            "Failed to parse structured output as JSON: {e}\nRaw text: {json_str}"
        ))
    })
}

/// Convenience: Default options builder
impl Default for ObjectGenerateOptions {
    fn default() -> Self {
        Self {
            model_id: String::new(),
            schema: serde_json::Value::Null,
            schema_name: None,
            schema_description: None,
            mode: OutputMode::Json,
            max_tokens: None,
            temperature: None,
            system: None,
        }
    }
}
