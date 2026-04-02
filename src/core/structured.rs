//! # Structured Output — `generate_object` / `stream_object`
//!
//! Forces a language model to return validated JSON conforming to a JSON Schema.
//! Mirrors the Vercel AI SDK's `generateObject` / `streamObject` pattern.
//!
//! **Output modes:**
//! - `Json` — uses native `response_format: { type: "json_schema" }` (OpenAI, Gemini)
//! - `Tool` — wraps the schema as a fake tool definition to force structured output
//! - `Auto` — defaults to `Json` mode
//!
//! **Features:**
//! - Schema validation with auto-retry on malformed output
//! - Streaming partial objects via `stream_object`

use crate::core::types::{
    Content, GenerateOptions, Message, Prompt, Role, StreamPart, ToolDefinition, Usage,
};
use crate::core::{LanguageModel, Result};
use futures::stream::BoxStream;
use futures_util::StreamExt;
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
    /// Maximum number of retries on schema validation failure (default: 2).
    pub max_retries: u32,
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

/// A streaming chunk from `stream_object`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ObjectStreamPart {
    /// A raw text delta from the model.
    TextDelta { delta: String },
    /// A partial JSON object successfully parsed from accumulated text so far.
    Partial { object: serde_json::Value },
    /// The final complete JSON object.
    Final {
        object: serde_json::Value,
        usage: Usage,
    },
    /// A streaming error.
    Error { message: String },
}

/// Generates a structured JSON object from a language model with schema
/// validation and auto-retry.
///
/// # Arguments
/// * `model` — Any `LanguageModel` implementation.
/// * `prompt_text` — The user prompt (text description of what to generate).
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
    let max_retries = options.max_retries;
    let mut last_error: Option<String> = None;

    for attempt in 0..=max_retries {
        let result = generate_object_once(model, prompt_text, &options, last_error.as_deref()).await;

        match result {
            Ok(gen_result) => {
                // Validate against JSON Schema
                match validate_schema(&gen_result.object, &options.schema) {
                    Ok(()) => return Ok(gen_result),
                    Err(validation_errors) => {
                        if attempt == max_retries {
                            return Err(crate::core::error::ProviderError::InvalidResponse(
                                format!(
                                    "Structured output failed schema validation after {} retries: {}",
                                    max_retries, validation_errors
                                ),
                            ));
                        }
                        last_error = Some(format!(
                            "Your JSON output did not match the required schema. Errors: {validation_errors}. Please fix and try again."
                        ));
                    }
                }
            }
            Err(e) => {
                if attempt == max_retries {
                    return Err(e);
                }
                last_error = Some(format!(
                    "Failed to produce valid JSON: {e}. Please respond with valid JSON only."
                ));
            }
        }
    }

    Err(crate::core::error::ProviderError::InvalidResponse(
        "Structured output generation exhausted all retries".to_string(),
    ))
}

/// Single attempt at generating an object (used by retry loop).
async fn generate_object_once(
    model: &dyn LanguageModel,
    prompt_text: &str,
    options: &ObjectGenerateOptions,
    retry_context: Option<&str>,
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

    // If retrying, add the error context as a follow-up message
    if let Some(context) = retry_context {
        messages.push(Message {
            role: Role::User,
            content: vec![Content::Text {
                text: context.to_string(),
            }],
        });
    }

    let prompt = Prompt { messages };

    match options.mode {
        OutputMode::Json => {
            let tool_name = options
                .schema_name
                .clone()
                .unwrap_or_else(|| "json_output".to_string());
            let tool_desc = options
                .schema_description
                .clone()
                .unwrap_or_else(|| "Generate a structured JSON object".to_string());

            let gen_options = GenerateOptions {
                model_id: options.model_id.clone(),
                max_tokens: options.max_tokens,
                temperature: options.temperature,
                top_p: None,
                stop_sequences: None,
                tools: None,
                response_format: Some(serde_json::json!({
                    "type": "json_schema",
                    "json_schema": {
                        "name": tool_name,
                        "description": tool_desc,
                        "schema": options.schema.clone(),
                        "strict": true
                    }
                })),
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
                .clone()
                .unwrap_or_else(|| "json_output".to_string());
            let tool_desc = options
                .schema_description
                .clone()
                .unwrap_or_else(|| "Generate a structured JSON object".to_string());

            let tool = ToolDefinition {
                name: tool_name.clone(),
                description: tool_desc,
                parameters: options.schema.clone(),
            };

            let gen_options = GenerateOptions {
                model_id: options.model_id.clone(),
                max_tokens: options.max_tokens,
                temperature: options.temperature,
                top_p: None,
                stop_sequences: None,
                tools: Some(vec![tool]),
                response_format: None,
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

/// Streams a structured JSON object from a language model.
///
/// Accumulates `TextDelta` chunks from the model's streaming response,
/// periodically attempts to parse partial JSON, and yields
/// `ObjectStreamPart::Partial` on each successful parse.
///
/// # Example
/// ```rust,ignore
/// use futures_util::StreamExt;
///
/// let mut stream = stream_object(
///     &model,
///     "Generate a user profile",
///     ObjectGenerateOptions {
///         model_id: "gpt-4o".to_string(),
///         schema: serde_json::json!({
///             "type": "object",
///             "properties": { "name": { "type": "string" } },
///             "required": ["name"]
///         }),
///         mode: OutputMode::Json,
///         ..Default::default()
///     },
/// ).await?;
///
/// while let Some(part) = stream.next().await {
///     match part {
///         ObjectStreamPart::Partial { object } => println!("Partial: {object}"),
///         ObjectStreamPart::Final { object, .. } => println!("Done: {object}"),
///         _ => {}
///     }
/// }
/// ```
pub async fn stream_object(
    model: &dyn LanguageModel,
    prompt_text: &str,
    options: ObjectGenerateOptions,
) -> Result<BoxStream<'static, ObjectStreamPart>> {
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

    let tool_name = options
        .schema_name
        .clone()
        .unwrap_or_else(|| "json_output".to_string());
    let tool_desc = options
        .schema_description
        .clone()
        .unwrap_or_else(|| "Generate a structured JSON object".to_string());

    let gen_options = match options.mode {
        OutputMode::Json => GenerateOptions {
            model_id: options.model_id.clone(),
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: None,
            stop_sequences: None,
            tools: None,
            response_format: Some(serde_json::json!({
                "type": "json_schema",
                "json_schema": {
                    "name": tool_name.clone(),
                    "description": tool_desc.clone(),
                    "schema": options.schema.clone(),
                    "strict": true
                }
            })),
        },
        OutputMode::Tool => {
            let tool = ToolDefinition {
                name: tool_name.clone(),
                description: tool_desc.clone(),
                parameters: options.schema.clone(),
            };
            GenerateOptions {
                model_id: options.model_id.clone(),
                max_tokens: options.max_tokens,
                temperature: options.temperature,
                top_p: None,
                stop_sequences: None,
                tools: Some(vec![tool]),
                response_format: None,
            }
        }
    };

    let mut inner_stream = model.generate_stream(prompt, gen_options).await?;

    let stream = async_stream::stream! {
        let mut accumulated = String::new();
        let mut last_usage = Usage { prompt_tokens: 0, completion_tokens: 0 };
        let mut chunk_count: u32 = 0;

        while let Some(part) = inner_stream.next().await {
            match part {
                StreamPart::TextDelta { delta } => {
                    if matches!(options.mode, OutputMode::Json) {
                        accumulated.push_str(&delta);
                        chunk_count += 1;
                        yield ObjectStreamPart::TextDelta { delta };

                        // Try partial parse every 5 chunks to avoid excessive parsing
                        if chunk_count.is_multiple_of(5) {
                            if let Ok(partial) = try_parse_partial_json(&accumulated) {
                                yield ObjectStreamPart::Partial { object: partial };
                            }
                        }
                    }
                }
                StreamPart::ToolCallDelta { arguments_delta, .. } => {
                    if matches!(options.mode, OutputMode::Tool) {
                        if let Some(delta) = arguments_delta {
                            accumulated.push_str(&delta);
                            chunk_count += 1;
                            yield ObjectStreamPart::TextDelta { delta: delta.clone() };

                            if chunk_count.is_multiple_of(5) {
                                if let Ok(partial) = try_parse_partial_json(&accumulated) {
                                    yield ObjectStreamPart::Partial { object: partial };
                                }
                            }
                        }
                    }
                }
                StreamPart::Usage { usage } => {
                    last_usage = usage;
                }
                StreamPart::Finish { .. } => {
                    // Final parse
                    match parse_json_from_text(&accumulated) {
                        Ok(object) => {
                            yield ObjectStreamPart::Final {
                                object,
                                usage: last_usage.clone(),
                            };
                        }
                        Err(e) => {
                            yield ObjectStreamPart::Error {
                                message: format!("Failed to parse final JSON: {e}"),
                            };
                        }
                    }
                }
                StreamPart::Error { message } => {
                    yield ObjectStreamPart::Error { message };
                }
            }
        }

        // If stream ended without a Finish event, try final parse
        if !accumulated.is_empty() {
            if let Ok(object) = parse_json_from_text(&accumulated) {
                yield ObjectStreamPart::Final {
                    object,
                    usage: last_usage,
                };
            }
        }
    };

    Ok(Box::pin(stream))
}

/// Validate a JSON value against a JSON Schema.
fn validate_schema(
    value: &serde_json::Value,
    schema: &serde_json::Value,
) -> std::result::Result<(), String> {
    let validator = jsonschema::validator_for(schema).map_err(|e| {
        format!("Invalid JSON Schema: {e}")
    })?;

    let errors: Vec<String> = validator
        .iter_errors(value)
        .map(|e| e.to_string())
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
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

/// Attempt to parse partial/incomplete JSON by closing open braces and brackets.
fn try_parse_partial_json(text: &str) -> std::result::Result<serde_json::Value, ()> {
    let trimmed = text.trim();

    // First try direct parse
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return Ok(v);
    }

    // Try closing open braces/brackets
    let mut open_braces: i32 = 0;
    let mut open_brackets: i32 = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for ch in trimmed.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }
        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => open_braces += 1,
            '}' if !in_string => open_braces -= 1,
            '[' if !in_string => open_brackets += 1,
            ']' if !in_string => open_brackets -= 1,
            _ => {}
        }
    }

    if open_braces <= 0 && open_brackets <= 0 {
        return Err(());
    }

    let mut patched = trimmed.to_string();

    // Remove trailing comma if present (common in partial JSON)
    if let Some(stripped) = patched.strip_suffix(',') {
        patched = stripped.to_string();
    }

    for _ in 0..open_brackets {
        patched.push(']');
    }
    for _ in 0..open_braces {
        patched.push('}');
    }

    serde_json::from_str::<serde_json::Value>(&patched).map_err(|_| ())
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
            max_retries: 2,
        }
    }
}
