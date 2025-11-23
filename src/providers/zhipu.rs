//! Zhipu GLM服务Provide商实现 - V2架构
//!
//! 这个模块ProvideZhipu GLM服务的完整实现，Support原生格式和OpenAI兼容格式。

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, Role, Tool, ToolChoice, Choice, Message as TypeMessage};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 从 Zhipu 响应中提取推理内容
///
/// Zhipu GLM-Z1 etc.推理模型将推理过程嵌入在 content 中，Use标记分隔：
/// - `###Thinking` 标记推理过程开始
/// - `###Response` 标记最终答案开始
///
/// # Parameters
/// - `content`: 原始 content 字符串
///
/// # Returns
/// - `(reasoning_content, final_content)`: 推理内容和最终答案
fn extract_zhipu_reasoning_content(content: &str) -> (Option<String>, String) {
    // Check是否Contains推理标记
    if content.contains("###Thinking") && content.contains("###Response") {
        // 分离推理内容和答案
        let parts: Vec<&str> = content.split("###Response").collect();
        if parts.len() >= 2 {
            let thinking = parts[0]
                .replace("###Thinking", "")
                .trim()
                .to_string();
            let response = parts[1..].join("###Response").trim().to_string();

            if !thinking.is_empty() {
                return (Some(thinking), response);
            }
        }
    }

    // 如果没有推理标记，Returns原始内容
    (None, content.to_string())
}

/// Zhipu 流式响应处理阶段
#[cfg(feature = "streaming")]
#[derive(Debug, Clone, PartialEq)]
enum ZhipuStreamPhase {
    /// 初始状态，etc.待检测是否为推理模型
    Initial,
    /// 在推理阶段（###Thinking 之后，###Response 之前）
    InThinking,
    /// 在答案阶段（###Response 之后）
    InResponse,
}

/// Zhipu 流式响应状态机
#[cfg(feature = "streaming")]
struct ZhipuStreamState {
    /// 缓冲区，用于累积内容
    buffer: String,
    /// 当前处理阶段
    phase: ZhipuStreamPhase,
}

#[cfg(feature = "streaming")]
impl ZhipuStreamState {
    fn new() -> Self {
        Self {
            buffer: String::new(),
            phase: ZhipuStreamPhase::Initial,
        }
    }

    /// 处理流式内容增量
    ///
    /// # Returns
    /// - `(reasoning_delta, content_delta)`: 推理内容增量和答案内容增量
    fn process(&mut self, delta_content: &str) -> (Option<String>, Option<String>) {
        self.buffer.push_str(delta_content);

        match self.phase {
            ZhipuStreamPhase::Initial => {
                // 检测是否Contains ###Thinking 标记
                if self.buffer.contains("###Thinking") {
                    // 移除标记并进入推理阶段
                    self.buffer = self.buffer.replace("###Thinking", "").trim_start().to_string();
                    self.phase = ZhipuStreamPhase::InThinking;

                    // Check是否立即Contains ###Response（完整推理在一个块中）
                    if self.buffer.contains("###Response") {
                        return self.handle_response_marker();
                    }

                    // Returns当前缓冲区作为推理内容
                    let reasoning = self.buffer.clone();
                    self.buffer.clear();
                    (Some(reasoning), None)
                } else {
                    // 不是推理模型，直接Returns内容
                    let content = self.buffer.clone();
                    self.buffer.clear();
                    (None, Some(content))
                }
            }
            ZhipuStreamPhase::InThinking => {
                // 检测是否Contains ###Response 标记
                if self.buffer.contains("###Response") {
                    self.handle_response_marker()
                } else {
                    // 继续累积推理内容
                    let reasoning = self.buffer.clone();
                    self.buffer.clear();
                    (Some(reasoning), None)
                }
            }
            ZhipuStreamPhase::InResponse => {
                // 在答案阶段，直接Returns内容
                let content = self.buffer.clone();
                self.buffer.clear();
                (None, Some(content))
            }
        }
    }

    /// 处理 ###Response 标记
    fn handle_response_marker(&mut self) -> (Option<String>, Option<String>) {
        let parts: Vec<&str> = self.buffer.split("###Response").collect();
        if parts.len() >= 2 {
            // 推理部分（###Response 之前）
            let thinking = parts[0].trim();
            let reasoning = if !thinking.is_empty() {
                Some(thinking.to_string())
            } else {
                None
            };

            // 答案部分（###Response 之后）
            let answer = parts[1..].join("###Response").trim_start().to_string();
            self.buffer = String::new();
            self.phase = ZhipuStreamPhase::InResponse;

            let content = if !answer.is_empty() {
                Some(answer)
            } else {
                None
            };

            (reasoning, content)
        } else {
            // 不应该发生，但为了安全
            (None, None)
        }
    }
}

// ============================================================================
// Zhipu Protocol Definition (Private)
// ============================================================================

/// Zhipu GLM私有协议实现
///
/// 智谱SupportOpenAI兼容格式，但有自己的认证和Errors处理。
/// 由于这是私有协议，Define在provider内部而不是公开的protocols模块中。
#[derive(Clone, Debug)]
pub struct ZhipuProtocol {
    api_key: String,
    use_openai_format: bool,
}

impl ZhipuProtocol {
    /// Create新的智谱协议实例 (Use原生格式)
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            use_openai_format: false,
        }
    }

    /// CreateUseOpenAI兼容格式的智谱协议实例
    pub fn new_openai_compatible(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            use_openai_format: true,
        }
    }

    /// GetAPI key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// 是否UseOpenAI兼容格式
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
            // 注意: Content-Type 由 HttpClient::post() 的 .json() 方法自动Set
            // 不要在这里重复Set，否则可能导致重复头部Errors
        ]
    }

    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        // 智谱UseOpenAI兼容格式
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
                // Zhipu Use纯文本格式
                content: msg.content_as_text(),
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
                // Convert ZhipuMessage 到 TypeMessage
                // 提取推理内容（如果存在）
                let (reasoning_content, final_content) =
                    extract_zhipu_reasoning_content(&first_choice.message.content);

                let type_message = TypeMessage {
                    role: match first_choice.message.role.as_str() {
                        "system" => Role::System,
                        "user" => Role::User,
                        "assistant" => Role::Assistant,
                        "tool" => Role::Tool,
                        _ => Role::Assistant,
                    },
                    content: vec![crate::types::MessageBlock::text(&final_content)],
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
                    content: final_content,
                    reasoning_content,
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

    /// 智谱专用流式Parse器
    ///
    /// 智谱 API Use单换行分隔 SSE 事件，而不是标准的双换行
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

                        // 智谱Use单换行分隔每个 data: 行
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

        // 将 JSON 字符串流Convert为 StreamingResponse 流
        // Use状态机处理 Zhipu 的 ###Thinking 和 ###Response 标记
        let response_stream = events_stream.scan(
            ZhipuStreamState::new(),
            |state, result| {
                let processed = result.and_then(|json_str| {
                    let mut response = serde_json::from_str::<StreamingResponse>(&json_str).map_err(|e| {
                        LlmConnectorError::ParseError(format!(
                            "Failed to parse Zhipu streaming response: {}. JSON: {}",
                            e, json_str
                        ))
                    })?;

                    // 处理推理内容标记
                    if let Some(first_choice) = response.choices.first_mut() {
                        if let Some(ref delta_content) = first_choice.delta.content {
                            // Use状态机处理内容
                            let (reasoning_delta, content_delta) = state.process(delta_content);

                            // 更新 delta
                            if let Some(reasoning) = reasoning_delta {
                                first_choice.delta.reasoning_content = Some(reasoning);
                            }

                            if let Some(content) = content_delta {
                                first_choice.delta.content = Some(content.clone());
                                // 同时更新 response.content
                                response.content = content;
                            } else {
                                // 如果没有内容增量，清空 delta.content
                                first_choice.delta.content = None;
                                response.content = String::new();
                            }
                        }
                    }

                    Ok(response)
                });

                std::future::ready(Some(processed))
            }
        );

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

/// Zhipu GLM服务Provide商类型
pub type ZhipuProvider = GenericProvider<ZhipuProtocol>;

/// CreateZhipu GLM服务Provide商 (Use原生格式)
///
/// # Parameters
/// - `api_key`: Zhipu GLM API key
///
/// # Returns
/// 配置好的智谱服务Provide商实例
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::zhipu;
///
/// let provider = zhipu("your-api-key").unwrap();
/// ```
pub fn zhipu(api_key: &str) -> Result<ZhipuProvider, LlmConnectorError> {
    zhipu_with_config(api_key, false, None, None, None)
}

/// CreateZhipu GLM服务Provide商 (UseOpenAI兼容格式)
///
/// # Parameters
/// - `api_key`: Zhipu GLM API key
///
/// # Returns
/// 配置好的智谱服务Provide商实例 (OpenAI兼容模式)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::zhipu_openai_compatible;
///
/// let provider = zhipu_openai_compatible("your-api-key").unwrap();
/// ```
pub fn zhipu_openai_compatible(api_key: &str) -> Result<ZhipuProvider, LlmConnectorError> {
    zhipu_with_config(api_key, true, None, None, None)
}

/// Create带有自Define配置的Zhipu GLM服务Provide商
///
/// # Parameters
/// - `api_key`: API key
/// - `openai_compatible`: 是否UseOpenAI兼容格式
/// - `base_url`: Custom base URL (可选)
/// - `timeout_secs`: 超时时间(秒) (可选)
/// - `proxy`: 代理URL (可选)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::zhipu_with_config;
///
/// let provider = zhipu_with_config(
///     "your-api-key",
///     true, // UseOpenAI兼容格式
///     None, // Use默认URL
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
    // Create协议实例
    let protocol = if openai_compatible {
        ZhipuProtocol::new_openai_compatible(api_key)
    } else {
        ZhipuProtocol::new(api_key)
    };

    // CreateHTTP客户端
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://open.bigmodel.cn"),
        timeout_secs,
        proxy,
    )?;

    // 添加认证头
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);

    // Create通用Provide商
    Ok(GenericProvider::new(protocol, client))
}

/// Create带有自Define超时的Zhipu GLM服务Provide商
///
/// # Parameters
/// - `api_key`: API key
/// - `timeout_secs`: 超时时间(秒)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::zhipu_with_timeout;
///
/// // Set120秒超时
/// let provider = zhipu_with_timeout("your-api-key", 120).unwrap();
/// ```
pub fn zhipu_with_timeout(
    api_key: &str,
    timeout_secs: u64,
) -> Result<ZhipuProvider, LlmConnectorError> {
    zhipu_with_config(api_key, true, None, Some(timeout_secs), None)
}

/// Create用于Zhipu GLM企业版的服务Provide商
///
/// # Parameters
/// - `api_key`: 企业版API key
/// - `enterprise_endpoint`: 企业版端点URL
///
/// # Example
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

/// ValidateZhipu GLM API key格式
///
/// # Parameters
/// - `api_key`: 要Validate的API key
///
/// # Returns
/// 如果格式看起来正确Returnstrue，否则Returnsfalse
///
/// # Example
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

    #[test]
    fn test_extract_zhipu_reasoning_content() {
        // 测试Contains推理内容的情况
        let content_with_thinking = "###Thinking\n这是推理过程\n分析步骤1\n分析步骤2\n###Response\n这是最终答案";
        let (reasoning, answer) = extract_zhipu_reasoning_content(content_with_thinking);
        assert!(reasoning.is_some());
        assert_eq!(reasoning.unwrap(), "这是推理过程\n分析步骤1\n分析步骤2");
        assert_eq!(answer, "这是最终答案");

        // 测试不Contains推理内容的情况
        let content_without_thinking = "这只是一个普通的回答";
        let (reasoning, answer) = extract_zhipu_reasoning_content(content_without_thinking);
        assert!(reasoning.is_none());
        assert_eq!(answer, "这只是一个普通的回答");

        // 测试只有 Thinking 没有 Response 的情况
        let content_only_thinking = "###Thinking\n这是推理过程";
        let (reasoning, answer) = extract_zhipu_reasoning_content(content_only_thinking);
        assert!(reasoning.is_none());
        assert_eq!(answer, "###Thinking\n这是推理过程");

        // 测试空推理内容的情况
        let content_empty_thinking = "###Thinking\n\n###Response\n答案";
        let (reasoning, answer) = extract_zhipu_reasoning_content(content_empty_thinking);
        assert!(reasoning.is_none());
        assert_eq!(answer, "###Thinking\n\n###Response\n答案");
    }

    #[cfg(feature = "streaming")]
    #[test]
    fn test_zhipu_stream_state() {
        // 测试推理模型流式响应
        let mut state = ZhipuStreamState::new();

        // 第一个块: ###Thinking
        let (reasoning, content) = state.process("###Thinking\n开始");
        assert_eq!(reasoning, Some("开始".to_string()));
        assert_eq!(content, None);

        // 第二个块: 推理过程
        let (reasoning, content) = state.process("推理");
        assert_eq!(reasoning, Some("推理".to_string()));
        assert_eq!(content, None);

        // 第三个块: ###Response
        let (reasoning, content) = state.process("过程\n###Response\n答案");
        assert_eq!(reasoning, Some("过程".to_string()));
        assert_eq!(content, Some("答案".to_string()));

        // 第四个块: 继续答案
        let (reasoning, content) = state.process("继续");
        assert_eq!(reasoning, None);
        assert_eq!(content, Some("继续".to_string()));
    }

    #[cfg(feature = "streaming")]
    #[test]
    fn test_zhipu_stream_state_non_reasoning() {
        // 测试非推理模型流式响应
        let mut state = ZhipuStreamState::new();

        // 第一个块: 普通内容
        let (reasoning, content) = state.process("这是");
        assert_eq!(reasoning, None);
        assert_eq!(content, Some("这是".to_string()));

        // 第二个块: 继续内容
        let (reasoning, content) = state.process("普通回答");
        assert_eq!(reasoning, None);
        assert_eq!(content, Some("普通回答".to_string()));
    }

    #[cfg(feature = "streaming")]
    #[test]
    fn test_zhipu_stream_state_complete_in_one_chunk() {
        // 测试完整推理在一个块中
        let mut state = ZhipuStreamState::new();

        let (reasoning, content) = state.process("###Thinking\n推理过程\n###Response\n答案");
        assert_eq!(reasoning, Some("推理过程".to_string()));
        assert_eq!(content, Some("答案".to_string()));
    }
}
