//! # QAI `DeepSeek`
//!
//! `DeepSeek` provider for the QAI SDK. Provides access to DeepSeek-Chat
//! and DeepSeek-Reasoner models through an OpenAI-compatible API adapter.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use qai_sdk::deepseek::create_deepseek;
//! use qai_sdk::core::types::ProviderSettings;
//!
//! let provider = create_deepseek(ProviderSettings {
//!     api_key: Some("sk-...".to_string()),
//!     ..Default::default()
//! });
//!
//! let model = provider.chat("deepseek-chat");
//! ```

pub mod error;
pub mod types;

use crate::core::types::{GenerateOptions, GenerateResult, Prompt, ProviderSettings, StreamPart};
use crate::openai::OpenAIModel;
use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;

pub struct DeepSeekModel {
    pub inner: OpenAIModel,
}

impl DeepSeekModel {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            inner: OpenAIModel {
                api_key,
                base_url: "https://api.deepseek.com".to_string(),
                client: Client::new(),
            },
        }
    }
}

#[async_trait]
impl crate::core::LanguageModel for DeepSeekModel {
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

/// `DeepSeek` provider with configurable settings.
pub struct DeepSeekProvider {
    settings: ProviderSettings,
}

impl DeepSeekProvider {
    /// Creates a chat language model.
    #[must_use]
    pub fn chat(&self, _model_id: &str) -> DeepSeekModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("DEEPSEEK_API_KEY").ok())
            .unwrap_or_default();
        let base_url = self
            .settings
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.deepseek.com".to_string());
        DeepSeekModel {
            inner: OpenAIModel {
                api_key,
                base_url,
                client: Client::new(),
            },
        }
    }

    /// Alias for `chat`.
    #[must_use]
    pub fn language_model(&self, model_id: &str) -> DeepSeekModel {
        self.chat(model_id)
    }
}

/// Create a `DeepSeek` provider instance with the given settings.
#[must_use]
pub fn create_deepseek(settings: ProviderSettings) -> DeepSeekProvider {
    DeepSeekProvider { settings }
}

impl crate::core::registry::Provider for DeepSeekProvider {
    fn language_model(&self, model_id: &str) -> Option<Box<dyn crate::core::LanguageModel>> {
        Some(Box::new(self.chat(model_id)))
    }
}
