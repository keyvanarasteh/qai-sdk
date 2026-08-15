//! xAI speech-to-text adapter.
//!
//! xAI's audio API is not OpenAI's `/audio/transcriptions` surface.  Keep the
//! wire contract here so an xAI base URL can never accidentally be sent to an
//! OpenAI endpoint.

use crate::core::audio::{
    DetailedTranscriptionModel, DetailedTranscriptionResult, TranscriptSegment, TranscriptWord,
    TranscriptionRequest,
};
use crate::core::types::{TranscriptionOptions, TranscriptionResult};
use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::Deserialize;

pub struct XaiTranscriptionModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl XaiTranscriptionModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: Client::new(),
        }
    }

    async fn request(
        &self,
        request: TranscriptionRequest,
    ) -> crate::core::Result<DetailedTranscriptionResult> {
        let mime = request.media_type.as_deref().unwrap_or("audio/wav");
        let audio = Part::bytes(request.audio)
            .file_name("audio")
            .mime_str(mime)?;
        let mut form = Form::new()
            .text("model", request.model_id)
            .part("file", audio);
        if let Some(language) = request.language {
            form = form.text("language", language);
        }
        if let Some(prompt) = request.prompt {
            form = form.text("prompt", prompt);
        }
        if !request.vocabulary.is_empty() {
            form = form.text("vocabulary", request.vocabulary.join(","));
        }
        if request.diarize {
            form = form.text("diarize", "true");
        }
        let response = self
            .client
            .post(format!("{}/stt", self.base_url.trim_end_matches('/')))
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(anyhow!("xAI STT API error: {}", response.text().await?).into());
        }
        let body: XaiSttResponse = response.json().await?;
        Ok(DetailedTranscriptionResult {
            text: body.text,
            language: body.language,
            duration_ms: body.duration.map(|value| (value * 1_000.0) as u64),
            words: body.words.into_iter().map(Into::into).collect(),
            segments: body.segments.into_iter().map(Into::into).collect(),
        })
    }
}

#[derive(Deserialize)]
struct XaiSttResponse {
    text: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    words: Vec<XaiWord>,
    #[serde(default)]
    segments: Vec<XaiSegment>,
}

#[derive(Deserialize)]
struct XaiWord {
    #[serde(alias = "word")]
    text: String,
    #[serde(default)]
    start: Option<f64>,
    #[serde(default)]
    end: Option<f64>,
    #[serde(default)]
    confidence: Option<f32>,
    #[serde(default)]
    speaker: Option<String>,
}

impl From<XaiWord> for TranscriptWord {
    fn from(value: XaiWord) -> Self {
        Self {
            text: value.text,
            start_ms: value.start.map(|v| (v * 1_000.0) as u64),
            end_ms: value.end.map(|v| (v * 1_000.0) as u64),
            confidence: value.confidence,
            speaker: value.speaker,
            language: None,
        }
    }
}

#[derive(Deserialize)]
struct XaiSegment {
    text: String,
    #[serde(default)]
    start: Option<f64>,
    #[serde(default)]
    end: Option<f64>,
    #[serde(default)]
    confidence: Option<f32>,
    #[serde(default)]
    speaker: Option<String>,
}

impl From<XaiSegment> for TranscriptSegment {
    fn from(value: XaiSegment) -> Self {
        Self {
            text: value.text,
            start_ms: value.start.map(|v| (v * 1_000.0) as u64),
            end_ms: value.end.map(|v| (v * 1_000.0) as u64),
            confidence: value.confidence,
            speaker: value.speaker,
            words: Vec::new(),
        }
    }
}

#[async_trait]
impl crate::core::TranscriptionModel for XaiTranscriptionModel {
    async fn transcribe(
        &self,
        options: TranscriptionOptions,
    ) -> crate::core::Result<TranscriptionResult> {
        self.request(TranscriptionRequest {
            model_id: options.model_id,
            audio: options.audio,
            media_type: Some("audio/wav".into()),
            language: options.language,
            prompt: options.prompt,
            vocabulary: Vec::new(),
            diarize: false,
            word_timestamps: false,
            segment_timestamps: false,
            channels: None,
            sample_rate_hz: None,
        })
        .await
        .map(Into::into)
    }
}

#[async_trait]
impl DetailedTranscriptionModel for XaiTranscriptionModel {
    async fn transcribe_detailed(
        &self,
        request: TranscriptionRequest,
    ) -> crate::core::Result<DetailedTranscriptionResult> {
        self.request(request).await
    }
}
