//! Anthropic Claude协议实现 - V2架构
//!
//! 这个模块实现了Anthropic Claude API协议规范。

use crate::core::Protocol;
use crate::types::{ChatRequest, ChatResponse, Message, Role, Choice, Usage};
use crate::error::LlmConnectorError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Anthropic Claude协议实现
#[derive(Clone, Debug)]
pub struct AnthropicProtocol {
    api_key: String,
}

impl AnthropicProtocol {
    /// 创建新的Anthropic协议实例
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
    
    /// 获取API密钥
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
        // Anthropic API 需要分离 system 消息
        let mut system_message = None;
        let mut messages = Vec::new();
        
        for msg in &request.messages {
            match msg.role {
                Role::System => {
                    // Anthropic 只支持一个 system 消息，放在单独的字段中
                    if system_message.is_none() {
                        system_message = Some(msg.content.clone());
                    } else {
                        // 如果有多个 system 消息，合并它们
                        let existing = system_message.take().unwrap_or_default();
                        system_message = Some(format!("{}\n\n{}", existing, msg.content));
                    }
                }
                Role::User => {
                    messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: msg.content.clone(),
                    });
                }
                Role::Assistant => {
                    messages.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: msg.content.clone(),
                    });
                }
                Role::Tool => {
                    // Anthropic 暂不支持 tool 角色，转换为 user
                    messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: format!("Tool result: {}", msg.content),
                    });
                }
            }
        }

        Ok(AnthropicRequest {
            model: request.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(1024), // Anthropic 要求必须设置
            messages,
            system: system_message,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: request.stream,
        })
    }
    
    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        let anthropic_response: AnthropicResponse = serde_json::from_str(response)
            .map_err(|e| LlmConnectorError::ParseError(format!("Failed to parse Anthropic response: {}", e)))?;

        // Anthropic 返回单个内容块
        let content = anthropic_response.content.first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        let choices = vec![Choice {
            index: 0,
            message: Message {
                role: Role::Assistant,
                content: content.clone(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                reasoning_content: None,
                reasoning: None,
                thought: None,
                thinking: None,
            },
            finish_reason: Some(anthropic_response.stop_reason.unwrap_or_else(|| "stop".to_string())),
            logprobs: None,
        }];

        let usage = Some(Usage {
            prompt_tokens: anthropic_response.usage.input_tokens,
            completion_tokens: anthropic_response.usage.output_tokens,
            total_tokens: anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens,
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
            
        let message = error_info.get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown Anthropic error");

        match status {
            400 => LlmConnectorError::InvalidRequest(format!("Anthropic: {}", message)),
            401 => LlmConnectorError::AuthenticationError(format!("Anthropic: {}", message)),
            403 => LlmConnectorError::PermissionError(format!("Anthropic: {}", message)),
            429 => LlmConnectorError::RateLimitError(format!("Anthropic: {}", message)),
            500..=599 => LlmConnectorError::ServerError(format!("Anthropic: {}", message)),
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

    /// 解析 Anthropic 流式响应
    ///
    /// Anthropic 使用不同的流式格式：
    /// - message_start: 包含 message 对象（有 id）
    /// - content_block_start: 开始内容块
    /// - content_block_delta: 内容增量（包含 text）
    /// - content_block_stop: 结束内容块
    /// - message_delta: 消息增量（包含 usage）
    /// - message_stop: 消息结束
    #[cfg(feature = "streaming")]
    async fn parse_stream_response(&self, response: reqwest::Response) -> Result<crate::types::ChatStream, LlmConnectorError> {
        use crate::types::{StreamingResponse, StreamingChoice, Delta, Usage};
        use futures_util::StreamExt;
        use std::sync::{Arc, Mutex};

        // 使用标准 SSE 解析器
        let events_stream = crate::sse::sse_events(response);

        // 共享状态：保存 message_id
        let message_id = Arc::new(Mutex::new(String::new()));

        // 转换事件流
        let response_stream = events_stream.filter_map(move |result| {
            let message_id = message_id.clone();
            async move {
                match result {
                    Ok(json_str) => {
                        // 解析 Anthropic 流式事件
                        match serde_json::from_str::<serde_json::Value>(&json_str) {
                            Ok(event) => {
                                let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");

                                match event_type {
                                    "message_start" => {
                                        // 提取并保存 message id
                                        if let Some(msg_id) = event.get("message")
                                            .and_then(|m| m.get("id"))
                                            .and_then(|id| id.as_str()) {
                                            if let Ok(mut id) = message_id.lock() {
                                                *id = msg_id.to_string();
                                            }
                                        }
                                        // message_start 不返回内容
                                        None
                                    }
                                    "content_block_delta" => {
                                        // 提取文本增量
                                        if let Some(text) = event.get("delta")
                                            .and_then(|d| d.get("text"))
                                            .and_then(|t| t.as_str()) {

                                            let id = message_id.lock().ok()
                                                .map(|id| id.clone())
                                                .unwrap_or_default();

                                            // 构造 StreamingResponse
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
                                        // 提取 usage 和 stop_reason
                                        let stop_reason = event.get("delta")
                                            .and_then(|d| d.get("stop_reason"))
                                            .and_then(|s| s.as_str())
                                            .map(|s| s.to_string());

                                        let usage = event.get("usage").and_then(|u| {
                                            let input_tokens = u.get("input_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
                                            let output_tokens = u.get("output_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
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

                                        let id = message_id.lock().ok()
                                            .map(|id| id.clone())
                                            .unwrap_or_default();

                                        // 返回最终的响应（包含 finish_reason 和 usage）
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
                                        // 忽略其他事件类型
                                        None
                                    }
                                }
                            }
                            Err(e) => {
                                Some(Err(LlmConnectorError::ParseError(format!(
                                    "Failed to parse Anthropic streaming event: {}. JSON: {}",
                                    e, json_str
                                ))))
                            }
                        }
                    }
                    Err(e) => Some(Err(e)),
                }
            }
        });

        Ok(Box::pin(response_stream))
    }
}

// Anthropic请求类型
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
    pub content: String,
}

// Anthropic响应类型
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
