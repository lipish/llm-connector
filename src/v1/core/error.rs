//! Error handling utilities for protocols and providers

use serde_json::Value;
use crate::error::LlmConnectorError;
use crate::v1::core::protocol::ProtocolError;

/// Standard error mapper for OpenAI-compatible APIs
pub struct StandardErrorMapper;

impl ProtocolError for StandardErrorMapper {
    fn map_http_error(status: u16, body: Value) -> LlmConnectorError {
        let error_message = body["error"]["message"].as_str().unwrap_or("Unknown error");
        let error_type = body["error"]["type"].as_str().unwrap_or("unknown_error");

        // Clean up error message if it contains OpenAI-specific URLs
        let cleaned_message = if error_message.contains("platform.openai.com") {
            // Extract just the main error message before the URL
            if let Some(idx) = error_message.find(". You can find your API key at") {
                &error_message[..idx]
            } else {
                error_message
            }
        } else {
            error_message
        };

        match status {
            400 => LlmConnectorError::InvalidRequest(cleaned_message.to_string()),
            401 => LlmConnectorError::AuthenticationError(format!(
                "{}. Please verify your API key is correct and has the necessary permissions.",
                cleaned_message
            )),
            403 => LlmConnectorError::PermissionError(cleaned_message.to_string()),
            404 => LlmConnectorError::NotFoundError(cleaned_message.to_string()),
            429 => LlmConnectorError::RateLimitError(cleaned_message.to_string()),
            500..=599 => {
                LlmConnectorError::ServerError(format!("HTTP {}: {}", status, cleaned_message))
            }
            _ => LlmConnectorError::ProviderError(format!(
                "HTTP {}: {} (type: {})",
                status, cleaned_message, error_type
            )),
        }
    }

    fn map_network_error(error: reqwest::Error) -> LlmConnectorError {
        if error.is_timeout() {
            LlmConnectorError::TimeoutError(error.to_string())
        } else if error.is_connect() {
            LlmConnectorError::ConnectionError(error.to_string())
        } else {
            LlmConnectorError::NetworkError(error.to_string())
        }
    }

    fn is_retriable_error(error: &LlmConnectorError) -> bool {
        matches!(
            error,
            LlmConnectorError::RateLimitError(_)
                | LlmConnectorError::ServerError(_)
                | LlmConnectorError::TimeoutError(_)
                | LlmConnectorError::ConnectionError(_)
        )
    }
}

/// Re-export for compatibility
pub use StandardErrorMapper as ErrorMapper;