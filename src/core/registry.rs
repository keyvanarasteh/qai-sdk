//! # Provider Registry
//!
//! A centralized registry that maps `"provider:model"` strings to trait objects.
//! Mirrors the Vercel AI SDK's `createProviderRegistry` pattern.
//!
//! # Example
//! ```rust,ignore
//! use qai_sdk::core::registry::*;
//!
//! let registry = ProviderRegistry::new()
//!     .register("openai", openai_provider)
//!     .register("anthropic", anthropic_provider);
//!
//! let model = registry.language_model("openai:gpt-4o")?;
//! let result = model.generate(prompt, options).await?;
//! ```

use crate::core::error::ProviderError;
use crate::core::{
    CompletionModel, DetailedSpeechModel, DetailedTranscriptionModel, EmbeddingModel, ImageModel,
    LanguageModel, MusicModel, ProviderCapabilities, RealtimeModel, Result, SpeechModel,
    StreamingSpeechModel, StreamingTranscriptionModel, TranscriptionModel, VideoModel,
};
use std::collections::HashMap;

/// A provider factory that can create model instances by ID.
pub trait Provider: Send + Sync {
    /// Create a language model by model ID.
    fn language_model(&self, _model_id: &str) -> Option<Box<dyn LanguageModel>> {
        None
    }

    /// Create an embedding model by model ID. Optional.
    fn embedding_model(&self, _model_id: &str) -> Option<Box<dyn EmbeddingModel>> {
        None
    }

    /// Create an image model by model ID. Optional.
    fn image_model(&self, _model_id: &str) -> Option<Box<dyn ImageModel>> {
        None
    }

    /// Create a transcription model by model ID. Optional.
    fn transcription_model(&self, _model_id: &str) -> Option<Box<dyn TranscriptionModel>> {
        None
    }

    /// Create a speech model by model ID. Optional.
    fn speech_model(&self, _model_id: &str) -> Option<Box<dyn SpeechModel>> {
        None
    }

    /// Create a text completion model by model ID. Optional.
    fn completion_model(&self, _model_id: &str) -> Option<Box<dyn CompletionModel>> {
        None
    }

    /// Create a video model by model ID. Optional.
    fn video_model(&self, _model_id: &str) -> Option<Box<dyn VideoModel>> {
        None
    }

    /// Create a music model by model ID. Optional.
    fn music_model(&self, _model_id: &str) -> Option<Box<dyn MusicModel>> {
        None
    }

    /// Create a realtime model by model ID. Optional.
    fn realtime_model(&self, _model_id: &str) -> Option<Box<dyn RealtimeModel>> {
        None
    }

    fn detailed_transcription_model(
        &self,
        _model_id: &str,
    ) -> Option<Box<dyn DetailedTranscriptionModel>> {
        None
    }

    fn streaming_transcription_model(
        &self,
        _model_id: &str,
    ) -> Option<Box<dyn StreamingTranscriptionModel>> {
        None
    }

    fn detailed_speech_model(&self, _model_id: &str) -> Option<Box<dyn DetailedSpeechModel>> {
        None
    }

    fn streaming_speech_model(&self, _model_id: &str) -> Option<Box<dyn StreamingSpeechModel>> {
        None
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::default()
    }
}

/// A registry of named providers that resolves `"provider:model"` strings.
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn Provider>>,
    separator: char,
}

impl ProviderRegistry {
    /// Create a new empty registry with the default `:` separator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            separator: ':',
        }
    }

    /// Create a new registry with a custom separator.
    #[must_use]
    pub fn with_separator(separator: char) -> Self {
        Self {
            providers: HashMap::new(),
            separator,
        }
    }

    /// Register a provider under a given name. Chainable.
    #[must_use]
    pub fn register(mut self, name: impl Into<String>, provider: impl Provider + 'static) -> Self {
        self.providers.insert(name.into(), Box::new(provider));
        self
    }

    /// Split a combined `"provider:model"` ID.
    fn split_id(&self, id: &str) -> Result<(String, String)> {
        let sep_pos = id.find(self.separator).ok_or_else(|| {
            ProviderError::InvalidResponse(format!(
                "Invalid model ID '{id}': expected format 'provider{sep}model'",
                sep = self.separator
            ))
        })?;
        let provider_id = id[..sep_pos].to_string();
        let model_id = id[sep_pos + 1..].to_string();
        Ok((provider_id, model_id))
    }

    /// Resolve a language model from a `"provider:model"` string.
    pub fn language_model(&self, id: &str) -> Result<Box<dyn LanguageModel>> {
        let (provider_id, model_id) = self.split_id(id)?;
        let provider = self.providers.get(&provider_id).ok_or_else(|| {
            ProviderError::NotSupported(format!(
                "No provider registered with name '{provider_id}'. Available: {:?}",
                self.providers.keys().collect::<Vec<_>>()
            ))
        })?;
        provider.language_model(&model_id).ok_or_else(|| {
            ProviderError::NotSupported(format!(
                "Provider '{provider_id}' does not support language model '{model_id}'"
            ))
        })
    }

    /// Resolve an embedding model from a `"provider:model"` string.
    pub fn embedding_model(&self, id: &str) -> Result<Box<dyn EmbeddingModel>> {
        let (provider_id, model_id) = self.split_id(id)?;
        let provider = self.providers.get(&provider_id).ok_or_else(|| {
            ProviderError::NotSupported(format!("No provider registered with name '{provider_id}'"))
        })?;
        provider.embedding_model(&model_id).ok_or_else(|| {
            ProviderError::NotSupported(format!(
                "Provider '{provider_id}' does not support embedding model '{model_id}'"
            ))
        })
    }

    /// Resolve an image model from a `"provider:model"` string.
    pub fn image_model(&self, id: &str) -> Result<Box<dyn ImageModel>> {
        let (provider_id, model_id) = self.split_id(id)?;
        let provider = self.providers.get(&provider_id).ok_or_else(|| {
            ProviderError::NotSupported(format!("No provider registered with name '{provider_id}'"))
        })?;
        provider.image_model(&model_id).ok_or_else(|| {
            ProviderError::NotSupported(format!(
                "Provider '{provider_id}' does not support image model '{model_id}'"
            ))
        })
    }

    /// Resolve a transcription model from a `"provider:model"` string.
    pub fn transcription_model(&self, id: &str) -> Result<Box<dyn TranscriptionModel>> {
        let (provider_id, model_id) = self.split_id(id)?;
        let provider = self.providers.get(&provider_id).ok_or_else(|| {
            ProviderError::NotSupported(format!("No provider registered with name '{provider_id}'"))
        })?;
        provider.transcription_model(&model_id).ok_or_else(|| {
            ProviderError::NotSupported(format!(
                "Provider '{provider_id}' does not support transcription model '{model_id}'"
            ))
        })
    }

    /// Resolve a speech model from a `"provider:model"` string.
    pub fn speech_model(&self, id: &str) -> Result<Box<dyn SpeechModel>> {
        let (provider_id, model_id) = self.split_id(id)?;
        let provider = self.providers.get(&provider_id).ok_or_else(|| {
            ProviderError::NotSupported(format!("No provider registered with name '{provider_id}'"))
        })?;
        provider.speech_model(&model_id).ok_or_else(|| {
            ProviderError::NotSupported(format!(
                "Provider '{provider_id}' does not support speech model '{model_id}'"
            ))
        })
    }

    /// Resolve a text completion model from a `"provider:model"` string.
    pub fn completion_model(&self, id: &str) -> Result<Box<dyn CompletionModel>> {
        let (provider_id, model_id) = self.split_id(id)?;
        let provider = self.providers.get(&provider_id).ok_or_else(|| {
            ProviderError::NotSupported(format!("No provider registered with name '{provider_id}'"))
        })?;
        provider.completion_model(&model_id).ok_or_else(|| {
            ProviderError::NotSupported(format!(
                "Provider '{provider_id}' does not support completion model '{model_id}'"
            ))
        })
    }

    /// Resolve a video model from a `"provider:model"` string.
    pub fn video_model(&self, id: &str) -> Result<Box<dyn VideoModel>> {
        let (provider_id, model_id) = self.split_id(id)?;
        let provider = self.providers.get(&provider_id).ok_or_else(|| {
            ProviderError::NotSupported(format!("No provider registered with name '{provider_id}'"))
        })?;
        provider.video_model(&model_id).ok_or_else(|| {
            ProviderError::NotSupported(format!(
                "Provider '{provider_id}' does not support video model '{model_id}'"
            ))
        })
    }

    /// Resolve a music model from a `"provider:model"` string.
    pub fn music_model(&self, id: &str) -> Result<Box<dyn MusicModel>> {
        let (provider_id, model_id) = self.split_id(id)?;
        let provider = self.providers.get(&provider_id).ok_or_else(|| {
            ProviderError::NotSupported(format!("No provider registered with name '{provider_id}'"))
        })?;
        provider.music_model(&model_id).ok_or_else(|| {
            ProviderError::NotSupported(format!(
                "Provider '{provider_id}' does not support music model '{model_id}'"
            ))
        })
    }

    /// Resolve a realtime model from a `"provider:model"` string.
    pub fn realtime_model(&self, id: &str) -> Result<Box<dyn RealtimeModel>> {
        let (provider_id, model_id) = self.split_id(id)?;
        let provider = self.providers.get(&provider_id).ok_or_else(|| {
            ProviderError::NotSupported(format!("No provider registered with name '{provider_id}'"))
        })?;
        provider.realtime_model(&model_id).ok_or_else(|| {
            ProviderError::NotSupported(format!(
                "Provider '{provider_id}' does not support realtime model '{model_id}'"
            ))
        })
    }

    pub fn detailed_transcription_model(
        &self,
        id: &str,
    ) -> Result<Box<dyn DetailedTranscriptionModel>> {
        let (provider_id, model_id) = self.split_id(id)?;
        let provider = self.providers.get(&provider_id).ok_or_else(|| {
            ProviderError::NotSupported(format!("No provider registered with name '{provider_id}'"))
        })?;
        provider
            .detailed_transcription_model(&model_id)
            .ok_or_else(|| {
                ProviderError::NotSupported(format!(
                    "Provider '{provider_id}' does not support detailed STT"
                ))
            })
    }

    pub fn streaming_transcription_model(
        &self,
        id: &str,
    ) -> Result<Box<dyn StreamingTranscriptionModel>> {
        let (provider_id, model_id) = self.split_id(id)?;
        let provider = self.providers.get(&provider_id).ok_or_else(|| {
            ProviderError::NotSupported(format!("No provider registered with name '{provider_id}'"))
        })?;
        provider
            .streaming_transcription_model(&model_id)
            .ok_or_else(|| {
                ProviderError::NotSupported(format!(
                    "Provider '{provider_id}' does not support streaming STT"
                ))
            })
    }

    pub fn detailed_speech_model(&self, id: &str) -> Result<Box<dyn DetailedSpeechModel>> {
        let (provider_id, model_id) = self.split_id(id)?;
        let provider = self.providers.get(&provider_id).ok_or_else(|| {
            ProviderError::NotSupported(format!("No provider registered with name '{provider_id}'"))
        })?;
        provider.detailed_speech_model(&model_id).ok_or_else(|| {
            ProviderError::NotSupported(format!(
                "Provider '{provider_id}' does not support detailed TTS"
            ))
        })
    }

    pub fn streaming_speech_model(&self, id: &str) -> Result<Box<dyn StreamingSpeechModel>> {
        let (provider_id, model_id) = self.split_id(id)?;
        let provider = self.providers.get(&provider_id).ok_or_else(|| {
            ProviderError::NotSupported(format!("No provider registered with name '{provider_id}'"))
        })?;
        provider.streaming_speech_model(&model_id).ok_or_else(|| {
            ProviderError::NotSupported(format!(
                "Provider '{provider_id}' does not support streaming TTS"
            ))
        })
    }

    pub fn capabilities(&self, provider_id: &str) -> Result<ProviderCapabilities> {
        self.providers
            .get(provider_id)
            .map(|provider| provider.capabilities())
            .ok_or_else(|| {
                ProviderError::NotSupported(format!(
                    "No provider registered with name '{provider_id}'"
                ))
            })
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
