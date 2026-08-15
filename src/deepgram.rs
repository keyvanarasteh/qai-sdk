//! Deepgram speech-to-text and text-to-speech provider.
//!
//! Deepgram is intentionally a speech-only provider: its `Provider`
//! implementation leaves `language_model` unset rather than pretending it can
//! serve an LLM request.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures_util::StreamExt;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::Deserialize;

use crate::core::audio::{
    AudioChunk, AudioEncoding, AudioSpec, DetailedSpeechModel, DetailedSpeechResult,
    DetailedTranscriptionModel, DetailedTranscriptionResult, ProviderCapabilities, SpeechRequest,
    StreamingSpeechModel, TranscriptSegment, TranscriptWord, TranscriptionRequest,
};
use crate::core::types::{
    ProviderSettings, SpeechOptions, SpeechResult, TranscriptionOptions, TranscriptionResult,
};

const BASE_URL: &str = "https://api.deepgram.com/v1";

pub struct DeepgramProvider {
    settings: ProviderSettings,
}

#[must_use]
pub fn create_deepgram(settings: ProviderSettings) -> DeepgramProvider {
    DeepgramProvider { settings }
}

impl DeepgramProvider {
    fn key(&self) -> String {
        self.settings
            .api_key
            .clone()
            .or_else(|| std::env::var("DEEPGRAM_API_KEY").ok())
            .unwrap_or_default()
    }

    fn base_url(&self) -> String {
        self.settings
            .base_url
            .clone()
            .unwrap_or_else(|| BASE_URL.into())
    }

    fn transcription(&self) -> DeepgramTranscriptionModel {
        DeepgramTranscriptionModel::new(self.key(), self.base_url())
    }

    fn speech(&self) -> DeepgramSpeechModel {
        DeepgramSpeechModel::new(self.key(), self.base_url())
    }
}

impl crate::core::registry::Provider for DeepgramProvider {
    fn transcription_model(
        &self,
        _model_id: &str,
    ) -> Option<Box<dyn crate::core::TranscriptionModel>> {
        Some(Box::new(self.transcription()))
    }

    fn speech_model(&self, _model_id: &str) -> Option<Box<dyn crate::core::SpeechModel>> {
        Some(Box::new(self.speech()))
    }

    fn detailed_transcription_model(
        &self,
        _model_id: &str,
    ) -> Option<Box<dyn DetailedTranscriptionModel>> {
        Some(Box::new(self.transcription()))
    }

    fn detailed_speech_model(&self, _model_id: &str) -> Option<Box<dyn DetailedSpeechModel>> {
        Some(Box::new(self.speech()))
    }

    fn streaming_speech_model(&self, _model_id: &str) -> Option<Box<dyn StreamingSpeechModel>> {
        Some(Box::new(self.speech()))
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            llm: false,
            batch_stt: true,
            streaming_stt: false,
            batch_tts: true,
            streaming_tts: true,
            diarization: true,
            word_timestamps: true,
            ssml: false,
            pitch: false,
        }
    }
}

pub struct DeepgramTranscriptionModel {
    api_key: String,
    base_url: String,
    client: Client,
}

impl DeepgramTranscriptionModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct ListenResponse {
    results: ListenResults,
    metadata: Option<ListenMetadata>,
}

#[derive(Deserialize)]
struct ListenResults {
    channels: Vec<ListenChannel>,
}

#[derive(Deserialize)]
struct ListenChannel {
    alternatives: Vec<ListenAlternative>,
}

#[derive(Deserialize)]
struct ListenAlternative {
    transcript: String,
    confidence: Option<f32>,
    #[serde(default)]
    words: Vec<ListenWord>,
}

#[derive(Deserialize)]
struct ListenWord {
    word: String,
    start: Option<f64>,
    end: Option<f64>,
    confidence: Option<f32>,
    speaker: Option<i64>,
    language: Option<String>,
    punctuated_word: Option<String>,
}

#[derive(Deserialize)]
struct ListenMetadata {
    duration: Option<f64>,
}

fn ms(value: Option<f64>) -> Option<u64> {
    value.and_then(|value| (value.is_finite() && value >= 0.0).then_some((value * 1_000.0) as u64))
}

#[async_trait]
impl DetailedTranscriptionModel for DeepgramTranscriptionModel {
    async fn transcribe_detailed(
        &self,
        request: TranscriptionRequest,
    ) -> crate::core::Result<DetailedTranscriptionResult> {
        let mut query = vec![format!("model={}", request.model_id)];
        if request.diarize {
            query.push("diarize=true".into());
        }
        if request.word_timestamps {
            query.push("utterances=true".into());
        }
        if let Some(language) = request.language.as_deref() {
            query.push(format!("language={language}"));
        }
        for term in &request.vocabulary {
            query.push(format!("keyterm={term}"));
        }
        let mime = request.media_type.as_deref().unwrap_or("audio/wav");
        let form = Form::new().part(
            "audio",
            Part::bytes(request.audio)
                .file_name("audio")
                .mime_str(mime)?,
        );
        let response = self
            .client
            .post(format!("{}/listen?{}", self.base_url, query.join("&")))
            .header("Authorization", format!("Token {}", self.api_key))
            .multipart(form)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(crate::core::ProviderError::InvalidResponse(
                response.text().await?,
            ));
        }
        let body: ListenResponse = response.json().await?;
        let alternative = body
            .results
            .channels
            .into_iter()
            .next()
            .and_then(|channel| channel.alternatives.into_iter().next())
            .ok_or_else(|| {
                crate::core::ProviderError::InvalidResponse(
                    "Deepgram returned no alternatives".into(),
                )
            })?;
        let words: Vec<TranscriptWord> = alternative
            .words
            .into_iter()
            .map(|word| TranscriptWord {
                text: word.punctuated_word.unwrap_or(word.word),
                start_ms: ms(word.start),
                end_ms: ms(word.end),
                confidence: word.confidence,
                speaker: word.speaker.map(|speaker| format!("spk{speaker}")),
                language: word.language,
            })
            .collect();
        let segments = (!alternative.transcript.trim().is_empty())
            .then(|| TranscriptSegment {
                text: alternative.transcript.clone(),
                start_ms: words.first().and_then(|word| word.start_ms),
                end_ms: words.last().and_then(|word| word.end_ms),
                confidence: alternative.confidence,
                speaker: None,
                words: words.clone(),
            })
            .into_iter()
            .collect();
        Ok(DetailedTranscriptionResult {
            text: alternative.transcript,
            language: request.language,
            duration_ms: body.metadata.and_then(|metadata| ms(metadata.duration)),
            words,
            segments,
        })
    }
}

#[async_trait]
impl crate::core::TranscriptionModel for DeepgramTranscriptionModel {
    async fn transcribe(
        &self,
        options: TranscriptionOptions,
    ) -> crate::core::Result<TranscriptionResult> {
        self.transcribe_detailed(TranscriptionRequest {
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

pub struct DeepgramSpeechModel {
    api_key: String,
    base_url: String,
    client: Client,
}

impl DeepgramSpeechModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: Client::new(),
        }
    }

    async fn request(&self, request: &SpeechRequest) -> crate::core::Result<reqwest::Response> {
        if request.pitch.is_some() || request.ssml.is_some() {
            return Err(crate::core::ProviderError::NotSupported(
                "Deepgram TTS does not expose pitch or SSML".into(),
            ));
        }
        let encoding = match request.audio.encoding {
            AudioEncoding::Pcm16 => "linear16",
            AudioEncoding::Wav => "linear16",
            AudioEncoding::Mp3 => "mp3",
            AudioEncoding::Opus => "opus",
            AudioEncoding::Flac => "flac",
            AudioEncoding::Mulaw => "mulaw",
            AudioEncoding::Alaw => "alaw",
        };
        let mut url = format!(
            "{}/speak?model={}&encoding={}",
            self.base_url, request.model_id, encoding
        );
        if let Some(rate) = request.rate {
            url.push_str(&format!("&speed={rate}"));
        }
        if let Some(sample_rate) = request.audio.sample_rate_hz {
            url.push_str(&format!("&sample_rate={sample_rate}"));
        }
        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Token {}", self.api_key))
            .json(&serde_json::json!({ "text": request.input }))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(crate::core::ProviderError::InvalidResponse(
                response.text().await?,
            ));
        }
        Ok(response)
    }
}

#[async_trait]
impl DetailedSpeechModel for DeepgramSpeechModel {
    async fn synthesize_detailed(
        &self,
        request: SpeechRequest,
    ) -> crate::core::Result<DetailedSpeechResult> {
        let audio_spec = request.audio.clone();
        let audio = self.request(&request).await?.bytes().await?.to_vec();
        Ok(DetailedSpeechResult { audio, audio_spec })
    }
}

#[async_trait]
impl StreamingSpeechModel for DeepgramSpeechModel {
    async fn synthesize_stream(
        &self,
        request: SpeechRequest,
    ) -> crate::core::Result<BoxStream<'static, crate::core::Result<AudioChunk>>> {
        let audio_spec = request.audio.clone();
        Ok(Box::pin(self.request(&request).await?.bytes_stream().map(
            move |item| {
                item.map(|data| AudioChunk {
                    data: data.to_vec(),
                    is_final: false,
                    audio_spec: audio_spec.clone(),
                })
                .map_err(Into::into)
            },
        )))
    }
}

#[async_trait]
impl crate::core::SpeechModel for DeepgramSpeechModel {
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
