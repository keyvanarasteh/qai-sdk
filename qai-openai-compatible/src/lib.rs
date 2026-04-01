//! # QAI OpenAI Compatible
//!
//! Provider for any OpenAI-compatible API endpoint. Use this to connect to
//! Ollama, LM Studio, vLLM, Together AI, Groq, or any other service that
//! implements the OpenAI chat completions format.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use qai_openai_compatible::{create_openai_compatible, OpenAICompatibleProviderSettings};
//!
//! let provider = create_openai_compatible(OpenAICompatibleProviderSettings {
//!     base_url: "http://localhost:11434/v1".to_string(),
//!     name: "ollama".to_string(),
//!     api_key: None,
//!     headers: None,
//! });
//!
//! let model = provider.chat("llama3");
//! ```

pub mod completion;
pub mod embedding;
pub mod error;
pub mod image;
pub mod types;

use async_trait::async_trait;
use futures::stream::BoxStream;
use qai_core::types::{GenerateOptions, GenerateResult, Prompt, StreamPart};
use qai_openai::OpenAIModel;
use reqwest::Client;

pub struct OpenAICompatibleModel {
    pub inner: OpenAIModel,
}

impl OpenAICompatibleModel {
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
impl qai_core::LanguageModel for OpenAICompatibleModel {
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

/// OpenAI-compatible provider settings.
#[derive(Debug, Clone)]
pub struct OpenAICompatibleProviderSettings {
    /// Base URL for the API (required).
    pub base_url: String,
    /// Provider name identifier.
    pub name: String,
    /// API key for authentication.
    pub api_key: Option<String>,
    /// Custom headers to include in requests.
    pub headers: Option<std::collections::HashMap<String, String>>,
}

/// OpenAI-compatible provider with configurable settings.
pub struct OpenAICompatibleProvider {
    settings: OpenAICompatibleProviderSettings,
}

impl OpenAICompatibleProvider {
    /// Creates a chat language model.
    pub fn chat(&self, _model_id: &str) -> OpenAICompatibleModel {
        let api_key = self.settings.api_key.clone().unwrap_or_default();
        OpenAICompatibleModel::new(api_key, self.settings.base_url.clone())
    }

    /// Alias for `chat`.
    pub fn language_model(&self, model_id: &str) -> OpenAICompatibleModel {
        self.chat(model_id)
    }

    /// Creates an embedding model.
    pub fn embedding(&self, _model_id: &str) -> crate::embedding::OpenAICompatibleEmbeddingModel {
        let api_key = self.settings.api_key.clone().unwrap_or_default();
        crate::embedding::OpenAICompatibleEmbeddingModel::new(
            api_key,
            self.settings.base_url.clone(),
        )
    }

    /// Creates an image generation model.
    pub fn image(&self, _model_id: &str) -> crate::image::OpenAICompatibleImageModel {
        let api_key = self.settings.api_key.clone().unwrap_or_default();
        crate::image::OpenAICompatibleImageModel::new(api_key, self.settings.base_url.clone())
    }

    /// Creates a completion model.
    pub fn completion(
        &self,
        _model_id: &str,
    ) -> crate::completion::OpenAICompatibleCompletionModel {
        let api_key = self.settings.api_key.clone().unwrap_or_default();
        crate::completion::OpenAICompatibleCompletionModel::new(
            api_key,
            self.settings.base_url.clone(),
        )
    }
}

/// Create an OpenAI-compatible provider instance with the given settings.
pub fn create_openai_compatible(
    settings: OpenAICompatibleProviderSettings,
) -> OpenAICompatibleProvider {
    OpenAICompatibleProvider { settings }
}
