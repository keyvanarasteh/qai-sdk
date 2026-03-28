pub mod types;

use async_trait::async_trait;
use qai_core::types::{Content, GenerateOptions, GenerateResult, Prompt, Role, Usage, ImageSource};
use crate::types::{OpenAIRequest, OpenAIResponse, OpenAIMessage, OpenAIContent, OpenAIImageUrl, OpenAIToolCall, OpenAIFunctionCall};
use anyhow::{Result, anyhow};
use reqwest::Client;

pub struct OpenAIModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl OpenAIModel {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl qai_core::LanguageModel for OpenAIModel {
    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> Result<GenerateResult> {
        let mut messages = Vec::new();

        for msg in prompt.messages {
            match msg.role {
                Role::System => {
                    let mut system_text = String::new();
                    for content in msg.content {
                        if let Content::Text { text } = content {
                            system_text.push_str(&text);
                        }
                    }
                    messages.push(OpenAIMessage::System { content: system_text });
                }
                Role::User => {
                    let mut user_contents = Vec::new();
                    for content in msg.content {
                        match content {
                            Content::Text { text } => {
                                user_contents.push(OpenAIContent::Text { text });
                            }
                            Content::Image { source } => {
                                match source {
                                    ImageSource::Base64 { media_type, data } => {
                                        user_contents.push(OpenAIContent::ImageUrl {
                                            image_url: OpenAIImageUrl {
                                                url: format!("data:{};base64,{}", media_type, data),
                                            },
                                        });
                                    }
                                    ImageSource::Url { url } => {
                                        user_contents.push(OpenAIContent::ImageUrl {
                                            image_url: OpenAIImageUrl { url },
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    messages.push(OpenAIMessage::User { content: user_contents });
                }
                Role::Assistant => {
                    let mut assistant_text = String::new();
                    let mut tool_calls = Vec::new();

                    for content in msg.content {
                        match content {
                            Content::Text { text } => {
                                assistant_text.push_str(&text);
                            }
                            Content::ToolCall { id, name, arguments } => {
                                tool_calls.push(OpenAIToolCall {
                                    id,
                                    call_type: "function".to_string(),
                                    function: OpenAIFunctionCall {
                                        name,
                                        arguments: arguments.to_string(),
                                    },
                                });
                            }
                            _ => {}
                        }
                    }
                    messages.push(OpenAIMessage::Assistant {
                        content: if assistant_text.is_empty() { None } else { Some(assistant_text) },
                        tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
                    });
                }
                Role::Tool => {
                    for content in msg.content {
                        if let Content::ToolResult { id, result } = content {
                            messages.push(OpenAIMessage::Tool {
                                content: result.to_string(),
                                tool_call_id: id,
                            });
                        }
                    }
                }
            }
        }

        let request = OpenAIRequest {
            model: options.model_id,
            messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: options.top_p,
            stop: options.stop_sequences,
            stream: Some(false),
        };

        let response = self.client.post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }

        let openai_response: OpenAIResponse = response.json().await?;

        let choice = openai_response.choices.get(0).ok_or_else(|| anyhow!("No choices returned from OpenAI"))?;
        let text = choice.message.content.clone().unwrap_or_default();

        Ok(GenerateResult {
            text,
            usage: Usage {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
            },
            finish_reason: choice.finish_reason.clone().unwrap_or_else(|| "unknown".to_string()),
        })
    }
}
