//! # QAI SDK
//!
//! A modular, type-safe Rust SDK for AI providers.
//!
//! This crate re-exports all provider crates for convenient single-import usage.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use qai_sdk::openai;
//! use qai_sdk::core::types::ProviderSettings;
//!
//! let provider = openai::create_openai(ProviderSettings {
//!     api_key: Some("sk-...".to_string()),
//!     base_url: None,
//!     headers: None,
//! });
//!
//! let chat = provider.chat("gpt-4o");
//! let embedding = provider.embedding("text-embedding-3-small");
//! let image = provider.image("dall-e-3");
//! let responses = provider.responses("gpt-4o");
//! ```

/// Core traits and types for all providers.
pub use qai_core as core;

/// Anthropic provider (Claude models).
pub use qai_anthropic as anthropic;

/// OpenAI provider (GPT, DALL-E, Whisper, TTS models).
pub use qai_openai as openai;

/// Google Generative AI provider (Gemini models).
pub use qai_google as google;

/// DeepSeek provider (DeepSeek-Chat, DeepSeek-Reasoner models).
pub use qai_deepseek as deepseek;

/// xAI provider (Grok models).
pub use qai_xai as xai;

/// OpenAI-compatible provider (any OpenAI-compatible API endpoint).
pub use qai_openai_compatible as openai_compatible;

// Re-export commonly used items at top level for convenience.
pub mod prelude {
    // Core traits
    pub use qai_core::CompletionModel;
    pub use qai_core::EmbeddingModel;
    pub use qai_core::ImageModel;
    pub use qai_core::LanguageModel;
    pub use qai_core::SpeechModel;
    pub use qai_core::TranscriptionModel;

    // Core types
    pub use qai_core::types::{
        Content, GenerateOptions, GenerateResult, Message, Prompt, ProviderSettings, Role,
        StreamPart, Usage,
    };

    // Provider factory functions
    pub use qai_anthropic::create_anthropic;
    pub use qai_deepseek::create_deepseek;
    pub use qai_google::create_google;
    pub use qai_openai::create_openai;
    pub use qai_openai_compatible::create_openai_compatible;
    pub use qai_xai::create_xai;
}
