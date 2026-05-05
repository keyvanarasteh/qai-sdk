//! # xAI Speech (TTS)
//!
//! xAI Speech provider implementation, compatible with OpenAI TTS API.

use crate::core::types::{SpeechOptions, SpeechResult};
use crate::openai::speech::OpenAISpeechModel;
use async_trait::async_trait;
use reqwest::Client;

/// xAI speech (text-to-speech) model.
pub struct XaiSpeechModel {
    pub inner: OpenAISpeechModel,
}

impl XaiSpeechModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            inner: OpenAISpeechModel {
                api_key,
                base_url,
                client: Client::new(),
            },
        }
    }
}

#[async_trait]
impl crate::core::SpeechModel for XaiSpeechModel {
    async fn synthesize(&self, options: SpeechOptions) -> crate::core::Result<SpeechResult> {
        self.inner.synthesize(options).await
    }
}
