use async_trait::async_trait;
use qai_core::types::{GenerateOptions, GenerateResult, Prompt};
use qai_openai::OpenAIModel;
use anyhow::Result;
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
    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> Result<GenerateResult> {
        self.inner.generate(prompt, options).await
    }
}
