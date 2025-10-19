//! 阿里云DashScope服务提供商实现 - V2架构
//!
//! 这个模块提供阿里云DashScope服务的完整实现，使用统一的V2架构。

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, Role};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Aliyun Protocol Definition (Private)
// ============================================================================

/// 阿里云DashScope私有协议实现
///
/// 这是阿里云专用的API格式，与OpenAI和Anthropic都不同。
/// 由于这是私有协议，定义在provider内部而不是公开的protocols模块中。
#[derive(Debug, Clone)]
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

impl Protocol for AliyunProtocol {
    type Request = AliyunRequest;
    type Response = AliyunResponse;

    fn name(&self) -> &str {
        "aliyun"
    }

    fn chat_endpoint(&self, base_url: &str) -> String {
        format!("{}/api/v1/services/aigc/text-generation/generation", base_url)
    }

    fn auth_headers(&self) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", self.api_key)),
            // 注意: Content-Type 由 HttpClient::post() 的 .json() 方法自动设置
            // 不要在这里重复设置，否则会导致 "Content-Type application/json,application/json is not supported" 错误
        ]
    }

    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        // 转换为阿里云格式
        let aliyun_messages: Vec<AliyunMessage> = request.messages.iter().map(|msg| {
            AliyunMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                content: msg.content.clone(),
            }
        }).collect();

        Ok(AliyunRequest {
            model: request.model.clone(),
            input: AliyunInput {
                messages: aliyun_messages,
            },
            parameters: AliyunParameters {
                max_tokens: request.max_tokens,
                temperature: request.temperature,
                top_p: request.top_p,
                result_format: "message".to_string(),
            },
        })
    }

    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        let parsed: AliyunResponse = serde_json::from_str(response)
            .map_err(|e| LlmConnectorError::InvalidRequest(format!("Failed to parse response: {}", e)))?;

        if let Some(choices) = parsed.output.choices {
            if let Some(first_choice) = choices.first() {
                return Ok(ChatResponse {
                    content: first_choice.message.content.clone(),
                    model: parsed.model.unwrap_or_else(|| "unknown".to_string()),
                    ..Default::default()
                });
            }
        }

        Err(LlmConnectorError::InvalidRequest("Empty or invalid response".to_string()))
    }

    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError {
        LlmConnectorError::from_status_code(status, format!("Aliyun API error: {}", body))
    }
}

// 阿里云专用数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunRequest {
    pub model: String,
    pub input: AliyunInput,
    pub parameters: AliyunParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunInput {
    pub messages: Vec<AliyunMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    pub result_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunResponse {
    pub model: Option<String>,
    pub output: AliyunOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunOutput {
    pub choices: Option<Vec<AliyunChoice>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunChoice {
    pub message: AliyunMessage,
}

// ============================================================================
// Aliyun Provider Implementation
// ============================================================================

/// 阿里云DashScope服务提供商类型
pub type AliyunProvider = GenericProvider<AliyunProtocol>;

/// 创建阿里云DashScope服务提供商
/// 
/// # 参数
/// - `api_key`: 阿里云DashScope API密钥
/// 
/// # 返回
/// 配置好的阿里云服务提供商实例
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::aliyun;
/// 
/// let provider = aliyun("sk-...").unwrap();
/// ```
pub fn aliyun(api_key: &str) -> Result<AliyunProvider, LlmConnectorError> {
    aliyun_with_config(api_key, None, None, None)
}

/// 创建带有自定义配置的阿里云服务提供商
/// 
/// # 参数
/// - `api_key`: API密钥
/// - `base_url`: 自定义基础URL (可选，默认为官方端点)
/// - `timeout_secs`: 超时时间(秒) (可选)
/// - `proxy`: 代理URL (可选)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::aliyun_with_config;
/// 
/// let provider = aliyun_with_config(
///     "sk-...",
///     None, // 使用默认URL
///     Some(60), // 60秒超时
///     Some("http://proxy:8080")
/// ).unwrap();
/// ```
pub fn aliyun_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<AliyunProvider, LlmConnectorError> {
    // 创建协议实例
    let protocol = AliyunProtocol::new(api_key);

    // 创建HTTP客户端
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://dashscope.aliyuncs.com"),
        timeout_secs,
        proxy,
    )?;

    // 添加认证头
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);

    // 创建通用提供商
    Ok(GenericProvider::new(protocol, client))
}

/// 创建用于阿里云国际版的服务提供商
/// 
/// # 参数
/// - `api_key`: 阿里云国际版API密钥
/// - `region`: 区域 (如 "us-west-1", "ap-southeast-1")
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::aliyun_international;
/// 
/// let provider = aliyun_international("sk-...", "us-west-1").unwrap();
/// ```
pub fn aliyun_international(
    api_key: &str,
    region: &str,
) -> Result<AliyunProvider, LlmConnectorError> {
    let base_url = format!("https://dashscope.{}.aliyuncs.com", region);
    aliyun_with_config(api_key, Some(&base_url), None, None)
}

/// 创建用于阿里云专有云的服务提供商
/// 
/// # 参数
/// - `api_key`: API密钥
/// - `endpoint`: 专有云端点URL
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::aliyun_private;
/// 
/// let provider = aliyun_private(
///     "sk-...",
///     "https://dashscope.your-private-cloud.com"
/// ).unwrap();
/// ```
pub fn aliyun_private(
    api_key: &str,
    endpoint: &str,
) -> Result<AliyunProvider, LlmConnectorError> {
    aliyun_with_config(api_key, Some(endpoint), None, None)
}

/// 创建带有自定义超时的阿里云服务提供商
/// 
/// 阿里云的某些模型可能需要较长的处理时间，这个函数提供便利的超时配置。
/// 
/// # 参数
/// - `api_key`: API密钥
/// - `timeout_secs`: 超时时间(秒)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::aliyun_with_timeout;
/// 
/// // 设置120秒超时，适用于长文本处理
/// let provider = aliyun_with_timeout("sk-...", 120).unwrap();
/// ```
pub fn aliyun_with_timeout(
    api_key: &str,
    timeout_secs: u64,
) -> Result<AliyunProvider, LlmConnectorError> {
    aliyun_with_config(api_key, None, Some(timeout_secs), None)
}

/// 验证Aliyun API密钥格式
pub fn validate_aliyun_key(api_key: &str) -> bool {
    api_key.starts_with("sk-") && api_key.len() > 20
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aliyun_provider_creation() {
        let provider = aliyun("test-key");
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.protocol().name(), "aliyun");
    }

    #[test]
    fn test_aliyun_with_config() {
        let provider = aliyun_with_config(
            "test-key",
            Some("https://custom.dashscope.com"),
            Some(60),
            None
        );
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.client().base_url(), "https://custom.dashscope.com");
    }

    #[test]
    fn test_aliyun_international() {
        let provider = aliyun_international("test-key", "us-west-1");
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.client().base_url(), "https://dashscope.us-west-1.aliyuncs.com");
    }

    #[test]
    fn test_aliyun_private() {
        let provider = aliyun_private("test-key", "https://private.dashscope.com");
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.client().base_url(), "https://private.dashscope.com");
    }

    #[test]
    fn test_aliyun_with_timeout() {
        let provider = aliyun_with_timeout("test-key", 120);
        assert!(provider.is_ok());
    }
}
