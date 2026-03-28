use serde::Deserialize;
use thiserror::Error;

/// Anthropic API error response.
#[derive(Debug, Deserialize)]
pub struct AnthropicErrorResponse {
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub error: AnthropicErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

/// Strongly-typed Anthropic error.
#[derive(Debug, Error)]
pub enum AnthropicError {
    #[error("Anthropic API error ({error_type}): {message}")]
    ApiError {
        error_type: String,
        message: String,
        status_code: u16,
    },

    #[error("Anthropic authentication error: {message}")]
    AuthenticationError { message: String },

    #[error("Anthropic rate limit error: {message}")]
    RateLimitError { message: String },

    #[error("Anthropic request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Anthropic error: {0}")]
    Other(String),
}

impl AnthropicError {
    /// Parse an error response from Anthropic API.
    pub fn from_response(status: u16, body: &str) -> Self {
        if let Ok(error_resp) = serde_json::from_str::<AnthropicErrorResponse>(body) {
            let error_type = error_resp.error.error_type.clone();
            let message = error_resp.error.message.clone();

            match error_type.as_str() {
                "authentication_error" => AnthropicError::AuthenticationError { message },
                "rate_limit_error" => AnthropicError::RateLimitError { message },
                _ => AnthropicError::ApiError {
                    error_type,
                    message,
                    status_code: status,
                },
            }
        } else {
            AnthropicError::Other(body.to_string())
        }
    }
}
