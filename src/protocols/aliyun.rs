//! Aliyun Protocol Implementation
//!
//! This module implements the Aliyun DashScope API protocol.
//! Aliyun uses a custom protocol that differs significantly from both OpenAI and Anthropic.

use crate::error::LlmConnectorError;
use crate::protocols::core::{ProviderAdapter, ErrorMapper};
use std::sync::Arc;
use crate::types::{ChatRequest, ChatResponse, Message, Usage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "streaming")]
use crate::types::StreamingResponse;

// ============================================================================
// Aliyun-specific Request Structures
// ============================================================================

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
    #[serde(skip_serializing_if = "Option::is_none")]
    result_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    incremental_output: Option<bool>,
}

// ============================================================================
// Aliyun-specific Response Structures
// ============================================================================

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

// ============================================================================
// Aliyun-specific Streaming Response
// ============================================================================

#[cfg(feature = "streaming")]
#[derive(Debug, Deserialize)]
pub struct AliyunStreamResponse {
    request_id: String,
    output: AliyunStreamOutput,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<AliyunUsage>,
}

#[cfg(feature = "streaming")]
#[derive(Debug, Deserialize)]
pub struct AliyunStreamOutput {
    choices: Vec<AliyunStreamChoice>,
}

#[cfg(feature = "streaming")]
#[derive(Debug, Deserialize)]
pub struct AliyunStreamChoice {
    message: AliyunResponseMessage,
    finish_reason: Option<String>,
}

// ============================================================================
// Aliyun Error Mapper
// ============================================================================

pub struct AliyunErrorMapper;

impl ErrorMapper for AliyunErrorMapper {
    fn map_http_error(status: u16, body: Value) -> LlmConnectorError {
        let error_message = body["error"]["message"]
            .as_str()
            .or_else(|| body["message"].as_str())
            .unwrap_or("Unknown Aliyun error");
            
        let error_code = body["error"]["code"]
            .as_str()
            .or_else(|| body["code"].as_str())
            .unwrap_or("unknown");

        match status {
            400 => LlmConnectorError::InvalidRequest(format!("Aliyun: {} ({})", error_message, error_code)),
            401 => LlmConnectorError::AuthenticationError(format!("Aliyun: {} ({})", error_message, error_code)),
            403 => LlmConnectorError::PermissionError(format!("Aliyun: {} ({})", error_message, error_code)),
            429 => LlmConnectorError::RateLimitError(format!("Aliyun: {} ({})", error_message, error_code)),
            500..=599 => LlmConnectorError::ServerError(format!("Aliyun HTTP {}: {} ({})", status, error_message, error_code)),
            _ => LlmConnectorError::ProviderError(format!("Aliyun HTTP {}: {} ({})", status, error_message, error_code)),
        }
    }

    fn map_network_error(error: reqwest::Error) -> LlmConnectorError {
        if error.is_timeout() {
            LlmConnectorError::TimeoutError(format!("Aliyun: {}", error))
        } else if error.is_connect() {
            LlmConnectorError::ConnectionError(format!("Aliyun: {}", error))
        } else {
            LlmConnectorError::NetworkError(format!("Aliyun: {}", error))
        }
    }

    fn is_retriable_error(error: &LlmConnectorError) -> bool {
        matches!(error, 
            LlmConnectorError::RateLimitError(_) |
            LlmConnectorError::ServerError(_) |
            LlmConnectorError::TimeoutError(_) |
            LlmConnectorError::ConnectionError(_)
        )
    }
}

// ============================================================================
// Aliyun Adapter Implementation
// ============================================================================

/// Aliyun Protocol adapter for DashScope API
///
/// Uses Arc for efficient sharing of strings across clones.
#[derive(Debug, Clone)]
pub struct AliyunProtocol {
    base_url: Arc<str>,
    supported_models: Arc<[String]>,
}

impl AliyunProtocol {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: Arc::from(base_url.unwrap_or("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation")),
            supported_models: Arc::from(vec![
                "qwen-turbo".to_string(),
                "qwen-plus".to_string(),
                "qwen-max".to_string(),
                "qwen-max-1201".to_string(),
                "qwen-max-longcontext".to_string(),
            ].into_boxed_slice()),
        }
    }
}

// Legacy compatibility
pub type AliyunAdapter = AliyunProtocol;

#[async_trait]
impl ProviderAdapter for AliyunProtocol {
    type RequestType = AliyunRequest;
    type ResponseType = AliyunResponse;
    #[cfg(feature = "streaming")]
    type StreamResponseType = AliyunStreamResponse;
    type ErrorMapperType = AliyunErrorMapper;

    fn name(&self) -> &str {
        "aliyun"
    }

    fn supported_models(&self) -> Vec<String> {
        self.supported_models.to_vec()
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        base_url
            .as_deref()
            .unwrap_or(&self.base_url)
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
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            seed: None, // Aliyun-specific field, could be configurable
            result_format: Some("message".to_string()),
            incremental_output: if stream { Some(true) } else { None },
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
            created: chrono::Utc::now().timestamp() as u64,
            model: "qwen".to_string(), // Aliyun doesn't return model in response
            choices: response.output.choices.into_iter().enumerate().map(|(index, choice)| {
                crate::types::Choice {
                    index: index as u32,
                    message: Message {
                        role: choice.message.role,
                        content: choice.message.content,
                        name: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(choice.finish_reason),
                    logprobs: None,
                }
            }).collect(),
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
            created: chrono::Utc::now().timestamp() as u64,
            model: "qwen".to_string(),
            choices: response.output.choices.into_iter().enumerate().map(|(index, choice)| {
                crate::types::StreamingChoice {
                    index: index as u32,
                    delta: crate::types::Delta {
                        role: Some(choice.message.role),
                        content: Some(choice.message.content),
                        tool_calls: None,
                    },
                    finish_reason: choice.finish_reason,
                }
            }).collect(),
            usage: response.usage.map(|usage| Usage {
                prompt_tokens: usage.input_tokens as u32,
                completion_tokens: usage.output_tokens as u32,
                total_tokens: (usage.input_tokens + usage.output_tokens) as u32,
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
// Convenience Functions and Type Aliases
// ============================================================================

/// Create an Aliyun protocol adapter
pub fn aliyun() -> AliyunProtocol {
    AliyunProtocol::new(None)
}

/// Get all providers that use the Aliyun protocol
pub fn aliyun_providers() -> Vec<(&'static str, AliyunProtocol)> {
    vec![
        ("aliyun", aliyun()),
        ("dashscope", aliyun()),
        ("qwen", aliyun()),
    ]
}

/// Aliyun provider type alias
pub type AliyunProvider = crate::protocols::core::GenericProvider<AliyunProtocol>;
