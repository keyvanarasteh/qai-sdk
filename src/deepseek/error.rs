//! `DeepSeek` API error types.
//!
//! `DeepSeek` uses the same error response format as `OpenAI` since it's
//! API-compatible. This module provides a `DeepSeekError` enum that
//! distinguishes between authentication, rate limit, and general API errors.

use serde::Deserialize;
use thiserror::Error;

/// `DeepSeek` API error response.
#[derive(Debug, Deserialize)]
pub struct DeepSeekErrorResponse {
    pub error: DeepSeekErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct DeepSeekErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub code: Option<serde_json::Value>,
}

/// Strongly-typed `DeepSeek` error.
#[derive(Debug, Error)]
pub enum DeepSeekError {
    #[error("DeepSeek API error ({error_type}): {message}")]
    ApiError {
        error_type: String,
        message: String,
        code: Option<String>,
        status_code: u16,
    },

    #[error("DeepSeek authentication error: {message}")]
    AuthenticationError { message: String },

    #[error("DeepSeek rate limit error: {message}")]
    RateLimitError { message: String },

    #[error("DeepSeek request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("DeepSeek error: {0}")]
    Other(String),
}

impl DeepSeekError {
    /// Parse an error response from `DeepSeek` API.
    #[must_use]
    pub fn from_response(status: u16, body: &str) -> Self {
        if let Ok(error_resp) = serde_json::from_str::<DeepSeekErrorResponse>(body) {
            let message = error_resp.error.message.clone();
            let error_type = error_resp.error.error_type.clone().unwrap_or_default();
            let code = error_resp
                .error
                .code
                .as_ref()
                .map(std::string::ToString::to_string);

            match status {
                401 => DeepSeekError::AuthenticationError { message },
                429 => DeepSeekError::RateLimitError { message },
                _ => DeepSeekError::ApiError {
                    error_type,
                    message,
                    code,
                    status_code: status,
                },
            }
        } else {
            DeepSeekError::Other(body.to_string())
        }
    }
}
