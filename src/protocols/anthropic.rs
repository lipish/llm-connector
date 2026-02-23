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

    fn chat_endpoint(&self, base_url: &str) -> String {
        format!("{}/v1/messages", base_url.trim_end_matches('/'))
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

        Ok(AnthropicRequest {
            model: request.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(1024), // Anthropic requires this to be set
            messages,
            system: system_message,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: request.stream,
        })
    }

    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        let anthropic_response: AnthropicResponse =
            serde_json::from_str(response).map_err(|e| {
                LlmConnectorError::ParseError(format!("Failed to parse Anthropic response: {}", e))
            })?;

        // Anthropic returns single content chunk
        // Convert Anthropic content to MessageBlock
        let message_blocks: Vec<crate::types::MessageBlock> = anthropic_response
            .content
            .iter()
            .map(|c| crate::types::MessageBlock::text(&c.text))
            .collect();

        let content = anthropic_response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

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
                thinking: None,
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
            prompt_cache_hit_tokens: None,
            prompt_cache_miss_tokens: None,
            prompt_tokens_details: None,
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
            content,
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
        vec![
            ("x-api-key".to_string(), self.api_key.clone()),
            ("Content-Type".to_string(), "application/json".to_string()),
            ("anthropic-version".to_string(), "2023-06-01".to_string()),
        ]
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
        use crate::types::{Delta, StreamingChoice, StreamingResponse, Usage};
        use futures_util::StreamExt;
        use std::sync::{Arc, Mutex};

        // Use standard SSE parser
        let events_stream = crate::sse::sse_events(response);

        // Shared state: save message_id
        let message_id = Arc::new(Mutex::new(String::new()));

        // Convert event stream
        let response_stream = events_stream.filter_map(move |result| {
            let message_id = message_id.clone();
            async move {
                match result {
                    Ok(json_str) => {
                        // Parse Anthropic streaming event
                        match serde_json::from_str::<serde_json::Value>(&json_str) {
                            Ok(event) => {
                                let event_type =
                                    event.get("type").and_then(|t| t.as_str()).unwrap_or("");

                                match event_type {
                                    "message_start" => {
                                        // Extract and save message id
                                        if let Some(msg_id) = event
                                            .get("message")
                                            .and_then(|m| m.get("id"))
                                            .and_then(|id| id.as_str())
                                        {
                                            if let Ok(mut id) = message_id.lock() {
                                                *id = msg_id.to_string();
                                            }
                                        }
                                        // message_start does not return content
                                        None
                                    }
                                    "content_block_delta" => {
                                        // Extract text delta
                                        if let Some(text) = event
                                            .get("delta")
                                            .and_then(|d| d.get("text"))
                                            .and_then(|t| t.as_str())
                                        {
                                            let id = message_id
                                                .lock()
                                                .ok()
                                                .map(|id| id.clone())
                                                .unwrap_or_default();

                                            // Construct StreamingResponse
                                            Some(Ok(StreamingResponse {
                                                id,
                                                object: "chat.completion.chunk".to_string(),
                                                created: std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_secs(),
                                                model: "anthropic".to_string(),
                                                choices: vec![StreamingChoice {
                                                    index: 0,
                                                    delta: Delta {
                                                        role: Some(crate::types::Role::Assistant),
                                                        content: Some(text.to_string()),
                                                        tool_calls: None,
                                                        reasoning_content: None,
                                                        reasoning: None,
                                                        thought: None,
                                                        thinking: None,
                                                    },
                                                    finish_reason: None,
                                                    logprobs: None,
                                                }],
                                                content: text.to_string(),
                                                reasoning_content: None,
                                                usage: None,
                                                system_fingerprint: None,
                                            }))
                                        } else {
                                            None
                                        }
                                    }
                                    "message_delta" => {
                                        // Extract usage and stop_reason
                                        let stop_reason = event
                                            .get("delta")
                                            .and_then(|d| d.get("stop_reason"))
                                            .and_then(|s| s.as_str())
                                            .map(|s| s.to_string());

                                        let usage = event.get("usage").and_then(|u| {
                                            let input_tokens = u
                                                .get("input_tokens")
                                                .and_then(|t| t.as_u64())
                                                .unwrap_or(0)
                                                as u32;
                                            let output_tokens = u
                                                .get("output_tokens")
                                                .and_then(|t| t.as_u64())
                                                .unwrap_or(0)
                                                as u32;
                                            Some(Usage {
                                                prompt_tokens: input_tokens,
                                                completion_tokens: output_tokens,
                                                total_tokens: input_tokens + output_tokens,
                                                completion_tokens_details: None,
                                                prompt_cache_hit_tokens: None,
                                                prompt_cache_miss_tokens: None,
                                                prompt_tokens_details: None,
                                            })
                                        });

                                        let id = message_id
                                            .lock()
                                            .ok()
                                            .map(|id| id.clone())
                                            .unwrap_or_default();

                                        // Return final response (contains finish_reason and usage)
                                        Some(Ok(StreamingResponse {
                                            id,
                                            object: "chat.completion.chunk".to_string(),
                                            created: std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_secs(),
                                            model: "anthropic".to_string(),
                                            choices: vec![StreamingChoice {
                                                index: 0,
                                                delta: Delta {
                                                    role: None,
                                                    content: None,
                                                    tool_calls: None,
                                                    reasoning_content: None,
                                                    reasoning: None,
                                                    thought: None,
                                                    thinking: None,
                                                },
                                                finish_reason: stop_reason,
                                                logprobs: None,
                                            }],
                                            content: String::new(),
                                            reasoning_content: None,
                                            usage,
                                            system_fingerprint: None,
                                        }))
                                    }
                                    _ => {
                                        // Ignore other event types
                                        None
                                    }
                                }
                            }
                            Err(e) => Some(Err(LlmConnectorError::ParseError(format!(
                                "Failed to parse Anthropic streaming event: {}. JSON: {}",
                                e, json_str
                            )))),
                        }
                    }
                    Err(e) => Some(Err(e)),
                }
            }
        });

        Ok(Box::pin(response_stream))
    }
}

// Anthropicrequesttype
#[derive(Serialize, Debug)]
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
}

#[derive(Serialize, Debug)]
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
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}
