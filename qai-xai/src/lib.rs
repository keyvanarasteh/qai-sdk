pub mod image;

use async_trait::async_trait;
use qai_core::types::{GenerateOptions, GenerateResult, Prompt, StreamPart, ProviderSettings};
use qai_openai::OpenAIModel;
use anyhow::Result;
use futures::stream::BoxStream;
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
    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> Result<GenerateResult> {
        self.inner.generate(prompt, options).await
    }

    async fn generate_stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> Result<BoxStream<'static, StreamPart>> {
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
        let api_key = self.settings.api_key.clone()
            .or_else(|| std::env::var("XAI_API_KEY").ok())
            .unwrap_or_default();
        let base_url = self.settings.base_url.clone()
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
}

/// Create an xAI provider instance with the given settings.
pub fn create_xai(settings: ProviderSettings) -> XAIProvider {
    XAIProvider { settings }
}
