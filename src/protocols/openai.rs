//! OpenAI Protocol Implementation
//!
//! This module implements the OpenAI protocol for OpenAI's API.
//!
//! # Protocol Details
//!
//! ## Endpoint
//! - Standard: `POST /v1/chat/completions`
//! - All providers follow the same endpoint structure
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
//!
//! # Example
//!
//! ```rust
//! use llm_connector::LlmClient;
//! use llm_connector::types::{ChatRequest, Message, Role};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create OpenAI client
//! let client = LlmClient::openai("your-api-key", None);
//!
//! // Create request
//! let request = ChatRequest {
//!     model: "gpt-4".to_string(),
//!     messages: vec![Message {
//!         role: Role::User,
//!         content: "Hello!".to_string(),
//!         ..Default::default()
//!     }],
//!     ..Default::default()
//! };
//!
//! // Send request
//! let response = client.chat(&request).await?;
//! println!("Response: {}", response.choices[0].message.content);
//! # Ok(())
//! # }
//! ```

use crate::protocols::core::{ProviderAdapter, StandardErrorMapper};
use crate::types::{ChatRequest, ChatResponse, Choice, Message, Role, ToolCall, Usage};
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
// OpenAI-Compatible Request/Response Structures
// ============================================================================

/// Standard OpenAI-compatible chat completion request
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
    pub tool_choice: Option<Value>,
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
    pub parameters: Option<Value>,
}

/// Standard OpenAI-compatible chat completion response
#[derive(Deserialize, Debug)]
pub struct OpenAIResponse {
    pub id: String,
    /// Object type - optional for compatibility with providers like Zhipu
    #[serde(default = "default_object_type")]
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// OpenAI model information from /models endpoint
#[derive(Deserialize, Debug)]
pub struct OpenAIModel {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

/// OpenAI models response from /models endpoint
#[derive(Deserialize, Debug)]
pub struct OpenAIModelsResponse {
    pub object: String,
    pub data: Vec<OpenAIModel>,
}

fn default_object_type() -> String {
    "chat.completion".to_string()
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

/// Standard OpenAI-compatible streaming response
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

// ============================================================================
// Conversion Utilities
// ============================================================================

impl OpenAIRequest {
    pub fn from_chat_request(request: &ChatRequest, stream: bool) -> Self {
        Self {
            model: request.model.clone(),
            messages: request.messages.iter().map(OpenAIMessage::from).collect(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: if stream { Some(true) } else { None },
            stop: request.stop.clone(),
            tools: request.tools.as_ref().map(|tools| {
                tools
                    .iter()
                    .map(|tool| OpenAITool {
                        r#type: "function".to_string(),
                        function: OpenAIFunction {
                            name: tool.function.name.clone(),
                            description: tool.function.description.clone().unwrap_or_default(),
                            parameters: Some(tool.function.parameters.clone()),
                        },
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

impl From<&Message> for OpenAIMessage {
    fn from(message: &Message) -> Self {
        Self {
            role: match message.role {
                crate::types::Role::System => "system".to_string(),
                crate::types::Role::User => "user".to_string(),
                crate::types::Role::Assistant => "assistant".to_string(),
                crate::types::Role::Tool => "tool".to_string(),
            },
            content: Some(message.content.clone()),
            name: message.name.clone(),
            tool_calls: message.tool_calls.clone(),
            tool_call_id: message.tool_call_id.clone(),
        }
    }
}

impl OpenAIResponse {
    pub fn to_chat_response(self) -> ChatResponse {
        let first_content = self
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        ChatResponse {
            id: self.id,
            object: self.object,
            created: self.created,
            model: self.model,
            choices: self
                .choices
                .into_iter()
                .map(|choice| Choice {
                    index: choice.index,
                    message: Message {
                        role: parse_role(&choice.message.role),
                        content: choice.message.content.unwrap_or_default(),
                        name: choice.message.name,
                        tool_calls: choice.message.tool_calls,
                        tool_call_id: choice.message.tool_call_id,
                        ..Default::default()
                    },
                    finish_reason: choice.finish_reason,
                    logprobs: None,
                })
                .collect(),
            content: first_content,
            usage: Some(self.usage),
            system_fingerprint: self.system_fingerprint,
        }
    }
}

#[cfg(feature = "streaming")]
impl OpenAIStreamResponse {
    pub fn to_streaming_response(self) -> StreamingResponse {
        let first_chunk_content = self
            .choices
            .get(0)
            .and_then(|c| c.delta.content.clone())
            .unwrap_or_default();

        StreamingResponse {
            id: self.id,
            object: self.object.unwrap_or_else(|| "chat.completion.chunk".to_string()),
            created: self.created,
            model: self.model,
            choices: self
                .choices
                .into_iter()
                .map(|choice| StreamingChoice {
                    index: choice.index,
                    delta: Delta {
                        role: choice
                            .delta
                            .role
                            .as_ref()
                            .map(|r| parse_role(r)),
                        content: choice.delta.content,
                        tool_calls: choice.delta.tool_calls,
                        reasoning_content: None,
                        ..Default::default()
                    },
                    finish_reason: choice.finish_reason,
                    logprobs: None,
                })
                .collect(),
            content: first_chunk_content,
            reasoning_content: None,
            usage: self.usage,
            system_fingerprint: self.system_fingerprint,
        }
    }
}

// ============================================================================
// Standard Adapter for OpenAI-compatible providers
// ============================================================================

/// OpenAI Protocol implementation
///
/// Uses Arc for efficient sharing of strings across clones.
#[derive(Debug, Clone)]
pub struct OpenAIProtocol {
    name: Arc<str>,
    base_url: Arc<str>,
}



#[async_trait]
impl ProviderAdapter for OpenAIProtocol {
    type RequestType = OpenAIRequest;
    type ResponseType = OpenAIResponse;
    #[cfg(feature = "streaming")]
    type StreamResponseType = OpenAIStreamResponse;
    type ErrorMapperType = StandardErrorMapper;

    fn name(&self) -> &str {
        &self.name
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        let base = base_url.as_deref().unwrap_or(&self.base_url);
        format!("{}/chat/completions", base)
    }

    fn models_endpoint_url(&self, base_url: &Option<String>) -> Option<String> {
        let base = base_url.as_deref().unwrap_or(&self.base_url);
        Some(format!("{}/models", base))
    }

    fn build_request_data(&self, request: &ChatRequest, stream: bool) -> Self::RequestType {
        OpenAIRequest::from_chat_request(request, stream)
    }

    fn parse_response_data(&self, response: Self::ResponseType) -> ChatResponse {
        response.to_chat_response()
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> StreamingResponse {
        response.to_streaming_response()
    }
}

// ============================================================================
// OpenAI Protocol Implementation
// ============================================================================

/// OpenAI protocol implementation
///
/// Implements the OpenAI API protocol.
/// Base URL: https://api.openai.com/v1

impl OpenAIProtocol {
    /// Create new OpenAI protocol
    ///
    /// Uses default OpenAI base URL: https://api.openai.com/v1
    pub fn new(_api_key: &str) -> Self {
        Self {
            name: Arc::from("openai"),
            base_url: Arc::from("https://api.openai.com/v1"),
        }
    }

    /// Create new OpenAI protocol with custom base URL
    ///
    /// This can be used for OpenAI-compatible endpoints if needed
    pub fn with_url(_api_key: &str, base_url: &str) -> Self {
        Self {
            name: Arc::from("openai"),
            base_url: Arc::from(base_url),
        }
    }
}

impl Default for OpenAIProtocol {
    fn default() -> Self {
        Self::new("") // Empty API key, user must set it
    }
}
