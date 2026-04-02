//! # Middleware Layer
//!
//! Composable wrappers that intercept `generate` and `generate_stream` calls,
//! allowing cross-cutting behavior like default settings injection, reasoning
//! extraction, JSON extraction, and streaming simulation.
//!
//! Mirrors the Vercel AI SDK's `wrapLanguageModel` and middleware pattern.
//!
//! # Example
//! ```rust,ignore
//! use qai_sdk::core::middleware::*;
//!
//! let wrapped = wrap_language_model(
//!     model,
//!     vec![Box::new(DefaultSettingsMiddleware {
//!         temperature: Some(0.7),
//!         max_tokens: Some(2048),
//!         top_p: None,
//!     })],
//! );
//! // `wrapped` now applies default settings before every call.
//! ```

use crate::core::types::{GenerateOptions, GenerateResult, Prompt, StreamPart};
use crate::core::{LanguageModel, Result};
use async_trait::async_trait;
use futures::stream::BoxStream;

/// A middleware that can intercept and transform language model calls.
#[async_trait]
pub trait LanguageModelMiddleware: Send + Sync {
    /// Transform the generation options before the model call.
    /// Default: passes through unmodified.
    async fn transform_params(
        &self,
        options: GenerateOptions,
    ) -> Result<GenerateOptions> {
        Ok(options)
    }

    /// Optionally wrap the generate call. Return `None` to delegate to the
    /// next middleware or the underlying model.
    async fn wrap_generate(
        &self,
        _prompt: &Prompt,
        _options: &GenerateOptions,
        _model: &dyn LanguageModel,
    ) -> Option<Result<GenerateResult>> {
        None
    }

    /// Optionally wrap the streaming generate call. Return `None` to delegate
    /// to the next middleware or the underlying model.
    async fn wrap_generate_stream(
        &self,
        _prompt: &Prompt,
        _options: &GenerateOptions,
        _model: &dyn LanguageModel,
    ) -> Option<Result<BoxStream<'static, StreamPart>>> {
        None
    }
}

/// Wraps a language model with a chain of middleware.
///
/// Middlewares are applied in order: first middleware transforms params first,
/// last middleware wraps closest to the model.
pub fn wrap_language_model(
    model: Box<dyn LanguageModel>,
    middlewares: Vec<Box<dyn LanguageModelMiddleware>>,
) -> Box<dyn LanguageModel> {
    let mut wrapped: Box<dyn LanguageModel> = model;
    // Apply in reverse order so first middleware is outermost
    for mw in middlewares.into_iter().rev() {
        wrapped = Box::new(WrappedModel {
            inner: wrapped,
            middleware: mw,
        });
    }
    wrapped
}

/// Internal struct that chains a middleware with an inner model.
struct WrappedModel {
    inner: Box<dyn LanguageModel>,
    middleware: Box<dyn LanguageModelMiddleware>,
}

#[async_trait]
impl LanguageModel for WrappedModel {
    async fn generate(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> Result<GenerateResult> {
        let transformed = self.middleware.transform_params(options).await?;

        // Check if middleware wants to fully handle the call
        if let Some(result) = self
            .middleware
            .wrap_generate(&prompt, &transformed, self.inner.as_ref())
            .await
        {
            return result;
        }

        self.inner.generate(prompt, transformed).await
    }

    async fn generate_stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> Result<BoxStream<'static, StreamPart>> {
        let transformed = self.middleware.transform_params(options).await?;

        // Check if middleware wants to fully handle the streaming call
        if let Some(result) = self
            .middleware
            .wrap_generate_stream(&prompt, &transformed, self.inner.as_ref())
            .await
        {
            return result;
        }

        self.inner.generate_stream(prompt, transformed).await
    }
}

// ---------------------------------------------------------------------------
// Built-in Middlewares
// ---------------------------------------------------------------------------

/// Injects default settings (temperature, max_tokens) when not explicitly set.
pub struct DefaultSettingsMiddleware {
    /// Default temperature to use if none is specified.
    pub temperature: Option<f32>,
    /// Default max_tokens to use if none is specified.
    pub max_tokens: Option<u32>,
    /// Default top_p to use if none is specified.
    pub top_p: Option<f32>,
}

#[async_trait]
impl LanguageModelMiddleware for DefaultSettingsMiddleware {
    async fn transform_params(
        &self,
        mut options: GenerateOptions,
    ) -> Result<GenerateOptions> {
        if options.temperature.is_none() {
            options.temperature = self.temperature;
        }
        if options.max_tokens.is_none() {
            options.max_tokens = self.max_tokens;
        }
        if options.top_p.is_none() {
            options.top_p = self.top_p;
        }
        Ok(options)
    }
}

/// Extracts `<think>...</think>` reasoning blocks from model output into a
/// separate field, leaving only the clean response text.
pub struct ExtractReasoningMiddleware {
    /// The opening tag to look for. Default: `<think>`
    pub open_tag: String,
    /// The closing tag to look for. Default: `</think>`
    pub close_tag: String,
}

impl Default for ExtractReasoningMiddleware {
    fn default() -> Self {
        Self {
            open_tag: "<think>".to_string(),
            close_tag: "</think>".to_string(),
        }
    }
}

#[async_trait]
impl LanguageModelMiddleware for ExtractReasoningMiddleware {
    async fn wrap_generate(
        &self,
        prompt: &Prompt,
        options: &GenerateOptions,
        model: &dyn LanguageModel,
    ) -> Option<Result<GenerateResult>> {
        let result = model.generate(prompt.clone(), options.clone()).await;
        match result {
            Ok(mut gen_result) => {
                gen_result.text = extract_reasoning(
                    &gen_result.text,
                    &self.open_tag,
                    &self.close_tag,
                );
                Some(Ok(gen_result))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

/// Extracts JSON from markdown code fences in model output.
///
/// If the model wraps its JSON in ` ```json ... ``` ` or ` ``` ... ``` `,
/// this middleware strips the fences and replaces the response text with
/// the raw JSON content.
pub struct ExtractJsonMiddleware;

#[async_trait]
impl LanguageModelMiddleware for ExtractJsonMiddleware {
    async fn wrap_generate(
        &self,
        prompt: &Prompt,
        options: &GenerateOptions,
        model: &dyn LanguageModel,
    ) -> Option<Result<GenerateResult>> {
        let result = model.generate(prompt.clone(), options.clone()).await;
        match result {
            Ok(mut gen_result) => {
                gen_result.text = extract_json_from_fences(&gen_result.text);
                Some(Ok(gen_result))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

/// Converts a non-streaming model into a streaming one by calling `generate()`
/// and then splitting the result into character-sized `TextDelta` chunks.
///
/// Useful for models/providers that don't natively support streaming.
pub struct SimulateStreamingMiddleware;

#[async_trait]
impl LanguageModelMiddleware for SimulateStreamingMiddleware {
    async fn wrap_generate_stream(
        &self,
        prompt: &Prompt,
        options: &GenerateOptions,
        model: &dyn LanguageModel,
    ) -> Option<Result<BoxStream<'static, StreamPart>>> {
        let result = model.generate(prompt.clone(), options.clone()).await;
        match result {
            Ok(gen_result) => {
                let text = gen_result.text;
                let usage = gen_result.usage;
                let finish_reason = gen_result.finish_reason;

                let stream = async_stream::stream! {
                    // Emit usage first
                    yield StreamPart::Usage { usage };

                    // Split text into character-sized deltas
                    for ch in text.chars() {
                        yield StreamPart::TextDelta { delta: ch.to_string() };
                    }

                    // Emit finish
                    yield StreamPart::Finish { finish_reason };
                };

                Some(Ok(Box::pin(stream)))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

/// Strips `<think>...</think>` blocks from text, returning only the clean output.
fn extract_reasoning(text: &str, open_tag: &str, close_tag: &str) -> String {
    let mut result = text.to_string();
    while let Some(start) = result.find(open_tag) {
        if let Some(end) = result[start..].find(close_tag) {
            result = format!(
                "{}{}",
                &result[..start],
                &result[start + end + close_tag.len()..]
            );
        } else {
            break;
        }
    }
    result.trim().to_string()
}

/// Extract JSON content from markdown code fences.
fn extract_json_from_fences(text: &str) -> String {
    let trimmed = text.trim();

    if trimmed.starts_with("```json") {
        if let Some(stripped) = trimmed
            .strip_prefix("```json")
            .and_then(|s| s.strip_suffix("```"))
        {
            return stripped.trim().to_string();
        }
    }

    if trimmed.starts_with("```") {
        if let Some(stripped) = trimmed
            .strip_prefix("```")
            .and_then(|s| s.strip_suffix("```"))
        {
            return stripped.trim().to_string();
        }
    }

    trimmed.to_string()
}
