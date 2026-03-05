//! OpenAI Protocol Implementation - V2 Architecture
//!
//! This module implements the standard OpenAI API protocol specification.

use crate::core::Protocol;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, EmbedRequest, EmbedResponse, ReasoningEffort};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// OpenAIprotocolimplementation
#[derive(Clone, Debug)]
pub struct OpenAIProtocol {
    api_key: String,
}

impl OpenAIProtocol {
    /// Create new OpenAI Protocol instance
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }

    /// GetAPI key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

#[async_trait]
impl Protocol for OpenAIProtocol {
    type Request = OpenAIRequest;
    type Response = crate::protocols::formats::chat_completions::ChatCompletionsResponse;

    fn name(&self) -> &str {
        "openai"
    }

    fn chat_endpoint(&self, base_url: &str, _model: &str) -> String {
        format!("{}/chat/completions", base_url.trim_end_matches('/'))
    }

    fn models_endpoint(&self, base_url: &str) -> Option<String> {
        Some(format!("{}/models", base_url.trim_end_matches('/')))
    }

    fn embed_endpoint(&self, base_url: &str, _model: &str) -> Option<String> {
        Some(format!("{}/embeddings", base_url.trim_end_matches('/')))
    }

    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        let messages =
            crate::protocols::common::request::openai_message_converter(&request.messages);

        // Convert tools
        let tools = request.tools.as_ref().map(|tools| {
            tools
                .iter()
                .map(|tool| {
                    serde_json::json!({
                        "type": tool.tool_type,
                        "function": {
                            "name": tool.function.name,
                            "description": tool.function.description,
                            "parameters": tool.function.parameters,
                        }
                    })
                })
                .collect()
        });

        // Convert tool_choice
        let tool_choice = request
            .tool_choice
            .as_ref()
            .map(|choice| serde_json::to_value(choice).unwrap_or(serde_json::json!("auto")));

        // Convert response_format
        let response_format = request
            .response_format
            .as_ref()
            .map(|rf| serde_json::to_value(rf).unwrap_or(serde_json::json!({"type": "text"})));

        Ok(OpenAIRequest {
            model: request.model.clone(),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
            stream: request.stream,
            tools,
            tool_choice,
            response_format,
            reasoning_effort: request.reasoning_effort,
        })
    }

    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        crate::protocols::formats::chat_completions::parse_chat_completions_chat_response(
            response,
            self.name(),
        )
    }

    fn parse_models(&self, response: &str) -> Result<Vec<String>, LlmConnectorError> {
        let models_response: OpenAIModelsResponse =
            serde_json::from_str(response).map_err(|e| {
                LlmConnectorError::ParseError(format!("Failed to parse models response: {}", e))
            })?;

        Ok(models_response
            .data
            .into_iter()
            .map(|model| model.id)
            .collect())
    }

    fn build_embed_request(
        &self,
        request: &EmbedRequest,
    ) -> Result<serde_json::Value, LlmConnectorError> {
        let req = OpenAIEmbedRequest {
            model: request.model.clone(),
            input: request.input.clone(),
            encoding_format: request.encoding_format.clone(),
            user: request.user.clone(),
        };
        serde_json::to_value(req).map_err(|e| {
            LlmConnectorError::ParseError(format!("Failed to serialize embed request: {}", e))
        })
    }

    fn parse_embed_response(&self, response: &str) -> Result<EmbedResponse, LlmConnectorError> {
        crate::protocols::formats::chat_completions::parse_chat_completions_embed_response(
            response,
            self.name(),
        )
    }

    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError {
        let error_info = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("error").cloned())
            .unwrap_or_else(|| serde_json::json!({"message": body}));

        let message = error_info
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown OpenAI error");

        // Check error code for context length exceeded
        let error_code = error_info
            .get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("");

        let msg = format!("OpenAI: {}", message);

        // Detect context length exceeded from error code or message content
        if error_code == "context_length_exceeded"
            || message.contains("maximum context length")
            || message.contains("context_length_exceeded")
        {
            return LlmConnectorError::ContextLengthExceeded(msg);
        }

        match status {
            400 => LlmConnectorError::InvalidRequest(msg),
            401 => LlmConnectorError::AuthenticationError(msg),
            403 => LlmConnectorError::PermissionError(msg),
            429 => LlmConnectorError::RateLimitError(msg),
            500..=599 => LlmConnectorError::ServerError(msg),
            _ => LlmConnectorError::ApiError(format!("OpenAI HTTP {}: {}", status, message)),
        }
    }

    fn auth_headers(&self) -> Vec<(String, String)> {
        crate::protocols::common::auth::bearer_auth(&self.api_key)
    }
    #[cfg(feature = "streaming")]
    async fn parse_stream_response(
        &self,
        response: reqwest::Response,
    ) -> Result<crate::types::ChatStream, LlmConnectorError> {
        Ok(crate::sse::sse_to_streaming_response_with_mode(
            response,
            crate::sse::StreamingParseMode::OpenAIOnly,
        ))
    }
}

// OpenAIrequesttype
#[derive(Serialize, Deserialize, Debug)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: serde_json::Value, // Support String or Array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// Model list response
#[derive(Deserialize, Debug)]
pub struct OpenAIModelsResponse {
    pub data: Vec<OpenAIModel>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIModel {
    pub id: String,
}

// Embedding Data Structures
#[derive(Serialize, Deserialize, Debug)]
pub struct OpenAIEmbedRequest {
    pub model: String,
    pub input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}
