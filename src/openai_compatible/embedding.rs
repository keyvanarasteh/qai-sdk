use crate::core::types::{EmbeddingOptions, EmbeddingResult};
use crate::openai::embedding::OpenAIEmbeddingModel;
use async_trait::async_trait;
use reqwest::Client;

/// OpenAI-Compatible embedding model that delegates to the `OpenAI` embedding implementation.
pub struct OpenAICompatibleEmbeddingModel {
    pub inner: OpenAIEmbeddingModel,
}

impl OpenAICompatibleEmbeddingModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            inner: OpenAIEmbeddingModel {
                api_key,
                base_url,
                client: Client::new(),
            },
        }
    }
}

#[async_trait]
impl crate::core::EmbeddingModel for OpenAICompatibleEmbeddingModel {
    async fn embed(
        &self,
        values: Vec<String>,
        options: EmbeddingOptions,
    ) -> crate::core::Result<EmbeddingResult> {
        self.inner.embed(values, options).await
    }
}
