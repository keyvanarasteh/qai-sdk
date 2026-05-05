//! # QAI Anthropic
//!
//! Anthropic Claude provider for the QAI SDK. Supports chat, streaming,
//! tool calling, vision (images), and PDF document input.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use qai_sdk::anthropic::create_anthropic;
//! use qai_sdk::core::types::ProviderSettings;
//!
//! let provider = create_anthropic(ProviderSettings {
//!     api_key: Some("sk-ant-...".to_string()),
//!     ..Default::default()
//! });
//!
//! let model = provider.chat("claude-sonnet-4-20250514");
//! ```

pub mod error;
#[cfg(test)]
mod tests;
pub mod tools;
pub mod types;

use crate::anthropic::types::{
    AnthropicContent, AnthropicImageSource, AnthropicMessage, AnthropicRequest, AnthropicResponse,
    AnthropicStreamEvent, AnthropicSystemContent, AnthropicThinkingConfig, AnthropicTool,
};
use crate::core::types::{
    Content, FileSource, GenerateOptions, GenerateResult, ImageSource, Prompt, Role, StreamPart,
    Usage,
};
use anyhow::anyhow;
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures::stream::BoxStream;
use futures_util::StreamExt;
use reqwest::Client;

pub struct AnthropicModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl AnthropicModel {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl crate::core::LanguageModel for AnthropicModel {
    #[tracing::instrument(skip(self, prompt), fields(model = options.model_id))]
    async fn generate(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> crate::core::Result<GenerateResult> {
        let (request, _) = self.prepare_request(prompt, options)?;

        let mut req_builder = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01");

        let mut has_beta_tools = false;
        if let Some(tools) = &request.tools {
            for tool in tools {
                match tool {
                    AnthropicTool::ComputerUse { .. } | AnthropicTool::BashOrTextEditor { .. } => {
                        has_beta_tools = true;
                        break;
                    }
                    _ => {}
                }
            }
        }
        if has_beta_tools {
            req_builder = req_builder.header("anthropic-beta", "computer-use-2024-10-22");
        }

        let response = req_builder.json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Anthropic API error: {error_text}").into());
        }

        let headers = response.headers().clone();
        let anthropic_response: AnthropicResponse = response.json().await?;

        let mut usage = Usage {
            prompt_tokens: anthropic_response.usage.input_tokens,
            completion_tokens: anthropic_response.usage.output_tokens,
            cache_hit_tokens: None,
            cache_miss_tokens: None,
        };

        // Header extraction as fallback/supplement
        if let Some(header_usage) = Usage::from_headers(&headers) {
            usage = header_usage;
        }

        let text = anthropic_response
            .content
            .iter()
            .filter_map(|c| {
                if let AnthropicContent::Text { text } = c {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect::<String>();

        // Extract thinking/reasoning content
        let reasoning_parts: Vec<String> = anthropic_response
            .content
            .iter()
            .filter_map(|c| {
                if let AnthropicContent::Thinking { thinking, .. } = c {
                    if !thinking.is_empty() {
                        Some(thinking.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        let reasoning = if reasoning_parts.is_empty() {
            None
        } else {
            Some(reasoning_parts.join("\n"))
        };

        // Extract native tool calls from ToolUse content blocks
        let tool_calls = anthropic_response
            .content
            .iter()
            .filter_map(|c| {
                if let AnthropicContent::ToolUse { id: _, name, input } = c {
                    Some(crate::core::types::ToolCallResult {
                        name: name.clone(),
                        arguments: input.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(GenerateResult {
            text,
            usage,
            finish_reason: anthropic_response
                .stop_reason
                .unwrap_or_else(|| "stop".to_string()),
            tool_calls,
            reasoning,
            executed_tools: Vec::new(),
        })
    }

    async fn generate_stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> crate::core::Result<BoxStream<'static, StreamPart>> {
        let (mut request, _) = self.prepare_request(prompt, options)?;
        request.stream = Some(true);

        let mut req_builder = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01");

        let mut has_beta_tools = false;
        if let Some(tools) = &request.tools {
            for tool in tools {
                match tool {
                    AnthropicTool::ComputerUse { .. } | AnthropicTool::BashOrTextEditor { .. } => {
                        has_beta_tools = true;
                        break;
                    }
                    _ => {}
                }
            }
        }
        if has_beta_tools {
            req_builder = req_builder.header("anthropic-beta", "computer-use-2024-10-22");
        }

        let response = req_builder.json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Anthropic API error: {error_text}").into());
        }

        let mut event_stream = response.bytes_stream().eventsource();
        let mut prompt_tokens = 0;

        let stream = async_stream::stream! {
            while let Some(event) = event_stream.next().await {
                match event {
                    Ok(event) => {
                        let parsed: Result<AnthropicStreamEvent, _> = serde_json::from_str(&event.data);
                        match parsed {
                            Ok(AnthropicStreamEvent::MessageStart { message }) => {
                                prompt_tokens = message.usage.input_tokens;
                                yield StreamPart::Usage { usage: Usage { prompt_tokens, completion_tokens: 0, cache_hit_tokens: None, cache_miss_tokens: None } };
                            }
                            Ok(AnthropicStreamEvent::ContentBlockDelta { delta, .. }) => {
                                match delta {
                                    types::AnthropicDelta::TextDelta { text } => {
                                        yield StreamPart::TextDelta { delta: text };
                                    }
                                    types::AnthropicDelta::InputJsonDelta { partial_json } => {
                                        yield StreamPart::ToolCallDelta {
                                            index: 0,
                                            id: None,
                                            name: None,
                                            arguments_delta: Some(partial_json)
                                        };
                                    }
                                    types::AnthropicDelta::ThinkingDelta { thinking } => {
                                        yield StreamPart::ReasoningDelta { delta: thinking };
                                    }
                                    types::AnthropicDelta::SignatureDelta { .. } => {
                                        // Signature deltas are internal; no user-facing stream part needed
                                    }
                                }
                            }
                            Ok(AnthropicStreamEvent::MessageDelta { delta, usage }) => {
                                yield StreamPart::Usage { usage: Usage { prompt_tokens, completion_tokens: usage.output_tokens, cache_hit_tokens: None, cache_miss_tokens: None } };
                                if let Some(reason) = delta.stop_reason {
                                    yield StreamPart::Finish { finish_reason: reason };
                                }
                            }
                            Ok(AnthropicStreamEvent::Error { error }) => {
                                yield StreamPart::Error { message: error.message };
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        yield StreamPart::Error { message: e.to_string() };
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

impl AnthropicModel {
    fn prepare_request(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> crate::core::Result<(AnthropicRequest, Vec<crate::core::types::ToolDefinition>)> {
        let mut system_content = Vec::new();
        let mut messages = Vec::new();

        for msg in prompt.messages {
            match msg.role {
                Role::System => {
                    for content in msg.content {
                        if let Content::Text { text } = content {
                            system_content.push(AnthropicSystemContent {
                                content_type: "text".to_string(),
                                text,
                            });
                        }
                    }
                }
                role => {
                    let mut anthropic_contents = Vec::new();
                    for content in msg.content {
                        match content {
                            Content::Text { text } => {
                                anthropic_contents.push(AnthropicContent::Text { text });
                            }
                            Content::Image { source } => {
                                if let ImageSource::Base64 { media_type, data } = source {
                                    anthropic_contents.push(AnthropicContent::Image {
                                        source: AnthropicImageSource {
                                            source_type: "base64".to_string(),
                                            media_type,
                                            data,
                                        },
                                    });
                                } else {
                                    return Err(
                                        anyhow!("Unsupported image source for Anthropic").into()
                                    );
                                }
                            }
                            Content::File { source } => {
                                let FileSource::Base64 { media_type, data } = source;
                                if media_type == "application/pdf" {
                                    anthropic_contents.push(AnthropicContent::Document {
                                        source: AnthropicImageSource {
                                            source_type: "base64".to_string(),
                                            media_type,
                                            data,
                                        },
                                    });
                                }
                            }
                            Content::ToolCall {
                                id,
                                name,
                                arguments,
                            } => {
                                anthropic_contents.push(AnthropicContent::ToolUse {
                                    id: id.clone(),
                                    name: name.clone(),
                                    input: arguments,
                                });
                            }
                            Content::ToolResult { id, result } => {
                                anthropic_contents.push(AnthropicContent::ToolResult {
                                    tool_use_id: id,
                                    content: result.to_string(),
                                    is_error: None,
                                });
                            }
                        }
                    }
                    messages.push(AnthropicMessage {
                        role: match role {
                            Role::User => "user".to_string(),
                            Role::Assistant => "assistant".to_string(),
                            _ => "user".to_string(),
                        },
                        content: anthropic_contents,
                    });
                }
            }
        }

        let anthropic_tools = if options.tools.as_ref().is_some_and(|t| !t.is_empty()) {
            Some(
                options
                    .tools
                    .unwrap()
                    .into_iter()
                    .map(|t| {
                        if t.name == "computer_20241022" {
                            let width = t.parameters.get("display_width_px").and_then(|v| v.as_u64()).unwrap_or(1024) as u32;
                            let height = t.parameters.get("display_height_px").and_then(|v| v.as_u64()).unwrap_or(768) as u32;
                            let display_number = t.parameters.get("display_number").and_then(|v| v.as_u64()).map(|v| v as u32);
                            AnthropicTool::ComputerUse {
                                tool_type: "computer_20241022".to_string(),
                                name: "computer".to_string(),
                                display_width_px: width,
                                display_height_px: height,
                                display_number,
                            }
                        } else if t.name == "bash_20241022" {
                            AnthropicTool::BashOrTextEditor {
                                tool_type: "bash_20241022".to_string(),
                                name: "bash".to_string(),
                            }
                        } else if t.name == "text_editor_20241022" {
                            AnthropicTool::BashOrTextEditor {
                                tool_type: "text_editor_20241022".to_string(),
                                name: "str_replace_editor".to_string(),
                            }
                        } else {
                            AnthropicTool::Custom {
                                name: t.name,
                                description: t.description,
                                input_schema: t.parameters,
                            }
                        }
                    })
                    .collect(),
            )
        } else {
            None
        };

        // Build thinking config from GenerateOptions
        let thinking = if options.reasoning_format.is_some()
            || options.reasoning_effort.is_some()
        {
            let effort = options.reasoning_effort.as_deref().unwrap_or("high");
            let effort_lower = effort.to_lowercase();

            // Determine thinking type and budget
            let (thinking_type, budget_tokens) = match effort_lower.as_str() {
                "off" | "none" | "disabled" => ("disabled".to_string(), None),
                "adaptive" | "auto" | "dynamic" => ("adaptive".to_string(), None),
                // Named effort levels → adaptive mode (Anthropic's effort parameter is separate)
                "low" | "medium" | "high" | "max" | "xhigh" | "minimal" => {
                    ("adaptive".to_string(), None)
                }
                _ => {
                    // Try to parse as integer for manual budget_tokens
                    if let Ok(budget) = effort.parse::<u32>() {
                        ("enabled".to_string(), Some(budget))
                    } else {
                        ("adaptive".to_string(), None)
                    }
                }
            };

            // Determine display mode from reasoning_format
            let display = match options.reasoning_format.as_deref() {
                Some("parsed" | "summarized") => Some("summarized".to_string()),
                Some("omitted" | "hidden") => Some("omitted".to_string()),
                Some("raw") => Some("summarized".to_string()),
                _ => Some("summarized".to_string()),
            };

            Some(AnthropicThinkingConfig {
                thinking_type,
                budget_tokens,
                display,
            })
        } else {
            None
        };

        let request = AnthropicRequest {
            model: options.model_id,
            messages,
            system: if system_content.is_empty() {
                None
            } else {
                Some(system_content)
            },
            max_tokens: options.max_tokens.unwrap_or(1024),
            temperature: if thinking.is_some() { None } else { options.temperature },
            top_p: if thinking.is_some() { None } else { options.top_p },
            top_k: None,
            stop_sequences: options.stop_sequences,
            stream: None,
            tools: anthropic_tools,
            tool_choice: None,
            thinking,
        };

        Ok((request, Vec::new())) // Tool list not used for return currently
    }
}

// --- Provider Factory ---

use crate::core::types::ProviderSettings;

/// Anthropic provider with configurable settings.
pub struct AnthropicProvider {
    settings: ProviderSettings,
}

impl AnthropicProvider {
    /// Creates a language model with the given model ID.
    #[must_use]
    pub fn language_model(&self, _model_id: &str) -> AnthropicModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .unwrap_or_default();
        let mut model = AnthropicModel::new(api_key);
        if let Some(ref base_url) = self.settings.base_url {
            model.base_url = base_url.clone();
        }
        model
    }

    /// Alias for `language_model`.
    #[must_use]
    pub fn chat(&self, model_id: &str) -> AnthropicModel {
        self.language_model(model_id)
    }
}

/// Create an Anthropic provider instance with the given settings.
#[must_use]
pub fn create_anthropic(settings: ProviderSettings) -> AnthropicProvider {
    AnthropicProvider { settings }
}

impl crate::core::registry::Provider for AnthropicProvider {
    fn language_model(&self, model_id: &str) -> Option<Box<dyn crate::core::LanguageModel>> {
        Some(Box::new(self.chat(model_id)))
    }
}
