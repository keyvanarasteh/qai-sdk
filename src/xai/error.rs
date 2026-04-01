//! xAI API error types.
//!
//! xAI uses the same error response format as `OpenAI` since it's
//! API-compatible. This module provides an `XaiError` enum that
//! distinguishes between authentication, rate limit, and general API errors.

use serde::Deserialize;
use thiserror::Error;

/// xAI API error response.
#[derive(Debug, Deserialize)]
pub struct XaiErrorResponse {
    pub error: XaiErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct XaiErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub code: Option<serde_json::Value>,
}

/// Strongly-typed xAI error.
#[derive(Debug, Error)]
pub enum XaiError {
    #[error("xAI API error ({error_type}): {message}")]
    ApiError {
        error_type: String,
        message: String,
        code: Option<String>,
        status_code: u16,
    },

    #[error("xAI authentication error: {message}")]
    AuthenticationError { message: String },

    #[error("xAI rate limit error: {message}")]
    RateLimitError { message: String },

    #[error("xAI request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("xAI error: {0}")]
    Other(String),
}

impl XaiError {
    /// Parse an error response from xAI API.
    #[must_use]
    pub fn from_response(status: u16, body: &str) -> Self {
        if let Ok(error_resp) = serde_json::from_str::<XaiErrorResponse>(body) {
            let message = error_resp.error.message.clone();
            let error_type = error_resp.error.error_type.clone().unwrap_or_default();
            let code = error_resp
                .error
                .code
                .as_ref()
                .map(std::string::ToString::to_string);

            match status {
                401 => XaiError::AuthenticationError { message },
                429 => XaiError::RateLimitError { message },
                _ => XaiError::ApiError {
                    error_type,
                    message,
                    code,
                    status_code: status,
                },
            }
        } else {
            XaiError::Other(body.to_string())
        }
    }
}
