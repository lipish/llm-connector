//! Unified Trait Definitions - V2 Architecture Core
//!
//! This module defines core traits for V2 architecture, providing clear and unified abstraction layer。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

// Reuse existing types, maintain compatibility
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse};

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// Protocol trait - Defines pure API specification
///
/// This trait represents an LLM API protocol specification, such as OpenAI API, Anthropic API, etc.
/// It only focuses on API format conversion, not specific network communication.
#[async_trait]
pub trait Protocol: Send + Sync + Clone + 'static {
    /// Protocol-specific request type
    type Request: Serialize + Send + Sync;

    /// Protocol-specific response type  
    type Response: for<'de> Deserialize<'de> + Send + Sync;

    /// Protocol name (such as "openai", "anthropic")
    fn name(&self) -> &str;

    /// Get chat completion endpoint URL
    fn chat_endpoint(&self, base_url: &str) -> String;

    /// Get model list endpoint URL (optional)
    fn models_endpoint(&self, _base_url: &str) -> Option<String> {
        None
    }

    /// Build protocol-specific request
    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError>;

    /// Parse protocol-specific response
    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError>;

    /// Parse model list response
    fn parse_models(&self, _response: &str) -> Result<Vec<String>, LlmConnectorError> {
        Err(LlmConnectorError::UnsupportedOperation(format!(
            "{} does not support model listing",
            self.name()
        )))
    }

    /// Map HTTP errors to unified error type
    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError;

    /// Get authentication headers (optional)
    fn auth_headers(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// Parse streaming response (optional)
    #[cfg(feature = "streaming")]
    async fn parse_stream_response(
        &self,
        response: reqwest::Response,
    ) -> Result<ChatStream, LlmConnectorError> {
        // Default to use generic SSE stream parser
        Ok(crate::sse::sse_to_streaming_response(response))
    }
}

/// Service Provider trait - Define unified service interface
///
/// This trait represents a specific LLM service provider, providing complete service functionality.
/// It is the direct user interaction interface.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Provider name (such as "openai", "aliyun", "ollama")
    fn name(&self) -> &str;

    /// Chat completion
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError>;

    /// Streaming chat completion
    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError>;

    /// Get available models list
    async fn models(&self) -> Result<Vec<String>, LlmConnectorError>;

    /// Type conversion support (for special feature access)
    fn as_any(&self) -> &dyn Any;
}

/// Build per-request header overrides from ChatRequest (api_key + extra_headers)
fn build_request_overrides(request: &ChatRequest) -> HashMap<String, String> {
    let mut overrides = HashMap::new();
    if let Some(ref key) = request.api_key {
        overrides.insert("Authorization".to_string(), format!("Bearer {}", key));
        overrides.insert("x-api-key".to_string(), key.clone());
    }
    if let Some(ref extra) = request.extra_headers {
        overrides.extend(extra.clone());
    }
    overrides
}

/// Generic provider implementation
///
/// This struct provides generic implementation for most standard LLM APIs.
/// It uses Protocol trait to handle API-specific format conversion,
/// uses HttpClient to handle network communication.
pub struct GenericProvider<P: Protocol> {
    protocol: P,
    client: super::HttpClient,
}

impl<P: Protocol> GenericProvider<P> {
    /// Create new generic provider
    pub fn new(protocol: P, client: super::HttpClient) -> Self {
        Self { protocol, client }
    }

    /// Get protocol reference
    pub fn protocol(&self) -> &P {
        &self.protocol
    }

    /// Get client reference
    pub fn client(&self) -> &super::HttpClient {
        &self.client
    }
}

impl<P: Protocol> Clone for GenericProvider<P> {
    fn clone(&self) -> Self {
        Self {
            protocol: self.protocol.clone(),
            client: self.client.clone(),
        }
    }
}

#[async_trait]
impl<P: Protocol> Provider for GenericProvider<P> {
    fn name(&self) -> &str {
        self.protocol.name()
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        let protocol_request = self.protocol.build_request(request)?;
        let base_url = request
            .base_url
            .as_deref()
            .unwrap_or_else(|| self.client.base_url());
        let url = self.protocol.chat_endpoint(base_url);
        let overrides = build_request_overrides(request);

        let response = if overrides.is_empty() {
            self.client.post(&url, &protocol_request).await?
        } else {
            self.client
                .post_with_overrides(&url, &protocol_request, &overrides)
                .await?
        };
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;

        if !status.is_success() {
            return Err(self.protocol.map_error(status.as_u16(), &text));
        }
        self.protocol.parse_response(&text)
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        let mut streaming_request = request.clone();
        streaming_request.stream = Some(true);

        let protocol_request = self.protocol.build_request(&streaming_request)?;
        let base_url = request
            .base_url
            .as_deref()
            .unwrap_or_else(|| self.client.base_url());
        let url = self.protocol.chat_endpoint(base_url);
        let overrides = build_request_overrides(request);

        let response = if overrides.is_empty() {
            self.client.stream(&url, &protocol_request).await?
        } else {
            self.client
                .stream_with_overrides(&url, &protocol_request, &overrides)
                .await?
        };
        let status = response.status();

        if !status.is_success() {
            let text = response
                .text()
                .await
                .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;
            return Err(self.protocol.map_error(status.as_u16(), &text));
        }
        self.protocol.parse_stream_response(response).await
    }

    async fn models(&self) -> Result<Vec<String>, LlmConnectorError> {
        let endpoint = self
            .protocol
            .models_endpoint(self.client.base_url())
            .ok_or_else(|| {
                LlmConnectorError::UnsupportedOperation(format!(
                    "{} does not support model listing",
                    self.protocol.name()
                ))
            })?;

        let response = self.client.get(&endpoint).await?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;

        if !status.is_success() {
            return Err(self.protocol.map_error(status.as_u16(), &text));
        }

        self.protocol.parse_models(&text)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
