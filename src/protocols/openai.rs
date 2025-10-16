//! OpenAI协议实现 - V2架构
//!
//! 这个模块实现了标准的OpenAI API协议规范。

use crate::core::Protocol;
use crate::types::{ChatRequest, ChatResponse, Message, Role, Choice, Usage};
use crate::error::LlmConnectorError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// OpenAI协议实现
#[derive(Clone, Debug)]
pub struct OpenAIProtocol {
    api_key: String,
}

impl OpenAIProtocol {
    /// 创建新的OpenAI协议实例
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
impl Protocol for OpenAIProtocol {
    type Request = OpenAIRequest;
    type Response = OpenAIResponse;
    
    fn name(&self) -> &str {
        "openai"
    }
    
    fn chat_endpoint(&self, base_url: &str) -> String {
        format!("{}/v1/chat/completions", base_url.trim_end_matches('/'))
    }
    
    fn models_endpoint(&self, base_url: &str) -> Option<String> {
        Some(format!("{}/v1/models", base_url.trim_end_matches('/')))
    }
    
    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        let messages = request.messages.iter()
            .map(|msg| OpenAIMessage {
                role: match msg.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::System => "system".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect();

        Ok(OpenAIRequest {
            model: request.model.clone(),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
            stream: request.stream,
        })
    }
    
    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        let openai_response: OpenAIResponse = serde_json::from_str(response)
            .map_err(|e| LlmConnectorError::ParseError(format!("Failed to parse OpenAI response: {}", e)))?;

        if openai_response.choices.is_empty() {
            return Err(LlmConnectorError::ParseError("No choices in response".to_string()));
        }

        let choices: Vec<Choice> = openai_response.choices.into_iter()
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

        let usage = openai_response.usage.map(|u| Usage {
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
            id: openai_response.id,
            object: openai_response.object,
            created: openai_response.created,
            model: openai_response.model,
            choices,
            content,
            usage,
            system_fingerprint: openai_response.system_fingerprint,
        })
    }
    
    fn parse_models(&self, response: &str) -> Result<Vec<String>, LlmConnectorError> {
        let models_response: OpenAIModelsResponse = serde_json::from_str(response)
            .map_err(|e| LlmConnectorError::ParseError(format!("Failed to parse models response: {}", e)))?;

        Ok(models_response.data.into_iter().map(|model| model.id).collect())
    }
    
    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError {
        let error_info = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("error").cloned())
            .unwrap_or_else(|| serde_json::json!({"message": body}));
            
        let message = error_info.get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown OpenAI error");

        match status {
            400 => LlmConnectorError::InvalidRequest(format!("OpenAI: {}", message)),
            401 => LlmConnectorError::AuthenticationError(format!("OpenAI: {}", message)),
            403 => LlmConnectorError::PermissionError(format!("OpenAI: {}", message)),
            429 => LlmConnectorError::RateLimitError(format!("OpenAI: {}", message)),
            500..=599 => LlmConnectorError::ServerError(format!("OpenAI: {}", message)),
            _ => LlmConnectorError::ApiError(format!("OpenAI HTTP {}: {}", status, message)),
        }
    }
    
    fn auth_headers(&self) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", self.api_key)),
            ("Content-Type".to_string(), "application/json".to_string()),
        ]
    }
}

// OpenAI请求类型
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
}

#[derive(Serialize, Debug)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

// OpenAI响应类型
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
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// 模型列表响应
#[derive(Deserialize, Debug)]
pub struct OpenAIModelsResponse {
    pub data: Vec<OpenAIModel>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIModel {
    pub id: String,
}
