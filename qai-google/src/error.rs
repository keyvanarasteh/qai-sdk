use serde::Deserialize;
use thiserror::Error;

/// Google API error response.
#[derive(Debug, Deserialize)]
pub struct GoogleErrorResponse {
    pub error: GoogleErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct GoogleErrorDetail {
    pub code: Option<i32>,
    pub message: String,
    pub status: Option<String>,
}

/// Strongly-typed Google error.
#[derive(Debug, Error)]
pub enum GoogleError {
    #[error("Google API error ({status}): {message}")]
    ApiError {
        code: Option<i32>,
        message: String,
        status: String,
        status_code: u16,
    },

    #[error("Google authentication error: {message}")]
    AuthenticationError { message: String },

    #[error("Google rate limit error: {message}")]
    RateLimitError { message: String },

    #[error("Google request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Google error: {0}")]
    Other(String),
}

impl GoogleError {
    /// Parse an error response from Google API.
    pub fn from_response(status: u16, body: &str) -> Self {
        if let Ok(error_resp) = serde_json::from_str::<GoogleErrorResponse>(body) {
            let message = error_resp.error.message.clone();
            let error_status = error_resp.error.status.clone().unwrap_or_default();

            match status {
                401 | 403 => GoogleError::AuthenticationError { message },
                429 => GoogleError::RateLimitError { message },
                _ => GoogleError::ApiError {
                    code: error_resp.error.code,
                    message,
                    status: error_status,
                    status_code: status,
                },
            }
        } else {
            GoogleError::Other(body.to_string())
        }
    }
}
