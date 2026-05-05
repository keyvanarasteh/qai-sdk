//! # QAI `GroqCloud`
//!
//! Provider for GroqCloud's ultra-fast API endpoints. Groq implements the `OpenAI`
//! API format for chat, speech-to-text, and text-to-speech, plus a Responses API
//! for agentic tool-use workflows.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use qai_sdk::groqcloud::create_groqcloud;
//! use qai_sdk::core::types::ProviderSettings;
//!
//! let provider = create_groqcloud(ProviderSettings {
//!     api_key: Some("gsk_...".to_string()),
//!     ..Default::default()
//! });
//!
//! let chat = provider.chat("llama-3.3-70b-versatile");
//! let compound = provider.compound("groq/compound");
//! let stt = provider.transcription("whisper-large-v3-turbo");
//! let tts = provider.speech("canopylabs/orpheus-v1-english");
//! let responses = provider.responses("groq/compound");
//! ```

pub mod tools;

use crate::core::types::{
    GenerateOptions, GenerateResult, Prompt, ProviderSettings, SpeechOptions, SpeechResult,
    StreamPart, TranscriptionOptions, TranscriptionResult,
};
use crate::openai::responses::OpenAIResponsesModel;
use crate::openai::speech::OpenAISpeechModel;
use crate::openai::transcription::OpenAITranscriptionModel;
use crate::openai::OpenAIModel;
use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;

const GROQ_BASE_URL: &str = "https://api.groq.com/openai/v1";

/// GroqCloud model wrapper.
pub struct GroqCloudModel {
    pub inner: OpenAIModel,
}

impl GroqCloudModel {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            inner: OpenAIModel {
                api_key,
                base_url: GROQ_BASE_URL.to_string(),
                client: Client::new(),
            },
        }
    }
}

#[async_trait]
impl crate::core::LanguageModel for GroqCloudModel {
    #[tracing::instrument(skip(self, prompt), fields(model = options.model_id))]
    async fn generate(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> crate::core::Result<GenerateResult> {
        self.inner.generate(prompt, options).await
    }

    async fn generate_stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> crate::core::Result<BoxStream<'static, StreamPart>> {
        self.inner.generate_stream(prompt, options).await
    }
}

/// GroqCloud Responses API model wrapper.
///
/// Provides access to Groq's Responses API for agentic tool-use workflows,
/// including built-in tools (web search, visit website) and remote MCP servers.
pub struct GroqCloudResponsesModel {
    pub inner: OpenAIResponsesModel,
}

impl GroqCloudResponsesModel {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            inner: OpenAIResponsesModel {
                api_key,
                base_url: GROQ_BASE_URL.to_string(),
                client: Client::new(),
            },
        }
    }
}

#[async_trait]
impl crate::core::LanguageModel for GroqCloudResponsesModel {
    async fn generate(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> crate::core::Result<GenerateResult> {
        self.inner.generate(prompt, options).await
    }

    async fn generate_stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> crate::core::Result<BoxStream<'static, StreamPart>> {
        self.inner.generate_stream(prompt, options).await
    }
}

/// GroqCloud Transcription (STT) model wrapper.
pub struct GroqCloudTranscriptionModel {
    pub inner: OpenAITranscriptionModel,
}

impl GroqCloudTranscriptionModel {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            inner: OpenAITranscriptionModel {
                api_key,
                base_url: GROQ_BASE_URL.to_string(),
                client: Client::new(),
            },
        }
    }
}

#[async_trait]
impl crate::core::TranscriptionModel for GroqCloudTranscriptionModel {
    async fn transcribe(
        &self,
        options: TranscriptionOptions,
    ) -> crate::core::Result<TranscriptionResult> {
        self.inner.transcribe(options).await
    }
}

/// GroqCloud Speech (TTS) model wrapper.
pub struct GroqCloudSpeechModel {
    pub inner: OpenAISpeechModel,
}

impl GroqCloudSpeechModel {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            inner: OpenAISpeechModel {
                api_key,
                base_url: GROQ_BASE_URL.to_string(),
                client: Client::new(),
            },
        }
    }
}

#[async_trait]
impl crate::core::SpeechModel for GroqCloudSpeechModel {
    async fn synthesize(&self, options: SpeechOptions) -> crate::core::Result<SpeechResult> {
        self.inner.synthesize(options).await
    }
}

// --- Provider Factory ---

/// GroqCloud provider instance.
pub struct GroqCloudProvider {
    settings: ProviderSettings,
}

impl GroqCloudProvider {
    /// Creates a chat language model.
    #[must_use]
    pub fn chat(&self, _model_id: &str) -> GroqCloudModel {
        let api_key = self.get_api_key();
        GroqCloudModel::new(api_key)
    }

    /// Alias for `chat`.
    #[must_use]
    pub fn language_model(&self, model_id: &str) -> GroqCloudModel {
        self.chat(model_id)
    }

    /// Creates a compound model instance for built-in tool use (web search, visit website).
    ///
    /// Compound models (`groq/compound`, `groq/compound-mini`) automatically
    /// select and execute built-in tools server-side. Use this with
    /// [`tools::GroqTool::builtin_web_search`] and [`tools::GroqTool::builtin_visit_website`].
    #[must_use]
    pub fn compound(&self, _model_id: &str) -> GroqCloudModel {
        let api_key = self.get_api_key();
        GroqCloudModel::new(api_key)
    }

    /// Creates a Responses API model for agentic workflows.
    ///
    /// The Responses API supports richer output including multi-step tool execution,
    /// MCP integration, and reasoning content.
    #[must_use]
    pub fn responses(&self, _model_id: &str) -> GroqCloudResponsesModel {
        let api_key = self.get_api_key();
        GroqCloudResponsesModel::new(api_key)
    }

    /// Creates a transcription (Speech-to-Text) model.
    #[must_use]
    pub fn transcription(&self, _model_id: &str) -> GroqCloudTranscriptionModel {
        let api_key = self.get_api_key();
        GroqCloudTranscriptionModel::new(api_key)
    }

    /// Creates a speech (Text-to-Speech) model.
    #[must_use]
    pub fn speech(&self, _model_id: &str) -> GroqCloudSpeechModel {
        let api_key = self.get_api_key();
        GroqCloudSpeechModel::new(api_key)
    }

    fn get_api_key(&self) -> String {
        self.settings
            .api_key
            .clone()
            .unwrap_or_else(|| std::env::var("GROQ_API_KEY").unwrap_or_default())
    }
}

/// Create a GroqCloud provider instance with the given settings.
#[must_use]
pub fn create_groqcloud(settings: ProviderSettings) -> GroqCloudProvider {
    GroqCloudProvider { settings }
}

impl crate::core::registry::Provider for GroqCloudProvider {
    fn language_model(&self, model_id: &str) -> Option<Box<dyn crate::core::LanguageModel>> {
        Some(Box::new(self.chat(model_id)))
    }

    fn transcription_model(
        &self,
        model_id: &str,
    ) -> Option<Box<dyn crate::core::TranscriptionModel>> {
        Some(Box::new(self.transcription(model_id)))
    }

    fn speech_model(&self, model_id: &str) -> Option<Box<dyn crate::core::SpeechModel>> {
        Some(Box::new(self.speech(model_id)))
    }
}
