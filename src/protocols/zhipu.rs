//! Zhipu GLM Protocol Implementation
//!
//! This module provides the private Zhipu GLM API protocol.

use crate::core::Protocol;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, ImageSource, MessageBlock, Role, Tool, ToolChoice};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

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
    type Response = crate::protocols::utils::OpenAICompatibleResponse;

    fn name(&self) -> &str {
        "zhipu"
    }

    fn chat_endpoint(&self, base_url: &str, _model: &str) -> String {
        format!("{}/api/paas/v4/chat/completions", base_url)
    }

    fn auth_headers(&self) -> Vec<(String, String)> {
        vec![(
            "Authorization".to_string(),
            format!("Bearer {}", self.api_key),
        )]
    }

    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        let messages: Vec<ZhipuMessage> = request
            .messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "tool".to_string(),
                };

                let has_image = msg.content.iter().any(|block| block.is_image());

                let content = if has_image {
                    let blocks: Vec<Value> = msg.content.iter().map(|block| {
                        match block {
                            MessageBlock::Text { text } => json!({
                                "type": "text",
                                "text": text
                            }),
                            MessageBlock::Image { source } => json!({
                                "type": "image_url",
                                "image_url": {
                                    "url": match source {
                                        ImageSource::Base64 { media_type, data } => format!("data:{};base64,{}", media_type, data),
                                        ImageSource::Url { url } => url.clone(),
                                    }
                                }
                            }),
                            MessageBlock::ImageUrl { image_url } => json!({
                                "type": "image_url",
                                "image_url": { "url": image_url.url }
                            }),
                            MessageBlock::Document { .. } | MessageBlock::DocumentUrl { .. } => json!({
                                "type": "text",
                                "text": "[Document]"
                            }),
                        }
                    }).collect();
                    json!(blocks)
                } else {
                    json!(msg.content_as_text())
                };

                ZhipuMessage {
                    role,
                    content,
                    tool_calls: msg.tool_calls.as_ref().map(|calls| {
                        calls.iter().map(|c| serde_json::to_value(c).unwrap_or_default()).collect()
                    }),
                    tool_call_id: msg.tool_call_id.clone(),
                    name: msg.name.clone(),
                }
            })
            .collect();

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
        crate::protocols::utils::parse_openai_compatible_chat_response(response, self.name())
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
    pub messages: Vec<ZhipuMessage>,
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
