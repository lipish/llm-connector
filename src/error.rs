//! Error types for llm-connector

/// Error types for llm-connector operations
#[derive(thiserror::Error, Debug)]
pub enum LlmConnectorError {
    /// Authentication failed with the provider
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    /// Network-related error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Invalid request format or parameters
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Model not supported by any provider
    #[error("Unsupported model: {0}")]
    UnsupportedModel(String),

    /// Provider-specific error
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Streaming-related error
    #[error("Streaming error: {0}")]
    StreamingError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// HTTP request error
    #[cfg(feature = "reqwest")]
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}

impl LlmConnectorError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            LlmConnectorError::NetworkError(_)
            | LlmConnectorError::RateLimitError(_)
            | LlmConnectorError::ProviderError(_)
        )
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            LlmConnectorError::AuthenticationError(_) => 401,
            LlmConnectorError::RateLimitError(_) => 429,
            LlmConnectorError::InvalidRequest(_) => 400,
            LlmConnectorError::UnsupportedModel(_) => 400,
            LlmConnectorError::ConfigError(_) => 500,
            LlmConnectorError::JsonError(_) => 400,
            LlmConnectorError::NetworkError(_) => 502,
            LlmConnectorError::ProviderError(_) => 502,
            LlmConnectorError::StreamingError(_) => 500,
            #[cfg(feature = "reqwest")]
            LlmConnectorError::HttpError(_) => 502,
        }
    }

    /// Create error from HTTP status code
    pub fn from_status_code(status: u16, message: String) -> Self {
        match status {
            401 | 403 => LlmConnectorError::AuthenticationError(message),
            429 => LlmConnectorError::RateLimitError(message),
            400 => LlmConnectorError::InvalidRequest(message),
            _ if status >= 500 => LlmConnectorError::ProviderError(message),
            _ => LlmConnectorError::NetworkError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LlmConnectorError;

    #[test]
    fn test_status_codes() {
        assert_eq!(LlmConnectorError::AuthenticationError("test".to_string()).status_code(), 401);
        assert_eq!(LlmConnectorError::RateLimitError("test".to_string()).status_code(), 429);
        assert_eq!(LlmConnectorError::InvalidRequest("test".to_string()).status_code(), 400);
        assert_eq!(LlmConnectorError::UnsupportedModel("test".to_string()).status_code(), 400);
        assert_eq!(LlmConnectorError::NetworkError("test".to_string()).status_code(), 502);
    }

    #[test]
    fn test_retryable() {
        assert!(LlmConnectorError::NetworkError("test".to_string()).is_retryable());
        assert!(LlmConnectorError::RateLimitError("test".to_string()).is_retryable());
        assert!(LlmConnectorError::ProviderError("test".to_string()).is_retryable());

        assert!(!LlmConnectorError::AuthenticationError("test".to_string()).is_retryable());
        assert!(!LlmConnectorError::InvalidRequest("test".to_string()).is_retryable());
        assert!(!LlmConnectorError::UnsupportedModel("test".to_string()).is_retryable());
    }

    #[test]
    fn test_from_status_code() {
        assert!(matches!(
            LlmConnectorError::from_status_code(401, "test".to_string()),
            LlmConnectorError::AuthenticationError(_)
        ));
        assert!(matches!(
            LlmConnectorError::from_status_code(429, "test".to_string()),
            LlmConnectorError::RateLimitError(_)
        ));
        assert!(matches!(
            LlmConnectorError::from_status_code(400, "test".to_string()),
            LlmConnectorError::InvalidRequest(_)
        ));
    }
}

