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
pub mod music;
pub mod realtime;
pub mod speech;
#[cfg(test)]
mod tests;
pub mod tools;
pub mod types;
pub mod video;
use serde_json::json;

use crate::core::types::{
    Content, FileSource, GenerateOptions, GenerateResult, ImageSource, Prompt, Role, StreamPart,
    Usage,
};
use crate::google::types::{
    GoogleContent, GoogleFunctionDeclaration, GoogleGenerationConfig, GooglePart, GoogleRequest,
    GoogleResponse, GoogleThinkingConfig, GoogleTool,
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
            cache_hit_tokens: None,
            cache_miss_tokens: None,
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
        let mut thought_parts = Vec::new();
        let mut tool_calls = Vec::new();
        let mut executed_tools = Vec::new();

        for part in &candidate.content.parts {
            match part {
                GooglePart::Text { text, thought } => {
                    if thought.unwrap_or(false) {
                        thought_parts.push(text.clone());
                    } else {
                        text_parts.push(text.clone());
                    }
                }
                GooglePart::FunctionCall { name, args } => {
                    tool_calls.push(crate::core::types::ToolCallResult {
                        name: name.clone(),
                        arguments: args.clone(),
                    });
                }
                GooglePart::ExecutableCode { language, code } => {
                    executed_tools.push(crate::core::types::ExecutedTool {
                        name: "code_execution".to_string(),
                        tool_type: "code_execution".to_string(),
                        arguments: Some(json!({ "language": language, "code": code })),
                        output: None,
                        server_label: Some("Executing Code".to_string()),
                    });
                }
                GooglePart::CodeExecutionResult { outcome, output } => {
                    if let Some(last_tool) = executed_tools.last_mut() {
                        if last_tool.tool_type == "code_execution" {
                            last_tool.output =
                                Some(json!({ "outcome": outcome, "output": output }));
                        }
                    }
                }
                _ => {}
            }
        }

        let text = text_parts.join("");
        let reasoning = if thought_parts.is_empty() {
            None
        } else {
            Some(thought_parts.join(""))
        };

        let mut executed_tools = Vec::new();
        if let Some(metadata) = &candidate.grounding_metadata {
            executed_tools.push(crate::core::types::ExecutedTool {
                name: "google_search_retrieval".to_string(),
                tool_type: "web_search".to_string(),
                arguments: None,
                output: Some(metadata.clone()),
                server_label: None,
            });
        }

        Ok(GenerateResult {
            text,
            usage,
            finish_reason: candidate
                .finish_reason
                .clone()
                .unwrap_or_else(|| "stop".to_string()),
            tool_calls,
            reasoning,
            executed_tools,
            citations: Vec::new(),
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
                                        completion_tokens: google_response.usage_metadata.candidates_token_count,
                                        cache_hit_tokens: None,
                                        cache_miss_tokens: None,
                                    }
                                };

                                if let Some(candidate) = google_response.candidates.first() {
                                    for part in &candidate.content.parts {
                                        match part {
                                            GooglePart::Text { text, thought } => {
                                                if thought.unwrap_or(false) {
                                                    yield StreamPart::ReasoningDelta { delta: text.clone() };
                                                } else {
                                                    yield StreamPart::TextDelta { delta: text.clone() };
                                                }
                                            }
                                            GooglePart::FunctionCall { name, args } => {
                                                yield StreamPart::ToolCallDelta {
                                                    index: 0,
                                                    id: None,
                                                    name: Some(name.clone()),
                                                    arguments_delta: Some(args.to_string()),
                                                };
                                            }
                                            GooglePart::ExecutableCode { language, code } => {
                                                yield StreamPart::ExecutedTool {
                                                    tool: crate::core::types::ExecutedTool {
                                                        name: "code_execution".to_string(),
                                                        tool_type: "code_execution".to_string(),
                                                        arguments: Some(serde_json::json!({ "language": language, "code": code })),
                                                        output: None,
                                                        server_label: Some("Executing Code".to_string()),
                                                    }
                                                };
                                            }
                                            GooglePart::CodeExecutionResult { outcome, output } => {
                                                yield StreamPart::ExecutedTool {
                                                    tool: crate::core::types::ExecutedTool {
                                                        name: "code_execution".to_string(),
                                                        tool_type: "code_execution".to_string(),
                                                        arguments: None,
                                                        output: Some(serde_json::json!({ "outcome": outcome, "output": output })),
                                                        server_label: Some("Code Execution Result".to_string()),
                                                    }
                                                };
                                            }
                                            _ => {}
                                        }
                                    }

                                    if let Some(metadata) = &candidate.grounding_metadata {
                                        yield StreamPart::ExecutedTool {
                                            tool: crate::core::types::ExecutedTool {
                                                name: "google_search_retrieval".to_string(),
                                                tool_type: "web_search".to_string(),
                                                arguments: None,
                                                output: Some(metadata.clone()),
                                                server_label: None,
                                            }
                                        };
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
                            parts.push(GooglePart::Text {
                                text,
                                thought: None,
                            });
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
                        parts.push(GooglePart::Text {
                            text,
                            thought: None,
                        });
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
                    Content::Spatial { boxes } => {
                        let mut text_boxes = Vec::new();
                        for b in boxes {
                            let label_str = b.label.as_deref().unwrap_or("object");
                            text_boxes.push(format!(
                                "[{}, {}, {}, {}] {}",
                                b.y_min, b.x_min, b.y_max, b.x_max, label_str
                            ));
                        }
                        parts.push(GooglePart::Text {
                            text: text_boxes.join("\n"),
                            thought: None,
                        });
                    }
                }
            }

            contents.push(GoogleContent {
                role: role.to_string(),
                parts,
            });
        }

        let mut google_tools = Vec::new();
        let mut functions = Vec::new();

        if let Some(tools) = &options.tools {
            for t in tools {
                functions.push(GoogleFunctionDeclaration {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    parameters: t.parameters.clone(),
                });
            }
        }

        if !functions.is_empty() {
            google_tools.push(GoogleTool {
                function_declarations: Some(functions),
                google_search_retrieval: None,
                code_execution: None,
            });
        }

        if let Some(server_tools) = &options.server_tools {
            for st in server_tools {
                match st.tool_type.as_str() {
                    "web_search" | "google_search" => {
                        google_tools.push(GoogleTool {
                            function_declarations: None,
                            google_search_retrieval: Some(types::GoogleSearchRetrieval {
                                dynamic_retrieval_config: Some(types::DynamicRetrievalConfig {
                                    mode: Some("MODE_DYNAMIC".to_string()),
                                    dynamic_threshold: Some(0.3),
                                }),
                            }),
                            code_execution: None,
                        });
                    }
                    "code_execution" | "code_interpreter" => {
                        google_tools.push(GoogleTool {
                            function_declarations: None,
                            google_search_retrieval: None,
                            code_execution: Some(serde_json::json!({})),
                        });
                    }
                    _ => {}
                }
            }
        }

        let google_tools_opt = if google_tools.is_empty() {
            None
        } else {
            Some(google_tools)
        };

        let mut response_mime_type = None;
        let mut response_schema = None;
        if let Some(format) = &options.response_format {
            if format.get("type").and_then(|t| t.as_str()) == Some("json_schema") {
                response_mime_type = Some("application/json".to_string());
                if let Some(schema) = format.get("json_schema").and_then(|s| s.get("schema")) {
                    response_schema = Some(schema.clone());
                }
            } else if format.get("type").and_then(|t| t.as_str()) == Some("json_object") {
                response_mime_type = Some("application/json".to_string());
            }
        }

        // Build thinking config from GenerateOptions
        let thinking_config =
            if options.reasoning_format.is_some() || options.reasoning_effort.is_some() {
                let mut tc = GoogleThinkingConfig {
                    include_thoughts: None,
                    thinking_level: None,
                    thinking_budget: None,
                };

                // "parsed" format → include thought summaries in response
                if options
                    .reasoning_format
                    .as_deref()
                    .is_some_and(|f| f == "parsed" || f == "raw")
                {
                    tc.include_thoughts = Some(true);
                }

                // Map reasoning_effort to thinking_level (Gemini 3) or thinking_budget (Gemini 2.5)
                if let Some(ref effort) = options.reasoning_effort {
                    let effort_lower = effort.to_lowercase();
                    match effort_lower.as_str() {
                        "minimal" | "low" | "medium" | "high" => {
                            tc.thinking_level = Some(effort_lower);
                        }
                        "off" | "none" => {
                            tc.thinking_budget = Some(0);
                        }
                        "dynamic" => {
                            tc.thinking_budget = Some(-1);
                        }
                        _ => {
                            // Try to parse as integer for thinking_budget
                            if let Ok(budget) = effort.parse::<i32>() {
                                tc.thinking_budget = Some(budget);
                            } else {
                                tc.thinking_level = Some(effort_lower);
                            }
                        }
                    }
                }

                Some(tc)
            } else {
                None
            };

        Ok(GoogleRequest {
            system_instruction,
            contents,
            tools: google_tools_opt,
            generation_config: Some(GoogleGenerationConfig {
                max_output_tokens: options.max_tokens,
                temperature: options.temperature,
                top_p: options.top_p,
                top_k: None,
                stop_sequences: options.stop_sequences.clone(),
                response_mime_type,
                response_schema,
                response_modalities: None,
                speech_config: None,
                thinking_config,
            }),
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

    /// Creates a speech generation (TTS) model.
    #[must_use]
    pub fn speech_model(&self, _model_id: &str) -> speech::GoogleSpeechModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("GOOGLE_GENERATIVE_AI_API_KEY").ok())
            .unwrap_or_default();
        let base_url = self
            .settings
            .base_url
            .clone()
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());
        speech::GoogleSpeechModel::new(api_key, base_url)
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

    /// Creates a video generation model.
    #[must_use]
    pub fn video_model(&self, _model_id: &str) -> video::GoogleVideoModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("GOOGLE_GENERATIVE_AI_API_KEY").ok())
            .unwrap_or_default();
        let mut model = video::GoogleVideoModel::new(api_key);
        if let Some(ref base_url) = self.settings.base_url {
            model.base_url = base_url.clone();
        }
        model
    }

    /// Creates a music generation model.
    #[must_use]
    pub fn music_model(&self, _model_id: &str) -> music::GoogleMusicModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("GOOGLE_GENERATIVE_AI_API_KEY").ok())
            .unwrap_or_default();
        let base_url = self
            .settings
            .base_url
            .clone()
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());
        music::GoogleMusicModel::new(api_key, base_url)
    }

    /// Creates a realtime model.
    #[must_use]
    pub fn realtime_model(&self, model_id: &str) -> realtime::GoogleRealtimeModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("GOOGLE_GENERATIVE_AI_API_KEY").ok())
            .unwrap_or_default();
        let base_url = "generativelanguage.googleapis.com".to_string();
        realtime::GoogleRealtimeModel::new(api_key, model_id.to_string(), base_url)
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
    fn speech_model(&self, model_id: &str) -> Option<Box<dyn crate::core::SpeechModel>> {
        Some(Box::new(self.speech_model(model_id)))
    }
    fn video_model(&self, model_id: &str) -> Option<Box<dyn crate::core::VideoModel>> {
        Some(Box::new(self.video_model(model_id)))
    }
    fn music_model(&self, model_id: &str) -> Option<Box<dyn crate::core::MusicModel>> {
        Some(Box::new(self.music_model(model_id)))
    }
    fn realtime_model(&self, model_id: &str) -> Option<Box<dyn crate::core::RealtimeModel>> {
        Some(Box::new(self.realtime_model(model_id)))
    }
}
