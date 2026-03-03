//! Aliyun DashScope Protocol Implementation
//!
//! This module provides the private Aliyun DashScope API protocol.

use crate::core::Protocol;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, EmbedRequest, EmbedResponse, Role};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Aliyun DashScope private protocol implementation
#[derive(Debug, Clone)]
pub struct AliyunProtocol {
    api_key: String,
}

impl AliyunProtocol {
    /// Create new Aliyun Protocol instance
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }

    /// Get API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get streaming request additional headers
    pub fn streaming_headers(&self) -> Vec<(String, String)> {
        vec![("X-DashScope-SSE".to_string(), "enable".to_string())]
    }
}

#[async_trait]
impl Protocol for AliyunProtocol {
    type Request = AliyunRequest;
    type Response = crate::protocols::utils::OpenAICompatibleResponse;

    fn name(&self) -> &str {
        "aliyun"
    }

    fn chat_endpoint(&self, base_url: &str, _model: &str) -> String {
        format!(
            "{}/api/v1/services/aigc/text-generation/generation",
            base_url
        )
    }

    fn embed_endpoint(&self, base_url: &str, _model: &str) -> Option<String> {
        Some(format!(
            "{}/api/v1/services/embeddings/text-embedding/text-embedding",
            base_url
        ))
    }

    fn auth_headers(&self) -> Vec<(String, String)> {
        vec![(
            "Authorization".to_string(),
            format!("Bearer {}", self.api_key),
        )]
    }

    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        let aliyun_messages: Vec<AliyunMessage> = request
            .messages
            .iter()
            .map(|msg| AliyunMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                content: msg.content_as_text(),
                tool_calls: msg.tool_calls.clone(),
            })
            .collect();

        Ok(AliyunRequest {
            model: request.model.clone(),
            input: AliyunInput {
                messages: aliyun_messages,
            },
            parameters: AliyunParameters {
                max_tokens: request.max_tokens,
                temperature: request.temperature,
                top_p: request.top_p,
                result_format: "message".to_string(),
                incremental_output: if request.stream.unwrap_or(false) {
                    Some(true)
                } else {
                    None
                },
                enable_thinking: request.enable_thinking,
                tools: request.tools.clone(),
                tool_choice: request.tool_choice.clone(),
            },
        })
    }

    fn build_embed_request(
        &self,
        request: &EmbedRequest,
    ) -> Result<serde_json::Value, LlmConnectorError> {
        let req = AliyunEmbedRequest {
            model: request.model.clone(),
            input: AliyunEmbedInput {
                texts: request.input.clone(),
            },
            parameters: request
                .encoding_format
                .as_deref()
                .map(|f| AliyunEmbedParameters {
                    text_type: f.to_string(),
                }),
        };
        serde_json::to_value(req).map_err(|e| {
            LlmConnectorError::ParseError(format!("Failed to serialize embed request: {}", e))
        })
    }

    fn parse_embed_response(&self, response: &str) -> Result<EmbedResponse, LlmConnectorError> {
        crate::protocols::utils::parse_openai_compatible_embed_response(response, self.name())
    }

    #[cfg(feature = "streaming")]
    async fn parse_stream_response(
        &self,
        response: reqwest::Response,
    ) -> Result<crate::types::ChatStream, LlmConnectorError> {
        Ok(crate::sse::sse_to_streaming_response(response))
    }

    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        crate::protocols::utils::parse_openai_compatible_chat_response(response, self.name())
    }

    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError {
        let body_lower = body.to_lowercase();
        if body_lower.contains("context_length_exceeded")
            || body_lower.contains("maximum context length")
            || body_lower.contains("input is too long")
        {
            return LlmConnectorError::ContextLengthExceeded(format!("Aliyun: {}", body));
        }
        LlmConnectorError::from_status_code(status, format!("Aliyun API error: {}", body))
    }
}

// Aliyun-specific data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunRequest {
    pub model: String,
    pub input: AliyunInput,
    pub parameters: AliyunParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunInput {
    pub messages: Vec<AliyunMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunEmbedRequest {
    pub model: String,
    pub input: AliyunEmbedInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<AliyunEmbedParameters>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunEmbedInput {
    pub texts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunEmbedParameters {
    pub text_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<crate::types::ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    pub result_format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental_output: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_thinking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<crate::types::Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<crate::types::ToolChoice>,
}
