pub mod types;
#[cfg(test)]
mod tests;

use async_trait::async_trait;
use crate::types::{
    GenerateOptions, GenerateResult, Prompt, StreamPart,
    EmbeddingOptions, EmbeddingResult,
    ImageGenerateOptions, ImageGenerateResult,
    CompletionOptions, CompletionResult,
    SpeechOptions, SpeechResult,
    TranscriptionOptions, TranscriptionResult,
};
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

#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    /// Generates embeddings for the given input values.
    async fn embed(&self, values: Vec<String>, options: EmbeddingOptions) -> Result<EmbeddingResult>;
}

#[async_trait]
pub trait ImageModel: Send + Sync {
    /// Generates images from a text prompt.
    async fn generate(&self, options: ImageGenerateOptions) -> Result<ImageGenerateResult>;
}

#[async_trait]
pub trait CompletionModel: Send + Sync {
    /// Generates a text completion.
    async fn complete(&self, options: CompletionOptions) -> Result<CompletionResult>;
}

#[async_trait]
pub trait SpeechModel: Send + Sync {
    /// Generates speech audio from text input.
    async fn synthesize(&self, options: SpeechOptions) -> Result<SpeechResult>;
}

#[async_trait]
pub trait TranscriptionModel: Send + Sync {
    /// Transcribes audio to text.
    async fn transcribe(&self, options: TranscriptionOptions) -> Result<TranscriptionResult>;
}
