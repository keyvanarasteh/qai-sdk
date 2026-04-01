use thiserror::Error;

/// Core error type representing various API and provider failures across the SDK.
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("API configuration error: {0}")]
    Configuration(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Context length exceeded: {0}")]
    ContextLengthExceeded(String),

    #[error("Unauthorized or authentication failed: {0}")]
    Unauthorized(String),

    #[error("Provider network or HTTP error: {0}")]
    Network(String),

    #[error("Invalid or structural response from provider: {0}")]
    InvalidResponse(String),

    #[error("Feature not supported natively by this provider instance: {0}")]
    NotSupported(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        ProviderError::Network(err.to_string())
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(err: serde_json::Error) -> Self {
        ProviderError::InvalidResponse(err.to_string())
    }
}

impl From<std::io::Error> for ProviderError {
    fn from(err: std::io::Error) -> Self {
        ProviderError::Other(anyhow::anyhow!(err))
    }
}
