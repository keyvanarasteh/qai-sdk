//! xAI Responses API model.
//!
//! xAI uses the same Responses API shape as `OpenAI`, so this module wraps
//! the `OpenAI` implementation with xAI's base URL and API key.

use crate::core::types::{GenerateOptions, GenerateResult, Prompt, StreamPart};
use crate::openai::responses::OpenAIResponsesModel;
use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;

/// xAI Responses API model wrapping `OpenAI`'s Responses implementation.
pub struct XaiResponsesModel {
    pub inner: OpenAIResponsesModel,
}

impl XaiResponsesModel {
    #[must_use]
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
impl crate::core::LanguageModel for XaiResponsesModel {
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
