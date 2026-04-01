use serde::Deserialize;
use thiserror::Error;

/// `OpenAI` API error response.
#[derive(Debug, Deserialize)]
pub struct OpenAIErrorResponse {
    pub error: OpenAIErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub param: Option<serde_json::Value>,
    pub code: Option<serde_json::Value>,
}

/// Strongly-typed `OpenAI` error.
#[derive(Debug, Error)]
pub enum OpenAIError {
    #[error("OpenAI API error ({error_type}): {message}")]
    ApiError {
        error_type: String,
        message: String,
        code: Option<String>,
        status_code: u16,
    },

    #[error("OpenAI authentication error: {message}")]
    AuthenticationError { message: String },

    #[error("OpenAI rate limit error: {message}")]
    RateLimitError { message: String },

    #[error("OpenAI request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("OpenAI error: {0}")]
    Other(String),
}

impl OpenAIError {
    /// Parse an error response from `OpenAI` API.
    #[must_use]
    pub fn from_response(status: u16, body: &str) -> Self {
        if let Ok(error_resp) = serde_json::from_str::<OpenAIErrorResponse>(body) {
            let message = error_resp.error.message.clone();
            let error_type = error_resp.error.error_type.clone().unwrap_or_default();
            let code = error_resp.error.code.as_ref().map(ToString::to_string);

            match status {
                401 => OpenAIError::AuthenticationError { message },
                429 => OpenAIError::RateLimitError { message },
                _ => OpenAIError::ApiError {
                    error_type,
                    message,
                    code,
                    status_code: status,
                },
            }
        } else {
            OpenAIError::Other(body.to_string())
        }
    }
}
