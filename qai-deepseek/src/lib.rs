pub mod types;

use async_trait::async_trait;
use qai_core::types::{GenerateOptions, GenerateResult, Prompt, StreamPart};
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
