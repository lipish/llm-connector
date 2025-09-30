//! Provider adapters for converting between unified API and provider-specific protocols
//!
//! This module contains concrete implementations of the `ProviderAdapter` trait for different
//! LLM providers. Each adapter handles:
//! - Request format conversion (unified → provider-specific)
//! - Response format conversion (provider-specific → unified)
//! - Error mapping
//! - Endpoint URL construction

use crate::error::LlmConnectorError;
use crate::providers::errors::ErrorMapper;
use crate::providers::traits::ProviderAdapter;
use crate::types::{ChatRequest, ChatResponse, Choice, Message, Usage};
#[cfg(feature = "streaming")]
use crate::types::{Delta, StreamingChoice, StreamingResponse};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================================
// Aliyun Adapter
// ============================================================================

/// Adapter for Alibaba Cloud's Qwen models (DashScope API)
///
/// Supports models: qwen-turbo, qwen-plus, etc.
/// API Documentation: https://help.aliyun.com/zh/dashscope/
#[derive(Debug, Clone)]
pub struct AliyunAdapter;

#[derive(Debug, Serialize)]
pub struct AliyunRequest {
    model: String,
    input: AliyunInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<AliyunParameters>,
}

#[derive(Debug, Serialize)]
pub struct AliyunInput {
    messages: Vec<AliyunMessage>,
}

#[derive(Debug, Serialize)]
pub struct AliyunMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Default)]
pub struct AliyunParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    result_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    incremental_output: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AliyunResponse {
    request_id: String,
    output: AliyunOutput,
    usage: AliyunUsage,
}

#[derive(Debug, Deserialize)]
pub struct AliyunOutput {
    choices: Vec<AliyunChoice>,
}

#[derive(Debug, Deserialize)]
pub struct AliyunChoice {
    message: AliyunResponseMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct AliyunResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct AliyunUsage {
    input_tokens: i32,
    output_tokens: i32,
}

#[derive(Debug, Deserialize)]
pub struct AliyunStreamResponse {
    request_id: String,
    output: AliyunOutput,
    usage: AliyunUsage,
}

pub struct AliyunErrorMapper;

impl ErrorMapper for AliyunErrorMapper {
    fn map_http_error(status: u16, body: Value) -> LlmConnectorError {
        let error_message = body["error"]["message"]
            .as_str()
            .unwrap_or("Unknown Aliyun error");
        LlmConnectorError::ProviderError(format!(
            "HTTP status: {}, message: {}",
            status, error_message
        ))
    }

    fn map_network_error(error: reqwest::Error) -> LlmConnectorError {
        LlmConnectorError::NetworkError(error.to_string())
    }

    fn is_retriable_error(_error: &LlmConnectorError) -> bool {
        false
    }
}

#[async_trait]
impl ProviderAdapter for AliyunAdapter {
    type RequestType = AliyunRequest;
    type ResponseType = AliyunResponse;
    type StreamResponseType = AliyunStreamResponse;
    type ErrorMapperType = AliyunErrorMapper;

    fn name(&self) -> &str {
        "aliyun"
    }

    fn supported_models(&self) -> Vec<String> {
        vec!["qwen-turbo".to_string(), "qwen-plus".to_string()]
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        base_url
            .as_deref()
            .unwrap_or(
                "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation",
            )
            .to_string()
    }

    fn build_request_data(&self, request: &ChatRequest, stream: bool) -> Self::RequestType {
        let messages = request
            .messages
            .iter()
            .map(|msg| AliyunMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect();

        let parameters = AliyunParameters {
            temperature: request.temperature,
            top_p: request.top_p,
            max_tokens: request.max_tokens,
            seed: request.seed,
            result_format: Some("message".to_string()),
            incremental_output: if stream { Some(true) } else { None },
            ..Default::default()
        };

        AliyunRequest {
            model: request.model.clone(),
            input: AliyunInput { messages },
            parameters: Some(parameters),
        }
    }

    fn parse_response_data(&self, response: Self::ResponseType) -> ChatResponse {
        ChatResponse {
            id: response.request_id,
            object: "chat.completion".to_string(),
            created: 0,
            model: "".to_string(),
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: response.output.choices[0].message.role.clone(),
                    content: response.output.choices[0].message.content.clone(),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                logprobs: None,
                finish_reason: Some(response.output.choices[0].finish_reason.clone()),
            }],
            usage: Some(Usage {
                prompt_tokens: response.usage.input_tokens as u32,
                completion_tokens: response.usage.output_tokens as u32,
                total_tokens: (response.usage.input_tokens + response.usage.output_tokens) as u32,
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        }
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> StreamingResponse {
        StreamingResponse {
            id: response.request_id,
            object: "chat.completion.chunk".to_string(),
            created: 0,
            model: "".to_string(),
            choices: vec![StreamingChoice {
                index: 0,
                delta: Delta {
                    role: Some(response.output.choices[0].message.role.clone()),
                    content: Some(response.output.choices[0].message.content.clone()),
                    tool_calls: None,
                    reasoning_content: None,
                },
                finish_reason: Some(response.output.choices[0].finish_reason.clone()),
                logprobs: None,
            }],
            usage: Some(Usage {
                prompt_tokens: response.usage.input_tokens as u32,
                completion_tokens: response.usage.output_tokens as u32,
                total_tokens: (response.usage.input_tokens + response.usage.output_tokens) as u32,
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
// DeepSeek Adapter
// ============================================================================

/// Adapter for DeepSeek's chat and coder models
///
/// Supports models: deepseek-chat, deepseek-coder, deepseek-reasoner
/// API Documentation: https://api-docs.deepseek.com/
///
/// DeepSeek uses OpenAI-compatible API format, making integration straightforward.
#[derive(Debug, Clone)]
pub struct DeepSeekAdapter;

#[derive(Serialize, Debug)]
pub struct DeepSeekRequest {
    model: String,
    messages: Vec<DeepSeekMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<DeepSeekTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<Value>,
}

#[derive(Serialize, Debug)]
struct DeepSeekMessage {
    role: String,
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<crate::types::ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Serialize, Debug)]
struct DeepSeekTool {
    r#type: String,
    function: DeepSeekFunction,
}

#[derive(Serialize, Debug)]
struct DeepSeekFunction {
    name: String,
    description: String,
    parameters: Option<Value>,
    strict: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct DeepSeekResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<DeepSeekChoice>,
    usage: Usage,
    system_fingerprint: Option<String>,
}

#[derive(Deserialize, Debug)]
struct DeepSeekChoice {
    index: u32,
    message: DeepSeekResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct DeepSeekResponseMessage {
    role: String,
    content: Option<String>,
    name: Option<String>,
    tool_calls: Option<Vec<crate::types::ToolCall>>,
    tool_call_id: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DeepSeekStreamResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<DeepSeekStreamChoice>,
    usage: Option<Usage>,
    system_fingerprint: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct DeepSeekStreamChoice {
    index: u32,
    delta: DeepSeekStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct DeepSeekStreamDelta {
    role: Option<String>,
    content: Option<String>,
    tool_calls: Option<Vec<crate::types::ToolCall>>,
}

pub struct DeepSeekErrorMapper;

impl ErrorMapper for DeepSeekErrorMapper {
    fn map_http_error(status: u16, body: Value) -> LlmConnectorError {
        let error_message = body["error"]["message"]
            .as_str()
            .unwrap_or("Unknown DeepSeek error");
        LlmConnectorError::ProviderError(format!(
            "HTTP status: {}, message: {}",
            status, error_message
        ))
    }

    fn map_network_error(error: reqwest::Error) -> LlmConnectorError {
        LlmConnectorError::NetworkError(error.to_string())
    }

    fn is_retriable_error(_error: &LlmConnectorError) -> bool {
        false
    }
}

#[async_trait]
impl ProviderAdapter for DeepSeekAdapter {
    type RequestType = DeepSeekRequest;
    type ResponseType = DeepSeekResponse;
    type StreamResponseType = DeepSeekStreamResponse;
    type ErrorMapperType = DeepSeekErrorMapper;

    fn name(&self) -> &str {
        "deepseek"
    }

    fn supported_models(&self) -> Vec<String> {
        vec!["deepseek-chat".to_string(), "deepseek-coder".to_string()]
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        let base = base_url
            .as_deref()
            .unwrap_or("https://api.deepseek.com/v1");
        format!("{}/chat/completions", base)
    }

    fn build_request_data(&self, request: &ChatRequest, stream: bool) -> Self::RequestType {
        let messages = request
            .messages
            .iter()
            .map(|msg| DeepSeekMessage {
                role: msg.role.clone(),
                content: Some(msg.content.clone()),
                name: msg.name.clone(),
                tool_calls: msg.tool_calls.clone(),
                tool_call_id: msg.tool_call_id.clone(),
            })
            .collect();

        let tools = request.tools.as_ref().map(|tools| {
            tools
                .iter()
                .map(|t| DeepSeekTool {
                    r#type: t.tool_type.clone(),
                    function: DeepSeekFunction {
                        name: t.function.name.clone(),
                        description: t.function.description.clone().unwrap_or_default(),
                        parameters: Some(t.function.parameters.clone()),
                        strict: None,
                    },
                })
                .collect()
        });

        let tool_choice = request
            .tool_choice
            .as_ref()
            .map(|tc| serde_json::to_value(tc).unwrap_or(Value::Null));

        DeepSeekRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: Some(stream),
            tools,
            tool_choice,
        }
    }

    fn parse_response_data(&self, response: Self::ResponseType) -> ChatResponse {
        ChatResponse {
            id: response.id,
            model: response.model,
            created: response.created,
            object: response.object,
            choices: response
                .choices
                .into_iter()
                .map(|choice| Choice {
                    index: choice.index,
                    message: Message {
                        role: choice.message.role,
                        content: choice.message.content.unwrap_or_default(),
                        name: choice.message.name,
                        tool_calls: choice.message.tool_calls,
                        tool_call_id: choice.message.tool_call_id,
                    },
                    logprobs: None,
                    finish_reason: choice.finish_reason,
                })
                .collect(),
            usage: Some(response.usage),
            system_fingerprint: response.system_fingerprint,
        }
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> StreamingResponse {
        StreamingResponse {
            id: response.id,
            object: response.object,
            created: response.created,
            model: response.model,
            choices: response
                .choices
                .into_iter()
                .map(|choice| StreamingChoice {
                    index: choice.index,
                    delta: Delta {
                        role: choice.delta.role,
                        content: choice.delta.content,
                        tool_calls: choice.delta.tool_calls,
                        reasoning_content: None,
                    },
                    finish_reason: choice.finish_reason,
                    logprobs: None,
                })
                .collect(),
            usage: response.usage.clone(),
            system_fingerprint: response.system_fingerprint,
        }
    }
}

// ============================================================================
// Zhipu Adapter
// ============================================================================

/// Adapter for Zhipu AI's GLM models (ChatGLM)
///
/// Supports models: glm-4, glm-3-turbo, etc.
/// API Documentation: https://open.bigmodel.cn/dev/api
///
/// Zhipu uses OpenAI-compatible API format.
#[derive(Debug, Clone)]
pub struct ZhipuAdapter;

#[derive(Serialize, Debug)]
pub struct ZhipuRequest {
    model: String,
    messages: Vec<ZhipuMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Serialize, Debug)]
pub struct ZhipuMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
pub struct ZhipuResponse {
    id: String,
    created: u64,
    model: String,
    choices: Vec<ZhipuChoice>,
    usage: crate::types::Usage,
}

#[derive(Deserialize, Debug)]
pub struct ZhipuChoice {
    index: u32,
    finish_reason: String,
    message: ZhipuResponseMessage,
}

#[derive(Deserialize, Debug)]
pub struct ZhipuResponseMessage {
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<crate::types::ToolCall>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ZhipuStreamResponse {
    id: String,
    created: u64,
    model: String,
    choices: Vec<ZhipuStreamChoice>,
    usage: Option<crate::types::Usage>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ZhipuStreamChoice {
    delta: ZhipuStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ZhipuStreamDelta {
    content: String,
}

pub struct ZhipuErrorMapper;

impl ErrorMapper for ZhipuErrorMapper {
    fn map_http_error(status: u16, body: Value) -> LlmConnectorError {
        let error_message = body["error"]["message"]
            .as_str()
            .unwrap_or("Unknown Zhipu error");
        LlmConnectorError::ProviderError(format!(
            "HTTP status: {}, message: {}",
            status, error_message
        ))
    }

    fn map_network_error(error: reqwest::Error) -> LlmConnectorError {
        LlmConnectorError::NetworkError(error.to_string())
    }

    fn is_retriable_error(_error: &LlmConnectorError) -> bool {
        false
    }
}

#[async_trait]
impl ProviderAdapter for ZhipuAdapter {
    type RequestType = ZhipuRequest;
    type ResponseType = ZhipuResponse;
    type StreamResponseType = ZhipuStreamResponse;
    type ErrorMapperType = ZhipuErrorMapper;

    fn name(&self) -> &str {
        "zhipu"
    }

    fn supported_models(&self) -> Vec<String> {
        vec!["glm-4".to_string(), "glm-3-turbo".to_string()]
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        let base = base_url
            .as_deref()
            .unwrap_or("https://open.bigmodel.cn/api/paas/v4");
        format!("{}/chat/completions", base)
    }

    fn build_request_data(&self, request: &ChatRequest, stream: bool) -> Self::RequestType {
        let messages = request
            .messages
            .iter()
            .map(|msg| ZhipuMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect();

        ZhipuRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: Some(stream),
            stop: request.stop.clone(),
        }
    }

    fn parse_response_data(&self, response: Self::ResponseType) -> ChatResponse {
        ChatResponse {
            id: response.id,
            object: "chat.completion".to_string(),
            created: response.created,
            model: response.model,
            choices: response
                .choices
                .into_iter()
                .map(|choice| Choice {
                    index: choice.index,
                    message: Message {
                        role: choice.message.role,
                        content: choice.message.content.unwrap_or_default(),
                        name: None,
                        tool_calls: choice.message.tool_calls,
                        tool_call_id: None,
                    },
                    logprobs: None,
                    finish_reason: Some(choice.finish_reason),
                })
                .collect(),
            usage: Some(response.usage),
            system_fingerprint: None,
        }
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> StreamingResponse {
        let choices = response
            .choices
            .iter()
            .map(|choice| StreamingChoice {
                index: 0,
                delta: Delta {
                    role: None,
                    content: Some(choice.delta.content.clone()),
                    tool_calls: None,
                    reasoning_content: None,
                },
                finish_reason: choice.finish_reason.clone(),
                logprobs: None,
            })
            .collect();

        StreamingResponse {
            id: response.id,
            object: "chat.completion.chunk".to_string(),
            created: response.created,
            model: response.model.clone(),
            choices,
            usage: response.usage.clone(),
            system_fingerprint: None,
        }
    }
}
