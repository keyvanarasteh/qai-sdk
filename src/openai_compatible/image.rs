use crate::core::types::{ImageGenerateOptions, ImageGenerateResult};
use crate::openai::image::OpenAIImageModel;
use async_trait::async_trait;
use reqwest::Client;

/// OpenAI-Compatible image generation model delegating to the `OpenAI` implementation.
pub struct OpenAICompatibleImageModel {
    pub inner: OpenAIImageModel,
}

impl OpenAICompatibleImageModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            inner: OpenAIImageModel {
                api_key,
                base_url,
                client: Client::new(),
            },
        }
    }
}

#[async_trait]
impl crate::core::ImageModel for OpenAICompatibleImageModel {
    async fn generate(
        &self,
        options: ImageGenerateOptions,
    ) -> crate::core::Result<ImageGenerateResult> {
        self.inner.generate(options).await
    }
}
