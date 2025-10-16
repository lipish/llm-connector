//! Provider trait for service implementations
//!
//! This module defines the unified `Provider` trait that all LLM service
//! implementations must implement. This is the main interface for LLM services.

use async_trait::async_trait;

use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse};
use crate::v1::core::Protocol;
use crate::v1::core::protocol::ProtocolError;

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// Provider trait for service implementations
///
/// Providers can implement custom APIs or use standard protocols.
/// They handle all service-specific logic including authentication,
/// error mapping, and special features.
///
/// # Examples
///
/// ```rust,no_run
/// use llm_connector::core::Provider;
/// use llm_connector::error::LlmConnectorError;
/// use llm_connector::types::{Request, Response};
///
/// struct MyProvider {
///     client: reqwest::Client,
///     api_key: String,
/// }
///
/// #[async_trait]
/// impl Provider for MyProvider {
///     fn name(&self) -> &str {
///         "my_provider"
///     }
///
///     async fn chat(&self, request: &Request) -> Result<Response, LlmConnectorError> {
///         // Custom implementation
///         let endpoint = "https://api.example.com/chat";
///         let response = self.client
///             .post(endpoint)
///             .header("Authorization", format!("Bearer {}", self.api_key))
///             .json(&self.build_request(request))
///             .send()
///             .await?;
///
///         self.parse_response(response).await
///     }
///
///     async fn fetch_models(&self) -> Result<Vec<String>, LlmConnectorError> {
///         // Model fetching implementation
///         Ok(vec!["model-1".to_string(), "model-2".to_string()])
///     }
/// }
/// ```

#[async_trait]
pub trait Provider: Send + Sync + 'static {
    /// Provider name (e.g., "aliyun", "zhipu", "ollama")
    fn name(&self) -> &str;

    /// Send a chat completion request
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError>;

    /// Send a streaming chat completion request
    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError>;

    /// Fetch available models from the provider
    ///
    /// Returns an error if the provider doesn't support model listing
    async fn fetch_models(&self) -> Result<Vec<String>, LlmConnectorError>;

    /// Get provider as Any for downcasting to specific types
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Generic provider implementation for protocol-compatible services
///
/// This wrapper allows services that use standard protocols (like OpenAI)
/// to be used as providers with minimal customization.
///
/// # Examples
///
/// ```rust,no_run
/// use llm_connector::core::{Protocol, Provider, ProtocolProvider};
/// use llm_connector::protocols::OpenAIProtocol;
///
/// // Create a provider from OpenAI protocol
/// let protocol = OpenAIProtocol::new();
/// let provider = ProtocolProvider::new(
///     protocol,
///     "https://api.deepseek.com/v1",
///     "sk-your-deepseek-key"
/// );
/// ```

pub struct ProtocolProvider<P: Protocol> {
    protocol: P,
    base_url: String,
    transport: crate::v1::core::HttpTransport,
}

impl<P: Protocol> ProtocolProvider<P> {
    /// Create a new provider from a protocol
    pub fn new(
        protocol: P,
        base_url: &str,
        api_key: &str,
    ) -> Result<Self, LlmConnectorError> {
        let config = crate::config::ProviderConfig::new(api_key)
            .with_base_url(base_url.to_string());

        let client = crate::v1::core::HttpTransport::build_client(
            &config.proxy,
            config.timeout_ms,
            config.base_url.as_ref(),
        )?;

        let transport = crate::v1::core::HttpTransport::new(client, config);

        Ok(Self {
            protocol,
            base_url: base_url.to_string(),
            transport,
        })
    }

    /// Create a new provider from pre-configured parts
    pub fn from_parts(protocol: P, base_url: &str, transport: crate::v1::core::HttpTransport) -> Self {
        Self {
            protocol,
            base_url: base_url.to_string(),
            transport,
        }
    }

    /// Get reference to the underlying protocol
    pub fn protocol(&self) -> &P {
        &self.protocol
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream_sse_impl(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        use futures_util::{StreamExt, TryStreamExt, stream};

        let endpoint = self.protocol.endpoint(&self.base_url);
        let protocol_request = self.protocol.build_request(request, true);

        // Get streaming response
        let byte_stream = self.transport.stream(&endpoint, &protocol_request).await?;

        // Process SSE stream and convert to streaming responses
        let sse_stream = byte_stream
            .map_ok(|chunk| String::from_utf8_lossy(&chunk).to_string())
            .map(|result| {
                match result {
                    Ok(chunk_text) => {
                        // Process all SSE lines in this chunk
                        let mut responses = Vec::new();
                        for line in chunk_text.lines() {
                            if line.starts_with("data: ") {
                                let data = line[6..].trim(); // Remove "data: " prefix
                                if data == "[DONE]" {
                                    break; // End of stream marker
                                }

                                // Try to parse as generic streaming response
                                if let Ok(mut response) = serde_json::from_str::<crate::types::StreamingResponse>(data) {
                                    // Ensure content field is populated from delta.content
                                    if response.content.is_empty() {
                                        if let Some(first_choice) = response.choices.first() {
                                            if let Some(ref delta_content) = first_choice.delta.content {
                                                response.content = delta_content.clone();
                                            }
                                        }
                                    }
                                    responses.push(Ok(response));
                                } else {
                                    // Silently skip invalid JSON chunks (common in SSE streams)
                                    continue;
                                }
                            }
                        }
                        responses
                    }
                    Err(e) => vec![Err(LlmConnectorError::NetworkError(e.to_string()))],
                }
            })
            .flat_map(stream::iter);

        Ok(Box::pin(sse_stream))
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream_fallback_impl(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        use futures_util::stream;

        // For non-SSE protocols, make a regular request and convert to single chunk stream
        let response = self.chat(request).await?;

        let stream_response = crate::types::StreamingResponse {
            id: response.id,
            object: "chat.completion.chunk".to_string(),
            created: response.created,
            model: response.model,
            choices: response.choices.into_iter().map(|choice| {
                crate::types::StreamingChoice {
                    index: choice.index,
                    delta: crate::types::Delta {
                        role: Some(crate::types::Role::Assistant),
                        content: Some(choice.message.content),
                        ..Default::default()
                    },
                    finish_reason: choice.finish_reason,
                    logprobs: None,
                }
            }).collect(),
            content: response.content,
            reasoning_content: None,
            usage: response.usage,
            system_fingerprint: response.system_fingerprint,
        };

        let single_chunk_stream = stream::once(async { Ok(stream_response) });
        Ok(Box::pin(single_chunk_stream))
    }
}

#[async_trait]
impl<P: Protocol> Provider for ProtocolProvider<P>
where
    P::Error: Send + Sync,
{
    fn name(&self) -> &str {
        self.protocol.name()
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        let endpoint = self.protocol.endpoint(&self.base_url);
        let protocol_request = self.protocol.build_request(request, false);

        let response = self.transport.post(&endpoint, &protocol_request).await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            return Err(P::Error::map_http_error(status, body));
        }

        let status = response.status().as_u16();
        let text = response.text().await.map_err(|e| {
            LlmConnectorError::ParseError(e.to_string())
        })?;
        let raw: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();

        // Allow protocol-specific validation
        self.protocol.validate_success_body(status, &raw)?;

        // Parse response
        let protocol_response: P::Response = serde_json::from_str(&text)
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;

        Ok(self.protocol.parse_response(protocol_response))
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        if self.protocol.uses_sse_stream() {
            // For SSE protocols (like OpenAI), use real streaming
            self.chat_stream_sse_impl(request).await
        } else {
            // For non-SSE protocols, fall back to single chunk
            self.chat_stream_fallback_impl(request).await
        }
    }

    async fn fetch_models(&self) -> Result<Vec<String>, LlmConnectorError> {
        let models_endpoint = self.protocol.models_endpoint(&self.base_url)
            .ok_or_else(|| LlmConnectorError::UnsupportedOperation(
                format!("{} does not support model listing", self.protocol.name())
            ))?;

        let response = self.transport.get(&models_endpoint).await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            return Err(P::Error::map_http_error(status, body));
        }

        // Parse models response (assuming OpenAI-compatible format)
        let models_response: serde_json::Value = response.json().await
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;

        if let Some(data) = models_response.get("data").and_then(|d| d.as_array()) {
            let models = data
                .iter()
                .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(String::from))
                .collect();
            Ok(models)
        } else {
            Err(LlmConnectorError::ParseError("Invalid models response format".to_string()))
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Bridge implementation: Implement old Provider trait for ProtocolProvider
/// This allows gradual migration from the old architecture to the new one
#[async_trait]
impl<P: Protocol> crate::v1::protocols::Provider for ProtocolProvider<P>
where
    P::Error: Send + Sync,
{
    fn name(&self) -> &str {
        self.protocol.name()
    }

    async fn chat(&self, request: &crate::types::ChatRequest) -> Result<crate::types::ChatResponse, LlmConnectorError> {
        let endpoint = self.protocol.endpoint(&self.base_url);
        let protocol_request = self.protocol.build_request(request, false);

        let response = self.transport.post(&endpoint, &protocol_request).await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            return Err(P::Error::map_http_error(status, body));
        }

        let status = response.status().as_u16();
        let text = response.text().await.map_err(|e| {
            LlmConnectorError::ParseError(e.to_string())
        })?;
        let raw: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();

        // Allow protocol-specific validation
        self.protocol.validate_success_body(status, &raw)?;

        // Parse response
        let protocol_response: P::Response = serde_json::from_str(&text)
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;

        Ok(self.protocol.parse_response(protocol_response))
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &crate::types::ChatRequest) -> Result<crate::types::ChatStream, LlmConnectorError> {
        // Delegate to the new Provider implementation
        <Self as crate::v1::core::Provider>::chat_stream(self, request).await
    }

    async fn fetch_models(&self) -> Result<Vec<String>, LlmConnectorError> {
        let models_endpoint = self.protocol.models_endpoint(&self.base_url)
            .ok_or_else(|| LlmConnectorError::UnsupportedOperation(
                format!("{} does not support model listing", self.protocol.name())
            ))?;

        let response = self.transport.get(&models_endpoint).await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            return Err(P::Error::map_http_error(status, body));
        }

        // Parse models response (assuming OpenAI-compatible format)
        let models_response: serde_json::Value = response.json().await
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;

        if let Some(data) = models_response.get("data").and_then(|d| d.as_array()) {
            let models = data
                .iter()
                .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(String::from))
                .collect();
            Ok(models)
        } else {
            Err(LlmConnectorError::ParseError("Invalid models response format".to_string()))
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}