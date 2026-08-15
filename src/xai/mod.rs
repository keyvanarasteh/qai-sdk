//! # QAI xAI
//!
//! xAI Grok provider for the QAI SDK. Provides access to Grok models
//! through an OpenAI-compatible API adapter, with additional support for
//! image generation and the Responses API.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use qai_sdk::xai::create_xai;
//! use qai_sdk::core::types::ProviderSettings;
//!
//! let provider = create_xai(ProviderSettings {
//!     api_key: Some("xai-...".to_string()),
//!     ..Default::default()
//! });
//!
//! let model = provider.chat("grok-2");
//! ```

pub mod error;
pub mod image;
pub mod realtime;
pub mod responses;
pub mod speech;
pub mod tools;
pub mod transcription;
pub mod types;
pub mod video;

use crate::core::audio::{
    DetailedSpeechModel, DetailedTranscriptionModel, ProviderCapabilities, StreamingSpeechModel,
};
use crate::core::types::{GenerateOptions, GenerateResult, Prompt, ProviderSettings, StreamPart};
use crate::openai::OpenAIModel;
use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;

pub struct XAIModel {
    pub inner: OpenAIModel,
}

impl XAIModel {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            inner: OpenAIModel {
                api_key,
                base_url: "https://api.x.ai/v1".to_string(),
                client: Client::new(),
            },
        }
    }
}

#[async_trait]
impl crate::core::LanguageModel for XAIModel {
    #[tracing::instrument(skip(self, prompt), fields(model = options.model_id))]
    async fn generate(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> crate::core::Result<GenerateResult> {
        self.inner.generate(prompt, options).await
    }

    async fn generate_stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> crate::core::Result<BoxStream<'static, StreamPart>> {
        self.inner.generate_stream(prompt, options).await
    }
}

// --- Provider Factory ---

/// xAI provider with configurable settings.
pub struct XAIProvider {
    settings: ProviderSettings,
}

impl XAIProvider {
    /// Creates a chat language model.
    #[must_use]
    pub fn chat(&self, _model_id: &str) -> XAIModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("XAI_API_KEY").ok())
            .unwrap_or_default();
        let base_url = self
            .settings
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.x.ai/v1".to_string());
        XAIModel {
            inner: OpenAIModel {
                api_key,
                base_url,
                client: Client::new(),
            },
        }
    }

    /// Alias for `chat`.
    #[must_use]
    pub fn language_model(&self, model_id: &str) -> XAIModel {
        self.chat(model_id)
    }

    /// Creates an image generation model.
    #[must_use]
    pub fn image(&self, _model_id: &str) -> image::XaiImageModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("XAI_API_KEY").ok())
            .unwrap_or_default();
        let base_url = self
            .settings
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.x.ai/v1".to_string());
        image::XaiImageModel {
            api_key,
            base_url,
            client: Client::new(),
        }
    }

    /// Creates a Responses API model.
    #[must_use]
    pub fn responses(&self, _model_id: &str) -> responses::XaiResponsesModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("XAI_API_KEY").ok())
            .unwrap_or_default();
        responses::XaiResponsesModel::new(api_key)
    }

    /// Creates a speech (TTS) model.
    #[must_use]
    pub fn speech(&self, _model_id: &str) -> speech::XaiSpeechModel {
        let api_key = self.resolve_api_key();
        let base_url = self.resolve_base_url();
        speech::XaiSpeechModel::new(api_key, base_url)
    }

    /// Creates a transcription (STT) model.
    #[must_use]
    pub fn transcription(&self, _model_id: &str) -> transcription::XaiTranscriptionModel {
        let api_key = self.resolve_api_key();
        let base_url = self.resolve_base_url();
        transcription::XaiTranscriptionModel::new(api_key, base_url)
    }

    /// Creates a video generation model.
    #[must_use]
    pub fn video(&self, _model_id: &str) -> video::XaiVideoModel {
        let api_key = self.resolve_api_key();
        let base_url = self.resolve_base_url();
        video::XaiVideoModel::new(api_key, base_url)
    }

    fn resolve_api_key(&self) -> String {
        self.settings
            .api_key
            .clone()
            .or_else(|| std::env::var("XAI_API_KEY").ok())
            .unwrap_or_default()
    }

    fn resolve_base_url(&self) -> String {
        self.settings
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.x.ai/v1".to_string())
    }
}

/// Create an xAI provider instance with the given settings.
#[must_use]
pub fn create_xai(settings: ProviderSettings) -> XAIProvider {
    XAIProvider { settings }
}

impl crate::core::registry::Provider for XAIProvider {
    fn language_model(&self, model_id: &str) -> Option<Box<dyn crate::core::LanguageModel>> {
        Some(Box::new(self.chat(model_id)))
    }

    fn image_model(&self, model_id: &str) -> Option<Box<dyn crate::core::ImageModel>> {
        Some(Box::new(self.image(model_id)))
    }

    fn speech_model(&self, model_id: &str) -> Option<Box<dyn crate::core::SpeechModel>> {
        Some(Box::new(self.speech(model_id)))
    }

    fn transcription_model(
        &self,
        model_id: &str,
    ) -> Option<Box<dyn crate::core::TranscriptionModel>> {
        Some(Box::new(self.transcription(model_id)))
    }

    fn detailed_speech_model(&self, model_id: &str) -> Option<Box<dyn DetailedSpeechModel>> {
        Some(Box::new(self.speech(model_id)))
    }

    fn streaming_speech_model(&self, model_id: &str) -> Option<Box<dyn StreamingSpeechModel>> {
        Some(Box::new(self.speech(model_id)))
    }

    fn detailed_transcription_model(
        &self,
        model_id: &str,
    ) -> Option<Box<dyn DetailedTranscriptionModel>> {
        Some(Box::new(self.transcription(model_id)))
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            llm: true,
            batch_stt: true,
            streaming_stt: false,
            batch_tts: true,
            streaming_tts: true,
            diarization: true,
            word_timestamps: true,
            ssml: false,
            pitch: false,
        }
    }
}
