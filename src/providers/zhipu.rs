//! 智谱GLM服务提供商实现 - V2架构
//!
//! 这个模块提供智谱GLM服务的完整实现，支持原生格式和OpenAI兼容格式。

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, Role, Tool, ToolChoice, Choice, Message as TypeMessage};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Zhipu Protocol Definition (Private)
// ============================================================================

/// 智谱GLM私有协议实现
///
/// 智谱支持OpenAI兼容格式，但有自己的认证和错误处理。
/// 由于这是私有协议，定义在provider内部而不是公开的protocols模块中。
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

#[async_trait::async_trait]
impl Protocol for ZhipuProtocol {
    type Request = ZhipuRequest;
    type Response = ZhipuResponse;

    fn name(&self) -> &str {
        "zhipu"
    }

    fn chat_endpoint(&self, base_url: &str) -> String {
        format!("{}/api/paas/v4/chat/completions", base_url)
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

    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        // 智谱使用OpenAI兼容格式
        let messages: Vec<ZhipuMessage> = request
            .messages
            .iter()
            .map(|msg| ZhipuMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                content: msg.content.clone(),
                tool_calls: msg.tool_calls.as_ref().map(|calls| {
                    calls.iter().map(|c| serde_json::to_value(c).unwrap_or_default()).collect()
                }),
                tool_call_id: msg.tool_call_id.clone(),
                name: msg.name.clone(),
            })
            .collect();

        Ok(ZhipuRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: request.stream,
            tools: request.tools.clone(),
            tool_choice: request.tool_choice.clone(),
        })
    }

    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        let parsed: ZhipuResponse = serde_json::from_str(response).map_err(|e| {
            LlmConnectorError::InvalidRequest(format!("Failed to parse response: {}", e))
        })?;

        if let Some(choices) = parsed.choices {
            if let Some(first_choice) = choices.first() {
                // 转换 ZhipuMessage 到 TypeMessage
                let type_message = TypeMessage {
                    role: match first_choice.message.role.as_str() {
                        "system" => Role::System,
                        "user" => Role::User,
                        "assistant" => Role::Assistant,
                        "tool" => Role::Tool,
                        _ => Role::Assistant,
                    },
                    content: first_choice.message.content.clone(),
                    tool_calls: first_choice.message.tool_calls.as_ref().map(|calls| {
                        calls.iter().filter_map(|v| {
                            serde_json::from_value(v.clone()).ok()
                        }).collect()
                    }),
                    ..Default::default()
                };
                
                let choice = Choice {
                    index: first_choice.index.unwrap_or(0),
                    message: type_message,
                    finish_reason: first_choice.finish_reason.clone(),
                    logprobs: None,
                };
                
                return Ok(ChatResponse {
                    id: parsed.id.unwrap_or_else(|| "unknown".to_string()),
                    object: "chat.completion".to_string(),
                    created: parsed.created.unwrap_or(0),
                    model: parsed.model.unwrap_or_else(|| "unknown".to_string()),
                    content: first_choice.message.content.clone(),
                    choices: vec![choice],
                    usage: parsed.usage.and_then(|v| serde_json::from_value(v).ok()),
                    system_fingerprint: None,
                });
            }
        }

        Err(LlmConnectorError::InvalidRequest(
            "Empty or invalid response".to_string(),
        ))
    }

    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError {
        LlmConnectorError::from_status_code(status, format!("Zhipu API error: {}", body))
    }

    /// 智谱专用流式解析器
    ///
    /// 智谱 API 使用单换行分隔 SSE 事件，而不是标准的双换行
    /// 格式: data: {...}\n 而不是 data: {...}\n\n
    #[cfg(feature = "streaming")]
    async fn parse_stream_response(
        &self,
        response: reqwest::Response,
    ) -> Result<crate::types::ChatStream, LlmConnectorError> {
        use crate::types::StreamingResponse;
        use futures_util::StreamExt;

        let stream = response.bytes_stream();

        let events_stream = stream
            .scan(String::new(), |buffer, chunk_result| {
                let mut out: Vec<Result<String, LlmConnectorError>> = Vec::new();
                match chunk_result {
                    Ok(chunk) => {
                        let chunk_str = String::from_utf8_lossy(&chunk).replace("\r\n", "\n");
                        buffer.push_str(&chunk_str);

                        // 智谱使用单换行分隔每个 data: 行
                        while let Some(newline_idx) = buffer.find('\n') {
                            let line: String = buffer.drain(..newline_idx + 1).collect();
                            let trimmed = line.trim();

                            // 跳过空行
                            if trimmed.is_empty() {
                                continue;
                            }

                            // 提取 data: 后的内容
                            if let Some(payload) = trimmed
                                .strip_prefix("data: ")
                                .or_else(|| trimmed.strip_prefix("data:"))
                            {
                                let payload = payload.trim();

                                // 跳过 [DONE] 标记
                                if payload == "[DONE]" {
                                    continue;
                                }

                                // 跳过空 payload
                                if payload.is_empty() {
                                    continue;
                                }

                                out.push(Ok(payload.to_string()));
                            }
                        }
                    }
                    Err(e) => {
                        out.push(Err(LlmConnectorError::NetworkError(e.to_string())));
                    }
                }
                std::future::ready(Some(out))
            })
            .flat_map(futures_util::stream::iter);

        // 将 JSON 字符串流转换为 StreamingResponse 流
        let response_stream = events_stream.map(|result| {
            result.and_then(|json_str| {
                let mut response = serde_json::from_str::<StreamingResponse>(&json_str).map_err(|e| {
                    LlmConnectorError::ParseError(format!(
                        "Failed to parse Zhipu streaming response: {}. JSON: {}",
                        e, json_str
                    ))
                })?;
                
                // 填充 content 字段（从 delta.content 复制）
                if response.content.is_empty() {
                    if let Some(first_choice) = response.choices.first() {
                        if let Some(ref delta_content) = first_choice.delta.content {
                            response.content = delta_content.clone();
                        }
                    }
                }
                
                Ok(response)
            })
        });

        Ok(Box::pin(response_stream))
    }
}

// 智谱专用数据结构 (OpenAI兼容格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZhipuRequest {
    pub model: String,
    pub messages: Vec<ZhipuMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZhipuMessage {
    pub role: String,
    #[serde(default)]
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZhipuResponse {
    pub id: Option<String>,
    pub created: Option<u64>,
    pub model: Option<String>,
    pub choices: Option<Vec<ZhipuChoice>>,
    pub usage: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZhipuChoice {
    pub index: Option<u32>,
    pub message: ZhipuMessage,
    pub finish_reason: Option<String>,
}

// ============================================================================
// Zhipu Provider Implementation
// ============================================================================

/// 智谱GLM服务提供商类型
pub type ZhipuProvider = GenericProvider<ZhipuProtocol>;

/// 创建智谱GLM服务提供商 (使用原生格式)
///
/// # 参数
/// - `api_key`: 智谱GLM API密钥
///
/// # 返回
/// 配置好的智谱服务提供商实例
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::zhipu;
///
/// let provider = zhipu("your-api-key").unwrap();
/// ```
pub fn zhipu(api_key: &str) -> Result<ZhipuProvider, LlmConnectorError> {
    zhipu_with_config(api_key, false, None, None, None)
}

/// 创建智谱GLM服务提供商 (使用OpenAI兼容格式)
///
/// # 参数
/// - `api_key`: 智谱GLM API密钥
///
/// # 返回
/// 配置好的智谱服务提供商实例 (OpenAI兼容模式)
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::zhipu_openai_compatible;
///
/// let provider = zhipu_openai_compatible("your-api-key").unwrap();
/// ```
pub fn zhipu_openai_compatible(api_key: &str) -> Result<ZhipuProvider, LlmConnectorError> {
    zhipu_with_config(api_key, true, None, None, None)
}

/// 创建带有自定义配置的智谱GLM服务提供商
///
/// # 参数
/// - `api_key`: API密钥
/// - `openai_compatible`: 是否使用OpenAI兼容格式
/// - `base_url`: 自定义基础URL (可选)
/// - `timeout_secs`: 超时时间(秒) (可选)
/// - `proxy`: 代理URL (可选)
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::zhipu_with_config;
///
/// let provider = zhipu_with_config(
///     "your-api-key",
///     true, // 使用OpenAI兼容格式
///     None, // 使用默认URL
///     Some(60), // 60秒超时
///     None
/// ).unwrap();
/// ```
pub fn zhipu_with_config(
    api_key: &str,
    openai_compatible: bool,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<ZhipuProvider, LlmConnectorError> {
    // 创建协议实例
    let protocol = if openai_compatible {
        ZhipuProtocol::new_openai_compatible(api_key)
    } else {
        ZhipuProtocol::new(api_key)
    };

    // 创建HTTP客户端
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://open.bigmodel.cn"),
        timeout_secs,
        proxy,
    )?;

    // 添加认证头
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);

    // 创建通用提供商
    Ok(GenericProvider::new(protocol, client))
}

/// 创建带有自定义超时的智谱GLM服务提供商
///
/// # 参数
/// - `api_key`: API密钥
/// - `timeout_secs`: 超时时间(秒)
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::zhipu_with_timeout;
///
/// // 设置120秒超时
/// let provider = zhipu_with_timeout("your-api-key", 120).unwrap();
/// ```
pub fn zhipu_with_timeout(
    api_key: &str,
    timeout_secs: u64,
) -> Result<ZhipuProvider, LlmConnectorError> {
    zhipu_with_config(api_key, true, None, Some(timeout_secs), None)
}

/// 创建用于智谱GLM企业版的服务提供商
///
/// # 参数
/// - `api_key`: 企业版API密钥
/// - `enterprise_endpoint`: 企业版端点URL
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::zhipu_enterprise;
///
/// let provider = zhipu_enterprise(
///     "your-enterprise-key",
///     "https://enterprise.bigmodel.cn"
/// ).unwrap();
/// ```
pub fn zhipu_enterprise(
    api_key: &str,
    enterprise_endpoint: &str,
) -> Result<ZhipuProvider, LlmConnectorError> {
    zhipu_with_config(api_key, true, Some(enterprise_endpoint), None, None)
}

/// 验证智谱GLM API密钥格式
///
/// # 参数
/// - `api_key`: 要验证的API密钥
///
/// # 返回
/// 如果格式看起来正确返回true，否则返回false
///
/// # 示例
/// ```rust
/// use llm_connector::providers::validate_zhipu_key;
///
/// assert!(validate_zhipu_key("your-valid-key"));
/// assert!(!validate_zhipu_key(""));
/// ```
pub fn validate_zhipu_key(api_key: &str) -> bool {
    !api_key.is_empty() && api_key.len() > 10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zhipu_provider_creation() {
        let provider = zhipu("test-key");
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.protocol().name(), "zhipu");
    }

    #[test]
    fn test_zhipu_openai_compatible() {
        let provider = zhipu_openai_compatible("test-key");
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.protocol().name(), "zhipu");
        assert!(provider.protocol().is_openai_compatible());
    }

    #[test]
    fn test_zhipu_with_config() {
        let provider = zhipu_with_config(
            "test-key",
            true,
            Some("https://custom.bigmodel.cn"),
            Some(60),
            None,
        );
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.client().base_url(), "https://custom.bigmodel.cn");
        assert!(provider.protocol().is_openai_compatible());
    }

    #[test]
    fn test_zhipu_with_timeout() {
        let provider = zhipu_with_timeout("test-key", 120);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_zhipu_enterprise() {
        let provider = zhipu_enterprise("test-key", "https://enterprise.bigmodel.cn");
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(
            provider.client().base_url(),
            "https://enterprise.bigmodel.cn"
        );
    }

    #[test]
    fn test_validate_zhipu_key() {
        assert!(validate_zhipu_key("valid-test-key"));
        assert!(validate_zhipu_key("another-valid-key-12345"));
        assert!(!validate_zhipu_key("short"));
        assert!(!validate_zhipu_key(""));
    }
}
