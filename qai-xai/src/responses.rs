//! xAI Responses API model.
//!
//! xAI uses the same Responses API shape as OpenAI, so this module wraps
//! the OpenAI implementation with xAI's base URL and API key.

use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
use qai_core::types::{GenerateOptions, GenerateResult, Prompt, StreamPart};
use qai_openai::responses::OpenAIResponsesModel;
use reqwest::Client;

/// xAI Responses API model wrapping OpenAI's Responses implementation.
pub struct XaiResponsesModel {
    pub inner: OpenAIResponsesModel,
}

impl XaiResponsesModel {
    pub fn new(api_key: String) -> Self {
        Self {
            inner: OpenAIResponsesModel {
                api_key,
                base_url: "https://api.x.ai/v1".to_string(),
                client: Client::new(),
            },
        }
    }
}

#[async_trait]
impl qai_core::LanguageModel for XaiResponsesModel {
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
