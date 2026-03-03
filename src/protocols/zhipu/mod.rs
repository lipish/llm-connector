//! Zhipu GLM Protocol Implementation
//!
//! This module provides the private Zhipu GLM API protocol.

use crate::core::Protocol;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, Tool, ToolChoice};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Extract reasoning content from Zhipu response

/// Zhipu GLM private protocol implementation
#[derive(Clone, Debug)]
pub struct ZhipuProtocol {
    api_key: String,
    use_openai_format: bool,
}

impl ZhipuProtocol {
    /// Create new Zhipu Protocol instance (using native format)
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            use_openai_format: false,
        }
    }

    /// Create Zhipu Protocol instance using OpenAI compatible format
    pub fn new_openai_compatible(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            use_openai_format: true,
        }
    }

    /// Get API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Whether to use OpenAI compatible format
    pub fn is_openai_compatible(&self) -> bool {
        self.use_openai_format
    }
}

#[async_trait::async_trait]
impl Protocol for ZhipuProtocol {
    type Request = ZhipuRequest;
    type Response = crate::protocols::common::openai::OpenAICompatibleResponse;

    fn name(&self) -> &str {
        "zhipu"
    }

    fn chat_endpoint(&self, base_url: &str, _model: &str) -> String {
        let base = base_url.trim_end_matches('/');
        if base.ends_with("/api/paas/v4") {
            format!("{}/chat/completions", base)
        } else {
            format!("{}/api/paas/v4/chat/completions", base)
        }
    }

    fn auth_headers(&self) -> Vec<(String, String)> {
        crate::protocols::common::auth::bearer_auth(&self.api_key)
    }

    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        let messages =
            crate::protocols::common::request::openai_message_converter(&request.messages);

        Ok(ZhipuRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: request.stream,
            tools: request.tools.clone(),
            tool_choice: request.tool_choice.clone(),
        })
    }

    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        crate::protocols::common::openai::parse_openai_compatible_chat_response(
            response,
            self.name(),
        )
    }

    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError {
        let body_lower = body.to_lowercase();
        if body_lower.contains("context_length_exceeded")
            || body_lower.contains("maximum context length")
            || body_lower.contains("token limit")
        {
            return LlmConnectorError::ContextLengthExceeded(format!("Zhipu: {}", body));
        }
        LlmConnectorError::from_status_code(status, format!("Zhipu API error: {}", body))
    }

    #[cfg(feature = "streaming")]
    async fn parse_stream_response(
        &self,
        response: reqwest::Response,
    ) -> Result<crate::types::ChatStream, LlmConnectorError> {
        Ok(crate::sse::sse_to_streaming_response(response))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZhipuRequest {
    pub model: String,
    pub messages: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZhipuMessage {
    pub role: String,
    #[serde(default)]
    pub content: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
