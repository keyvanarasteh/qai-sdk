use async_trait::async_trait;
use qai_core::types::{CompletionOptions, CompletionResult};
use qai_openai::completion::OpenAICompletionModel;
use reqwest::Client;

/// OpenAI-Compatible completion model delegating to the OpenAI implementation.
pub struct OpenAICompatibleCompletionModel {
    pub inner: OpenAICompletionModel,
}

impl OpenAICompatibleCompletionModel {
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
impl qai_core::CompletionModel for OpenAICompatibleCompletionModel {
    async fn complete(&self, options: CompletionOptions) -> qai_core::Result<CompletionResult> {
        self.inner.complete(options).await
    }
}
