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

    /// Send a streaming chat completion request in pure Ollama format
    #[cfg(feature = "streaming")]
    async fn chat_stream_ollama_pure(&self, request: &ChatRequest) -> Result<crate::types::OllamaChatStream, LlmConnectorError> {
        use futures_util::StreamExt;
        use crate::types::OllamaStreamChunk;

        let stream = self.chat_stream(request).await?;
        let model = request.model.clone();

        let ollama_stream = stream.map(move |result| {
            match result {
                Ok(openai_chunk) => {
                    // Check if this is the final chunk
                    let is_final = openai_chunk.usage.is_some() ||
                        openai_chunk.choices.iter().any(|c| c.finish_reason.is_some());

                    // Extract content from OpenAI format
                    let content = if !openai_chunk.content.is_empty() {
                        openai_chunk.content.clone()
                    } else {
                        openai_chunk.choices.get(0)
                            .and_then(|choice| choice.delta.content.clone())
                            .unwrap_or_default()
                    };

                    if is_final {
                        // Create final chunk with usage statistics
                        Ok(OllamaStreamChunk::final_chunk(model.clone(), openai_chunk.usage.as_ref()))
                    } else {
                        // Create regular chunk
                        Ok(OllamaStreamChunk::new(model.clone(), content, false))
                    }
                }
                Err(e) => Err(e),
            }
        });

        Ok(Box::pin(ollama_stream))
    }

    /// Send a streaming chat completion request with format configuration
    #[cfg(feature = "streaming")]
    async fn chat_stream_with_format(
        &self,
        request: &ChatRequest,
        config: &crate::types::StreamingConfig,
    ) -> Result<ChatStream, LlmConnectorError> {
        // Default implementation: use standard chat_stream and convert format
        use futures_util::StreamExt;
        use crate::types::{convert_streaming_format, create_final_ollama_chunk, StreamingFormat};

        let stream = self.chat_stream(request).await?;
        let format = config.format;
        let model = request.model.clone();

        match format {
            StreamingFormat::OpenAI => Ok(stream), // No conversion needed
            StreamingFormat::Ollama => {
                use std::sync::{Arc, Mutex};

                // Track if we've seen the final chunk
                let seen_final = Arc::new(Mutex::new(false));
                let seen_final_clone = seen_final.clone();
                let model_name = Arc::new(model.clone());

                // Convert OpenAI format to Ollama format
                let converted_stream = stream.map(move |result| {
                    match result {
                        Ok(chunk) => {
                            // Check if this is the final chunk (has usage data or finish_reason)
                            let is_final = chunk.usage.is_some() ||
                                chunk.choices.iter().any(|c| c.finish_reason.is_some());

                            if is_final {
                                let mut seen = seen_final_clone.lock().unwrap();
                                *seen = true;
                            }

                            convert_streaming_format(&chunk, StreamingFormat::Ollama, is_final)
                                .map_err(|e| crate::error::LlmConnectorError::ParseError(e.to_string()))
                                .map(|json_str| {
                                    // Create a new StreamingResponse with Ollama JSON in content
                                    let mut response = chunk.clone();
                                    response.content = json_str;
                                    response
                                })
                        }
                        Err(e) => Err(e),
                    }
                });

                // Create a stream that ensures we send a final done:true chunk
                let final_stream = converted_stream.chain(futures_util::stream::once(async move {
                    let seen = seen_final.lock().unwrap();
                    if !*seen {
                        // If we haven't seen a final chunk, create one
                        let final_json = create_final_ollama_chunk(&model_name, None);
                        let mut final_response = crate::types::StreamingResponse::default();
                        final_response.model = (*model_name).clone();
                        final_response.content = final_json;
                        Ok(final_response)
                    } else {
                        // Skip if we already sent a final chunk
                        Err(crate::error::LlmConnectorError::ParseError("Stream ended".to_string()))
                    }
                })).filter_map(|result| async move {
                    match result {
                        Ok(chunk) => Some(Ok(chunk)),
                        Err(e) if e.to_string().contains("Stream ended") => None, // Filter out our sentinel
                        Err(e) => Some(Err(e)),
                    }
                });

                Ok(Box::pin(final_stream))
            }
        }
    }

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

    #[cfg(feature = "streaming")]
    fn uses_sse_stream(&self) -> bool { true }

    /// Optional hook: validate success body when HTTP status is OK but provider returns error in JSON
    /// Default: no-op (assume body is valid)
    fn validate_success_body(&self, _status: u16, _raw: &Value) -> Result<(), LlmConnectorError> { Ok(()) }
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
        if std::env::var("LLM_DEBUG_REQUEST_RAW").map(|v| v == "1").unwrap_or(false) {
            if let Ok(j) = serde_json::to_string(&request_data) {
                eprintln!("[request-raw] {}", j);
            }
        }

        let response = self.transport.post(&url, &request_data).await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body: Value = response.json().await.unwrap_or_default();
            return Err(A::ErrorMapperType::map_http_error(status, body));
        }

        // Read body once, then parse into both typed and raw JSON
        let status_code = response.status().as_u16();
        let text = response
            .text()
            .await
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;
        if std::env::var("LLM_DEBUG_RESPONSE_RAW").map(|v| v == "1").unwrap_or(false) {
            eprintln!("[response-raw] {}", text);
        }
        let raw: Value = serde_json::from_str(&text).unwrap_or_default();

        // Allow protocol-specific validation of body-level errors under HTTP 200
        if let Err(err) = self.adapter.validate_success_body(status_code, &raw) {
            return Err(err);
        }

        // Try strict typed parse first
        match serde_json::from_str::<A::ResponseType>(&text) {
            Ok(response_data) => {
                let mut chat_response = self.adapter.parse_response_data(response_data);
                // Provider-agnostic synonym extraction
                chat_response.populate_reasoning_synonyms(&raw);
                Ok(chat_response)
            }
            Err(e) => {
                // Fallback: best-effort extraction to avoid hard failure on minor incompatibilities
                if std::env::var("LLM_DEBUG_PARSE_FALLBACK").map(|v| v == "1").unwrap_or(false) {
                    eprintln!("[parse-fallback] strict parse failed: {}\nbody: {}", e, text);
                }

                // Extract common fields with defaults
                let model = raw.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let id = raw
                    .get("id")
                    .or_else(|| raw.get("request_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let object = raw
                    .get("object")
                    .and_then(|v| v.as_str())
                    .unwrap_or("chat.completion")
                    .to_string();
                let created = raw.get("created").and_then(|v| v.as_u64()).unwrap_or(0);

                // Try to read usage if present
                let usage = raw
                    .get("usage")
                    .and_then(|u| serde_json::from_value::<crate::types::Usage>(u.clone()).ok());

                // Extract first choice content if available
                let (choice_msg, finish_reason) = if let Some(choices) = raw.get("choices").and_then(|c| c.as_array()) {
                    let first = choices.get(0);
                    let content = first
                        .and_then(|c| c.get("message"))
                        .and_then(|m| m.get("content"))
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();
                    let fr = first
                        .and_then(|c| c.get("finish_reason"))
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string());
                    (crate::types::Message::assistant(content), fr)
                } else {
                    (crate::types::Message::assistant(String::new()), None)
                };

                let choices = vec![crate::types::Choice {
                    index: 0,
                    message: choice_msg,
                    finish_reason,
                    logprobs: None,
                }];

                let mut chat_response = crate::types::ChatResponse {
                    id,
                    object,
                    created,
                    model,
                    choices,
                    content: raw
                        .get("choices")
                        .and_then(|c| c.as_array())
                        .and_then(|arr| arr.get(0))
                        .and_then(|c0| c0.get("message"))
                        .and_then(|m| m.get("content"))
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string(),
                    usage,
                    system_fingerprint: raw
                        .get("system_fingerprint")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                };

                // Populate reasoning fields from raw JSON
                chat_response.populate_reasoning_synonyms(&raw);
                Ok(chat_response)
            }
        }
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        use crate::sse::json_lines_events;
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

        // Clone adapter to satisfy 'static lifetime in the async stream closure
        let adapter = self.adapter.clone();

        let event_stream = if self.adapter.uses_sse_stream() {
            crate::sse::sse_events(response)
        } else {
            json_lines_events(response)
        };

        let mapped_stream = event_stream.filter_map(move |event| {
            let adapter = adapter.clone();
            async move {
                match event {
                    Ok(data) => {
                        // 原始事件调试：设置环境变量 LLM_DEBUG_STREAM_RAW=1 启用
                        if std::env::var("LLM_DEBUG_STREAM_RAW").map(|v| v == "1").unwrap_or(false) {
                            eprintln!("[stream-raw] {}", data);
                        }
                        if data.trim() == "[DONE]" { return None; }

                        match serde_json::from_str::<A::StreamResponseType>(&data) {
                            Ok(stream_response) => {
                                let raw: Value = serde_json::from_str(&data).unwrap_or_default();
                                let mut sr = adapter.parse_stream_response_data(stream_response);
                                // Provider-agnostic synonym extraction
                                sr.populate_reasoning_synonyms(&raw);
                                Some(Ok(sr))
                            }
                            Err(_e) => {
                                // Fallback: best-effort mapping to StreamingResponse to avoid interrupting stream
                                let raw: Value = serde_json::from_str(&data).unwrap_or_default();
                                let model = raw
                                    .get("model")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                let id = raw
                                    .get("id")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| format!("{}-{}", adapter.name(), model));

                                let content_opt = raw
                                    .pointer("/choices/0/delta/content")
                                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                                    .or_else(|| raw.pointer("/message/content").and_then(|v| v.as_str().map(|s| s.to_string())));
                                let content = content_opt.unwrap_or_default();

                                let finish_reason = raw
                                    .pointer("/choices/0/finish_reason")
                                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                                let mut sr = crate::types::StreamingResponse {
                                    id,
                                    object: "chat.completion.chunk".to_string(),
                                    created: chrono::Utc::now().timestamp() as u64,
                                    model,
                                    choices: vec![crate::types::StreamingChoice {
                                        index: 0,
                                        delta: crate::types::Delta {
                                            role: None,
                                            content: if content.is_empty() { None } else { Some(content.clone()) },
                                            tool_calls: None,
                                            reasoning_content: None,
                                            ..Default::default()
                                        },
                                        finish_reason,
                                        logprobs: None,
                                    }],
                                    content,
                                    reasoning_content: None,
                                    usage: None,
                                    system_fingerprint: None,
                                };
                                // Provider-agnostic synonym extraction from raw JSON
                                sr.populate_reasoning_synonyms(&raw);
                                Some(Ok(sr))
                            }
                        }
                    }
                    Err(e) => Some(Err(LlmConnectorError::StreamingError(e.to_string()))),
                }
            }
        });

        Ok(Box::pin(mapped_stream))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
