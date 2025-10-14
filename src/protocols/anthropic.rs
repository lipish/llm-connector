//! Anthropic Protocol Implementation
//!
//! This module implements the Anthropic Messages API protocol used by Claude models.
//!
//! # Supported Providers
//!
//! - **Anthropic Claude** - `claude()` - Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku
//!
//! # Protocol Differences from OpenAI
//!
//! The Anthropic protocol has several key differences:
//!
//! ## 1. Endpoint
//! - Anthropic: `POST /v1/messages`
//! - OpenAI: `POST /v1/chat/completions`
//!
//! ## 2. Request Structure
//! - **System message**: Separate `system` field instead of message with role "system"
//! - **Max tokens**: Required field (OpenAI makes it optional)
//! - **Stop sequences**: `stop_sequences` instead of `stop`
//!
//! ## 3. Response Structure
//! - **Content**: Array of content blocks instead of single string
//! - **Role**: Always "assistant" (no "system" or "user" in responses)
//! - **Usage**: Different field names (`input_tokens` vs `prompt_tokens`)
//!
//! ## 4. Streaming Format
//! - Different event types: `message_start`, `content_block_delta`, `message_delta`
//! - More granular streaming events
//!
//! # Request Format
//!
//! ```json
//! {
//!   "model": "claude-3-5-sonnet-20241022",
//!   "max_tokens": 1024,
//!   "messages": [
//!     {"role": "user", "content": "Hello"}
//!   ],
//!   "system": "You are a helpful assistant",
//!   "temperature": 0.7
//! }
//! ```
//!
//! # Response Format
//!
//! ```json
//! {
//!   "id": "msg_123",
//!   "type": "message",
//!   "role": "assistant",
//!   "content": [
//!     {"type": "text", "text": "Hello! How can I help you?"}
//!   ],
//!   "model": "claude-3-5-sonnet-20241022",
//!   "usage": {
//!     "input_tokens": 10,
//!     "output_tokens": 20
//!   }
//! }
//! ```
//!
//! # Example
//!
//! ```rust
//! use llm_connector::LlmClient;
//! use llm_connector::types::{ChatRequest, Message};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create Anthropic client (Claude)
//! let client = LlmClient::anthropic("your-api-key");
//!
//! // Create request
//! let request = ChatRequest {
//!     model: "claude-3-5-sonnet-20241022".to_string(),
//!     messages: vec![Message::user("Hello!")],
//!     max_tokens: Some(1024), // Required for Anthropic
//!     ..Default::default()
//! };
//!
//! // Send request
//! let response = client.chat(&request).await?;
//! println!("Response: {}", response.choices[0].message.content);
//! # Ok(())
//! # }
//! ```

use crate::core::Protocol;
use crate::core::error::StandardErrorMapper;
use crate::types::{ChatRequest as Request, ChatResponse as Response, Role, Usage};
use crate::protocols::{ProviderAdapter, ErrorMapper};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Parse a role string into a Role enum
fn parse_role(role: &str) -> Role {
    match role {
        "system" => Role::System,
        "user" => Role::User,
        "assistant" => Role::Assistant,
        "tool" => Role::Tool,
        _ => Role::User, // Default to user for unknown roles
    }
}

#[cfg(feature = "streaming")]
use crate::types::{Delta, StreamingChoice, StreamingResponse};

// ============================================================================
// Anthropic Protocol Request/Response Structures
// ============================================================================

/// Anthropic Messages API request
#[derive(Serialize, Debug, Clone)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AnthropicTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
}

#[derive(Serialize, Debug, Clone)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: AnthropicContent,
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum AnthropicContent {
    Text(String),
    Blocks(Vec<AnthropicContentBlock>),
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum AnthropicContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: AnthropicImageSource },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

#[derive(Serialize, Debug, Clone)]
pub struct AnthropicImageSource {
    pub r#type: String,
    pub media_type: String,
    pub data: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct AnthropicTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Anthropic Messages API response
#[derive(Deserialize, Debug)]
pub struct AnthropicResponse {
    pub id: String,
    pub r#type: String,
    pub role: String,
    pub content: Vec<AnthropicResponseContent>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: AnthropicUsage,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum AnthropicResponseContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Dummy streaming type for when streaming feature is not enabled
#[cfg(not(feature = "streaming"))]
#[derive(Debug, Clone)]
pub struct AnthropicStreamResponse;

/// Anthropic streaming response
#[cfg(feature = "streaming")]
#[derive(Deserialize, Debug, Clone)]
pub struct AnthropicStreamResponse {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<AnthropicStreamMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_block: Option<AnthropicResponseContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<AnthropicStreamDelta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<AnthropicUsage>,
}

/// Anthropic streaming response state manager
#[cfg(feature = "streaming")]
#[derive(Debug, Default)]
pub struct AnthropicStreamState {
    pub id: Option<String>,
    pub model: Option<String>,
    pub role: Option<String>,
    pub content: Vec<String>,
    pub usage: Option<AnthropicUsage>,
    pub finished: bool,
}

#[cfg(feature = "streaming")]
impl AnthropicStreamState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process_event(&mut self, event: AnthropicStreamResponse) -> Option<StreamingResponse> {
        match event.r#type.as_str() {
            "message_start" => {
                if let Some(message) = event.message {
                    self.id = Some(message.id);
                    self.model = Some(message.model);
                    self.role = Some(message.role);
                    self.usage = Some(message.usage);
                }
                None
            }
            "content_block_delta" => {
                if let Some(delta) = event.delta {
                    if let Some(text) = delta.text {
                        self.content.push(text.clone());
                        return Some(StreamingResponse {
                            id: self.id.clone().unwrap_or_default(),
                            object: "chat.completion.chunk".to_string(),
                            created: chrono::Utc::now().timestamp() as u64,
                            model: self.model.clone().unwrap_or_default(),
                            choices: vec![StreamingChoice {
                                index: event.index.unwrap_or(0),
                                delta: Delta {
                                    role: self.role.clone().and_then(|r| match r.as_str() {
                                        "system" => Some(crate::types::Role::System),
                                        "user" => Some(crate::types::Role::User),
                                        "assistant" => Some(crate::types::Role::Assistant),
                                        "tool" => Some(crate::types::Role::Tool),
                                        _ => Some(crate::types::Role::User),
                                    }),
                                    content: Some(text.clone()),
                                    tool_calls: None,
                                    reasoning_content: None,
                                    ..Default::default()
                                },
                                finish_reason: None,
                                logprobs: None,
                            }],
                            content: text,
                            reasoning_content: None,
                            usage: None,
                            system_fingerprint: None,
                        });
                    }
                }
                None
            }
            "message_delta" => {
                if let Some(usage) = event.usage {
                    self.usage = Some(usage);
                }
                let finish_reason = if self.finished {
                    Some("stop".to_string())
                } else {
                    None
                };

                Some(StreamingResponse {
                    id: self.id.clone().unwrap_or_default(),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp() as u64,
                    model: self.model.clone().unwrap_or_default(),
                    choices: vec![StreamingChoice {
                        index: 0,
                        delta: Delta {
                            role: None,
                            content: None,
                            tool_calls: None,
                            reasoning_content: None,
                            ..Default::default()
                        },
                        finish_reason,
                        logprobs: None,
                    }],
                    content: self.content.join(""),
                    reasoning_content: None,
                    usage: self.usage.clone().map(|usage| Usage {
                        prompt_tokens: usage.input_tokens,
                        completion_tokens: usage.output_tokens,
                        total_tokens: usage.input_tokens + usage.output_tokens,
                        prompt_cache_hit_tokens: None,
                        prompt_cache_miss_tokens: None,
                        prompt_tokens_details: None,
                        completion_tokens_details: None,
                    }),
                    system_fingerprint: None,
                })
            }
            "message_stop" => {
                self.finished = true;
                None
            }
            _ => None,
        }
    }
}

/// Anthropic streaming response processor
#[cfg(feature = "streaming")]
#[derive(Debug, Clone)]
pub struct AnthropicStreamProcessor {
    state: Arc<std::sync::Mutex<AnthropicStreamState>>,
}

#[cfg(feature = "streaming")]
impl AnthropicStreamProcessor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(std::sync::Mutex::new(AnthropicStreamState::new())),
        }
    }

    pub fn process_event(&self, event: AnthropicStreamResponse) -> Option<StreamingResponse> {
        if let Ok(mut state) = self.state.lock() {
            state.process_event(event)
        } else {
            None
        }
    }
}

#[cfg(feature = "streaming")]
#[derive(Deserialize, Debug, Clone)]
pub struct AnthropicStreamMessage {
    pub id: String,
    pub r#type: String,
    pub role: String,
    pub model: String,
    pub content: Vec<Value>,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: AnthropicUsage,
}

#[cfg(feature = "streaming")]
#[derive(Deserialize, Debug, Clone)]
pub struct AnthropicStreamDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_json: Option<String>,
}

// ============================================================================
// Anthropic Error Mapper
// ============================================================================

pub struct AnthropicErrorMapper;

impl ErrorMapper for AnthropicErrorMapper {
    fn map_http_error(status: u16, body: Value) -> crate::error::LlmConnectorError {
        let error_message = body["error"]["message"]
            .as_str()
            .or_else(|| body["message"].as_str())
            .unwrap_or("Unknown Anthropic error");

        let error_type = body["error"]["type"]
            .as_str()
            .or_else(|| body["type"].as_str())
            .unwrap_or("unknown_error");

        match status {
            400 => crate::error::LlmConnectorError::InvalidRequest(format!(
                "Anthropic: {} ({})",
                error_message, error_type
            )),
            401 => crate::error::LlmConnectorError::AuthenticationError(format!(
                "Anthropic: {} ({})",
                error_message, error_type
            )),
            403 => crate::error::LlmConnectorError::PermissionError(format!(
                "Anthropic: {} ({})",
                error_message, error_type
            )),
            429 => crate::error::LlmConnectorError::RateLimitError(format!(
                "Anthropic: {} ({})",
                error_message, error_type
            )),
            500..=599 => crate::error::LlmConnectorError::ServerError(format!(
                "Anthropic HTTP {}: {} ({})",
                status, error_message, error_type
            )),
            _ => crate::error::LlmConnectorError::ProviderError(format!(
                "Anthropic HTTP {}: {} ({})",
                status, error_message, error_type
            )),
        }
    }

    fn map_network_error(error: reqwest::Error) -> crate::error::LlmConnectorError {
        if error.is_timeout() {
            crate::error::LlmConnectorError::TimeoutError(format!("Anthropic: {}", error))
        } else if error.is_connect() {
            crate::error::LlmConnectorError::ConnectionError(format!("Anthropic: {}", error))
        } else {
            crate::error::LlmConnectorError::NetworkError(format!("Anthropic: {}", error))
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

// ============================================================================
// Conversion Utilities
// ============================================================================

impl AnthropicRequest {
    /// Create Anthropic request from generic chat request
    pub fn from_chat_request(request: &Request, stream: bool) -> Self {
        // Extract system message if present
        let (system_message, user_messages): (Vec<_>, Vec<_>) = request
            .messages
            .iter()
            .partition(|msg| msg.role == Role::System);

        let system = system_message.first().map(|msg| msg.content.clone());

        let messages = user_messages
            .iter()
            .map(|msg| AnthropicMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                content: AnthropicContent::Text(msg.content.clone()),
            })
            .collect();

        Self {
            model: request.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(1024),
            messages,
            system,
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: request.stop.clone(),
            stream: if stream { Some(true) } else { None },
            tools: request.tools.as_ref().map(|tools| {
                tools
                    .iter()
                    .map(|tool| AnthropicTool {
                        name: tool.function.name.clone(),
                        description: tool.function.description.clone().unwrap_or_default(),
                        input_schema: tool.function.parameters.clone(),
                    })
                    .collect()
            }),
            tool_choice: request
                .tool_choice
                .as_ref()
                .map(|tc| serde_json::to_value(tc).unwrap_or_default()),
        }
    }
}

impl AnthropicResponse {
    /// Convert to generic chat response
    pub fn to_chat_response(self) -> Response {
        let content = self
            .content
            .into_iter()
            .filter_map(|block| match block {
                AnthropicResponseContent::Text { text } => Some(text),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        // Convenience field mirrors the first choice content
        let first_content = content.clone();

        Response {
            id: self.id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: self.model,
            choices: vec![crate::types::Choice {
                index: 0,
                message: crate::types::Message {
                    role: parse_role(&self.role),
                    content,
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    ..Default::default()
                },
                finish_reason: self.stop_reason,
                logprobs: None,
            }],
            content: first_content,
            usage: Some(Usage {
                prompt_tokens: self.usage.input_tokens,
                completion_tokens: self.usage.output_tokens,
                total_tokens: self.usage.input_tokens + self.usage.output_tokens,
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        }
    }
}

// ============================================================================
// Anthropic Protocol Adapter
// ============================================================================

/// Anthropic Protocol implementation
///
/// Uses Arc for efficient sharing of strings across clones.
#[derive(Debug, Clone)]
pub struct AnthropicProtocol {
    base_url: Arc<str>,
    #[cfg(feature = "streaming")]
    stream_processor: Arc<AnthropicStreamProcessor>,
}

impl AnthropicProtocol {
    /// Create new Anthropic protocol with default base URL
    pub fn new() -> Self {
        Self {
            base_url: Arc::from("https://api.anthropic.com"),
            #[cfg(feature = "streaming")]
            stream_processor: Arc::new(AnthropicStreamProcessor::new()),
        }
    }

    /// Create new Anthropic protocol with custom base URL
    pub fn with_url(base_url: &str) -> Self {
        Self {
            base_url: Arc::from(base_url),
            #[cfg(feature = "streaming")]
            stream_processor: Arc::new(AnthropicStreamProcessor::new()),
        }
    }
}

impl Default for AnthropicProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Protocol for AnthropicProtocol {
    type Request = AnthropicRequest;
    type Response = AnthropicResponse;
    type StreamResponse = AnthropicStreamResponse;
    type Error = StandardErrorMapper;

    fn name(&self) -> &str {
        "anthropic"
    }

    fn endpoint(&self, base_url: &str) -> String {
        format!("{}/v1/messages", base_url)
    }

    fn models_endpoint(&self, _base_url: &str) -> Option<String> {
        // Anthropic doesn't have a models endpoint, return None
        None
    }

    fn build_request(&self, request: &Request, stream: bool) -> Self::Request {
        // Extract system message if present
        let (system_message, user_messages): (Vec<_>, Vec<_>) = request
            .messages
            .iter()
            .partition(|msg| msg.role == Role::System);

        let system = system_message.first().map(|msg| msg.content.clone());

        let messages = user_messages
            .iter()
            .map(|msg| AnthropicMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                content: AnthropicContent::Text(msg.content.clone()),
            })
            .collect();

        AnthropicRequest {
            model: request.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(1024),
            messages,
            system,
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: request.stop.clone(),
            stream: if stream { Some(true) } else { None },
            tools: request.tools.as_ref().map(|tools| {
                tools
                    .iter()
                    .map(|tool| AnthropicTool {
                        name: tool.function.name.clone(),
                        description: tool.function.description.clone().unwrap_or_default(),
                        input_schema: tool.function.parameters.clone(),
                    })
                    .collect()
            }),
            tool_choice: request
                .tool_choice
                .as_ref()
                .map(|tc| serde_json::to_value(tc).unwrap_or_default()),
        }
    }

    fn parse_response(&self, response: Self::Response) -> Response {
        let content = response
            .content
            .into_iter()
            .filter_map(|block| match block {
                AnthropicResponseContent::Text { text } => Some(text),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        // Convenience field mirrors the first choice content
        let first_content = content.clone();

        Response {
            id: response.id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: response.model,
            choices: vec![crate::types::Choice {
                index: 0,
                message: crate::types::Message {
                    role: parse_role(&response.role),
                    content,
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    ..Default::default()
                },
                finish_reason: response.stop_reason,
                logprobs: None,
            }],
            content: first_content,
            usage: Some(Usage {
                prompt_tokens: response.usage.input_tokens,
                completion_tokens: response.usage.output_tokens,
                total_tokens: response.usage.input_tokens + response.usage.output_tokens,
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        }
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response(&self, response: Self::StreamResponse) -> crate::types::ChatStream {
        use futures_util::stream;

        // Use the stream processor to handle complex Anthropic streaming events
        let processed = self.stream_processor.process_event(response.clone());
        let streaming_response = if let Some(sr) = processed {
            sr
        } else {
            // Fallback response
            crate::types::StreamingResponse {
                id: "anthropic-stream".to_string(),
                object: "chat.completion.chunk".to_string(),
                created: chrono::Utc::now().timestamp() as u64,
                model: "claude".to_string(),
                choices: vec![],
                content: String::new(),
                reasoning_content: None,
                usage: None,
                system_fingerprint: None,
            }
        };

        // Convert single response to a stream with one chunk
        let single_chunk_stream = stream::once(async { Ok(streaming_response) });
        Box::pin(single_chunk_stream)
    }

    #[cfg(feature = "streaming")]
    fn uses_sse_stream(&self) -> bool {
        true
    }
}

// Legacy constructors for backward compatibility
impl AnthropicProtocol {
    /// Create new Anthropic protocol with API key (legacy)
    pub fn new_with_key(_api_key: &str) -> Self {
        Self::new()
    }

    /// Create new Anthropic protocol with custom API key and base URL (legacy)
    pub fn with_url_and_key(_api_key: &str, base_url: &str) -> Self {
        Self::with_url(base_url)
    }
}

#[async_trait]
impl ProviderAdapter for AnthropicProtocol {
    type RequestType = AnthropicRequest;
    type ResponseType = AnthropicResponse;
    #[cfg(feature = "streaming")]
    type StreamResponseType = AnthropicStreamResponse;
    type ErrorMapperType = crate::protocols::core::StandardErrorMapper;

    fn name(&self) -> &str {
        <Self as Protocol>::name(self)
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        let base = base_url.as_deref().unwrap_or(&self.base_url);
        <Self as Protocol>::endpoint(self, base)
    }

    fn models_endpoint_url(&self, base_url: &Option<String>) -> Option<String> {
        let base = base_url.as_deref().unwrap_or(&self.base_url);
        <Self as Protocol>::models_endpoint(self, base)
    }

    fn build_request_data(&self, request: &crate::types::ChatRequest, stream: bool) -> Self::RequestType {
        <Self as Protocol>::build_request(self, request, stream)
    }

    fn parse_response_data(&self, response: Self::ResponseType) -> crate::types::ChatResponse {
        <Self as Protocol>::parse_response(self, response)
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> crate::types::StreamingResponse {
        // Use the stream processor to handle complex Anthropic streaming events
        let processed = self.stream_processor.process_event(response.clone());
        processed.unwrap_or_else(|| crate::types::StreamingResponse {
                id: "anthropic-stream".to_string(),
                object: "chat.completion.chunk".to_string(),
                created: chrono::Utc::now().timestamp() as u64,
                model: "claude".to_string(),
                choices: vec![crate::types::StreamingChoice {
                    index: 0,
                    delta: crate::types::Delta {
                        role: None,
                        content: response.delta.as_ref().and_then(|d| d.text.clone()),
                        tool_calls: None,
                        reasoning_content: None,
                        ..Default::default()
                    },
                    finish_reason: None,
                    logprobs: None,
                }],
                content: response
                    .delta
                    .and_then(|d| d.text)
                    .unwrap_or_default(),
                reasoning_content: None,
                usage: response.usage.clone().map(|usage| Usage {
                    prompt_tokens: usage.input_tokens,
                    completion_tokens: usage.output_tokens,
                    total_tokens: usage.input_tokens + usage.output_tokens,
                    prompt_cache_hit_tokens: None,
                    prompt_cache_miss_tokens: None,
                    prompt_tokens_details: None,
                    completion_tokens_details: None,
                }),
                system_fingerprint: None,
            })
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Create an Anthropic protocol adapter
pub fn anthropic() -> AnthropicProtocol {
    AnthropicProtocol::new()
}

/// Get all providers that use the Anthropic protocol
pub fn anthropic_providers() -> Vec<(&'static str, AnthropicProtocol)> {
    vec![("anthropic", anthropic()), ("claude", anthropic())]
}

/// Anthropic provider type alias
pub type AnthropicProvider = crate::protocols::core::GenericProvider<AnthropicProtocol>;
