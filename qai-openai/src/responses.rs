//! OpenAI Responses API model implementation.
//!
//! The Responses API is OpenAI's newer API surface that supports reasoning models,
//! multi-turn conversations via `previous_response_id`, and server-executed tools
//! (shell, code_interpreter, web_search, apply_patch, MCP, etc.).

use async_trait::async_trait;
use qai_core::types::{GenerateOptions, GenerateResult, Prompt, StreamPart, Usage};
use crate::responses_types::*;
use anyhow::{Result, anyhow};
use reqwest::Client;
use futures::stream::BoxStream;
use futures_util::StreamExt;
use eventsource_stream::Eventsource;

/// OpenAI Responses API model.
pub struct OpenAIResponsesModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl OpenAIResponsesModel {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: Client::new(),
        }
    }

    /// Build a ResponsesRequest from core Prompt and GenerateOptions.
    fn build_request(&self, prompt: Prompt, options: GenerateOptions) -> ResponsesRequest {
        let input = prompt.messages.into_iter().map(|msg| {
            match msg.role {
                qai_core::types::Role::System => ResponsesInputItem::Message {
                    role: ResponsesRole::System,
                    content: ResponsesMessageContent::Text(
                        msg.content.into_iter().filter_map(|c| match c {
                            qai_core::types::Content::Text { text } => Some(text),
                            _ => None,
                        }).collect::<Vec<_>>().join("")
                    ),
                },
                qai_core::types::Role::User => ResponsesInputItem::Message {
                    role: ResponsesRole::User,
                    content: ResponsesMessageContent::Parts(
                        msg.content.into_iter().map(|c| match c {
                            qai_core::types::Content::Text { text } => ResponsesContentPart::InputText { text },
                            qai_core::types::Content::Image { source } => {
                                let url = match source {
                                    qai_core::types::ImageSource::Url { url } => url,
                                    qai_core::types::ImageSource::Base64 { data, media_type } => {
                                        format!("data:{};base64,{}", media_type, data)
                                    }
                                };
                                ResponsesContentPart::InputImage { image_url: url }
                            }
                            _ => ResponsesContentPart::InputText { text: String::new() },
                        }).collect()
                    ),
                },
                qai_core::types::Role::Assistant => ResponsesInputItem::Message {
                    role: ResponsesRole::Assistant,
                    content: ResponsesMessageContent::Parts(
                        msg.content.into_iter().filter_map(|c| match c {
                            qai_core::types::Content::Text { text } => Some(ResponsesContentPart::OutputText { text }),
                            _ => None,
                        }).collect()
                    ),
                },
                qai_core::types::Role::Tool => {
                    // Tool results: extract text + id from ToolResult content
                    let mut call_id = String::new();
                    let mut output_text = String::new();
                    for c in msg.content {
                        match c {
                            qai_core::types::Content::ToolResult { id, result } => {
                                call_id = id;
                                output_text = result.to_string();
                            }
                            qai_core::types::Content::Text { text } => {
                                output_text = text;
                            }
                            _ => {}
                        }
                    }
                    ResponsesInputItem::FunctionCallOutput {
                        call_id,
                        output: serde_json::Value::String(output_text),
                    }
                }
            }
        }).collect();

        let tools = options.tools.map(|tool_defs| {
            tool_defs.into_iter().map(|t| {
                ResponsesTool::Function {
                    name: t.name,
                    description: Some(t.description),
                    parameters: t.parameters,
                    strict: None,
                }
            }).collect()
        });

        ResponsesRequest {
            model: options.model_id,
            input,
            temperature: options.temperature,
            top_p: options.top_p,
            max_output_tokens: options.max_tokens,
            tools,
            tool_choice: None,
            text: None,
            reasoning: None,
            previous_response_id: None,
            store: None,
            include: None,
            stream: None,
            instructions: None,
            metadata: None,
            truncation: None,
            service_tier: None,
            user: None,
        }
    }

    /// Extract text from response output items.
    fn extract_text(output: &[ResponsesOutputItem]) -> String {
        let mut texts = Vec::new();
        for item in output {
            match item {
                ResponsesOutputItem::Message { content: parts, .. } => {
                    for part in parts {
                        texts.push(part.text.clone());
                    }
                }
                ResponsesOutputItem::Reasoning { summary, .. } => {
                    for s in summary {
                        texts.push(s.text.clone());
                    }
                }
                _ => {}
            }
        }
        texts.join("")
    }

    /// Determine finish reason from output items.
    fn finish_reason(output: &[ResponsesOutputItem]) -> String {
        for item in output {
            if matches!(item, ResponsesOutputItem::FunctionCall { .. } | ResponsesOutputItem::CustomToolCall { .. }) {
                return "tool_calls".to_string();
            }
        }
        "stop".to_string()
    }
}

#[async_trait]
impl qai_core::LanguageModel for OpenAIResponsesModel {
    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> Result<GenerateResult> {
        let request = self.build_request(prompt, options);

        let resp = self.client.post(&format!("{}/responses", self.base_url))
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(anyhow!("OpenAI Responses API error: {}", error_text));
        }

        let response: ResponsesResponse = resp.json().await?;

        if let Some(err) = response.error {
            return Err(anyhow!("OpenAI Responses API error: {}", err.message));
        }

        let text = Self::extract_text(&response.output);
        let finish_reason = Self::finish_reason(&response.output);

        let usage = response.usage.map(|u| Usage {
            prompt_tokens: u.input_tokens,
            completion_tokens: u.output_tokens,
        }).unwrap_or(Usage { prompt_tokens: 0, completion_tokens: 0 });

        Ok(GenerateResult {
            text,
            usage,
            finish_reason,
        })
    }

    async fn generate_stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> Result<BoxStream<'static, StreamPart>> {
        let mut request = self.build_request(prompt, options);
        request.stream = Some(true);

        let resp = self.client.post(&format!("{}/responses", self.base_url))
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(anyhow!("OpenAI Responses API stream error: {}", error_text));
        }

        let stream = resp.bytes_stream().eventsource();

        let mapped = stream.filter_map(|event| async move {
            match event {
                Ok(ev) => {
                    if ev.data == "[DONE]" {
                        return Some(StreamPart::Finish { finish_reason: "stop".to_string() });
                    }
                    match serde_json::from_str::<ResponsesStreamEvent>(&ev.data) {
                        Ok(ResponsesStreamEvent::OutputTextDelta { delta, .. }) => {
                            Some(StreamPart::TextDelta { delta })
                        }
                        Ok(ResponsesStreamEvent::ReasoningSummaryTextDelta { delta, .. }) => {
                            Some(StreamPart::TextDelta { delta })
                        }
                        Ok(ResponsesStreamEvent::FunctionCallArgumentsDelta { delta, item_id, call_id, .. }) => {
                            Some(StreamPart::ToolCallDelta {
                                index: 0,
                                id: Some(call_id),
                                name: Some(item_id),
                                arguments_delta: Some(delta),
                            })
                        }
                        Ok(ResponsesStreamEvent::ResponseCompleted { response }) => {
                            if let Some(usage) = response.usage {
                                Some(StreamPart::Usage { usage: Usage {
                                    prompt_tokens: usage.input_tokens,
                                    completion_tokens: usage.output_tokens,
                                }})
                            } else {
                                Some(StreamPart::Finish { finish_reason: "stop".to_string() })
                            }
                        }
                        Ok(ResponsesStreamEvent::ResponseFailed { response }) => {
                            let msg = response.error
                                .map(|e| e.message)
                                .unwrap_or_else(|| "Unknown error".to_string());
                            Some(StreamPart::Error { message: msg })
                        }
                        Ok(_) => None,
                        Err(_) => None,
                    }
                }
                Err(e) => Some(StreamPart::Error { message: format!("SSE error: {}", e) }),
            }
        });

        Ok(Box::pin(mapped))
    }
}
