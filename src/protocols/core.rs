//! Core provider traits, transport, and error handling
//!
//! This module contains all the fundamental abstractions needed for provider implementations.
//!
//! # Architecture
//!
//! The module is organized into three main components:
//!
//! ## 1. Core Traits
//!
//! - **`Provider`**: Public-facing trait for external API consumers
//!   - Provides high-level methods: `chat()`, `chat_stream()`
//!   - Used by `LlmClient`
//!
//! - **`ProviderAdapter`**: Internal trait for protocol implementations
//!   - Handles protocol-specific request/response transformations
//!   - Implemented by `OpenAIProtocol`, `AnthropicProtocol`, `AliyunProtocol`
//!
//! - **`ErrorMapper`**: Protocol-specific error handling
//!   - Maps HTTP status codes to appropriate errors
//!   - Determines which errors are retriable
//!
//! ## 2. HTTP Transport
//!
//! - **`HttpTransport`**: Shared HTTP client and configuration
//!   - Uses `Arc` for zero-copy sharing across providers
//!   - Handles proxy, timeout, and custom headers
//!
//! ## 3. Generic Provider
//!
//! - **`GenericProvider<A>`**: Universal provider implementation
//!   - Works with any `ProviderAdapter`
//!   - Handles HTTP communication, retries, and streaming
//!   - Single implementation for all protocols
//!
//! # Example
//!
//! ```rust,no_run
//! use llm_connector::{LlmClient, ChatRequest, Message};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create client with OpenAI protocol
//! let client = LlmClient::openai("sk-...", None);
//!
//! // Create request
//! let request = ChatRequest {
//!     model: "gpt-4".to_string(),
//!     messages: vec![Message::user("Hello!")],
//!     ..Default::default()
//! };
//!
//! // Send request
//! let response = client.chat(&request).await?;
//! println!("Response: {}", response.choices[0].message.content);
//! # Ok(())
//! # }
//! ```

use crate::config::{ProviderConfig, SharedProviderConfig};
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse};
use async_trait::async_trait;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

#[cfg(feature = "streaming")]
use crate::types::{ChatStream, StreamingResponse};

// ============================================================================
// Core Traits
// ============================================================================

/// Main provider trait for external API
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Fetch available models from the API (online)
    ///
    /// This makes an API call to retrieve the list of available models.
    /// Returns an error if the provider doesn't support model listing or if the API call fails.
    async fn fetch_models(&self) -> Result<Vec<String>, LlmConnectorError>;

    /// Send a chat completion request
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError>;

    /// Send a streaming chat completion request
    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError>;

    /// Get as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Internal adapter trait for different provider APIs
#[async_trait]
pub trait ProviderAdapter: Send + Sync + Clone + 'static {
    type RequestType: Serialize + Send + Sync;
    type ResponseType: DeserializeOwned + Send + Sync;
    #[cfg(feature = "streaming")]
    type StreamResponseType: DeserializeOwned + Send + Sync;
    type ErrorMapperType: ErrorMapper;

    fn name(&self) -> &str;
    fn endpoint_url(&self, base_url: &Option<String>) -> String;

    /// Get the models endpoint URL (for fetching available models)
    ///
    /// Returns None if the provider doesn't support model listing
    fn models_endpoint_url(&self, base_url: &Option<String>) -> Option<String> {
        let _ = base_url;
        None // Default: no model listing support
    }

    fn build_request_data(&self, request: &ChatRequest, stream: bool) -> Self::RequestType;
    fn parse_response_data(&self, response: Self::ResponseType) -> ChatResponse;

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> StreamingResponse;
}

/// Error mapping trait for provider-specific error handling
pub trait ErrorMapper {
    fn map_http_error(status: u16, body: Value) -> LlmConnectorError;
    fn map_network_error(error: reqwest::Error) -> LlmConnectorError;
    fn is_retriable_error(error: &LlmConnectorError) -> bool;
}

// ============================================================================
// HTTP Transport
// ============================================================================

/// HTTP transport layer for making requests to provider APIs
///
/// Uses Arc for efficient sharing of client and config across clones.
#[derive(Clone, Debug)]
pub struct HttpTransport {
    pub client: Arc<Client>,
    pub config: SharedProviderConfig,
}

impl HttpTransport {
    pub fn new(client: Client, config: ProviderConfig) -> Self {
        Self {
            client: Arc::new(client),
            config: SharedProviderConfig::new(config),
        }
    }

    /// Create from shared components (zero-copy)
    pub fn from_shared(client: Arc<Client>, config: SharedProviderConfig) -> Self {
        Self { client, config }
    }

    /// Build HTTP client with proxy and timeout configuration
    pub fn build_client(
        proxy: &Option<String>,
        timeout_ms: Option<u64>,
        base_url: Option<&String>,
    ) -> Result<Client, LlmConnectorError> {
        let mut client_builder = Client::builder();

        if let Some(proxy) = proxy {
            client_builder = client_builder.proxy(reqwest::Proxy::all(proxy)?);
        }

        if let Some(timeout) = timeout_ms {
            client_builder = client_builder.timeout(std::time::Duration::from_millis(timeout));
        }

        // If the base_url points to localhost, disable proxy to avoid 502 from system proxies
        if let Some(base) = base_url {
            if let Ok(url) = reqwest::Url::parse(base) {
                if matches!(url.host_str(), Some("localhost") | Some("127.0.0.1")) {
                    client_builder = client_builder.no_proxy();
                }
            }
        }

        client_builder
            .build()
            .map_err(|e| LlmConnectorError::ConfigError(e.to_string()))
    }

    /// Send GET request
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, LlmConnectorError> {
        let mut request = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", &self.config.api_key));

        // Apply custom headers if configured
        if let Some(headers) = &self.config.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        request
            .send()
            .await
            .map_err(LlmConnectorError::from)
    }

    /// Send POST request with JSON body
    pub async fn post<T: Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<reqwest::Response, LlmConnectorError> {
        let mut request = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", &self.config.api_key))
            .header("Content-Type", "application/json");

        // Apply custom headers if configured
        if let Some(headers) = &self.config.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        request
            .json(body)
            .send()
            .await
            .map_err(LlmConnectorError::from)
    }

    /// Send streaming POST request
    #[cfg(feature = "streaming")]
    pub async fn stream<T: Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<
        impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>>,
        LlmConnectorError,
    > {
        let mut request = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", &self.config.api_key))
            .header("Content-Type", "application/json");

        // Apply custom headers if configured
        if let Some(headers) = &self.config.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        let response = request
            .json(body)
            .send()
            .await
            .map_err(LlmConnectorError::from)?;

        if !response.status().is_success() {
            return Err(LlmConnectorError::ProviderError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        Ok(response.bytes_stream())
    }
}

// ============================================================================
// Standard Error Mapper for OpenAI-compatible providers
// ============================================================================

/// Standard error mapper for OpenAI-compatible APIs
pub struct StandardErrorMapper;

impl ErrorMapper for StandardErrorMapper {
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

// ============================================================================
// Generic Provider Implementation
// ============================================================================

/// Generic provider that works with any adapter
#[derive(Clone)]
pub struct GenericProvider<A: ProviderAdapter> {
    adapter: A,
    transport: HttpTransport,
}

impl<A: ProviderAdapter> GenericProvider<A> {
    pub fn new(config: ProviderConfig, adapter: A) -> Result<Self, LlmConnectorError> {
        let client = HttpTransport::build_client(
            &config.proxy,
            config.timeout_ms,
            config.base_url.as_ref(),
        )?;

        let transport = HttpTransport::new(client, config);

        Ok(Self { adapter, transport })
    }

    /// Get access to the underlying adapter
    pub fn adapter(&self) -> &A {
        &self.adapter
    }
}

#[async_trait]
impl<A: ProviderAdapter> Provider for GenericProvider<A> {
    fn name(&self) -> &str {
        self.adapter.name()
    }

    async fn fetch_models(&self) -> Result<Vec<String>, LlmConnectorError> {
        // Check if the adapter supports model listing
        let url = self.adapter.models_endpoint_url(&self.transport.config.base_url)
            .ok_or_else(|| LlmConnectorError::UnsupportedOperation(
                format!("{} does not support model listing", self.adapter.name())
            ))?;

        let response = self.transport.get(&url).await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body: Value = response.json().await.unwrap_or_default();
            return Err(A::ErrorMapperType::map_http_error(status, body));
        }

        // Parse the response - for OpenAI-compatible APIs
        let models_response: Value = response
            .json()
            .await
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;

        // Extract model IDs from the response
        // OpenAI format: { "data": [{ "id": "model-name", ... }] }
        if let Some(data) = models_response.get("data").and_then(|d| d.as_array()) {
            let models = data
                .iter()
                .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(String::from))
                .collect();
            Ok(models)
        } else {
            Err(LlmConnectorError::ParseError(
                "Invalid models response format".to_string()
            ))
        }
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        let url = self.adapter.endpoint_url(&self.transport.config.base_url);
        let request_data = self.adapter.build_request_data(request, false);

        let response = self.transport.post(&url, &request_data).await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body: Value = response.json().await.unwrap_or_default();
            return Err(A::ErrorMapperType::map_http_error(status, body));
        }

        let response_data: A::ResponseType = response
            .json()
            .await
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;

        Ok(self.adapter.parse_response_data(response_data))
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        use crate::sse::sse_events;
        use futures_util::StreamExt;

        let url = self.adapter.endpoint_url(&self.transport.config.base_url);
        let request_data = self.adapter.build_request_data(request, true);

        let response = self
            .transport
            .client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", &self.transport.config.api_key),
            )
            .header("Content-Type", "application/json")
            .json(&request_data)
            .send()
            .await
            .map_err(LlmConnectorError::from)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body: Value = response.json().await.unwrap_or_default();
            return Err(A::ErrorMapperType::map_http_error(status, body));
        }

        let mapped_stream = crate::sse::sse_events(response).filter_map(|event| async move {
            match event {
                Ok(data) => {
                    if data.trim() == "[DONE]" {
                        return None;
                    }

                    match serde_json::from_str::<A::StreamResponseType>(&data) {
                        Ok(stream_response) => {
                            Some(Ok(self.adapter.parse_stream_response_data(stream_response)))
                        }
                        Err(e) => Some(Err(LlmConnectorError::ParseError(e.to_string()))),
                    }
                }
                Err(e) => Some(Err(LlmConnectorError::StreamingError(e.to_string()))),
            }
        });

        Ok(Box::pin(mapped_stream))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
