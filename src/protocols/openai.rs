//! OpenAI Protocol Implementation - V2 Architecture
//!
//! This module implements the standard OpenAI API protocol specification.

use crate::core::Protocol;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, Choice, Message, Role, Usage};
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
    type Response = OpenAIResponse;

    fn name(&self) -> &str {
        "openai"
    }

    fn chat_endpoint(&self, base_url: &str) -> String {
        format!("{}/chat/completions", base_url.trim_end_matches('/'))
    }

    fn models_endpoint(&self, base_url: &str) -> Option<String> {
        Some(format!("{}/models", base_url.trim_end_matches('/')))
    }

    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        let messages = request
            .messages
            .iter()
            .map(|msg| {
                // Convert MessageBlock to OpenAI format
                let content = if msg.content.len() == 1 && msg.content[0].is_text() {
                    // Plain text: use string format
                    serde_json::json!(msg.content[0].as_text().unwrap())
                } else {
                    // Multi-modal: use array format
                    serde_json::to_value(&msg.content).unwrap()
                };

                OpenAIMessage {
                    role: match msg.role {
                        Role::User => "user".to_string(),
                        Role::Assistant => "assistant".to_string(),
                        Role::System => "system".to_string(),
                        Role::Tool => "tool".to_string(),
                    },
                    content,
                    tool_calls: msg.tool_calls.as_ref().map(|calls| {
                        calls
                            .iter()
                            .map(|call| {
                                serde_json::json!({
                                    "id": call.id,
                                    "type": call.call_type,
                                    "function": {
                                        "name": call.function.name,
                                        "arguments": call.function.arguments,
                                    }
                                })
                            })
                            .collect()
                    }),
                    tool_call_id: msg.tool_call_id.clone(),
                    name: msg.name.clone(),
                }
            })
            .collect();

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
        })
    }

    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        let openai_response: OpenAIResponse = serde_json::from_str(response).map_err(|e| {
            LlmConnectorError::ParseError(format!("Failed to parse OpenAI response: {}", e))
        })?;

        if openai_response.choices.is_empty() {
            return Err(LlmConnectorError::ParseError(
                "No choices in response".to_string(),
            ));
        }

        let choices: Vec<Choice> = openai_response
            .choices
            .into_iter()
            .map(|choice| {
                // Convert tool_calls
                let tool_calls = choice.message.tool_calls.as_ref().map(|calls| {
                    calls
                        .iter()
                        .filter_map(|call| {
                            Some(crate::types::ToolCall {
                                id: call.get("id")?.as_str()?.to_string(),
                                call_type: call.get("type")?.as_str()?.to_string(),
                                function: crate::types::FunctionCall {
                                    name: call.get("function")?.get("name")?.as_str()?.to_string(),
                                    arguments: call
                                        .get("function")?
                                        .get("arguments")?
                                        .as_str()?
                                        .to_string(),
                                },
                                index: None, // Non-streaming responses don't have index
                            })
                        })
                        .collect()
                });

                // Convert content to MessageBlock
                let content = if let Some(content_value) = &choice.message.content {
                    if let Some(text) = content_value.as_str() {
                        // Plain text
                        vec![crate::types::MessageBlock::text(text)]
                    } else if let Some(array) = content_value.as_array() {
                        // Multi-modal array
                        serde_json::from_value(serde_json::Value::Array(array.clone()))
                            .unwrap_or_else(|_| vec![])
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };

                Choice {
                    index: choice.index,
                    message: Message {
                        role: Role::Assistant,
                        content,
                        name: None,
                        tool_calls,
                        tool_call_id: None,
                        reasoning_content: choice.message.reasoning_content.clone(),
                        reasoning: None,
                        thought: None,
                        thinking: None,
                    },
                    finish_reason: choice.finish_reason,
                    logprobs: None,
                }
            })
            .collect();

        let usage = openai_response.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
            completion_tokens_details: None,
            prompt_cache_hit_tokens: None,
            prompt_cache_miss_tokens: None,
            prompt_tokens_details: None,
        });

        // Extract first choice content as convenience field (plain text)
        let content = choices
            .first()
            .map(|choice| choice.message.content_as_text())
            .unwrap_or_default();

        // Extract first choice reasoning_content
        let reasoning_content = choices
            .first()
            .and_then(|c| c.message.reasoning_content.clone());

        Ok(ChatResponse {
            id: openai_response.id,
            object: openai_response.object,
            created: openai_response.created,
            model: openai_response.model,
            choices,
            content,
            reasoning_content,
            usage,
            system_fingerprint: openai_response.system_fingerprint,
        })
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
        vec![
            (
                "Authorization".to_string(),
                format!("Bearer {}", self.api_key),
            ),
            ("Content-Type".to_string(), "application/json".to_string()),
        ]
    }
}

// OpenAIrequesttype
#[derive(Serialize, Debug)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
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
}

#[derive(Serialize, Debug)]
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

// OpenAIresponsetype
#[derive(Deserialize, Debug)]
pub struct OpenAIResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    pub usage: Option<OpenAIUsage>,
    pub system_fingerprint: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIChoice {
    pub index: u32,
    pub message: OpenAIResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIResponseMessage {
    pub content: Option<serde_json::Value>, // Support String or Array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// modellistresponse
#[derive(Deserialize, Debug)]
pub struct OpenAIModelsResponse {
    pub data: Vec<OpenAIModel>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIModel {
    pub id: String,
}
