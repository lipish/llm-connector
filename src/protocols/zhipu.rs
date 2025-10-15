//! Zhipu Protocol Implementation
//!
//! Zhipu provides both OpenAI-compatible (`/api/openai/v1`) and PaaS v4 (`/api/paas/v4`) endpoints.
//! This adapter targets Zhipu, reusing OpenAI request/response shapes on success, while handling
//! Zhipu-specific error bodies like `{"code":500,"msg":"404 NOT_FOUND","success":false}`.

use crate::core::{Protocol, protocol::ProtocolError};
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest as Request, ChatResponse as Response};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

// Reuse OpenAI-compatible structures for request/response on success
use crate::protocols::openai::{OpenAIRequest, OpenAIResponse};
#[cfg(feature = "streaming")]
use crate::protocols::openai::OpenAIStreamResponse;

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// Zhipu-specific error mapper
pub struct ZhipuErrorMapper;

impl ProtocolError for ZhipuErrorMapper {
    fn map_http_error(status: u16, body: Value) -> crate::error::LlmConnectorError {
        // Prefer Zhipu's code/msg if present
        let code = body.get("code").and_then(|v| v.as_i64()).unwrap_or(status as i64);
        let msg = body.get("msg").and_then(|v| v.as_str()).unwrap_or("Unknown error");
        match (status, code) {
            (404, _) | (_, 404) => crate::error::LlmConnectorError::NotFoundError(msg.to_string()),
            (401, _) | (_, 401) => crate::error::LlmConnectorError::AuthenticationError(msg.to_string()),
            (403, _) | (_, 403) => crate::error::LlmConnectorError::PermissionError(msg.to_string()),
            (429, _) | (_, 429) => crate::error::LlmConnectorError::RateLimitError(msg.to_string()),
            (500..=599, _) | (_, 500..=599) => crate::error::LlmConnectorError::ServerError(format!("HTTP {}: {}", status, msg)),
            _ => crate::error::LlmConnectorError::ProviderError(format!("HTTP {}: {} (code: {})", status, msg, code)),
        }
    }

    fn map_network_error(error: reqwest::Error) -> crate::error::LlmConnectorError {
        if error.is_timeout() {
            crate::error::LlmConnectorError::TimeoutError(error.to_string())
        } else if error.is_connect() {
            crate::error::LlmConnectorError::ConnectionError(error.to_string())
        } else {
            crate::error::LlmConnectorError::NetworkError(error.to_string())
        }
    }

    fn is_retriable_error(error: &crate::error::LlmConnectorError) -> bool {
        matches!(
            error,
            crate::error::LlmConnectorError::RateLimitError(_)
                | crate::error::LlmConnectorError::ServerError(_)
                | crate::error::LlmConnectorError::TimeoutError(_)
                | crate::error::LlmConnectorError::ConnectionError(_)
        )
    }
}

/// Zhipu Protocol for ChatGLM API
///
/// Protocol implementation using OpenAI-compatible format with Zhipu-specific error handling.
#[derive(Debug, Clone)]
pub struct ZhipuProtocol {
    name: Arc<str>,
}

impl ZhipuProtocol {
    /// Create new Zhipu protocol
    pub fn new() -> Self {
        Self {
            name: Arc::from("zhipu"),
        }
    }
}

impl Default for ZhipuProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Protocol for ZhipuProtocol {
    type Request = OpenAIRequest;
    type Response = OpenAIResponse;
    type Error = ZhipuErrorMapper;

    #[cfg(feature = "streaming")]
    type StreamResponse = OpenAIStreamResponse;

    #[cfg(not(feature = "streaming"))]
    type StreamResponse = ();

    fn name(&self) -> &str {
        &self.name
    }

    fn endpoint(&self, base_url: &str) -> String {
        // Zhipu uses PaaS v4 endpoint
        format!("{}/chat/completions", base_url)
    }

    fn models_endpoint(&self, _base_url: &str) -> Option<String> {
        // Zhipu doesn't provide a reliable models endpoint
        None
    }

    fn build_request(&self, request: &Request, stream: bool) -> Self::Request {
        OpenAIRequest::from_chat_request(request, stream)
    }

    fn parse_response(&self, response: Self::Response) -> Response {
        response.to_chat_response()
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response(&self, response: Self::StreamResponse) -> ChatStream {
        use futures_util::stream;

        let streaming_response = response.to_streaming_response();

        // Convert to stream of streaming responses
        let stream = stream::once(async { Ok(streaming_response) });
        Box::pin(stream)
    }

    #[cfg(feature = "streaming")]
    fn uses_sse_stream(&self) -> bool {
        true // Zhipu supports OpenAI-compatible SSE streaming
    }

    fn validate_success_body(&self, status: u16, raw: &Value) -> Result<(), LlmConnectorError> {
        // If Zhipu wraps errors under HTTP 200 with { success: false, code, msg }
        if let Some(success) = raw.get("success").and_then(|v| v.as_bool()) {
            if !success {
                return Err(ZhipuErrorMapper::map_http_error(status, raw.clone()));
            }
        }
        // Some payloads may use `code != 0` as error, even without `success`
        if let Some(code) = raw.get("code").and_then(|v| v.as_i64()) {
            if code != 0 && raw.get("choices").is_none() {
                return Err(ZhipuErrorMapper::map_http_error(status, raw.clone()));
            }
        }
        Ok(())
    }
}

/// Convenience function to create a Zhipu provider
pub fn zhipu(base_url: &str, api_key: &str) -> Result<crate::core::provider::ProtocolProvider<ZhipuProtocol>, LlmConnectorError> {
    let protocol = ZhipuProtocol::new();
    crate::core::provider::ProtocolProvider::new(protocol, base_url, api_key)
}

/// Convenience function to create a Zhipu provider with default endpoint
pub fn zhipu_default(api_key: &str) -> Result<crate::core::provider::ProtocolProvider<ZhipuProtocol>, LlmConnectorError> {
    zhipu("https://open.bigmodel.cn/api/paas/v4", api_key)
}

/// Convenience function to create a Zhipu provider with default endpoint and custom timeout
pub fn zhipu_with_timeout(api_key: &str, timeout_ms: u64) -> Result<crate::core::provider::ProtocolProvider<ZhipuProtocol>, LlmConnectorError> {
    let protocol = ZhipuProtocol::new();
    let config = crate::config::ProviderConfig::new(api_key)
        .with_base_url("https://open.bigmodel.cn/api/paas/v4")
        .with_timeout_ms(timeout_ms);

    let client = crate::core::HttpTransport::build_client(
        &config.proxy,
        config.timeout_ms,
        config.base_url.as_ref(),
    )?;

    let transport = crate::core::HttpTransport::new(client, config);

    Ok(crate::core::provider::ProtocolProvider::from_parts(protocol, "https://open.bigmodel.cn/api/paas/v4", transport))
}