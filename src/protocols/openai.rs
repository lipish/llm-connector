//! OpenAI Protocol Implementation
//!
//! This module implements the pure OpenAI API specification.
//!
//! # Protocol Details
//!
//! ## Endpoint
//! - Standard: `POST /v1/chat/completions`
//! - Models: `GET /v1/models`
//!
//! ## Request Format
//! ```json
//! {
//!   "model": "gpt-4",
//!   "messages": [
//!     {"role": "user", "content": "Hello"}
//!   ],
//!   "temperature": 0.7,
//!   "max_tokens": 1000,
//!   "stream": false
//! }
//! ```
//!
//! ## Response Format
//! ```json
//! {
//!   "id": "chatcmpl-123",
//!   "object": "chat.completion",
//!   "created": 1677652288,
//!   "model": "gpt-4",
//!   "choices": [{
//!     "index": 0,
//!     "message": {
//!       "role": "assistant",
//!       "content": "Hello! How can I help you?"
//!     },
//!     "finish_reason": "stop"
//!   }],
//!   "usage": {
//!     "prompt_tokens": 10,
//!     "completion_tokens": 20,
//!     "total_tokens": 30
//!   }
//! }
//! ```
//!
//! ## Streaming Format
//! - Uses Server-Sent Events (SSE)
//! - Each chunk: `data: {"choices": [{"delta": {"content": "..."}}]}`
//! - Final marker: `data: [DONE]`

use crate::core::Protocol;
use crate::core::error::StandardErrorMapper;
use crate::types::{ChatRequest as Request, ChatResponse as Response, Role, ToolCall, Usage};
use crate::protocols::ProviderAdapter;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
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

// ============================================================================
// OpenAI Protocol Request/Response Types
// ============================================================================

/// Standard OpenAI chat completion request
#[derive(Serialize, Debug, Clone)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
}

#[derive(Serialize, Debug, Clone)]
pub struct OpenAIMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct OpenAITool {
    pub r#type: String,
    pub function: OpenAIFunction,
}

#[derive(Serialize, Debug, Clone)]
pub struct OpenAIFunction {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Standard OpenAI chat completion response
#[derive(Deserialize, Debug)]
pub struct OpenAIResponse {
    pub id: String,
    #[serde(default = "default_object_type")]
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIChoice {
    pub index: u32,
    pub message: OpenAIResponseMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIResponseMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// OpenAI streaming response
#[cfg(feature = "streaming")]
#[derive(Deserialize, Debug, Clone)]
pub struct OpenAIStreamResponse {
    pub id: String,
    #[serde(default)]
    pub object: Option<String>,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIStreamChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

#[cfg(feature = "streaming")]
#[derive(Deserialize, Debug, Clone)]
pub struct OpenAIStreamChoice {
    pub index: u32,
    pub delta: OpenAIStreamDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[cfg(feature = "streaming")]
#[derive(Deserialize, Debug, Clone)]
pub struct OpenAIStreamDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

fn default_object_type() -> String {
    "chat.completion".to_string()
}

/// Dummy streaming type for when streaming feature is not enabled
#[cfg(not(feature = "streaming"))]
#[derive(Debug, Clone)]
pub struct OpenAIStreamResponse;

// ============================================================================
// Compatibility Methods (used by other protocols)
// ============================================================================

impl OpenAIRequest {
    /// Create OpenAI request from generic chat request
    pub fn from_chat_request(request: &crate::types::ChatRequest, stream: bool) -> Self {
        Self {
            model: request.model.clone(),
            messages: request.messages.iter().map(|msg| {
                let content = if msg.content.is_empty() { None } else { Some(msg.content.clone()) };
                OpenAIMessage {
                    role: match msg.role {
                        crate::types::Role::System => "system".to_string(),
                        crate::types::Role::User => "user".to_string(),
                        crate::types::Role::Assistant => "assistant".to_string(),
                        crate::types::Role::Tool => "tool".to_string(),
                    },
                    content,
                    name: msg.name.clone(),
                    tool_calls: msg.tool_calls.clone(),
                    tool_call_id: msg.tool_call_id.clone(),
                }
            }).collect(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: if stream { Some(true) } else { None },
            stop: request.stop.clone(),
            tools: request.tools.as_ref().map(|tools| {
                tools.iter().map(|tool| OpenAITool {
                    r#type: "function".to_string(),
                    function: OpenAIFunction {
                        name: tool.function.name.clone(),
                        description: tool.function.description.clone().unwrap_or_default(),
                        parameters: Some(tool.function.parameters.clone()),
                    },
                }).collect()
            }),
            tool_choice: request.tool_choice.as_ref()
                .map(|tc| serde_json::to_value(tc).unwrap_or_default()),
        }
    }
}

impl OpenAIResponse {
    /// Convert to generic chat response
    pub fn to_chat_response(self) -> crate::types::ChatResponse {
        let first_content = self
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        crate::types::ChatResponse {
            id: self.id,
            object: self.object,
            created: self.created,
            model: self.model,
            choices: self.choices.into_iter().map(|choice| {
                crate::types::Choice {
                    index: choice.index,
                    message: crate::types::Message {
                        role: parse_role(&choice.message.role),
                        content: choice.message.content.unwrap_or_default(),
                        name: choice.message.name,
                        tool_calls: choice.message.tool_calls,
                        tool_call_id: choice.message.tool_call_id,
                        ..Default::default()
                    },
                    finish_reason: choice.finish_reason,
                    logprobs: None,
                }
            }).collect(),
            content: first_content,
            usage: Some(self.usage),
            system_fingerprint: self.system_fingerprint,
        }
    }
}

#[cfg(feature = "streaming")]
impl OpenAIStreamResponse {
    /// Convert to streaming response
    pub fn to_streaming_response(self) -> crate::types::StreamingResponse {
        let first_chunk_content = self
            .choices
            .first()
            .and_then(|c| c.delta.content.clone())
            .unwrap_or_default();

        crate::types::StreamingResponse {
            id: self.id,
            object: self.object.unwrap_or_else(|| "chat.completion.chunk".to_string()),
            created: self.created,
            model: self.model,
            choices: self.choices.into_iter().map(|choice| {
                crate::types::StreamingChoice {
                    index: choice.index,
                    delta: crate::types::Delta {
                        role: choice.delta.role.as_ref().map(|r| parse_role(r)),
                        content: choice.delta.content,
                        tool_calls: choice.delta.tool_calls,
                        reasoning_content: None,
                        ..Default::default()
                    },
                    finish_reason: choice.finish_reason,
                    logprobs: None,
                }
            }).collect(),
            content: first_chunk_content,
            reasoning_content: None,
            usage: self.usage,
            system_fingerprint: self.system_fingerprint,
        }
    }
}

// ============================================================================
// OpenAI Protocol Implementation
// ============================================================================

/// Pure OpenAI protocol implementation
///
/// Implements the official OpenAI API specification exactly.
#[derive(Debug)]
pub struct OpenAIProtocol {
    name: Arc<str>,
    base_url: Arc<str>,
}

impl OpenAIProtocol {
    /// Create new OpenAI protocol with default base URL
    pub fn new() -> Self {
        Self {
            name: Arc::from("openai"),
            base_url: Arc::from("https://api.openai.com/v1"),
        }
    }

    /// Create new OpenAI protocol with custom base URL
    ///
    /// This can be used for OpenAI-compatible endpoints
    pub fn with_url(base_url: &str) -> Self {
        Self {
            name: Arc::from("openai"),
            base_url: Arc::from(base_url),
        }
    }
}

impl Default for OpenAIProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Protocol for OpenAIProtocol {
    type Request = OpenAIRequest;
    type Response = OpenAIResponse;
    type StreamResponse = OpenAIStreamResponse;
    type Error = StandardErrorMapper;

    fn name(&self) -> &str {
        &self.name
    }

    fn endpoint(&self, base_url: &str) -> String {
        format!("{}/chat/completions", base_url)
    }

    fn models_endpoint(&self, base_url: &str) -> Option<String> {
        Some(format!("{}/models", base_url))
    }

    fn build_request(&self, request: &Request, stream: bool) -> Self::Request {
        Self::Request {
            model: request.model.clone(),
            messages: request.messages.iter().map(|msg| {
                let content = if msg.content.is_empty() { None } else { Some(msg.content.clone()) };
                OpenAIMessage {
                    role: match msg.role {
                        Role::System => "system".to_string(),
                        Role::User => "user".to_string(),
                        Role::Assistant => "assistant".to_string(),
                        Role::Tool => "tool".to_string(),
                    },
                    content,
                    name: msg.name.clone(),
                    tool_calls: msg.tool_calls.clone(),
                    tool_call_id: msg.tool_call_id.clone(),
                }
            }).collect(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: if stream { Some(true) } else { None },
            stop: request.stop.clone(),
            tools: request.tools.as_ref().map(|tools| {
                tools.iter().map(|tool| OpenAITool {
                    r#type: "function".to_string(),
                    function: OpenAIFunction {
                        name: tool.function.name.clone(),
                        description: tool.function.description.clone().unwrap_or_default(),
                        parameters: Some(tool.function.parameters.clone()),
                    },
                }).collect()
            }),
            tool_choice: request.tool_choice.as_ref()
                .map(|tc| serde_json::to_value(tc).unwrap_or_default()),
        }
    }

    fn parse_response(&self, response: Self::Response) -> Response {
        let first_content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Response {
            id: response.id,
            object: response.object,
            created: response.created,
            model: response.model,
            choices: response.choices.into_iter().map(|choice| {
                crate::types::Choice {
                    index: choice.index,
                    message: crate::types::Message {
                        role: parse_role(&choice.message.role),
                        content: choice.message.content.unwrap_or_default(),
                        name: choice.message.name,
                        tool_calls: choice.message.tool_calls,
                        tool_call_id: choice.message.tool_call_id,
                        ..Default::default()
                    },
                    finish_reason: choice.finish_reason,
                    logprobs: None,
                }
            }).collect(),
            content: first_content,
            usage: Some(response.usage),
            system_fingerprint: response.system_fingerprint,
        }
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response(&self, response: Self::StreamResponse) -> crate::types::ChatStream {
        use futures_util::stream;

        let first_chunk_content = response
            .choices
            .first()
            .and_then(|c| c.delta.content.clone())
            .unwrap_or_default();

        let streaming_response = crate::types::StreamingResponse {
            id: response.id,
            object: response.object.unwrap_or_else(|| "chat.completion.chunk".to_string()),
            created: response.created,
            model: response.model,
            choices: response.choices.into_iter().map(|choice| {
                crate::types::StreamingChoice {
                    index: choice.index,
                    delta: crate::types::Delta {
                        role: choice.delta.role.as_ref().map(|r| parse_role(r)),
                        content: choice.delta.content,
                        tool_calls: choice.delta.tool_calls,
                        reasoning_content: None,
                        ..Default::default()
                    },
                    finish_reason: choice.finish_reason,
                    logprobs: None,
                }
            }).collect(),
            content: first_chunk_content,
            reasoning_content: None,
            usage: response.usage,
            system_fingerprint: response.system_fingerprint,
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

// ============================================================================
// Compatibility Layer (Temporary during migration)
// ============================================================================

impl Clone for OpenAIProtocol {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            base_url: self.base_url.clone(),
        }
    }
}

impl ProviderAdapter for OpenAIProtocol {
    type RequestType = OpenAIRequest;
    type ResponseType = OpenAIResponse;
    #[cfg(feature = "streaming")]
    type StreamResponseType = OpenAIStreamResponse;
    type ErrorMapperType = crate::protocols::core::StandardErrorMapper;

    fn name(&self) -> &str {
        &self.name
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        match base_url {
            Some(url) => format!("{}/chat/completions", url),
            None => format!("{}/chat/completions", &self.base_url),
        }
    }

    fn models_endpoint_url(&self, base_url: &Option<String>) -> Option<String> {
        Some(match base_url {
            Some(url) => format!("{}/models", url),
            None => format!("{}/models", &self.base_url),
        })
    }

    fn build_request_data(&self, request: &crate::types::ChatRequest, stream: bool) -> Self::RequestType {
        // Use the new Protocol implementation
        <Self as Protocol>::build_request(self, request, stream)
    }

    fn parse_response_data(&self, response: Self::ResponseType) -> crate::types::ChatResponse {
        // Use the new Protocol implementation
        <Self as Protocol>::parse_response(self, response)
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> crate::types::StreamingResponse {
        // Direct implementation for compatibility
        let first_chunk_content = response
            .choices
            .first()
            .and_then(|c| c.delta.content.clone())
            .unwrap_or_default();

        crate::types::StreamingResponse {
            id: response.id,
            object: response.object.unwrap_or_else(|| "chat.completion.chunk".to_string()),
            created: response.created,
            model: response.model,
            choices: response.choices.into_iter().map(|choice| {
                crate::types::StreamingChoice {
                    index: choice.index,
                    delta: crate::types::Delta {
                        role: choice.delta.role.as_ref().map(|r| parse_role(r)),
                        content: choice.delta.content,
                        tool_calls: choice.delta.tool_calls,
                        reasoning_content: None,
                        ..Default::default()
                    },
                    finish_reason: choice.finish_reason,
                    logprobs: None,
                }
            }).collect(),
            content: first_chunk_content,
            reasoning_content: None,
            usage: response.usage,
            system_fingerprint: response.system_fingerprint,
        }
    }

    #[cfg(feature = "streaming")]
    fn uses_sse_stream(&self) -> bool {
        true
    }
}
