//! # QAI SDK
//!
//! Universal Rust SDK for AI Providers.
//!
//! Provides a unified trait `LanguageModel` implemented across various AI providers.
//!
//! ## Features
//! Enable the providers you need via Cargo features:
//! - `openai`
//! - `anthropic`
//! - `google`
//! - `deepseek`
//! - `xai`
//! - `openai-compatible`

pub mod core;

#[cfg(feature = "openai")]
pub mod openai;

#[cfg(feature = "anthropic")]
pub mod anthropic;

#[cfg(feature = "google")]
pub mod google;

#[cfg(feature = "deepseek")]
pub mod deepseek;

#[cfg(feature = "xai")]
pub mod xai;

#[cfg(feature = "openai-compatible")]
pub mod openai_compatible;

#[cfg(feature = "mcp")]
pub mod mcp;

#[cfg(test)]
pub mod test_utils;

pub use crate::core::types::*;
pub use crate::core::*;

// Export all providers if their features are enabled
#[cfg(feature = "openai")]
pub use crate::openai::create_openai;

#[cfg(feature = "anthropic")]
pub use crate::anthropic::create_anthropic;

#[cfg(feature = "google")]
pub use crate::google::create_google;

#[cfg(feature = "deepseek")]
pub use crate::deepseek::create_deepseek;

#[cfg(feature = "xai")]
pub use crate::xai::create_xai;

#[cfg(feature = "openai-compatible")]
pub use crate::openai_compatible::{create_openai_compatible, OpenAICompatibleProviderSettings};

#[cfg(feature = "openai")]
pub use crate::openai::OpenAIModel;

#[cfg(feature = "anthropic")]
pub use crate::anthropic::AnthropicModel;

#[cfg(feature = "google")]
pub use crate::google::GoogleModel;

#[cfg(feature = "deepseek")]
pub use crate::deepseek::DeepSeekModel;

#[cfg(feature = "xai")]
pub use crate::xai::XAIModel;

#[cfg(feature = "openai-compatible")]
pub use crate::openai_compatible::OpenAICompatibleModel;
