//! 阿里云DashScope协议实现 - V2架构
//!
//! 这个模块实现了阿里云DashScope API协议规范。

use crate::core::Protocol;
use crate::types::{ChatRequest, ChatResponse, Message, Role, Choice, Usage};
use crate::error::LlmConnectorError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 阿里云DashScope协议实现
#[derive(Clone, Debug)]
pub struct AliyunProtocol {
    api_key: String,
}

impl AliyunProtocol {
    /// 创建新的阿里云协议实例
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
impl Protocol for AliyunProtocol {
    type Request = AliyunRequest;
    type Response = AliyunResponse;
    
    fn name(&self) -> &str {
        "aliyun"
    }
    
    fn chat_endpoint(&self, base_url: &str) -> String {
        format!("{}/services/aigc/text-generation/generation", base_url.trim_end_matches('/'))
    }
    
    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        let messages = request.messages.iter()
            .map(|msg| AliyunMessage {
                role: match msg.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::System => "system".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect();

        let parameters = AliyunParameters {
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            result_format: "message".to_string(),
            incremental_output: request.stream,
        };

        Ok(AliyunRequest {
            model: request.model.clone(),
            input: AliyunInput { messages },
            parameters,
        })
    }
    
    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        let aliyun_response: AliyunResponse = serde_json::from_str(response)
            .map_err(|e| LlmConnectorError::ParseError(format!("Failed to parse Aliyun response: {}", e)))?;

        if aliyun_response.output.choices.is_empty() {
            return Err(LlmConnectorError::ParseError("No choices in response".to_string()));
        }

        let choices: Vec<Choice> = aliyun_response.output.choices.into_iter()
            .enumerate()
            .map(|(index, choice)| Choice {
                index: index as u32,
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

        let usage = aliyun_response.usage.map(|u| Usage {
            prompt_tokens: u.input_tokens,
            completion_tokens: u.output_tokens,
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
            id: aliyun_response.request_id.unwrap_or_else(|| "unknown".to_string()),
            object: "chat.completion".to_string(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            model: "unknown".to_string(), // Aliyun doesn't return model in response
            choices,
            content,
            usage,
            system_fingerprint: None,
        })
    }
    
    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError {
        let error_info = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .unwrap_or_else(|| serde_json::json!({"message": body}));
            
        let message = error_info.get("message")
            .or_else(|| error_info.get("error"))
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown Aliyun error");

        match status {
            400 => LlmConnectorError::InvalidRequest(format!("Aliyun: {}", message)),
            401 => LlmConnectorError::AuthenticationError(format!("Aliyun: {}", message)),
            403 => LlmConnectorError::PermissionError(format!("Aliyun: {}", message)),
            429 => LlmConnectorError::RateLimitError(format!("Aliyun: {}", message)),
            500..=599 => LlmConnectorError::ServerError(format!("Aliyun: {}", message)),
            _ => LlmConnectorError::ApiError(format!("Aliyun HTTP {}: {}", status, message)),
        }
    }
    
    fn auth_headers(&self) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", self.api_key)),
            ("Content-Type".to_string(), "application/json".to_string()),
        ]
    }
}

// 阿里云请求类型
#[derive(Serialize, Debug)]
pub struct AliyunRequest {
    pub model: String,
    pub input: AliyunInput,
    pub parameters: AliyunParameters,
}

#[derive(Serialize, Debug)]
pub struct AliyunInput {
    pub messages: Vec<AliyunMessage>,
}

#[derive(Serialize, Debug)]
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
}

#[derive(Serialize, Debug)]
pub struct AliyunMessage {
    pub role: String,
    pub content: String,
}

// 阿里云响应类型
#[derive(Deserialize, Debug)]
pub struct AliyunResponse {
    pub output: AliyunOutput,
    pub usage: Option<AliyunUsage>,
    pub request_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AliyunOutput {
    pub choices: Vec<AliyunChoice>,
}

#[derive(Deserialize, Debug)]
pub struct AliyunChoice {
    pub message: AliyunResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AliyunResponseMessage {
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct AliyunUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}
