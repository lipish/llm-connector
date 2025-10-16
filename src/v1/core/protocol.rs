//! Protocol trait for pure API specifications
//!
//! Protocols implement official API specifications exactly as defined by the service.
//! They should be minimal, pure, and free from provider-specific customizations.

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};

use crate::error::LlmConnectorError;
use crate::types::{ChatRequest as Request, ChatResponse as Response};

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// Protocol trait for pure API specification implementations
///
/// # Examples
///
/// ```rust,no_run
/// use llm_connector::core::Protocol;
/// use serde::{Serialize, Deserialize};
///
/// struct MyProtocol {
///     name: Arc<str>,
/// }
///
/// impl Protocol for MyProtocol {
///     type Request = MyRequest;
///     type Response = MyResponse;
///     type StreamResponse = MyStreamResponse;
///     type Error = MyError;
///
///     fn name(&self) -> &str {
///         &self.name
///     }
///
///     fn endpoint(&self, base_url: &str) -> String {
///         format!("{}/chat/completions", base_url)
///     }
///
///     fn build_request(&self, request: &Request, stream: bool) -> Self::Request {
///         MyRequest::from(request, stream)
///     }
///
///     fn parse_response(&self, response: Self::Response) -> Response {
///         response.into()
///     }
/// }
/// ```

#[async_trait]
pub trait Protocol: Send + Sync + 'static {
    /// Request type for this protocol
    type Request: Serialize + Send + Sync;

    /// Response type for this protocol
    type Response: DeserializeOwned + Send + Sync;

    /// Streaming response type for this protocol
    #[cfg(feature = "streaming")]
    type StreamResponse: DeserializeOwned + Send + Sync;

    /// Streaming response type for this protocol (dummy when no streaming)
    #[cfg(not(feature = "streaming"))]
    type StreamResponse: Send + Sync;

    /// Protocol-specific error type
    type Error: ProtocolError;

    /// Protocol name (e.g., "openai", "anthropic")
    fn name(&self) -> &str;

    /// Get the API endpoint URL for chat completions
    fn endpoint(&self, base_url: &str) -> String;

    /// Get the models endpoint URL (optional, not all APIs support model listing)
    fn models_endpoint(&self, base_url: &str) -> Option<String>;

    /// Build protocol-specific request from generic request
    fn build_request(&self, request: &Request, stream: bool) -> Self::Request;

    /// Parse protocol-specific response to generic response
    fn parse_response(&self, response: Self::Response) -> Response;

    /// Parse protocol-specific streaming response to generic response
    #[cfg(feature = "streaming")]
    fn parse_stream_response(&self, response: Self::StreamResponse) -> ChatStream;

    /// Check if this protocol uses Server-Sent Events for streaming
    #[cfg(feature = "streaming")]
    fn uses_sse_stream(&self) -> bool { true }

    /// Validate success response body for APIs that return HTTP 200 with error in JSON
    fn validate_success_body(&self, status: u16, raw: &serde_json::Value) -> Result<(), LlmConnectorError> {
        let _ = (status, raw);
        Ok(())
    }
}

/// Protocol-specific error handling trait
pub trait ProtocolError: Send + Sync {
    /// Map HTTP status and body to connector error
    fn map_http_error(status: u16, body: serde_json::Value) -> LlmConnectorError;

    /// Map network error to connector error
    fn map_network_error(error: reqwest::Error) -> LlmConnectorError;

    /// Check if an error is retriable
    fn is_retriable_error(error: &LlmConnectorError) -> bool;
}