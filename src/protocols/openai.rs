//! OpenAI Protocol Implementation
//!
//! This module implements the OpenAI-compatible protocol used by multiple providers.
//! The OpenAI protocol has become the de facto standard for LLM APIs.

use crate::types::{ChatRequest, ChatResponse, Choice, Message, Usage, ToolCall};
use crate::protocols::core::{ProviderAdapter, StandardErrorMapper};
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub object: String,
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
                tools.iter().map(|tool| OpenAITool {
                    r#type: "function".to_string(),
                    function: OpenAIFunction {
                        name: tool.function.name.clone(),
                        description: tool.function.description.clone().unwrap_or_default(),
                        parameters: Some(tool.function.parameters.clone()),
                    },
                }).collect()
            }),
            tool_choice: request.tool_choice.as_ref().map(|tc| serde_json::to_value(tc).unwrap_or_default()),
        }
    }
}

impl From<&Message> for OpenAIMessage {
    fn from(message: &Message) -> Self {
        Self {
            role: message.role.clone(),
            content: Some(message.content.clone()),
            name: message.name.clone(),
            tool_calls: message.tool_calls.clone(),
            tool_call_id: message.tool_call_id.clone(),
        }
    }
}

impl OpenAIResponse {
    pub fn to_chat_response(self) -> ChatResponse {
        ChatResponse {
            id: self.id,
            object: self.object,
            created: self.created,
            model: self.model,
            choices: self.choices.into_iter().map(|choice| Choice {
                index: choice.index,
                message: Message {
                    role: choice.message.role,
                    content: choice.message.content.unwrap_or_default(),
                    name: choice.message.name,
                    tool_calls: choice.message.tool_calls,
                    tool_call_id: choice.message.tool_call_id,
                },
                finish_reason: choice.finish_reason,
                logprobs: None,
            }).collect(),
            usage: Some(self.usage),
            system_fingerprint: self.system_fingerprint,
        }
    }
}

#[cfg(feature = "streaming")]
impl OpenAIStreamResponse {
    pub fn to_streaming_response(self) -> StreamingResponse {
        StreamingResponse {
            id: self.id,
            object: self.object,
            created: self.created,
            model: self.model,
            choices: self.choices.into_iter().map(|choice| StreamingChoice {
                index: choice.index,
                delta: Delta {
                    role: choice.delta.role,
                    content: choice.delta.content,
                    tool_calls: choice.delta.tool_calls,
                },
                finish_reason: choice.finish_reason,
            }).collect(),
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
    supported_models: Arc<[String]>,
}

impl OpenAIProtocol {
    pub fn new(name: &str, base_url: &str, models: Vec<&str>) -> Self {
        Self {
            name: Arc::from(name),
            base_url: Arc::from(base_url),
            supported_models: Arc::from(
                models.iter().map(|s| s.to_string()).collect::<Vec<_>>()
            ),
        }
    }
}

// Legacy compatibility
pub type StandardAdapter = OpenAIProtocol;

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

    fn supported_models(&self) -> Vec<String> {
        self.supported_models.to_vec()
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        let base = base_url.as_deref().unwrap_or(&self.base_url);
        format!("{}/chat/completions", base)
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
// Pre-configured Standard Adapters
// ============================================================================

/// DeepSeek provider using OpenAI protocol
pub fn deepseek() -> OpenAIProtocol {
    OpenAIProtocol::new(
        "deepseek",
        "https://api.deepseek.com/v1",
        vec!["deepseek-chat", "deepseek-coder"]
    )
}

/// Zhipu (GLM) provider using OpenAI protocol
pub fn zhipu() -> OpenAIProtocol {
    OpenAIProtocol::new(
        "zhipu",
        "https://open.bigmodel.cn/api/paas/v4",
        vec!["glm-4", "glm-3-turbo"]
    )
}

/// Moonshot provider using OpenAI protocol
pub fn moonshot() -> OpenAIProtocol {
    OpenAIProtocol::new(
        "moonshot",
        "https://api.moonshot.cn/v1",
        vec!["moonshot-v1-8k", "moonshot-v1-32k", "moonshot-v1-128k"]
    )
}

/// VolcEngine (Doubao) provider using OpenAI protocol
/// Note: VolcEngine uses endpoint IDs as model names
/// You need to create an endpoint in the console and use its ID
pub fn volcengine() -> OpenAIProtocol {
    OpenAIProtocol::new(
        "volcengine",
        "https://ark.cn-beijing.volces.com/api/v3",
        vec!["ep-*"]  // Endpoint IDs start with "ep-"
    )
}

/// Tencent Hunyuan provider using OpenAI protocol
pub fn tencent() -> OpenAIProtocol {
    OpenAIProtocol::new(
        "tencent",
        "https://hunyuan.tencentcloudapi.com/v1",
        vec!["hunyuan-lite", "hunyuan-standard", "hunyuan-pro"]
    )
}

/// MiniMax provider using OpenAI protocol
pub fn minimax() -> OpenAIProtocol {
    OpenAIProtocol::new(
        "minimax",
        "https://api.minimax.chat/v1",
        vec!["abab6.5s-chat", "abab6.5-chat", "abab5.5-chat"]
    )
}

/// StepFun provider using OpenAI protocol
pub fn stepfun() -> OpenAIProtocol {
    OpenAIProtocol::new(
        "stepfun",
        "https://api.stepfun.com/v1",
        vec!["step-1-8k", "step-1-32k", "step-1-128k"]
    )
}

/// LongCat provider using OpenAI protocol
pub fn longcat() -> OpenAIProtocol {
    OpenAIProtocol::new(
        "longcat",
        "https://api.longcat.chat/openai",
        vec!["LongCat-Flash-Chat", "LongCat-Flash-Thinking"]
    )
}

/// Get all providers that use the OpenAI protocol
pub fn openai_providers() -> Vec<(&'static str, OpenAIProtocol)> {
    vec![
        ("deepseek", deepseek()),
        ("zhipu", zhipu()),
        ("moonshot", moonshot()),
        ("volcengine", volcengine()),
        ("tencent", tencent()),
        ("minimax", minimax()),
        ("stepfun", stepfun()),
        ("longcat", longcat()),
    ]
}

// ============================================================================
// Convenience type aliases
// ============================================================================

pub type DeepSeekProvider = crate::protocols::core::GenericProvider<OpenAIProtocol>;
pub type ZhipuProvider = crate::protocols::core::GenericProvider<OpenAIProtocol>;
pub type MoonshotProvider = crate::protocols::core::GenericProvider<OpenAIProtocol>;
pub type VolcEngineProvider = crate::protocols::core::GenericProvider<OpenAIProtocol>;
pub type TencentProvider = crate::protocols::core::GenericProvider<OpenAIProtocol>;
pub type MiniMaxProvider = crate::protocols::core::GenericProvider<OpenAIProtocol>;
pub type StepFunProvider = crate::protocols::core::GenericProvider<OpenAIProtocol>;
