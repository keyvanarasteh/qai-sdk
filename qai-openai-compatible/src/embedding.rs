use async_trait::async_trait;
use qai_core::types::{EmbeddingOptions, EmbeddingResult};
use anyhow::Result;
use qai_openai::embedding::OpenAIEmbeddingModel;
use reqwest::Client;

/// OpenAI-Compatible embedding model that delegates to the OpenAI embedding implementation.
pub struct OpenAICompatibleEmbeddingModel {
    pub inner: OpenAIEmbeddingModel,
}

impl OpenAICompatibleEmbeddingModel {
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
impl qai_core::EmbeddingModel for OpenAICompatibleEmbeddingModel {
    async fn embed(&self, values: Vec<String>, options: EmbeddingOptions) -> Result<EmbeddingResult> {
        self.inner.embed(values, options).await
    }
}
