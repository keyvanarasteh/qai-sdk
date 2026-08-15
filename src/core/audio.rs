//! Rich audio contracts shared by provider adapters.
//!
//! These types are additive to the original `SpeechModel` and
//! `TranscriptionModel` APIs.  Applications that only need a text transcript
//! or a complete audio buffer can keep using those compatibility traits.

use async_trait::async_trait;
use futures::channel::mpsc;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

use super::{Result, SpeechResult, TranscriptionResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioEncoding {
    Pcm16,
    Wav,
    Mp3,
    Opus,
    Flac,
    Mulaw,
    Alaw,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioSpec {
    pub encoding: AudioEncoding,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u8>,
    pub content_type: Option<String>,
}

impl AudioSpec {
    #[must_use]
    pub fn wav() -> Self {
        Self {
            encoding: AudioEncoding::Wav,
            sample_rate_hz: None,
            channels: None,
            content_type: Some("audio/wav".into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptWord {
    pub text: String,
    pub start_ms: Option<u64>,
    pub end_ms: Option<u64>,
    pub confidence: Option<f32>,
    pub speaker: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub text: String,
    pub start_ms: Option<u64>,
    pub end_ms: Option<u64>,
    pub confidence: Option<f32>,
    pub speaker: Option<String>,
    pub words: Vec<TranscriptWord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptionRequest {
    pub model_id: String,
    pub audio: Vec<u8>,
    pub media_type: Option<String>,
    pub language: Option<String>,
    pub prompt: Option<String>,
    pub vocabulary: Vec<String>,
    pub diarize: bool,
    pub word_timestamps: bool,
    pub segment_timestamps: bool,
    pub channels: Option<u8>,
    pub sample_rate_hz: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetailedTranscriptionResult {
    pub text: String,
    pub language: Option<String>,
    pub duration_ms: Option<u64>,
    pub words: Vec<TranscriptWord>,
    pub segments: Vec<TranscriptSegment>,
}

impl From<DetailedTranscriptionResult> for TranscriptionResult {
    fn from(value: DetailedTranscriptionResult) -> Self {
        Self {
            text: value.text,
            language: value.language,
            duration: value.duration_ms.map(|value| value as f64 / 1_000.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptionEvent {
    pub text: String,
    pub is_final: bool,
    pub start_ms: Option<u64>,
    pub end_ms: Option<u64>,
    pub confidence: Option<f32>,
    pub speaker: Option<String>,
    pub words: Vec<TranscriptWord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeechRequest {
    pub model_id: String,
    pub input: String,
    pub voice: String,
    pub audio: AudioSpec,
    pub rate: Option<f32>,
    pub pitch: Option<f32>,
    pub ssml: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetailedSpeechResult {
    pub audio: Vec<u8>,
    pub audio_spec: AudioSpec,
}

impl From<DetailedSpeechResult> for SpeechResult {
    fn from(value: DetailedSpeechResult) -> Self {
        Self { audio: value.audio }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioChunk {
    pub data: Vec<u8>,
    pub is_final: bool,
    pub audio_spec: AudioSpec,
}

/// Bidirectional streaming STT handle. Providers expose it only when they
/// have a native streaming contract; callers must treat NotSupported as a
/// capability result, never as permission to emulate it with batch STT.
pub struct TranscriptionSession {
    pub audio_input: mpsc::Sender<Vec<u8>>,
    pub events: BoxStream<'static, Result<TranscriptionEvent>>,
}

/// Streaming TTS handle. Text chunks, flush, clear and close are explicit
/// controls so a voice agent can cancel stale audio during barge-in.
pub struct SpeechSession {
    pub text_input: mpsc::Sender<String>,
    pub flush: mpsc::Sender<()>,
    pub clear: mpsc::Sender<()>,
    pub close: mpsc::Sender<()>,
    pub audio: BoxStream<'static, Result<AudioChunk>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderCapabilities {
    pub llm: bool,
    pub batch_stt: bool,
    pub streaming_stt: bool,
    pub batch_tts: bool,
    pub streaming_tts: bool,
    pub diarization: bool,
    pub word_timestamps: bool,
    pub ssml: bool,
    pub pitch: bool,
}

#[async_trait]
pub trait DetailedTranscriptionModel: Send + Sync {
    async fn transcribe_detailed(
        &self,
        request: TranscriptionRequest,
    ) -> Result<DetailedTranscriptionResult>;
}

#[async_trait]
pub trait StreamingTranscriptionModel: Send + Sync {
    /// Open a bidirectional transcription stream. Implementations must not
    /// emulate streaming by running a batch request before yielding events.
    async fn transcribe_stream(
        &self,
        request: TranscriptionRequest,
    ) -> Result<BoxStream<'static, Result<TranscriptionEvent>>>;

    async fn open(&self, _model_id: String) -> Result<TranscriptionSession> {
        Err(anyhow::anyhow!("native streaming STT is not supported by this provider").into())
    }
}

#[async_trait]
pub trait DetailedSpeechModel: Send + Sync {
    async fn synthesize_detailed(&self, request: SpeechRequest) -> Result<DetailedSpeechResult>;
}

#[async_trait]
pub trait StreamingSpeechModel: Send + Sync {
    async fn synthesize_stream(
        &self,
        request: SpeechRequest,
    ) -> Result<BoxStream<'static, Result<AudioChunk>>>;

    async fn open(
        &self,
        _model_id: String,
        _voice: String,
        _audio: AudioSpec,
    ) -> Result<SpeechSession> {
        Err(anyhow::anyhow!("native streaming TTS is not supported by this provider").into())
    }
}
