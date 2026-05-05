//! # xAI Transcription (STT)
//!
//! xAI Transcription provider implementation, compatible with OpenAI Whisper API.

use crate::core::types::{TranscriptionOptions, TranscriptionResult};
use crate::openai::transcription::OpenAITranscriptionModel;
use async_trait::async_trait;
use reqwest::Client;

/// xAI transcription (speech-to-text) model.
pub struct XaiTranscriptionModel {
    pub inner: OpenAITranscriptionModel,
}

impl XaiTranscriptionModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            inner: OpenAITranscriptionModel {
                api_key,
                base_url,
                client: Client::new(),
            },
        }
    }
}

#[async_trait]
impl crate::core::TranscriptionModel for XaiTranscriptionModel {
    async fn transcribe(&self, options: TranscriptionOptions) -> crate::core::Result<TranscriptionResult> {
        self.inner.transcribe(options).await
    }
}
