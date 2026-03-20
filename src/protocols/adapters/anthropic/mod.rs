//! Anthropic Claude Protocol Implementation - V2 Architecture
//!
//! This module implements the Anthropic Claude API protocol specification.

use crate::core::Protocol;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, Choice, Message, Role, Usage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Anthropic Claudeprotocolimplementation
#[derive(Clone, Debug)]
pub struct AnthropicProtocol {
    api_key: String,
}

impl AnthropicProtocol {
    /// Create new Anthropic Protocol instance
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
impl Protocol for AnthropicProtocol {
    type Request = AnthropicRequest;
    type Response = AnthropicResponse;

    fn name(&self) -> &str {
        "anthropic"
    }

    fn chat_endpoint(&self, base_url: &str, _model: &str) -> String {
        let base = base_url.trim_end_matches('/');
        if base.ends_with("/v1") {
            format!("{}/messages", base)
        } else {
            format!("{}/v1/messages", base)
        }
    }

    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        // Anthropic API requires separating system messages
        let mut system_message = None;
        let mut messages = Vec::new();

        for msg in &request.messages {
            match msg.role {
                Role::System => {
                    // Anthropic only supports one system message, placed in a separate field
                    let text = msg.content_as_text();
                    if system_message.is_none() {
                        system_message = Some(text);
                    } else {
                        // If there are multiple system messages, merge them
                        let existing = system_message.take().unwrap_or_default();
                        system_message = Some(format!("{}\n\n{}", existing, text));
                    }
                }
                Role::User => {
                    // Anthropic always uses array format
                    let content = serde_json::to_value(&msg.content).unwrap();
                    messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content,
                    });
                }
                Role::Assistant => {
                    // Anthropic always uses array format
                    let content = serde_json::to_value(&msg.content).unwrap();
                    messages.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content,
                    });
                }
                Role::Tool => {
                    // Anthropic does not support tool role yet, convert to user
                    let text = format!("Tool result: {}", msg.content_as_text());
                    messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: serde_json::json!([{"type": "text", "text": text}]),
                    });
                }
            }
        }

        // Handle thinking/budget
        let mut max_tokens = request.max_tokens.unwrap_or(1024);
        let thinking = if let Some(budget) = request.thinking_budget {
            // If budget is set, enable thinking
            // Ensure max_tokens is larger than budget (Anthropic requirement)
            if max_tokens <= budget {
                // If user didn't set enough max_tokens, bump it
                max_tokens = budget + 4096;
            }
            Some(AnthropicThinking {
                thinking_type: "enabled".to_string(),
                budget_tokens: budget,
            })
        } else {
            None
        };

        Ok(AnthropicRequest {
            model: request.model.clone(),
            max_tokens,
            messages,
            system: system_message,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: request.stream,
            thinking,
        })
    }

    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        let anthropic_response: AnthropicResponse =
            serde_json::from_str(response).map_err(|e| {
                LlmConnectorError::ParseError(format!("Failed to parse Anthropic response: {}", e))
            })?;

        // Extract content blocks and thinking
        let mut message_blocks = Vec::new();
        let mut thinking_content = String::new();
        let mut text_content = String::new();

        for block in &anthropic_response.content {
            match block.content_type.as_str() {
                "text" => {
                    if let Some(text) = &block.text {
                        message_blocks.push(crate::types::MessageBlock::text(text));
                        text_content.push_str(text);
                    }
                }
                "thinking" => {
                    if let Some(thinking) = &block.thinking {
                        thinking_content.push_str(thinking);
                    }
                }
                _ => {
                    // Ignore unknown blocks
                }
            }
        }

        let thinking = if !thinking_content.is_empty() {
            Some(thinking_content)
        } else {
            None
        };

        let choices = vec![Choice {
            index: 0,
            message: Message {
                role: Role::Assistant,
                content: message_blocks,
                name: None,
                tool_calls: None,
                tool_call_id: None,
                reasoning_content: None,
                reasoning: None,
                thought: None,
                thinking,
            },
            finish_reason: Some(
                anthropic_response
                    .stop_reason
                    .unwrap_or_else(|| "stop".to_string()),
            ),
            logprobs: None,
        }];

        let usage = Some(Usage {
            prompt_tokens: anthropic_response.usage.input_tokens,
            completion_tokens: anthropic_response.usage.output_tokens,
            total_tokens: anthropic_response.usage.input_tokens
                + anthropic_response.usage.output_tokens,
            completion_tokens_details: None,
            prompt_cache_hit_tokens: anthropic_response.usage.cache_read_input_tokens,
            prompt_cache_miss_tokens: None,
            prompt_tokens_details: Some(crate::types::PromptTokensDetails {
                cached_tokens: anthropic_response.usage.cache_read_input_tokens,
                cache_read_input_tokens: anthropic_response.usage.cache_read_input_tokens,
                cache_creation_input_tokens: anthropic_response.usage.cache_creation_input_tokens,
            }),
        });

        Ok(ChatResponse {
            id: anthropic_response.id,
            object: "chat.completion".to_string(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            model: anthropic_response.model,
            choices,
            content: text_content,
            reasoning_content: None,
            usage,
            system_fingerprint: None,
        })
    }

    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError {
        let error_info = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("error").cloned())
            .unwrap_or_else(|| serde_json::json!({"message": body}));

        let message = error_info
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown Anthropic error");

        let error_type = error_info
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("");

        let msg = format!("Anthropic: {}", message);

        // Detect context length exceeded
        if error_type == "invalid_request_error"
            && (message.contains("too long")
                || message.contains("maximum")
                || message.contains("context"))
        {
            return LlmConnectorError::ContextLengthExceeded(msg);
        }

        match status {
            400 => LlmConnectorError::InvalidRequest(msg),
            401 => LlmConnectorError::AuthenticationError(msg),
            403 => LlmConnectorError::PermissionError(msg),
            429 => LlmConnectorError::RateLimitError(msg),
            500..=599 => LlmConnectorError::ServerError(msg),
            _ => LlmConnectorError::ApiError(format!("Anthropic HTTP {}: {}", status, message)),
        }
    }

    fn auth_headers(&self) -> Vec<(String, String)> {
        let mut headers =
            crate::protocols::common::auth::api_key_header(&self.api_key, "x-api-key");
        headers.push(("anthropic-version".to_string(), "2023-06-01".to_string()));
        headers
    }

    fn build_auth_headers_for_override(&self, api_key: &str) -> Vec<(String, String)> {
        crate::protocols::common::auth::api_key_header(api_key, "x-api-key")
    }

    /// Parse Anthropic streamingresponse
    ///
    /// Anthropic uses different streaming format:
    /// - message_start: Contains message object (with id)
    /// - content_block_start: Start content chunk
    /// - content_block_delta: contentdelta（Contains text）
    /// - content_block_stop: End content chunk
    /// - message_delta: Message delta (contains usage)
    /// - message_stop: Message end
    #[cfg(feature = "streaming")]
    async fn parse_stream_response(
        &self,
        response: reqwest::Response,
    ) -> Result<crate::types::ChatStream, LlmConnectorError> {
        use std::sync::{Arc, Mutex};

        let message_id = Arc::new(Mutex::new(String::new()));

        Ok(crate::protocols::common::streamers::map_sse_json_stream(
            response,
            move |json_str| {
                let message_id = message_id.clone();
                let event = serde_json::from_str::<serde_json::Value>(&json_str).map_err(|e| {
                    LlmConnectorError::ParseError(format!(
                        "Failed to parse Anthropic streaming event: {}. JSON: {}",
                        e, json_str
                    ))
                })?;

                let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");

                match event_type {
                    "message_start" => {
                        if let Some(msg_id) = event
                            .get("message")
                            .and_then(|m| m.get("id"))
                            .and_then(|id| id.as_str())
                            && let Ok(mut id) = message_id.lock()
                        {
                            *id = msg_id.to_string();
                        }
                        Ok(None)
                    }
                    "content_block_delta" | "message_delta" => {
                        let id = message_id.lock().map(|id| id.clone()).unwrap_or_default();
                        crate::protocols::common::streamers::interpret_anthropic_event(&event, &id)
                    }
                    _ => Ok(None),
                }
            },
        ))
    }
}

// Anthropicrequesttype
#[derive(Serialize, Deserialize, Debug)]
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
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<AnthropicThinking>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnthropicThinking {
    #[serde(rename = "type")]
    pub thinking_type: String,
    pub budget_tokens: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: serde_json::Value, // Support String or Array
}

// Anthropicresponsetype
#[derive(Deserialize, Debug)]
pub struct AnthropicResponse {
    pub id: String,
    pub model: String,
    pub content: Vec<AnthropicContent>,
    pub stop_reason: Option<String>,
    pub usage: AnthropicUsage,
}

#[derive(Deserialize, Debug)]
pub struct AnthropicContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
    pub thinking: Option<String>,
    pub signature: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_creation_input_tokens: Option<u32>,
    pub cache_read_input_tokens: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_parsing_with_thinking_and_text() {
        let response_json = r#"
        {
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "model": "claude-3-5-sonnet-20241022",
            "content": [
                {
                    "type": "thinking",
                    "thinking": "This is a thinking block",
                    "signature": "sig_123"
                },
                {
                    "type": "text",
                    "text": "Hello there, nice to meet."
                }
            ],
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {
                "input_tokens": 10,
                "output_tokens": 20
            }
        }
        "#;

        let protocol = AnthropicProtocol::new("test-key");
        let result = protocol.parse_response(response_json).unwrap();

        assert_eq!(result.content, "Hello there, nice to meet.");
        assert_eq!(
            result.choices[0].message.content[0].as_text().unwrap(),
            "Hello there, nice to meet."
        );
        assert_eq!(
            result.choices[0].message.thinking.as_deref(),
            Some("This is a thinking block")
        );
    }

    #[test]
    fn test_anthropic_parsing_only_thinking() {
        let response_json = r#"
        {
            "id": "msg_124",
            "type": "message",
            "role": "assistant",
            "model": "claude-3-5-sonnet-20241022",
            "content": [
                {
                    "type": "thinking",
                    "thinking": "Just thinking...",
                    "signature": "sig_124"
                }
            ],
            "stop_reason": "max_tokens",
            "stop_sequence": null,
            "usage": {
                "input_tokens": 10,
                "output_tokens": 20
            }
        }
        "#;

        let protocol = AnthropicProtocol::new("test-key");
        let result = protocol.parse_response(response_json).unwrap();

        assert_eq!(result.content, "");
        assert!(result.choices[0].message.content.is_empty());
        assert_eq!(
            result.choices[0].message.thinking.as_deref(),
            Some("Just thinking...")
        );
    }
}
