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

use crate::core::{
    EmbeddingModel, ImageModel, LanguageModel, Result,
};
use crate::core::error::ProviderError;
use std::collections::HashMap;

/// A provider factory that can create model instances by ID.
pub trait Provider: Send + Sync {
    /// Create a language model by model ID.
    fn language_model(&self, model_id: &str) -> Option<Box<dyn LanguageModel>>;

    /// Create an embedding model by model ID. Optional.
    fn embedding_model(&self, _model_id: &str) -> Option<Box<dyn EmbeddingModel>> {
        None
    }

    /// Create an image model by model ID. Optional.
    fn image_model(&self, _model_id: &str) -> Option<Box<dyn ImageModel>> {
        None
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
            ProviderError::NotSupported(format!(
                "No provider registered with name '{provider_id}'"
            ))
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
            ProviderError::NotSupported(format!(
                "No provider registered with name '{provider_id}'"
            ))
        })?;
        provider.image_model(&model_id).ok_or_else(|| {
            ProviderError::NotSupported(format!(
                "Provider '{provider_id}' does not support image model '{model_id}'"
            ))
        })
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
