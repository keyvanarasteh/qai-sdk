//! OpenAI-Compatible API error types.
//!
//! Since OpenAI-compatible providers follow the `OpenAI` error format,
//! this module provides an `OpenAICompatibleError` enum with a generic
//! provider name field for better error messages.

use serde::Deserialize;
use thiserror::Error;

/// OpenAI-compatible API error response.
#[derive(Debug, Deserialize)]
pub struct OpenAICompatibleErrorResponse {
    pub error: OpenAICompatibleErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct OpenAICompatibleErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub code: Option<serde_json::Value>,
}

/// Strongly-typed OpenAI-compatible provider error.
#[derive(Debug, Error)]
pub enum OpenAICompatibleError {
    #[error("{provider} API error ({error_type}): {message}")]
    ApiError {
        provider: String,
        error_type: String,
        message: String,
        code: Option<String>,
        status_code: u16,
    },

    #[error("{provider} authentication error: {message}")]
    AuthenticationError { provider: String, message: String },

    #[error("{provider} rate limit error: {message}")]
    RateLimitError { provider: String, message: String },

    #[error("API request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("{0}")]
    Other(String),
}

impl OpenAICompatibleError {
    /// Parse an error response from an OpenAI-compatible provider.
    #[must_use]
    pub fn from_response(provider: &str, status: u16, body: &str) -> Self {
        let provider = provider.to_string();
        if let Ok(error_resp) = serde_json::from_str::<OpenAICompatibleErrorResponse>(body) {
            let message = error_resp.error.message.clone();
            let error_type = error_resp.error.error_type.clone().unwrap_or_default();
            let code = error_resp
                .error
                .code
                .as_ref()
                .map(std::string::ToString::to_string);

            match status {
                401 => OpenAICompatibleError::AuthenticationError { provider, message },
                429 => OpenAICompatibleError::RateLimitError { provider, message },
                _ => OpenAICompatibleError::ApiError {
                    provider,
                    error_type,
                    message,
                    code,
                    status_code: status,
                },
            }
        } else {
            OpenAICompatibleError::Other(format!("{provider} error: {body}"))
        }
    }
}
