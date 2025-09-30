//! Anthropic Protocol Implementation
//! 
//! This module implements the Anthropic Messages API protocol used by Claude models.
//! The Anthropic protocol differs from OpenAI in several key ways:
//! - Uses `/v1/messages` endpoint instead of `/chat/completions`
//! - Response content is an array of content blocks
//! - Different streaming format
//! - Different error structure

use crate::types::{ChatRequest, ChatResponse, Choice, Message, Usage, ToolCall};
use crate::protocols::core::{ProviderAdapter, ErrorMapper};
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    ToolUse { id: String, name: String, input: Value },
    #[serde(rename = "tool_result")]
    ToolResult { tool_use_id: String, content: String },
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

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum AnthropicResponseContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse { id: String, name: String, input: Value },
}

#[derive(Deserialize, Debug)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

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
            400 => crate::error::LlmConnectorError::InvalidRequest(format!("Anthropic: {} ({})", error_message, error_type)),
            401 => crate::error::LlmConnectorError::AuthenticationError(format!("Anthropic: {} ({})", error_message, error_type)),
            403 => crate::error::LlmConnectorError::PermissionError(format!("Anthropic: {} ({})", error_message, error_type)),
            429 => crate::error::LlmConnectorError::RateLimitError(format!("Anthropic: {} ({})", error_message, error_type)),
            500..=599 => crate::error::LlmConnectorError::ServerError(format!("Anthropic HTTP {}: {} ({})", status, error_message, error_type)),
            _ => crate::error::LlmConnectorError::ProviderError(format!("Anthropic HTTP {}: {} ({})", status, error_message, error_type)),
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
        matches!(error, 
            crate::error::LlmConnectorError::RateLimitError(_) |
            crate::error::LlmConnectorError::ServerError(_) |
            crate::error::LlmConnectorError::TimeoutError(_) |
            crate::error::LlmConnectorError::ConnectionError(_)
        )
    }
}

// ============================================================================
// Conversion Utilities
// ============================================================================

impl AnthropicRequest {
    pub fn from_chat_request(request: &ChatRequest, stream: bool) -> Self {
        // Extract system message if present
        let (system_message, user_messages): (Vec<_>, Vec<_>) = request.messages
            .iter()
            .partition(|msg| msg.role == "system");
        
        let system = system_message.first().map(|msg| msg.content.clone());
        
        let messages = user_messages.iter().map(|msg| AnthropicMessage {
            role: msg.role.clone(),
            content: AnthropicContent::Text(msg.content.clone()),
        }).collect();

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
                tools.iter().map(|tool| AnthropicTool {
                    name: tool.function.name.clone(),
                    description: tool.function.description.clone().unwrap_or_default(),
                    input_schema: tool.function.parameters.clone(),
                }).collect()
            }),
            tool_choice: request.tool_choice.as_ref().map(|tc| serde_json::to_value(tc).unwrap_or_default()),
        }
    }
}

impl AnthropicResponse {
    pub fn to_chat_response(self) -> ChatResponse {
        let content = self.content.into_iter()
            .filter_map(|block| match block {
                AnthropicResponseContent::Text { text } => Some(text),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        ChatResponse {
            id: self.id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: self.model,
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: self.role,
                    content,
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: self.stop_reason,
                logprobs: None,
            }],
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
    supported_models: Arc<[String]>,
}

impl AnthropicProtocol {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: Arc::from(base_url.unwrap_or("https://api.anthropic.com")),
            supported_models: Arc::from(vec![
                "claude-3-5-sonnet-20241022".to_string(),
                "claude-3-5-haiku-20241022".to_string(),
                "claude-3-opus-20240229".to_string(),
                "claude-3-sonnet-20240229".to_string(),
                "claude-3-haiku-20240307".to_string(),
            ].into_boxed_slice()),
        }
    }
}

#[async_trait]
impl ProviderAdapter for AnthropicProtocol {
    type RequestType = AnthropicRequest;
    type ResponseType = AnthropicResponse;
    #[cfg(feature = "streaming")]
    type StreamResponseType = AnthropicStreamResponse;
    type ErrorMapperType = AnthropicErrorMapper;

    fn name(&self) -> &str {
        "anthropic"
    }

    fn supported_models(&self) -> Vec<String> {
        self.supported_models.to_vec()
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        let base = base_url.as_deref().unwrap_or(&self.base_url);
        format!("{}/v1/messages", base)
    }

    fn build_request_data(&self, request: &ChatRequest, stream: bool) -> Self::RequestType {
        AnthropicRequest::from_chat_request(request, stream)
    }

    fn parse_response_data(&self, response: Self::ResponseType) -> ChatResponse {
        response.to_chat_response()
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> StreamingResponse {
        // Simplified streaming response conversion
        // In practice, Anthropic streaming is more complex and requires state management
        StreamingResponse {
            id: "anthropic-stream".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: "claude".to_string(),
            choices: vec![StreamingChoice {
                index: 0,
                delta: Delta {
                    role: None,
                    content: response.delta.and_then(|d| d.text),
                    tool_calls: None,
                },
                finish_reason: None,
            }],
            usage: response.usage.map(|usage| Usage {
                prompt_tokens: usage.input_tokens,
                completion_tokens: usage.output_tokens,
                total_tokens: usage.input_tokens + usage.output_tokens,
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
// Convenience Functions
// ============================================================================

/// Create an Anthropic protocol adapter
pub fn anthropic() -> AnthropicProtocol {
    AnthropicProtocol::new(None)
}

/// Get all providers that use the Anthropic protocol
pub fn anthropic_providers() -> Vec<(&'static str, AnthropicProtocol)> {
    vec![
        ("anthropic", anthropic()),
        ("claude", anthropic()),
    ]
}

/// Anthropic provider type alias
pub type AnthropicProvider = crate::protocols::core::GenericProvider<AnthropicProtocol>;
