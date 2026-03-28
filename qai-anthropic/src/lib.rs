pub mod types;

use async_trait::async_trait;
use qai_core::types::{Content, GenerateOptions, GenerateResult, Prompt, Role, Usage, ImageSource};
use crate::types::{AnthropicRequest, AnthropicResponse, AnthropicMessage, AnthropicContent, AnthropicSystemContent, AnthropicImageSource};
use anyhow::{Result, anyhow};
use reqwest::Client;

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
                                match source {
                                    ImageSource::Base64 { media_type, data } => {
                                        anthropic_contents.push(AnthropicContent::Image {
                                            source: AnthropicImageSource {
                                                source_type: "base64".to_string(),
                                                media_type,
                                                data,
                                            },
                                        });
                                    }
                                    _ => return Err(anyhow!("Unsupported image source for Anthropic")),
                                }
                            }
                            Content::ToolCall { id, name, arguments } => {
                                anthropic_contents.push(AnthropicContent::ToolUse {
                                    id,
                                    name,
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
                            _ => "user".to_string(), // Default or error
                        },
                        content: anthropic_contents,
                    });
                }
            }
        }

        let request = AnthropicRequest {
            model: options.model_id,
            messages,
            system: if system_content.is_empty() { None } else { Some(system_content) },
            max_tokens: options.max_tokens.unwrap_or(1024),
            temperature: options.temperature,
            top_p: options.top_p,
            top_k: None,
            stop_sequences: options.stop_sequences,
        };

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

        let anthropic_response: AnthropicResponse = response.json().await?;

        let text = anthropic_response.content.iter()
            .filter_map(|c| if let AnthropicContent::Text { text } = c { Some(text.clone()) } else { None })
            .collect::<Vec<_>>()
            .join("");

        Ok(GenerateResult {
            text,
            usage: Usage {
                prompt_tokens: anthropic_response.usage.input_tokens,
                completion_tokens: anthropic_response.usage.output_tokens,
            },
            finish_reason: anthropic_response.stop_reason.unwrap_or_else(|| "unknown".to_string()),
        })
    }
}
