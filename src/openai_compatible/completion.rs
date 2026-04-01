use crate::core::types::{CompletionOptions, CompletionResult};
use crate::openai::completion::OpenAICompletionModel;
use async_trait::async_trait;
use reqwest::Client;

/// OpenAI-Compatible completion model delegating to the `OpenAI` implementation.
pub struct OpenAICompatibleCompletionModel {
    pub inner: OpenAICompletionModel,
}

impl OpenAICompatibleCompletionModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            inner: OpenAICompletionModel {
                api_key,
                base_url,
                client: Client::new(),
            },
        }
    }
}

#[async_trait]
impl crate::core::CompletionModel for OpenAICompatibleCompletionModel {
    async fn complete(&self, options: CompletionOptions) -> crate::core::Result<CompletionResult> {
        self.inner.complete(options).await
    }
}
