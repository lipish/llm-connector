//! Zhipu GLM Service Provider Implementation - V2 Architecture
//!
//! This module provides complete Zhipu GLM service implementation, supporting native format and OpenAI compatible format.

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::error::LlmConnectorError;
use crate::types::{
    ChatRequest, ChatResponse, Choice, ImageSource, Message as TypeMessage, MessageBlock, Role,
    Tool, ToolChoice,
};
use serde_json::{Value, json};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Extract reasoning content from Zhipu response
///
/// Zhipu GLM-Z1 and other reasoning models embed reasoning process in content, using markers to separate:
/// - `###Thinking` Marks the start of reasoning process
/// - `###Response` Marks the start of final answer
///
/// # Parameters
/// - `content`: Original content string
///
/// # Returns
/// - `(reasoning_content, final_content)`: Reasoning content and final answer
fn extract_zhipu_reasoning_content(content: &str) -> (Option<String>, String) {
    // Check if contains reasoning markers
    if content.contains("###Thinking") && content.contains("###Response") {
        // Separate reasoning content and answer
        let parts: Vec<&str> = content.split("###Response").collect();
        if parts.len() >= 2 {
            let thinking = parts[0].replace("###Thinking", "").trim().to_string();
            let response = parts[1..].join("###Response").trim().to_string();

            if !thinking.is_empty() {
                return (Some(thinking), response);
            }
        }
    }

    // If no reasoning markers, return original content
    (None, content.to_string())
}

/// Zhipu streaming response processing stage
#[cfg(feature = "streaming")]
#[derive(Debug, Clone, PartialEq)]
enum ZhipuStreamPhase {
    /// Initial state, waiting to detect if is reasoning model
    Initial,
    /// In reasoning stage (after ###Thinking, before ###Response)
    InThinking,
    /// In answer stage (after ###Response)
    InResponse,
}

/// Zhipu streaming response state machine
#[cfg(feature = "streaming")]
struct ZhipuStreamState {
    /// Buffer for accumulating content
    buffer: String,
    /// Current processing stage
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

    /// Process streaming content delta
    ///
    /// # Returns
    /// - `(reasoning_delta, content_delta)`: Reasoning content delta and answer content delta
    fn process(&mut self, delta_content: &str) -> (Option<String>, Option<String>) {
        self.buffer.push_str(delta_content);

        match self.phase {
            ZhipuStreamPhase::Initial => {
                // Detect if contains ###Thinking marker
                if self.buffer.contains("###Thinking") {
                    // Remove marker and enter reasoning stage
                    self.buffer = self
                        .buffer
                        .replace("###Thinking", "")
                        .trim_start()
                        .to_string();
                    self.phase = ZhipuStreamPhase::InThinking;

                    // Check if immediately contains ###Response (complete reasoning in one chunk)
                    if self.buffer.contains("###Response") {
                        return self.handle_response_marker();
                    }

                    // Return current buffer as reasoning content
                    let reasoning = self.buffer.clone();
                    self.buffer.clear();
                    (Some(reasoning), None)
                } else {
                    // Not a reasoning model, return content directly
                    let content = self.buffer.clone();
                    self.buffer.clear();
                    (None, Some(content))
                }
            }
            ZhipuStreamPhase::InThinking => {
                // Detect if contains ###Response marker
                if self.buffer.contains("###Response") {
                    self.handle_response_marker()
                } else {
                    // Continue accumulating reasoning content
                    let reasoning = self.buffer.clone();
                    self.buffer.clear();
                    (Some(reasoning), None)
                }
            }
            ZhipuStreamPhase::InResponse => {
                // In answer stage, return content directly
                let content = self.buffer.clone();
                self.buffer.clear();
                (None, Some(content))
            }
        }
    }

    /// Process ###Response marker
    fn handle_response_marker(&mut self) -> (Option<String>, Option<String>) {
        let parts: Vec<&str> = self.buffer.split("###Response").collect();
        if parts.len() >= 2 {
            // Reasoning part (before ###Response)
            let thinking = parts[0].trim();
            let reasoning = if !thinking.is_empty() {
                Some(thinking.to_string())
            } else {
                None
            };

            // Answer part (after ###Response)
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
            // Should not happen, but for safety
            (None, None)
        }
    }
}

// ============================================================================
// Zhipu Protocol Definition (Private)
// ============================================================================

/// Zhipu GLM private protocol implementation
///
/// Zhipu supports OpenAI compatible format, but has its own authentication and error handling.
/// Since this is a private protocol, it is defined inside the provider rather than in the public protocols module.
#[derive(Clone, Debug)]
pub struct ZhipuProtocol {
    api_key: String,
    use_openai_format: bool,
}

impl ZhipuProtocol {
    /// Create new Zhipu Protocol instance (using native format)
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            use_openai_format: false,
        }
    }

    /// Create Zhipu Protocol instance using OpenAI compatible format
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

    /// Whether to use OpenAI compatible format
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
            // Note: Content-Type is automatically set by HttpClient::post() .json() method
            // Do not set repeatedly here, otherwise may cause duplicate headers error
        ]
    }

    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        // Zhipu uses OpenAI compatible format
        let messages: Vec<ZhipuMessage> = request
            .messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "tool".to_string(),
                };

                // Check if message contains image content
                let has_image = msg.content.iter().any(|block| block.is_image());

                let content = if has_image {
                    // Multi-modal: build content array with text and image_url blocks
                    let blocks: Vec<Value> = msg.content.iter().map(|block| {
                        match block {
                            MessageBlock::Text { text } => json!({
                                "type": "text",
                                "text": text
                            }),
                            MessageBlock::Image { source } => json!({
                                "type": "image_url",
                                "image_url": {
                                    "url": match source {
                                        ImageSource::Base64 { media_type, data } => format!("data:{};base64,{}", media_type, data),
                                        ImageSource::Url { url } => url.clone(),
                                    }
                                }
                            }),
                            MessageBlock::ImageUrl { image_url } => json!({
                                "type": "image_url",
                                "image_url": { "url": image_url.url }
                            }),
                            MessageBlock::Document { .. } => json!({
                                "type": "text",
                                "text": "[Document content not supported by Zhipu provider]"
                            }),
                        }
                    }).collect();
                    json!(blocks)
                } else {
                    // Text only: use plain string
                    json!(msg.content_as_text())
                };

                ZhipuMessage {
                    role,
                    content,
                    tool_calls: msg.tool_calls.as_ref().map(|calls| {
                        calls.iter().map(|c| serde_json::to_value(c).unwrap_or_default()).collect()
                    }),
                    tool_call_id: msg.tool_call_id.clone(),
                    name: msg.name.clone(),
                }
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

        if let Some(choices) = parsed.choices
            && let Some(first_choice) = choices.first()
        {
            // Convert ZhipuMessage to TypeMessage
            // Extract reasoning content (if exists)
            let content_str = match &first_choice.message.content {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            let (reasoning_content, final_content) = extract_zhipu_reasoning_content(&content_str);

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
                    calls
                        .iter()
                        .filter_map(|v| serde_json::from_value(v.clone()).ok())
                        .collect()
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

        Err(LlmConnectorError::InvalidRequest(
            "Empty or invalid response".to_string(),
        ))
    }

    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError {
        // Detect context length exceeded from error body
        let body_lower = body.to_lowercase();
        if body_lower.contains("context_length_exceeded")
            || body_lower.contains("maximum context length")
            || body_lower.contains("token limit")
        {
            return LlmConnectorError::ContextLengthExceeded(format!("Zhipu: {}", body));
        }
        LlmConnectorError::from_status_code(status, format!("Zhipu API error: {}", body))
    }

    /// Zhipu-specific streaming parser
    ///
    /// Zhipu API uses single newline to separate SSE events, not standard double newline
    /// Format: data: {...}\n instead of data: {...}\n\n
    #[cfg(feature = "streaming")]
    async fn parse_stream_response(
        &self,
        response: reqwest::Response,
    ) -> Result<crate::types::ChatStream, LlmConnectorError> {
        use crate::types::StreamingResponse;
        use futures_util::StreamExt;

        // Use new robust text stream creator with NDJSON mode (single newline separator)
        // Zhipu's "SSE" is actually more like NDJSON where each line starts with "data:"
        let events_stream =
            crate::sse::create_text_stream(response, crate::sse::StreamFormat::NdJson);

        // Convert JSON string stream to StreamingResponse stream
        // Use state machine to handle Zhipu ###Thinking and ###Response markers
        let response_stream = events_stream.scan(ZhipuStreamState::new(), |state, result| {
            let processed = result.and_then(|json_str| {
                let mut response =
                    serde_json::from_str::<StreamingResponse>(&json_str).map_err(|e| {
                        LlmConnectorError::ParseError(format!(
                            "Failed to parse Zhipu streaming response: {}. JSON: {}",
                            e, json_str
                        ))
                    })?;

                // Process reasoning content markers
                if let Some(first_choice) = response.choices.first_mut()
                    && let Some(ref delta_content) = first_choice.delta.content
                {
                    // Use state machine to process content
                    let (reasoning_delta, content_delta) = state.process(delta_content);

                    // Update delta
                    if let Some(reasoning) = reasoning_delta {
                        first_choice.delta.reasoning_content = Some(reasoning);
                    }

                    if let Some(content) = content_delta {
                        first_choice.delta.content = Some(content.clone());
                        // Also update response.content
                        response.content = content;
                    } else {
                        // If no content delta, clear delta.content
                        first_choice.delta.content = None;
                        response.content = String::new();
                    }
                }

                Ok(response)
            });

            std::future::ready(Some(processed))
        });

        Ok(Box::pin(response_stream))
    }
}

// Zhipu-specific data structure (OpenAI compatible format)
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
    pub content: Value,
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

/// Zhipu GLM service provider type
pub type ZhipuProvider = GenericProvider<ZhipuProtocol>;

/// Create Zhipu GLM service provider (using native format)
///
/// # Parameters
/// - `api_key`: Zhipu GLM API key
///
/// # Returns
/// Configured Zhipu service provider instance
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

/// Create Zhipu GLM service provider (using OpenAI compatible format)
///
/// # Parameters
/// - `api_key`: Zhipu GLM API key
///
/// # Returns
/// Configured Zhipu service provider instance (OpenAI compatible mode)
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

/// Create Zhipu GLM service provider with custom configuration
///
/// # Parameters
/// - `api_key`: API key
/// - `openai_compatible`: Whether to use OpenAI compatible format
/// - `base_url`: Custom base URL (optional)
/// - `timeout_secs`: Timeout (seconds) (optional)
/// - `proxy`: Proxy URL (optional)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::zhipu_with_config;
///
/// let provider = zhipu_with_config(
///     "your-api-key",
///     true, // Use OpenAI compatible format
///     None, // Use default URL
///     Some(60), // 60 seconds timeout
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
    // CreateProtocol instance
    let protocol = if openai_compatible {
        ZhipuProtocol::new_openai_compatible(api_key)
    } else {
        ZhipuProtocol::new(api_key)
    };

    // CreateHTTP Client
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://open.bigmodel.cn"),
        timeout_secs,
        proxy,
    )?;

    // Add authentication headers
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);

    // Create generic provider
    Ok(GenericProvider::new(protocol, client))
}

/// Create Zhipu GLM service provider with custom timeout
///
/// # Parameters
/// - `api_key`: API key
/// - `timeout_secs`: Timeout (seconds)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::zhipu_with_timeout;
///
/// // Set 120 seconds timeout
/// let provider = zhipu_with_timeout("your-api-key", 120).unwrap();
/// ```
pub fn zhipu_with_timeout(
    api_key: &str,
    timeout_secs: u64,
) -> Result<ZhipuProvider, LlmConnectorError> {
    zhipu_with_config(api_key, true, None, Some(timeout_secs), None)
}

/// Create Zhipu GLM enterprise service provider
///
/// # Parameters
/// - `api_key`: Enterprise API key
/// - `enterprise_endpoint`: Enterprise endpoint URL
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

/// Validate Zhipu GLM API key format
///
/// # Parameters
/// - `api_key`: API key to validate
///
/// # Returns
/// Returns true if format looks correct, otherwise returns false
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
        // Test case with reasoning content
        let content_with_thinking = "###Thinking\nthis_is_reasoning_process\nanalysis_step_1\nanalysis_step_2\n###Response\nthis_is_final_answer";
        let (reasoning, answer) = extract_zhipu_reasoning_content(content_with_thinking);
        assert!(reasoning.is_some());
        assert_eq!(
            reasoning.unwrap(),
            "this_is_reasoning_process\nanalysis_step_1\nanalysis_step_2"
        );
        assert_eq!(answer, "this_is_final_answer");

        // Test case without reasoning content
        let content_without_thinking = "this_is_a_normal_answer";
        let (reasoning, answer) = extract_zhipu_reasoning_content(content_without_thinking);
        assert!(reasoning.is_none());
        assert_eq!(answer, "this_is_a_normal_answer");

        // Test case with only Thinking, no Response
        let content_only_thinking = "###Thinking\nthis_is_reasoning_process";
        let (reasoning, answer) = extract_zhipu_reasoning_content(content_only_thinking);
        assert!(reasoning.is_none());
        assert_eq!(answer, "###Thinking\nthis_is_reasoning_process");

        // Test case with empty reasoning content
        let content_empty_thinking = "###Thinking\n\n###Response\nanswer";
        let (reasoning, answer) = extract_zhipu_reasoning_content(content_empty_thinking);
        assert!(reasoning.is_none());
        assert_eq!(answer, "###Thinking\n\n###Response\nanswer");
    }

    #[test]
    fn test_zhipu_build_request_text_only() {
        use crate::types::Message;
        let protocol = ZhipuProtocol::new("test-key");
        let request = ChatRequest {
            model: "glm-4".to_string(),
            messages: vec![Message::user("Hello")],
            ..Default::default()
        };
        let zhipu_req = protocol.build_request(&request).unwrap();
        // Text-only message should produce a plain string content
        assert_eq!(zhipu_req.messages[0].content, json!("Hello"));
    }

    #[test]
    fn test_zhipu_build_request_with_image_url() {
        use crate::types::Message;
        let protocol = ZhipuProtocol::new("test-key");

        let mut msg = Message::user("Describe this image");
        msg.content
            .push(MessageBlock::image_url("https://example.com/cat.jpg"));

        let request = ChatRequest {
            model: "glm-4v".to_string(),
            messages: vec![msg],
            ..Default::default()
        };
        let zhipu_req = protocol.build_request(&request).unwrap();
        let content = &zhipu_req.messages[0].content;

        // Should be an array with text + image_url blocks
        assert!(content.is_array());
        let arr = content.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["type"], "text");
        assert_eq!(arr[0]["text"], "Describe this image");
        assert_eq!(arr[1]["type"], "image_url");
        assert_eq!(arr[1]["image_url"]["url"], "https://example.com/cat.jpg");
    }

    #[test]
    fn test_zhipu_build_request_with_base64_image() {
        use crate::types::Message;
        let protocol = ZhipuProtocol::new("test-key");

        let mut msg = Message::user("What is this?");
        msg.content
            .push(MessageBlock::image_base64("image/jpeg", "abc123"));

        let request = ChatRequest {
            model: "glm-4v".to_string(),
            messages: vec![msg],
            ..Default::default()
        };
        let zhipu_req = protocol.build_request(&request).unwrap();
        let content = &zhipu_req.messages[0].content;

        assert!(content.is_array());
        let arr = content.as_array().unwrap();
        assert_eq!(arr[1]["type"], "image_url");
        assert_eq!(arr[1]["image_url"]["url"], "data:image/jpeg;base64,abc123");
    }

    #[cfg(feature = "streaming")]
    #[test]
    fn test_zhipu_stream_state() {
        // Test reasoning model streaming response
        let mut state = ZhipuStreamState::new();

        // First chunk: ###Thinking
        let (reasoning, content) = state.process("###Thinking\nstart");
        assert_eq!(reasoning, Some("start".to_string()));
        assert_eq!(content, None);

        // Second chunk: Reasoning process
        let (reasoning, content) = state.process("reasoning");
        assert_eq!(reasoning, Some("reasoning".to_string()));
        assert_eq!(content, None);

        // Third chunk: ###Response
        let (reasoning, content) = state.process("process\n###Response\nanswer");
        assert_eq!(reasoning, Some("process".to_string()));
        assert_eq!(content, Some("answer".to_string()));

        // Fourth chunk: Continue answer
        let (reasoning, content) = state.process("continue");
        assert_eq!(reasoning, None);
        assert_eq!(content, Some("continue".to_string()));
    }

    #[cfg(feature = "streaming")]
    #[test]
    fn test_zhipu_stream_state_non_reasoning() {
        // Test non-reasoning model streaming response
        let mut state = ZhipuStreamState::new();

        // First chunk: Normal content
        let (reasoning, content) = state.process("this_is");
        assert_eq!(reasoning, None);
        assert_eq!(content, Some("this_is".to_string()));

        // Second chunk: Continue content
        let (reasoning, content) = state.process("normal_answer");
        assert_eq!(reasoning, None);
        assert_eq!(content, Some("normal_answer".to_string()));
    }

    #[cfg(feature = "streaming")]
    #[test]
    fn test_zhipu_stream_state_complete_in_one_chunk() {
        // Test complete reasoning in one chunk
        let mut state = ZhipuStreamState::new();

        let (reasoning, content) =
            state.process("###Thinking\nReasoning process\n###Response\nanswer");
        assert_eq!(reasoning, Some("Reasoning process".to_string()));
        assert_eq!(content, Some("answer".to_string()));
    }
}
