pub mod types;
#[cfg(test)]
mod tests;

use async_trait::async_trait;
use qai_core::types::{Content, GenerateOptions, GenerateResult, Prompt, Role, Usage, ImageSource, FileSource, StreamPart};
use crate::types::{AnthropicRequest, AnthropicResponse, AnthropicMessage, AnthropicContent, AnthropicSystemContent, AnthropicImageSource, AnthropicStreamEvent, AnthropicTool};
use anyhow::{Result, anyhow};
use reqwest::Client;
use futures::stream::BoxStream;
use futures_util::StreamExt;
use eventsource_stream::Eventsource;

pub struct AnthropicModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl AnthropicModel {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl qai_core::LanguageModel for AnthropicModel {
    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> Result<GenerateResult> {
        let (request, _) = self.prepare_request(prompt, options)?;

        let response = self.client.post(&format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Anthropic API error: {}", error_text));
        }

        let headers = response.headers().clone();
        let anthropic_response: AnthropicResponse = response.json().await?;

        let mut usage = Usage {
            prompt_tokens: anthropic_response.usage.input_tokens,
            completion_tokens: anthropic_response.usage.output_tokens,
        };

        // Header extraction as fallback/supplement
        if let Some(header_usage) = Usage::from_headers(&headers) {
            usage = header_usage;
        }

        let text = anthropic_response.content.iter()
            .filter_map(|c| if let AnthropicContent::Text { text } = c { Some(text.clone()) } else { None })
            .collect::<Vec<_>>()
            .join("");

        Ok(GenerateResult {
            text,
            usage,
            finish_reason: "stop".to_string(), // Anthropic has finish_reason in different events, for non-stream using "stop"
        })
    }

    async fn generate_stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> Result<BoxStream<'static, StreamPart>> {
        let (mut request, _) = self.prepare_request(prompt, options)?;
        request.stream = Some(true);

        let response = self.client.post(&format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Anthropic API error: {}", error_text));
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
                                yield StreamPart::Usage { usage: Usage { prompt_tokens, completion_tokens: 0 } };
                            }
                            Ok(AnthropicStreamEvent::ContentBlockDelta { delta, .. }) => {
                                match delta {
                                    crate::types::AnthropicDelta::TextDelta { text } => {
                                        yield StreamPart::TextDelta { delta: text };
                                    }
                                    crate::types::AnthropicDelta::InputJsonDelta { partial_json } => {
                                        yield StreamPart::ToolCallDelta { 
                                            index: 0, 
                                            id: None, 
                                            name: None, 
                                            arguments_delta: Some(partial_json) 
                                        };
                                    }
                                }
                            }
                            Ok(AnthropicStreamEvent::MessageDelta { delta, usage }) => {
                                yield StreamPart::Usage { usage: Usage { prompt_tokens, completion_tokens: usage.output_tokens } };
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
    fn prepare_request(&self, prompt: Prompt, options: GenerateOptions) -> Result<(AnthropicRequest, Vec<qai_core::types::ToolDefinition>)> {
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
                                    return Err(anyhow!("Unsupported image source for Anthropic"));
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
                            Content::ToolCall { id, name, arguments } => {
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

        let anthropic_tools = if options.tools.as_ref().map(|t| !t.is_empty()).unwrap_or(false) {
            Some(options.tools.unwrap().into_iter().map(|t| AnthropicTool {
                name: t.name,
                description: t.description,
                input_schema: t.parameters,
            }).collect())
        } else {
            None
        };

        let request = AnthropicRequest {
            model: options.model_id,
            messages,
            system: if system_content.is_empty() { None } else { Some(system_content) },
            max_tokens: options.max_tokens.unwrap_or(1024),
            temperature: options.temperature,
            top_p: options.top_p,
            top_k: None,
            stop_sequences: options.stop_sequences,
            stream: None,
            tools: anthropic_tools,
            tool_choice: None, // Default to auto
        };

        Ok((request, Vec::new())) // Tool list not used for return currently
    }
}

// --- Provider Factory ---

use qai_core::types::ProviderSettings;

/// Anthropic provider with configurable settings.
pub struct AnthropicProvider {
    settings: ProviderSettings,
}

impl AnthropicProvider {
    /// Creates a language model with the given model ID.
    pub fn language_model(&self, _model_id: &str) -> AnthropicModel {
        let api_key = self.settings.api_key.clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .unwrap_or_default();
        let mut model = AnthropicModel::new(api_key);
        if let Some(ref base_url) = self.settings.base_url {
            model.base_url = base_url.clone();
        }
        model
    }

    /// Alias for `language_model`.
    pub fn chat(&self, model_id: &str) -> AnthropicModel {
        self.language_model(model_id)
    }
}

/// Create an Anthropic provider instance with the given settings.
pub fn create_anthropic(settings: ProviderSettings) -> AnthropicProvider {
    AnthropicProvider { settings }
}
