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

/// `OpenAI` speech (text-to-speech) model.
pub struct OpenAISpeechModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl OpenAISpeechModel {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: Client::new(),
        }
    }

    async fn synthesize_request(
        &self,
        request: SpeechRequest,
    ) -> crate::core::Result<reqwest::Response> {
        if request.pitch.is_some() || request.ssml.is_some() {
            return Err(anyhow!("OpenAI speech does not support pitch or SSML").into());
        }
        let response_format = match request.audio.encoding {
            AudioEncoding::Pcm16 => "pcm",
            AudioEncoding::Wav => "wav",
            AudioEncoding::Mp3 => "mp3",
            AudioEncoding::Opus => "opus",
            AudioEncoding::Flac => "flac",
            AudioEncoding::Mulaw => "pcm",
            AudioEncoding::Alaw => "pcm",
        };
        let request = OpenAISpeechRequest {
            model: request.model_id,
            input: request.input,
            voice: request.voice,
            response_format: Some(response_format.to_string()),
            speed: request.rate,
        };
        let response = self
            .client
            .post(format!("{}/audio/speech", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(anyhow!("OpenAI Speech API error: {}", response.text().await?).into());
        }
        Ok(response)
    }
}

#[derive(Serialize)]
struct OpenAISpeechRequest {
    model: String,
    input: String,
    voice: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    speed: Option<f32>,
}

#[async_trait]
impl crate::core::SpeechModel for OpenAISpeechModel {
    async fn synthesize(&self, options: SpeechOptions) -> crate::core::Result<SpeechResult> {
        self.synthesize_detailed(SpeechRequest {
            model_id: options.model_id,
            input: options.input,
            voice: options.voice,
            audio: AudioSpec {
                encoding: match options.response_format.as_deref() {
                    Some("pcm") => AudioEncoding::Pcm16,
                    Some("wav") | None => AudioEncoding::Wav,
                    Some("opus") => AudioEncoding::Opus,
                    Some("flac") => AudioEncoding::Flac,
                    _ => AudioEncoding::Mp3,
                },
                sample_rate_hz: None,
                channels: None,
                content_type: None,
            },
            rate: options.speed,
            pitch: None,
            ssml: None,
        })
        .await
        .map(Into::into)
    }
}

#[async_trait]
impl DetailedSpeechModel for OpenAISpeechModel {
    async fn synthesize_detailed(
        &self,
        request: SpeechRequest,
    ) -> crate::core::Result<DetailedSpeechResult> {
        let audio_spec = request.audio.clone();
        let audio = self
            .synthesize_request(request)
            .await?
            .bytes()
            .await?
            .to_vec();
        Ok(DetailedSpeechResult { audio, audio_spec })
    }
}

#[async_trait]
impl StreamingSpeechModel for OpenAISpeechModel {
    async fn synthesize_stream(
        &self,
        request: SpeechRequest,
    ) -> crate::core::Result<BoxStream<'static, crate::core::Result<AudioChunk>>> {
        let audio_spec = request.audio.clone();
        let stream = self
            .synthesize_request(request)
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
