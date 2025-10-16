//! 智谱GLM协议实现 - V2架构
//!
//! 这个模块实现了智谱GLM API协议规范，支持OpenAI兼容模式。

use crate::core::Protocol;
use crate::types::{ChatRequest, ChatResponse, Message, Role, Choice, Usage};
use crate::error::LlmConnectorError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// 智谱GLM协议实现
#[derive(Clone, Debug)]
pub struct ZhipuProtocol {
    api_key: String,
    use_openai_format: bool,
}

impl ZhipuProtocol {
    /// 创建新的智谱协议实例 (使用原生格式)
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            use_openai_format: false,
        }
    }
    
    /// 创建使用OpenAI兼容格式的智谱协议实例
    pub fn new_openai_compatible(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            use_openai_format: true,
        }
    }
    
    /// 获取API密钥
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
    
    /// 是否使用OpenAI兼容格式
    pub fn is_openai_compatible(&self) -> bool {
        self.use_openai_format
    }
}

#[async_trait]
impl Protocol for ZhipuProtocol {
    type Request = ZhipuRequest;
    type Response = ZhipuResponse;
    
    fn name(&self) -> &str {
        "zhipu"
    }
    
    fn chat_endpoint(&self, base_url: &str) -> String {
        if self.use_openai_format {
            format!("{}/v1/chat/completions", base_url.trim_end_matches('/'))
        } else {
            format!("{}/api/paas/v4/chat/completions", base_url.trim_end_matches('/'))
        }
    }
    
    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        let messages = request.messages.iter()
            .map(|msg| ZhipuMessage {
                role: match msg.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::System => "system".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect();

        Ok(ZhipuRequest {
            model: request.model.clone(),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            top_p: request.top_p,
            stream: request.stream,
            // 智谱特有参数
            do_sample: Some(true),
            request_id: None,
        })
    }
    
    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        let zhipu_response: ZhipuResponse = serde_json::from_str(response)
            .map_err(|e| LlmConnectorError::ParseError(format!("Failed to parse Zhipu response: {}", e)))?;

        if zhipu_response.choices.is_empty() {
            return Err(LlmConnectorError::ParseError("No choices in response".to_string()));
        }

        let choices: Vec<Choice> = zhipu_response.choices.into_iter()
            .map(|choice| Choice {
                index: choice.index,
                message: Message {
                    role: Role::Assistant,
                    content: choice.message.content,
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    reasoning_content: None,
                    reasoning: None,
                    thought: None,
                    thinking: None,
                },
                finish_reason: choice.finish_reason,
                logprobs: None,
            })
            .collect();

        let usage = zhipu_response.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
            completion_tokens_details: None,
            prompt_cache_hit_tokens: None,
            prompt_cache_miss_tokens: None,
            prompt_tokens_details: None,
        });

        // 提取第一个选择的内容作为便利字段
        let content = choices.first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();

        Ok(ChatResponse {
            id: zhipu_response.id,
            object: zhipu_response.object,
            created: zhipu_response.created,
            model: zhipu_response.model,
            choices,
            content,
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
            .unwrap_or("Unknown Zhipu error");

        match status {
            400 => LlmConnectorError::InvalidRequest(format!("Zhipu: {}", message)),
            401 => LlmConnectorError::AuthenticationError(format!("Zhipu: {}", message)),
            403 => LlmConnectorError::PermissionError(format!("Zhipu: {}", message)),
            429 => LlmConnectorError::RateLimitError(format!("Zhipu: {}", message)),
            500..=599 => LlmConnectorError::ServerError(format!("Zhipu: {}", message)),
            _ => LlmConnectorError::ApiError(format!("Zhipu HTTP {}: {}", status, message)),
        }
    }
    
    fn auth_headers(&self) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", self.api_key)),
            ("Content-Type".to_string(), "application/json".to_string()),
        ]
    }
    
    #[cfg(feature = "streaming")]
    async fn parse_stream_response(&self, response: reqwest::Response) -> Result<ChatStream, LlmConnectorError> {
        // 智谱使用标准SSE格式
        Ok(crate::sse::sse_to_streaming_response(response))
    }
}

// 智谱请求类型
#[derive(Serialize, Debug)]
pub struct ZhipuRequest {
    pub model: String,
    pub messages: Vec<ZhipuMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub do_sample: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ZhipuMessage {
    pub role: String,
    pub content: String,
}

// 智谱响应类型
#[derive(Deserialize, Debug)]
pub struct ZhipuResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ZhipuChoice>,
    pub usage: Option<ZhipuUsage>,
}

#[derive(Deserialize, Debug)]
pub struct ZhipuChoice {
    pub index: u32,
    pub message: ZhipuResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ZhipuResponseMessage {
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct ZhipuUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
