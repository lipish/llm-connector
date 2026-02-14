//! Aliyun DashScope Service Provider Implementation - V2 Architecture
//!
//! This module provides complete Aliyun DashScope service implementation，using unified V2 architecture。

use crate::core::{HttpClient, Protocol};
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, Role};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

// ============================================================================
// Aliyun Protocol Definition (Private)
// ============================================================================

/// Aliyun DashScope private protocol implementation
///
/// This is Aliyun-specific API format，different from both OpenAI and Anthropic。
/// Since this is a private protocol, it is defined inside the provider rather than in the public protocols module.
#[derive(Debug, Clone)]
pub struct AliyunProtocol {
    api_key: String,
}

impl AliyunProtocol {
    /// Create new Aliyun Protocol instance
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }

    /// GetAPI key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// GetstreamingrequestAdditionalheaders
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
            // Note: Content-Type is automatically set by HttpClient::post() .json() method
            // Do not set repeatedly here，otherwise will cause "Content-Type application/json,application/json is not supported" error
        ]
    }

    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        // Convert to Aliyun format
        let aliyun_messages: Vec<AliyunMessage> = request.messages.iter().map(|msg| {
            AliyunMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                // Aliyun uses plain text format
                content: msg.content_as_text(),
                // Tool calls support
                tool_calls: msg.tool_calls.clone(),
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
                // Streaming mode requires incremental_output
                incremental_output: if request.stream.unwrap_or(false) {
                    Some(true)
                } else {
                    None
                },
                // Directly use user-specified values
                enable_thinking: request.enable_thinking,

                // Tools support
                tools: request.tools.clone(),
                tool_choice: request.tool_choice.clone(),
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

                            // Parse Aliyun response
                            if let Ok(aliyun_resp) = serde_json::from_str::<AliyunResponse>(json_str) {
                                if let Some(choices) = aliyun_resp.output.choices {
                                    if let Some(first_choice) = choices.first() {
                                        // Convertas StreamingResponse
                                        let streaming_choice = StreamingChoice {
                                            index: 0,
                                            delta: Delta {
                                                role: Some(Role::Assistant),
                                                content: if first_choice.message.content.is_empty() {
                                                    None
                                                } else {
                                                    Some(first_choice.message.content.clone())
                                                },
                                                // Extract tool_calls from streaming response
                                                tool_calls: first_choice.message.tool_calls.clone(),
                                                reasoning_content: None,
                                                reasoning: None,
                                                thought: None,
                                                thinking: None,
                                            },
                                            finish_reason: first_choice.finish_reason.clone(),
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

                    // Clear processed lines
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
                // Build choices array (conforming to OpenAI standard format)
                let choices = vec![crate::types::Choice {
                    index: 0,
                    message: crate::types::Message {
                        role: Role::Assistant,
                        content: vec![crate::types::MessageBlock::text(&first_choice.message.content)],
                        name: None,
                        // Extract tool_calls from Aliyun response
                        tool_calls: first_choice.message.tool_calls.clone(),
                        tool_call_id: None,
                        reasoning_content: None,
                        reasoning: None,
                        thought: None,
                        thinking: None,
                    },
                    finish_reason: first_choice.finish_reason.clone(),
                    logprobs: None,
                }];

                // Extract content from choices[0] as convenience field
                let content = first_choice.message.content.clone();

                // Extract usage information
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
                    created: 0,  // Aliyun does not provide created timestamp
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
        // Detect context length exceeded from error body
        let body_lower = body.to_lowercase();
        if body_lower.contains("context_length_exceeded")
            || body_lower.contains("maximum context length")
            || body_lower.contains("input is too long")
        {
            return LlmConnectorError::ContextLengthExceeded(format!("Aliyun: {}", body));
        }
        LlmConnectorError::from_status_code(status, format!("Aliyun API error: {}", body))
    }
}

// Aliyun-specific data structures
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

    /// Tool calls in the message (for assistant messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<crate::types::ToolCall>>,
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

    /// Enable thinking/reasoning mode for hybrid models
    ///
    /// When enabled, hybrid models like qwen-plus will return reasoning content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_thinking: Option<bool>,

    /// Tools available to the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<crate::types::Tool>>,

    /// Tool choice strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<crate::types::ToolChoice>,
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

/// custom Aliyun Provider implementation
///
/// Requires special handling for streaming requests，because Aliyun requires X-DashScope-SSE headers
pub struct AliyunProviderImpl {
    protocol: AliyunProtocol,
    client: HttpClient,
}

impl AliyunProviderImpl {
    /// Get Protocol instance reference
    pub fn protocol(&self) -> &AliyunProtocol {
        &self.protocol
    }

    /// Get HTTP client reference
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
        // Usestandardimplementation
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

        // Create temporary client，add streaming headers
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

/// Aliyun DashScopeserviceProvidertype
pub type AliyunProvider = AliyunProviderImpl;

/// CreateAliyun DashScopeserviceProvider
/// 
/// # Parameters
/// - `api_key`: Aliyun DashScope API key
/// 
/// # Returns
/// Configured Aliyun service Provider instance
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::aliyun;
/// 
/// let provider = aliyun("sk-...").unwrap();
/// ```
pub fn aliyun(api_key: &str) -> Result<AliyunProvider, LlmConnectorError> {
    aliyun_with_config(api_key, None, None, None)
}

/// Create Aliyun service Provider with custom configuration
///
/// # Parameters
/// - `api_key`: API key
/// - `base_url`: Custom base URL (optional, defaults to official endpoint)
/// - `timeout_secs`: Timeout (seconds) (optional)
/// - `proxy`: Proxy URL (optional)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::aliyun_with_config;
///
/// let provider = aliyun_with_config(
///     "sk-...",
///     None, // Use default URL
///     Some(60), // 60 seconds timeout
///     Some("http://proxy:8080")
/// ).unwrap();
/// ```
pub fn aliyun_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<AliyunProvider, LlmConnectorError> {
    // CreateProtocol instance
    let protocol = AliyunProtocol::new(api_key);

    // Create HTTP Client (without streaming headers)
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://dashscope.aliyuncs.com"),
        timeout_secs,
        proxy,
    )?;

    // Add authentication headers
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);

    // Createcustom Aliyun Provider（Requires special handling for streaming requests）
    Ok(AliyunProviderImpl {
        protocol,
        client,
    })
}

/// Create Aliyun international service Provider
/// 
/// # Parameters
/// - `api_key`: Aliyun international API key
/// - `region`: Region (such as "us-west-1", "ap-southeast-1")
/// 
/// # Example
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

/// Create Aliyun private cloud service Provider
/// 
/// # Parameters
/// - `api_key`: API key
/// - `endpoint`: Private cloud endpoint URL
/// 
/// # Example
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

/// Create Aliyun service Provider with custom timeout
/// 
/// Some Aliyun models may require longer processing time，this function provides convenient timeout configuration。
/// 
/// # Parameters
/// - `api_key`: API key
/// - `timeout_secs`: Timeout (seconds)
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::aliyun_with_timeout;
/// 
/// // Set 120 seconds timeout, suitable for long text processing
/// let provider = aliyun_with_timeout("sk-...", 120).unwrap();
/// ```
pub fn aliyun_with_timeout(
    api_key: &str,
    timeout_secs: u64,
) -> Result<AliyunProvider, LlmConnectorError> {
    aliyun_with_config(api_key, None, Some(timeout_secs), None)
}

/// ValidateAliyun API keyformat
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

    #[test]
    fn test_enable_thinking_explicit_control() {
        use crate::types::{ChatRequest, Message, Role};

        let protocol = AliyunProtocol::new("test-key");

        // Test explicit enable
        let request = ChatRequest {
            model: "qwen-plus".to_string(),
            messages: vec![Message::text(Role::User, "test")],
            enable_thinking: Some(true),  // Explicitly enable
            ..Default::default()
        };

        let aliyun_request = protocol.build_request(&request).unwrap();
        assert_eq!(aliyun_request.parameters.enable_thinking, Some(true));

        // Test explicit disable
        let request = ChatRequest {
            model: "qwen-plus".to_string(),
            messages: vec![Message::text(Role::User, "test")],
            enable_thinking: Some(false),  // Explicitly disable
            ..Default::default()
        };

        let aliyun_request = protocol.build_request(&request).unwrap();
        assert_eq!(aliyun_request.parameters.enable_thinking, Some(false));

        // Test unspecified (default not enabled)
        let request = ChatRequest {
            model: "qwen-plus".to_string(),
            messages: vec![Message::text(Role::User, "test")],
            // enable_thinking not specified
            ..Default::default()
        };

        let aliyun_request = protocol.build_request(&request).unwrap();
        assert_eq!(aliyun_request.parameters.enable_thinking, None);
    }
}
