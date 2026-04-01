//! # QAI xAI
//!
//! xAI Grok provider for the QAI SDK. Provides access to Grok models
//! through an OpenAI-compatible API adapter, with additional support for
//! image generation and the Responses API.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use qai_xai::create_xai;
//! use qai_core::types::ProviderSettings;
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
pub mod responses;
pub mod tools;
pub mod types;

use qai_core::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
use qai_core::types::{GenerateOptions, GenerateResult, Prompt, ProviderSettings, StreamPart};
use qai_openai::OpenAIModel;
use reqwest::Client;

pub struct XAIModel {
    pub inner: OpenAIModel,
}

impl XAIModel {
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
impl qai_core::LanguageModel for XAIModel {
    #[tracing::instrument(skip(self, prompt), fields(model = options.model_id))]
    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> qai_core::Result<GenerateResult> {
        self.inner.generate(prompt, options).await
    }

    async fn generate_stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> qai_core::Result<BoxStream<'static, StreamPart>> {
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
    pub fn language_model(&self, model_id: &str) -> XAIModel {
        self.chat(model_id)
    }

    /// Creates an image generation model.
    pub fn image(&self, _model_id: &str) -> crate::image::XaiImageModel {
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
        crate::image::XaiImageModel {
            api_key,
            base_url,
            client: Client::new(),
        }
    }

    /// Creates a Responses API model.
    pub fn responses(&self, _model_id: &str) -> crate::responses::XaiResponsesModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("XAI_API_KEY").ok())
            .unwrap_or_default();
        crate::responses::XaiResponsesModel::new(api_key)
    }
}

/// Create an xAI provider instance with the given settings.
pub fn create_xai(settings: ProviderSettings) -> XAIProvider {
    XAIProvider { settings }
}
