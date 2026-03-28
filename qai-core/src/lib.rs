pub mod types;

use async_trait::async_trait;
use crate::types::{GenerateOptions, GenerateResult, Prompt, StreamPart};
use anyhow::Result;
use futures::stream::BoxStream;

#[async_trait]
pub trait LanguageModel: Send + Sync {
    /// Generates a single response from the model.
    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> Result<GenerateResult>;

    /// Generates a streaming response from the model.
    async fn generate_stream(
        &self,
        _prompt: Prompt,
        _options: GenerateOptions,
    ) -> Result<BoxStream<'static, StreamPart>> {
        Err(anyhow::anyhow!("Streaming not implemented for this model"))
    }
}
