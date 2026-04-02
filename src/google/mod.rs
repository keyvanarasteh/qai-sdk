//! # QAI Google
//!
//! Google Gemini provider for the QAI SDK. Supports chat, streaming,
//! tool calling, vision, embeddings, and image generation via the
//! Generative Language API.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use qai_sdk::google::create_google;
//! use qai_sdk::core::types::ProviderSettings;
//!
//! let provider = create_google(ProviderSettings {
//!     api_key: Some("AIza...".to_string()),
//!     ..Default::default()
//! });
//!
//! let model = provider.chat("gemini-2.0-flash");
//! ```

pub mod embedding;
pub mod error;
pub mod image;
#[cfg(test)]
mod tests;
pub mod tools;
pub mod types;

use crate::core::types::{
    Content, FileSource, GenerateOptions, GenerateResult, ImageSource, Prompt, Role, StreamPart,
    Usage,
};
use crate::google::types::{
    GoogleContent, GoogleFunctionDeclaration, GoogleGenerationConfig, GooglePart, GoogleRequest,
    GoogleResponse, GoogleTool,
};
use anyhow::anyhow;
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures::stream::BoxStream;
use futures_util::StreamExt;
use reqwest::Client;

pub struct GoogleModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl GoogleModel {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl crate::core::LanguageModel for GoogleModel {
    #[tracing::instrument(skip(self, prompt), fields(model = options.model_id))]
    async fn generate(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> crate::core::Result<GenerateResult> {
        let request = self.prepare_request(prompt, &options)?;

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, options.model_id, self.api_key
        );

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Google API error: {error_text}").into());
        }

        let headers = response.headers().clone();
        let google_response: GoogleResponse = response.json().await?;

        let mut usage = Usage {
            prompt_tokens: google_response.usage_metadata.prompt_token_count,
            completion_tokens: google_response.usage_metadata.candidates_token_count,
        };

        // Header extraction as fallback/supplement
        if let Some(header_usage) = Usage::from_headers(&headers) {
            usage = header_usage;
        }

        let candidate =
            google_response
                .candidates
                .first()
                .ok_or_else(|| -> crate::core::ProviderError {
                    crate::core::ProviderError::Other(anyhow::anyhow!(
                        "No candidates returned from Google"
                    ))
                })?;

        let mut text_parts = Vec::new();
        let mut tool_calls = Vec::new();

        for part in &candidate.content.parts {
            match part {
                GooglePart::Text { text } => {
                    text_parts.push(text.clone());
                }
                GooglePart::FunctionCall { name, args } => {
                    tool_calls.push(crate::core::types::ToolCallResult {
                        name: name.clone(),
                        arguments: args.clone(),
                    });
                }
                _ => {}
            }
        }

        let text = text_parts.join("");

        Ok(GenerateResult {
            text,
            usage,
            finish_reason: candidate
                .finish_reason
                .clone()
                .unwrap_or_else(|| "stop".to_string()),
            tool_calls,
        })
    }

    async fn generate_stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> crate::core::Result<BoxStream<'static, StreamPart>> {
        let request = self.prepare_request(prompt, &options)?;
        let url = format!(
            "{}/models/{}:streamGenerateContent?alt=sse&key={}",
            self.base_url, options.model_id, self.api_key
        );

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Google API error: {error_text}").into());
        }

        let mut event_stream = response.bytes_stream().eventsource();

        let stream = async_stream::stream! {
            while let Some(event) = event_stream.next().await {
                match event {
                    Ok(event) => {
                        let parsed: Result<GoogleResponse, _> = serde_json::from_str(&event.data);
                        match parsed {
                            Ok(google_response) => {
                                // Gemini sends usage in the last chunk or sometimes in every chunk
                                yield StreamPart::Usage {
                                    usage: Usage {
                                        prompt_tokens: google_response.usage_metadata.prompt_token_count,
                                        completion_tokens: google_response.usage_metadata.candidates_token_count
                                    }
                                };

                                if let Some(candidate) = google_response.candidates.first() {
                                    for part in &candidate.content.parts {
                                        match part {
                                            GooglePart::Text { text } => {
                                                yield StreamPart::TextDelta { delta: text.clone() };
                                            }
                                            GooglePart::FunctionCall { name, args } => {
                                                yield StreamPart::ToolCallDelta {
                                                    index: 0,
                                                    id: None,
                                                    name: Some(name.clone()),
                                                    arguments_delta: Some(args.to_string()),
                                                };
                                            }
                                            _ => {}
                                        }
                                    }

                                    if let Some(reason) = &candidate.finish_reason {
                                        yield StreamPart::Finish { finish_reason: reason.clone() };
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

impl GoogleModel {
    fn prepare_request(
        &self,
        prompt: Prompt,
        options: &GenerateOptions,
    ) -> crate::core::Result<GoogleRequest> {
        let mut contents = Vec::new();
        let mut system_instruction = None;

        for msg in prompt.messages {
            let role = match msg.role {
                Role::System => {
                    let mut parts = Vec::new();
                    for content in msg.content {
                        if let Content::Text { text } = content {
                            parts.push(GooglePart::Text { text });
                        }
                    }
                    system_instruction = Some(GoogleContent {
                        role: "system".to_string(),
                        parts,
                    });
                    continue;
                }
                Role::User => "user",
                Role::Assistant => "model",
                Role::Tool => "user",
            };

            let mut parts = Vec::new();
            for content in msg.content {
                match content {
                    Content::Text { text } => {
                        parts.push(GooglePart::Text { text });
                    }
                    Content::Image { source } => {
                        let (mime_type, data) = match source {
                            ImageSource::Base64 { media_type, data } => (media_type, data),
                            _ => return Err(anyhow!("Unsupported image source for Google").into()),
                        };
                        parts.push(GooglePart::InlineData { mime_type, data });
                    }
                    Content::File { source } => {
                        let FileSource::Base64 { media_type, data } = source;
                        parts.push(GooglePart::InlineData {
                            mime_type: media_type,
                            data,
                        });
                    }
                    Content::ToolCall {
                        name, arguments, ..
                    } => {
                        parts.push(GooglePart::FunctionCall {
                            name,
                            args: arguments,
                        });
                    }
                    Content::ToolResult { id, result } => {
                        parts.push(GooglePart::FunctionResponse {
                            name: id,
                            response: result,
                        });
                    }
                }
            }

            contents.push(GoogleContent {
                role: role.to_string(),
                parts,
            });
        }

        let google_tools = if options.tools.as_ref().is_some_and(|t| !t.is_empty()) {
            Some(vec![GoogleTool {
                function_declarations: options
                    .tools
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|t| GoogleFunctionDeclaration {
                        name: t.name.clone(),
                        description: t.description.clone(),
                        parameters: t.parameters.clone(),
                    })
                    .collect(),
            }])
        } else {
            None
        };

        Ok(GoogleRequest {
            contents,
            system_instruction,
            generation_config: Some(GoogleGenerationConfig {
                max_output_tokens: options.max_tokens,
                temperature: options.temperature,
                top_p: options.top_p,
                top_k: None,
                stop_sequences: options.stop_sequences.clone(),
            }),
            tools: google_tools,
        })
    }
}

// --- Provider Factory ---

use crate::core::types::ProviderSettings;

/// Google provider with configurable settings.
pub struct GoogleProvider {
    settings: ProviderSettings,
}

impl GoogleProvider {
    /// Creates a chat language model.
    #[must_use]
    pub fn chat(&self, _model_id: &str) -> GoogleModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("GOOGLE_GENERATIVE_AI_API_KEY").ok())
            .unwrap_or_default();
        let mut model = GoogleModel::new(api_key);
        if let Some(ref base_url) = self.settings.base_url {
            model.base_url = base_url.clone();
        }
        model
    }

    /// Alias for `chat`.
    #[must_use]
    pub fn language_model(&self, model_id: &str) -> GoogleModel {
        self.chat(model_id)
    }

    /// Creates an embedding model.
    #[must_use]
    pub fn embedding(&self, _model_id: &str) -> embedding::GoogleEmbeddingModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("GOOGLE_GENERATIVE_AI_API_KEY").ok())
            .unwrap_or_default();
        let mut model = embedding::GoogleEmbeddingModel::new(api_key);
        if let Some(ref base_url) = self.settings.base_url {
            model.base_url = base_url.clone();
        }
        model
    }

    /// Creates an image generation model.
    #[must_use]
    pub fn image(&self, _model_id: &str) -> image::GoogleImageModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("GOOGLE_GENERATIVE_AI_API_KEY").ok())
            .unwrap_or_default();
        let mut model = image::GoogleImageModel::new(api_key);
        if let Some(ref base_url) = self.settings.base_url {
            model.base_url = base_url.clone();
        }
        model
    }
}

/// Create a Google provider instance with the given settings.
#[must_use]
pub fn create_google(settings: ProviderSettings) -> GoogleProvider {
    GoogleProvider { settings }
}

impl crate::core::registry::Provider for GoogleProvider {
    fn language_model(&self, model_id: &str) -> Option<Box<dyn crate::core::LanguageModel>> {
        Some(Box::new(self.chat(model_id)))
    }

    fn embedding_model(&self, model_id: &str) -> Option<Box<dyn crate::core::EmbeddingModel>> {
        Some(Box::new(self.embedding(model_id)))
    }

    fn image_model(&self, model_id: &str) -> Option<Box<dyn crate::core::ImageModel>> {
        Some(Box::new(self.image(model_id)))
    }
}
