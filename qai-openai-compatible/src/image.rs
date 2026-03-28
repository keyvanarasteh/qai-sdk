use async_trait::async_trait;
use qai_core::types::{ImageGenerateOptions, ImageGenerateResult};
use anyhow::Result;
use qai_openai::image::OpenAIImageModel;
use reqwest::Client;

/// OpenAI-Compatible image generation model delegating to the OpenAI implementation.
pub struct OpenAICompatibleImageModel {
    pub inner: OpenAIImageModel,
}

impl OpenAICompatibleImageModel {
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
impl qai_core::ImageModel for OpenAICompatibleImageModel {
    async fn generate(&self, options: ImageGenerateOptions) -> Result<ImageGenerateResult> {
        self.inner.generate(options).await
    }
}
