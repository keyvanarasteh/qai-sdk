pub mod types;
pub mod embedding;
pub mod image;
pub mod error;
pub mod tools;
pub mod completion;
pub mod speech;
pub mod transcription;
pub mod responses_types;
pub mod responses;
#[cfg(test)]
mod tests;

use async_trait::async_trait;
use qai_core::types::{Content, GenerateOptions, GenerateResult, Prompt, Role, Usage, ImageSource, StreamPart};
use crate::types::{OpenAIRequest, OpenAIResponse, OpenAIMessage, OpenAIContent, OpenAIImageUrl, OpenAIToolCall, OpenAIFunctionCall, OpenAIStreamChunk, OpenAITool, OpenAIFunctionDefinition};
use anyhow::{Result, anyhow};
use reqwest::Client;
use futures::stream::BoxStream;
use futures_util::StreamExt;
use eventsource_stream::Eventsource;

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
        let request = self.prepare_request(prompt, options)?;

        let response = self.client.post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }

        let headers = response.headers().clone();
        let openai_response: OpenAIResponse = response.json().await?;

        let mut usage = Usage {
            prompt_tokens: openai_response.usage.prompt_tokens,
            completion_tokens: openai_response.usage.completion_tokens,
        };

        // Header extraction as fallback/supplement
        if let Some(header_usage) = Usage::from_headers(&headers) {
            usage = header_usage;
        }

        Ok(GenerateResult {
            text: openai_response.choices[0].message.content.clone().unwrap_or_default(),
            usage,
            finish_reason: openai_response.choices[0].finish_reason.clone().unwrap_or_default(),
        })
    }

    async fn generate_stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> Result<BoxStream<'static, StreamPart>> {
        let mut request = self.prepare_request(prompt, options)?;
        request.stream = Some(true);

        let response = self.client.post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }

        let mut event_stream = response.bytes_stream().eventsource();

        let stream = async_stream::stream! {
            while let Some(event) = event_stream.next().await {
                match event {
                    Ok(event) => {
                        if event.data == "[DONE]" {
                            break;
                        }

                        let parsed: Result<OpenAIStreamChunk, _> = serde_json::from_str(&event.data);
                        match parsed {
                            Ok(chunk) => {
                                if let Some(usage) = chunk.usage {
                                    yield StreamPart::Usage { 
                                        usage: Usage { 
                                            prompt_tokens: usage.prompt_tokens, 
                                            completion_tokens: usage.completion_tokens 
                                        } 
                                    };
                                }

                                for choice in chunk.choices {
                                    if let Some(delta_content) = choice.delta.content {
                                        yield StreamPart::TextDelta { delta: delta_content };
                                    }

                                    if let Some(tool_calls) = choice.delta.tool_calls {
                                        for tc in tool_calls {
                                            yield StreamPart::ToolCallDelta {
                                                index: tc.index,
                                                id: tc.id,
                                                name: tc.function.as_ref().and_then(|f| f.name.clone()),
                                                arguments_delta: tc.function.as_ref().and_then(|f| f.arguments.clone()),
                                            };
                                        }
                                    }

                                    if let Some(reason) = choice.finish_reason {
                                        yield StreamPart::Finish { finish_reason: reason };
                                    }
                                }
                            }
                            Err(e) => {
                                yield StreamPart::Error { message: e.to_string() };
                            }
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

impl OpenAIModel {
    fn prepare_request(&self, prompt: Prompt, options: GenerateOptions) -> Result<OpenAIRequest> {
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
                            Content::File { .. } => {
                                return Err(anyhow!("File content is not yet supported for OpenAI"));
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

        let openai_tools = if options.tools.as_ref().map(|t| !t.is_empty()).unwrap_or(false) {
            Some(options.tools.unwrap().into_iter().map(|t| OpenAITool {
                tool_type: "function".to_string(),
                function: OpenAIFunctionDefinition {
                    name: t.name,
                    description: t.description,
                    parameters: t.parameters,
                },
            }).collect())
        } else {
            None
        };

        Ok(OpenAIRequest {
            model: options.model_id,
            messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: options.top_p,
            stop: options.stop_sequences,
            stream: Some(false),
            tools: openai_tools,
            tool_choice: None, // Default to auto
        })
    }
}

// --- Provider Factory ---

use qai_core::types::ProviderSettings;

/// OpenAI provider with configurable settings.
pub struct OpenAIProvider {
    settings: ProviderSettings,
}

impl OpenAIProvider {
    fn resolve_api_key(&self) -> String {
        self.settings.api_key.clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .unwrap_or_default()
    }

    fn resolve_base_url(&self) -> String {
        self.settings.base_url.clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string())
    }

    /// Creates a chat language model.
    pub fn chat(&self, _model_id: &str) -> OpenAIModel {
        OpenAIModel {
            api_key: self.resolve_api_key(),
            base_url: self.resolve_base_url(),
            client: Client::new(),
        }
    }

    /// Alias for `chat`.
    pub fn language_model(&self, model_id: &str) -> OpenAIModel {
        self.chat(model_id)
    }

    /// Creates an embedding model.
    pub fn embedding(&self, _model_id: &str) -> crate::embedding::OpenAIEmbeddingModel {
        crate::embedding::OpenAIEmbeddingModel {
            api_key: self.resolve_api_key(),
            base_url: self.resolve_base_url(),
            client: Client::new(),
        }
    }

    /// Creates an image generation model.
    pub fn image(&self, _model_id: &str) -> crate::image::OpenAIImageModel {
        crate::image::OpenAIImageModel {
            api_key: self.resolve_api_key(),
            base_url: self.resolve_base_url(),
            client: Client::new(),
        }
    }

    /// Creates a completion model.
    pub fn completion(&self, _model_id: &str) -> crate::completion::OpenAICompletionModel {
        crate::completion::OpenAICompletionModel {
            api_key: self.resolve_api_key(),
            base_url: self.resolve_base_url(),
            client: Client::new(),
        }
    }

    /// Creates a speech (TTS) model.
    pub fn speech(&self, _model_id: &str) -> crate::speech::OpenAISpeechModel {
        crate::speech::OpenAISpeechModel {
            api_key: self.resolve_api_key(),
            base_url: self.resolve_base_url(),
            client: Client::new(),
        }
    }

    /// Creates a transcription (STT) model.
    pub fn transcription(&self, _model_id: &str) -> crate::transcription::OpenAITranscriptionModel {
        crate::transcription::OpenAITranscriptionModel {
            api_key: self.resolve_api_key(),
            base_url: self.resolve_base_url(),
            client: Client::new(),
        }
    }

    /// Creates a Responses API model.
    pub fn responses(&self, _model_id: &str) -> crate::responses::OpenAIResponsesModel {
        crate::responses::OpenAIResponsesModel {
            api_key: self.resolve_api_key(),
            base_url: self.resolve_base_url(),
            client: Client::new(),
        }
    }
}

/// Create an OpenAI provider instance with the given settings.
pub fn create_openai(settings: ProviderSettings) -> OpenAIProvider {
    OpenAIProvider { settings }
}
