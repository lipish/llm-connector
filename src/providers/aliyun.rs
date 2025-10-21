//! 阿里云DashScope服务提供商实现 - V2架构
//!
//! 这个模块提供阿里云DashScope服务的完整实现，使用统一的V2架构。

use crate::core::{HttpClient, Protocol};
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, Role};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

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

    /// 获取流式请求的额外头部
    pub fn streaming_headers(&self) -> Vec<(String, String)> {
        vec![
            ("X-DashScope-SSE".to_string(), "enable".to_string()),
        ]
    }
}

#[async_trait]
#[async_trait]
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
                // 流式模式需要 incremental_output
                incremental_output: if request.stream.unwrap_or(false) {
                    Some(true)
                } else {
                    None
                },
            },
        })
    }

    #[cfg(feature = "streaming")]
    async fn parse_stream_response(&self, response: reqwest::Response) -> Result<crate::types::ChatStream, LlmConnectorError> {
        use futures_util::StreamExt;
        use crate::types::{StreamingResponse, StreamingChoice, Delta};

        let stream = response.bytes_stream();
        let mut lines_buffer = String::new();

        let mapped_stream = stream.map(move |result| {
            match result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    lines_buffer.push_str(&text);

                    let mut responses = Vec::new();
                    let lines: Vec<&str> = lines_buffer.lines().collect();

                    for line in &lines {
                        if line.starts_with("data:") {
                            let json_str = line.trim_start_matches("data:").trim();
                            if json_str.is_empty() {
                                continue;
                            }

                            // 解析 Aliyun 响应
                            if let Ok(aliyun_resp) = serde_json::from_str::<AliyunResponse>(json_str) {
                                if let Some(choices) = aliyun_resp.output.choices {
                                    if let Some(first_choice) = choices.first() {
                                        // 转换为 StreamingResponse
                                        let streaming_choice = StreamingChoice {
                                            index: 0,
                                            delta: Delta {
                                                role: Some(Role::Assistant),
                                                content: if first_choice.message.content.is_empty() {
                                                    None
                                                } else {
                                                    Some(first_choice.message.content.clone())
                                                },
                                                tool_calls: None,
                                                reasoning_content: None,
                                                reasoning: None,
                                                thought: None,
                                                thinking: None,
                                            },
                                            finish_reason: if first_choice.finish_reason.as_deref() == Some("stop") {
                                                Some("stop".to_string())
                                            } else {
                                                None
                                            },
                                            logprobs: None,
                                        };

                                        let content = first_choice.message.content.clone();

                                        let streaming_response = StreamingResponse {
                                            id: aliyun_resp.request_id.clone().unwrap_or_default(),
                                            object: "chat.completion.chunk".to_string(),
                                            created: 0,
                                            model: aliyun_resp.model.clone().unwrap_or_else(|| "unknown".to_string()),
                                            choices: vec![streaming_choice],
                                            content,
                                            reasoning_content: None,
                                            usage: aliyun_resp.usage.as_ref().map(|u| crate::types::Usage {
                                                prompt_tokens: u.input_tokens,
                                                completion_tokens: u.output_tokens,
                                                total_tokens: u.total_tokens,
                                                prompt_cache_hit_tokens: None,
                                                prompt_cache_miss_tokens: None,
                                                prompt_tokens_details: None,
                                                completion_tokens_details: None,
                                            }),
                                            system_fingerprint: None,
                                        };

                                        responses.push(Ok(streaming_response));
                                    }
                                }
                            }
                        }
                    }

                    // 清空已处理的行
                    if let Some(last_line) = lines.last() {
                        if !last_line.is_empty() && !last_line.starts_with("data:") {
                            lines_buffer = last_line.to_string();
                        } else {
                            lines_buffer.clear();
                        }
                    }

                    futures_util::stream::iter(responses)
                }
                Err(e) => {
                    futures_util::stream::iter(vec![Err(crate::error::LlmConnectorError::NetworkError(e.to_string()))])
                }
            }
        }).flatten();

        Ok(Box::pin(mapped_stream))
    }

    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        let parsed: AliyunResponse = serde_json::from_str(response)
            .map_err(|e| LlmConnectorError::InvalidRequest(format!("Failed to parse response: {}", e)))?;

        if let Some(aliyun_choices) = parsed.output.choices {
            if let Some(first_choice) = aliyun_choices.first() {
                // 构建 choices 数组（符合 OpenAI 标准格式）
                let choices = vec![crate::types::Choice {
                    index: 0,
                    message: crate::types::Message {
                        role: Role::Assistant,
                        content: first_choice.message.content.clone(),
                        name: None,
                        tool_calls: None,
                        tool_call_id: None,
                        reasoning_content: None,
                        reasoning: None,
                        thought: None,
                        thinking: None,
                    },
                    finish_reason: first_choice.finish_reason.clone(),
                    logprobs: None,
                }];

                // 从 choices[0] 提取 content 作为便利字段
                let content = first_choice.message.content.clone();

                // 提取 usage 信息
                let usage = parsed.usage.map(|u| crate::types::Usage {
                    prompt_tokens: u.input_tokens,
                    completion_tokens: u.output_tokens,
                    total_tokens: u.total_tokens,
                    prompt_cache_hit_tokens: None,
                    prompt_cache_miss_tokens: None,
                    prompt_tokens_details: None,
                    completion_tokens_details: None,
                });

                return Ok(ChatResponse {
                    id: parsed.request_id.unwrap_or_default(),
                    object: "chat.completion".to_string(),
                    created: 0,  // Aliyun 不提供 created 时间戳
                    model: parsed.model.unwrap_or_else(|| "unknown".to_string()),
                    choices,
                    content,
                    reasoning_content: None,
                    usage,
                    system_fingerprint: None,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental_output: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunResponse {
    pub model: Option<String>,
    pub output: AliyunOutput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<AliyunUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunOutput {
    pub choices: Option<Vec<AliyunChoice>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunChoice {
    pub message: AliyunMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

// ============================================================================
// Custom Aliyun Provider Implementation
// ============================================================================

/// 自定义 Aliyun Provider 实现
///
/// 需要特殊处理流式请求，因为 Aliyun 需要 X-DashScope-SSE 头部
pub struct AliyunProviderImpl {
    protocol: AliyunProtocol,
    client: HttpClient,
}

impl AliyunProviderImpl {
    /// 获取协议实例的引用
    pub fn protocol(&self) -> &AliyunProtocol {
        &self.protocol
    }

    /// 获取 HTTP 客户端的引用
    pub fn client(&self) -> &HttpClient {
        &self.client
    }
}

#[async_trait]
impl crate::core::Provider for AliyunProviderImpl {
    fn name(&self) -> &str {
        "aliyun"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        // 使用标准实现
        let protocol_request = self.protocol.build_request(request)?;
        let url = self.protocol.chat_endpoint(self.client.base_url());

        let response = self.client.post(&url, &protocol_request).await?;
        let status = response.status();

        if !status.is_success() {
            let text = response.text().await
                .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;
            return Err(self.protocol.map_error(status.as_u16(), &text));
        }

        let text = response.text().await
            .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;

        self.protocol.parse_response(&text)
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<crate::types::ChatStream, LlmConnectorError> {
        let mut streaming_request = request.clone();
        streaming_request.stream = Some(true);

        let protocol_request = self.protocol.build_request(&streaming_request)?;
        let url = self.protocol.chat_endpoint(self.client.base_url());

        // 创建临时客户端，添加流式头部
        let streaming_headers: HashMap<String, String> = self.protocol.streaming_headers().into_iter().collect();
        let streaming_client = self.client.clone().with_headers(streaming_headers);

        let response = streaming_client.stream(&url, &protocol_request).await?;
        let status = response.status();

        if !status.is_success() {
            let text = response.text().await
                .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;
            return Err(self.protocol.map_error(status.as_u16(), &text));
        }

        self.protocol.parse_stream_response(response).await
    }

    async fn models(&self) -> Result<Vec<String>, LlmConnectorError> {
        Err(LlmConnectorError::UnsupportedOperation(
            "Aliyun DashScope does not support model listing".to_string()
        ))
    }
}

// ============================================================================
// Aliyun Provider Public API
// ============================================================================

/// 阿里云DashScope服务提供商类型
pub type AliyunProvider = AliyunProviderImpl;

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

    // 创建HTTP客户端（不包含流式头部）
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://dashscope.aliyuncs.com"),
        timeout_secs,
        proxy,
    )?;

    // 添加认证头
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);

    // 创建自定义 Aliyun Provider（需要特殊处理流式请求）
    Ok(AliyunProviderImpl {
        protocol,
        client,
    })
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
