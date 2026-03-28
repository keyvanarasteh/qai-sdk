pub mod types;
pub mod error;

use async_trait::async_trait;
use qai_core::types::{GenerateOptions, GenerateResult, Prompt, StreamPart, ProviderSettings};
use qai_openai::OpenAIModel;
use anyhow::Result;
use reqwest::Client;
use futures::stream::BoxStream;

pub struct DeepSeekModel {
    pub inner: OpenAIModel,
}

impl DeepSeekModel {
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
impl qai_core::LanguageModel for DeepSeekModel {
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

/// DeepSeek provider with configurable settings.
pub struct DeepSeekProvider {
    settings: ProviderSettings,
}

impl DeepSeekProvider {
    /// Creates a chat language model.
    pub fn chat(&self, _model_id: &str) -> DeepSeekModel {
        let api_key = self.settings.api_key.clone()
            .or_else(|| std::env::var("DEEPSEEK_API_KEY").ok())
            .unwrap_or_default();
        let base_url = self.settings.base_url.clone()
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
    pub fn language_model(&self, model_id: &str) -> DeepSeekModel {
        self.chat(model_id)
    }
}

/// Create a DeepSeek provider instance with the given settings.
pub fn create_deepseek(settings: ProviderSettings) -> DeepSeekProvider {
    DeepSeekProvider { settings }
}
