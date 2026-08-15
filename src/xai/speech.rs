//! xAI text-to-speech adapter using xAI's native `/v1/tts` endpoint.

use crate::core::audio::{
    AudioChunk, AudioEncoding, AudioSpec, DetailedSpeechModel, DetailedSpeechResult, SpeechRequest,
    StreamingSpeechModel,
};
use crate::core::types::{SpeechOptions, SpeechResult};
use anyhow::anyhow;
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures_util::StreamExt;
use reqwest::Client;
use serde::Serialize;

pub struct XaiSpeechModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl XaiSpeechModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: Client::new(),
        }
    }

    async fn request(&self, request: SpeechRequest) -> crate::core::Result<reqwest::Response> {
        if request.pitch.is_some() || request.ssml.is_some() {
            return Err(
                anyhow!("xAI TTS does not support pitch or SSML on the HTTP TTS endpoint").into(),
            );
        }
        let format = match request.audio.encoding {
            AudioEncoding::Pcm16 => "pcm",
            AudioEncoding::Wav => "wav",
            AudioEncoding::Mp3 => "mp3",
            AudioEncoding::Opus => "opus",
            AudioEncoding::Flac => "flac",
            AudioEncoding::Mulaw => "mulaw",
            AudioEncoding::Alaw => "alaw",
        };
        let payload = XaiTtsRequest {
            model: request.model_id,
            input: request.input,
            voice: request.voice,
            response_format: format,
            speed: request.rate,
        };
        let response = self
            .client
            .post(format!("{}/tts", self.base_url.trim_end_matches('/')))
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(anyhow!("xAI TTS API error: {}", response.text().await?).into());
        }
        Ok(response)
    }
}

#[derive(Serialize)]
struct XaiTtsRequest {
    model: String,
    input: String,
    voice: String,
    response_format: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    speed: Option<f32>,
}

#[async_trait]
impl crate::core::SpeechModel for XaiSpeechModel {
    async fn synthesize(&self, options: SpeechOptions) -> crate::core::Result<SpeechResult> {
        self.synthesize_detailed(SpeechRequest {
            model_id: options.model_id,
            input: options.input,
            voice: options.voice,
            audio: AudioSpec::wav(),
            rate: options.speed,
            pitch: None,
            ssml: None,
        })
        .await
        .map(Into::into)
    }
}

#[async_trait]
impl DetailedSpeechModel for XaiSpeechModel {
    async fn synthesize_detailed(
        &self,
        request: SpeechRequest,
    ) -> crate::core::Result<DetailedSpeechResult> {
        let audio_spec = request.audio.clone();
        Ok(DetailedSpeechResult {
            audio: self.request(request).await?.bytes().await?.to_vec(),
            audio_spec,
        })
    }
}

#[async_trait]
impl StreamingSpeechModel for XaiSpeechModel {
    async fn synthesize_stream(
        &self,
        request: SpeechRequest,
    ) -> crate::core::Result<BoxStream<'static, crate::core::Result<AudioChunk>>> {
        let audio_spec = request.audio.clone();
        let stream = self
            .request(request)
            .await?
            .bytes_stream()
            .map(move |chunk| {
                chunk
                    .map(|data| AudioChunk {
                        data: data.to_vec(),
                        is_final: false,
                        audio_spec: audio_spec.clone(),
                    })
                    .map_err(Into::into)
            });
        Ok(Box::pin(stream))
    }
}
