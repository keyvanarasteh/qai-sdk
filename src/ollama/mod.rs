//! # QAI `Ollama`
//!
//! `Ollama` provider for the QAI SDK. Uses Ollama's recommended `/v1`
//! OpenAI-compatible API layer, guaranteeing native support for tool calls,
//! structured outputs, and streaming across both local servers and Ollama Cloud.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use qai_sdk::ollama::create_ollama;
//! use qai_sdk::core::types::ProviderSettings;
//!
//! // Automatically targets http://localhost:11434/v1 by default
//! let provider = create_ollama(ProviderSettings::default());
//!
//! let model = provider.chat("llama3.2");
//! ```

pub mod types;

use crate::core::types::{GenerateOptions, GenerateResult, Prompt, ProviderSettings, StreamPart};
use crate::openai::OpenAIModel;
use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;

pub struct OllamaModel {
    pub inner: OpenAIModel,
}

impl OllamaModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            inner: OpenAIModel {
                api_key,
                base_url,
                client: Client::new(),
            },
        }
    }
}

#[async_trait]
impl crate::core::LanguageModel for OllamaModel {
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

#[async_trait]
impl crate::core::EmbeddingModel for OllamaModel {
    #[tracing::instrument(skip(self, texts), fields(model = options.model_id))]
    async fn embed(
        &self,
        texts: Vec<String>,
        options: crate::core::types::EmbeddingOptions,
    ) -> crate::core::Result<crate::core::types::EmbeddingResult> {
        let embedding_model = crate::openai::embedding::OpenAIEmbeddingModel {
            api_key: self.inner.api_key.clone(),
            base_url: self.inner.base_url.clone(),
            client: self.inner.client.clone(),
        };
        embedding_model.embed(texts, options).await
    }
}

// --- Provider Factory ---

/// `Ollama` provider with configurable settings.
pub struct OllamaProvider {
    settings: ProviderSettings,
}

impl OllamaProvider {
    /// Creates a chat language model.
    #[must_use]
    pub fn chat(&self, _model_id: &str) -> OllamaModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("OLLAMA_API_KEY").ok())
            .unwrap_or_default();

        let base_url = self
            .settings
            .base_url
            .clone()
            .or_else(|| std::env::var("OLLAMA_BASE_URL").ok())
            .unwrap_or_else(|| {
                if !api_key.is_empty() {
                    types::DEFAULT_OLLAMA_CLOUD_URL.to_string()
                } else {
                    types::DEFAULT_OLLAMA_LOCAL_URL.to_string()
                }
            });

        OllamaModel::new(api_key, base_url)
    }

    /// Alias for `chat`.
    #[must_use]
    pub fn language_model(&self, model_id: &str) -> OllamaModel {
        self.chat(model_id)
    }

    /// Creates an embedding model.
    #[must_use]
    pub fn embedding(&self, model_id: &str) -> OllamaModel {
        self.chat(model_id)
    }
}

/// Create an `Ollama` provider instance with the given settings.
#[must_use]
pub fn create_ollama(settings: ProviderSettings) -> OllamaProvider {
    OllamaProvider { settings }
}

impl crate::core::registry::Provider for OllamaProvider {
    fn language_model(&self, model_id: &str) -> Option<Box<dyn crate::core::LanguageModel>> {
        Some(Box::new(self.chat(model_id)))
    }

    fn embedding_model(&self, model_id: &str) -> Option<Box<dyn crate::core::EmbeddingModel>> {
        Some(Box::new(self.embedding(model_id)))
    }
}
