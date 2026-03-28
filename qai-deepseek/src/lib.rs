pub mod types;

use async_trait::async_trait;
use qai_core::types::{Content, GenerateOptions, GenerateResult, Prompt, Role, Usage};
use crate::types::{DeepSeekRequest, DeepSeekResponse, DeepSeekMessage, DeepSeekToolCall, DeepSeekFunctionCall};
use anyhow::{Result, anyhow};
use reqwest::Client;

pub struct DeepSeekModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl DeepSeekModel {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.deepseek.com".to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl qai_core::LanguageModel for DeepSeekModel {
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
                    messages.push(DeepSeekMessage::System { content: system_text });
                }
                Role::User => {
                    let mut user_text = String::new();
                    for content in msg.content {
                        if let Content::Text { text } = content {
                            user_text.push_str(&text);
                        }
                    }
                    messages.push(DeepSeekMessage::User { content: user_text });
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
                                tool_calls.push(DeepSeekToolCall {
                                    id,
                                    call_type: "function".to_string(),
                                    function: DeepSeekFunctionCall {
                                        name,
                                        arguments: arguments.to_string(),
                                    },
                                });
                            }
                            _ => {}
                        }
                    }
                    messages.push(DeepSeekMessage::Assistant {
                        content: if assistant_text.is_empty() { None } else { Some(assistant_text) },
                        reasoning_content: None,
                        tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
                    });
                }
                Role::Tool => {
                    for content in msg.content {
                        if let Content::ToolResult { id, result } = content {
                            messages.push(DeepSeekMessage::Tool {
                                content: result.to_string(),
                                tool_call_id: id,
                            });
                        }
                    }
                }
            }
        }

        let request = DeepSeekRequest {
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
            return Err(anyhow!("DeepSeek API error: {}", error_text));
        }

        let deepseek_response: DeepSeekResponse = response.json().await?;

        let choice = deepseek_response.choices.get(0).ok_or_else(|| anyhow!("No choices returned from DeepSeek"))?;
        let text = choice.message.content.clone().unwrap_or_default();

        Ok(GenerateResult {
            text,
            usage: Usage {
                prompt_tokens: deepseek_response.usage.prompt_tokens,
                completion_tokens: deepseek_response.usage.completion_tokens,
            },
            finish_reason: choice.finish_reason.clone().unwrap_or_else(|| "unknown".to_string()),
        })
    }
}
