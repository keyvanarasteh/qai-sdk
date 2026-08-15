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

/// `OpenAI` transcription (speech-to-text) model.
pub struct OpenAITranscriptionModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl OpenAITranscriptionModel {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct OpenAITranscriptionResponse {
    text: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    words: Vec<OpenAIWord>,
    #[serde(default)]
    segments: Vec<OpenAISegment>,
}

#[derive(Deserialize)]
struct OpenAIWord {
    #[serde(alias = "word")]
    text: String,
    start: Option<f64>,
    end: Option<f64>,
    #[serde(default)]
    probability: Option<f32>,
}

#[derive(Deserialize)]
struct OpenAISegment {
    text: String,
    start: Option<f64>,
    end: Option<f64>,
    #[serde(default)]
    avg_logprob: Option<f32>,
}

fn millis(seconds: Option<f64>) -> Option<u64> {
    seconds
        .and_then(|value| (value.is_finite() && value >= 0.0).then_some((value * 1_000.0) as u64))
}

impl OpenAITranscriptionModel {
    async fn transcribe_request(
        &self,
        request: TranscriptionRequest,
    ) -> crate::core::Result<DetailedTranscriptionResult> {
        let mime = request.media_type.as_deref().unwrap_or("audio/mpeg");
        let audio_part = Part::bytes(request.audio)
            .file_name("audio")
            .mime_str(mime)?;
        let mut form = Form::new()
            .text("model", request.model_id)
            .text("response_format", "verbose_json")
            .part("file", audio_part);

        if let Some(language) = request.language {
            form = form.text("language", language);
        }
        let prompt = request
            .prompt
            .or_else(|| (!request.vocabulary.is_empty()).then(|| request.vocabulary.join(", ")));
        if let Some(prompt) = prompt {
            form = form.text("prompt", prompt);
        }
        if request.word_timestamps || request.segment_timestamps {
            if request.word_timestamps {
                form = form.text("timestamp_granularities[]", "word");
            }
            if request.segment_timestamps {
                form = form.text("timestamp_granularities[]", "segment");
            }
        }

        let resp = self
            .client
            .post(format!("{}/audio/transcriptions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow!("OpenAI Transcription API error: {}", resp.text().await?).into());
        }
        let transcription: OpenAITranscriptionResponse = resp.json().await?;
        let words = transcription
            .words
            .into_iter()
            .map(|word| TranscriptWord {
                text: word.text,
                start_ms: millis(word.start),
                end_ms: millis(word.end),
                confidence: word.probability,
                speaker: None,
                language: None,
            })
            .collect();
        let segments = transcription
            .segments
            .into_iter()
            .map(|segment| TranscriptSegment {
                text: segment.text,
                start_ms: millis(segment.start),
                end_ms: millis(segment.end),
                confidence: segment.avg_logprob.map(f32::exp),
                speaker: None,
                words: Vec::new(),
            })
            .collect();
        Ok(DetailedTranscriptionResult {
            text: transcription.text,
            language: transcription.language,
            duration_ms: millis(transcription.duration),
            words,
            segments,
        })
    }
}

#[async_trait]
impl crate::core::TranscriptionModel for OpenAITranscriptionModel {
    async fn transcribe(
        &self,
        options: TranscriptionOptions,
    ) -> crate::core::Result<TranscriptionResult> {
        let _ = options.temperature;
        self.transcribe_request(TranscriptionRequest {
            model_id: options.model_id,
            audio: options.audio,
            media_type: Some("audio/mpeg".into()),
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
impl DetailedTranscriptionModel for OpenAITranscriptionModel {
    async fn transcribe_detailed(
        &self,
        request: TranscriptionRequest,
    ) -> crate::core::Result<DetailedTranscriptionResult> {
        self.transcribe_request(request).await
    }
}
