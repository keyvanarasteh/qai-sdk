//! # QAI Core
//!
//! Core traits and types for the QAI SDK ecosystem.
//!
//! This crate defines the provider-agnostic interfaces that every AI provider
//! implements:
//!
//! - [`LanguageModel`] — Chat and text generation (with optional streaming)
//! - [`EmbeddingModel`] — Vector embeddings for text
//! - [`ImageModel`] — Image generation from text prompts
//! - [`CompletionModel`] — Legacy text completion
//! - [`SpeechModel`] — Text-to-speech synthesis
//! - [`TranscriptionModel`] — Speech-to-text transcription
//!
//! All shared types live in the [`types`] module.

pub mod error;
pub mod agent;
pub mod middleware;
pub mod registry;
pub mod structured;
#[cfg(test)]
mod tests;
pub mod types;

pub use error::ProviderError;
pub type Result<T> = std::result::Result<T, ProviderError>;

use crate::core::types::{
    CompletionOptions, CompletionResult, EmbeddingOptions, EmbeddingResult, GenerateOptions,
    GenerateResult, ImageGenerateOptions, ImageGenerateResult, Prompt, SpeechOptions, SpeechResult,
    StreamPart, TranscriptionOptions, TranscriptionResult, VideoGenerateOptions, VideoGenerateResult,
    MusicGenerateOptions, MusicGenerateResult,
    RealtimeEvent,
};
use async_trait::async_trait;
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
        Err(ProviderError::NotSupported(
            "Streaming not implemented for this model".to_string(),
        ))
    }
}

#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    /// Generates embeddings for the given input values.
    async fn embed(
        &self,
        values: Vec<String>,
        options: EmbeddingOptions,
    ) -> Result<EmbeddingResult>;
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

#[async_trait]
pub trait VideoModel: Send + Sync {
    /// Generates videos from a text prompt.
    async fn generate(&self, options: VideoGenerateOptions) -> Result<VideoGenerateResult>;
}

#[async_trait]
pub trait MusicModel: Send + Sync {
    /// Generates music from a text prompt.
    async fn generate(&self, options: MusicGenerateOptions) -> Result<MusicGenerateResult>;
}

#[async_trait]
pub trait RealtimeModel: Send + Sync {
    /// Connects to a real-time WebSocket session.
    async fn connect(&self) -> Result<BoxStream<'static, RealtimeEvent>>;

    /// Sends an event to the real-time session.
    async fn send(&self, event: RealtimeEvent) -> Result<()>;
}
