//! Aliyun Protocol Implementation
//!
//! This module implements the Aliyun DashScope API protocol.
//!
//! # Supported Providers
//!
//! - **Aliyun (DashScope)** - `qwen()` - Qwen-Max, Qwen-Plus, Qwen-Turbo
//!
//! # Protocol Differences
//!
//! Aliyun uses a custom protocol that differs significantly from both OpenAI and Anthropic:
//!
//! ## 1. Endpoint
//! - Aliyun: `POST /services/aigc/text-generation/generation`
//! - OpenAI: `POST /v1/chat/completions`
//!
//! ## 2. Request Structure
//! - **Nested structure**: Uses `input` and `parameters` objects
//! - **Model field**: At top level, not in parameters
//! - **Result format**: Explicit `result_format` field for response type
//!
//! ## 3. Response Structure
//! - **Nested output**: Response data in `output.choices` instead of top-level `choices`
//! - **Request ID**: Includes `request_id` for tracking
//! - **Usage**: Different field structure
//!
//! ## 4. Authentication
//! - Uses `Authorization: Bearer <api-key>` header
//! - API key format: `sk-...`
//!
//! # Request Format
//!
//! ```json
//! {
//!   "model": "qwen-max",
//!   "input": {
//!     "messages": [
//!       {"role": "user", "content": "Hello"}
//!     ]
//!   },
//!   "parameters": {
//!     "max_tokens": 1000,
//!     "temperature": 0.7,
//!     "result_format": "message"
//!   }
//! }
//! ```
//!
//! # Response Format
//!
//! ```json
//! {
//!   "request_id": "req_123",
//!   "output": {
//!     "choices": [{
//!       "message": {
//!         "role": "assistant",
//!         "content": "Hello! How can I help you?"
//!       },
//!       "finish_reason": "stop"
//!     }]
//!   },
//!   "usage": {
//!     "input_tokens": 10,
//!     "output_tokens": 20,
//!     "total_tokens": 30
//!   }
//! }
//! ```
//!
//! # Example
//!
//! ```rust
//! use llm_connector::{LlmClient};
//! use llm_connector::types::{ChatRequest, Message};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create Aliyun client (DashScope)
//! let client = LlmClient::aliyun("your-api-key");
//!
//! // Create request
//! let request = ChatRequest {
//!     model: "qwen-max".to_string(),
//!     messages: vec![Message::user("Hello!")],
//!     ..Default::default()
//! };
//!
//! // Send request
//! let response = client.chat(&request).await?;
//! println!("Response: {}", response.choices[0].message.content);
//! # Ok(())
//! # }
//! ```

use crate::core::Provider;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest as Request, ChatResponse as Response, Role, Usage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Parse a role string into a Role enum
fn parse_role(role: &str) -> Role {
    match role {
        "system" => Role::System,
        "user" => Role::User,
        "assistant" => Role::Assistant,
        "tool" => Role::Tool,
        _ => Role::User, // Default to user for unknown roles
    }
}

#[cfg(feature = "streaming")]
use crate::types::{StreamingResponse, ChatStream};

// ============================================================================
// Aliyun-specific Request Structures
// ============================================================================

#[derive(Debug, Serialize)]
pub struct AliyunRequest {
    model: String,
    input: AliyunInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<AliyunParameters>,
}

#[derive(Debug, Serialize)]
pub struct AliyunInput {
    messages: Vec<AliyunMessage>,
}

#[derive(Debug, Serialize)]
pub struct AliyunMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Default)]
pub struct AliyunParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    incremental_output: Option<bool>,
}

// ============================================================================
// Aliyun-specific Response Structures
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AliyunResponse {
    request_id: String,
    output: AliyunOutput,
    usage: AliyunUsage,
}

#[derive(Debug, Deserialize)]
pub struct AliyunOutput {
    choices: Vec<AliyunChoice>,
}

#[derive(Debug, Deserialize)]
pub struct AliyunChoice {
    message: AliyunResponseMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct AliyunResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct AliyunUsage {
    input_tokens: i32,
    output_tokens: i32,
}

// ============================================================================
// Aliyun-specific Streaming Response
// ============================================================================

#[cfg(feature = "streaming")]
#[derive(Debug, Deserialize)]
pub struct AliyunStreamResponse {
    request_id: String,
    output: AliyunStreamOutput,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<AliyunUsage>,
}

#[cfg(feature = "streaming")]
#[derive(Debug, Deserialize)]
pub struct AliyunStreamOutput {
    choices: Vec<AliyunStreamChoice>,
}

#[cfg(feature = "streaming")]
#[derive(Debug, Deserialize)]
pub struct AliyunStreamChoice {
    message: AliyunResponseMessage,
    finish_reason: Option<String>,
}

// ============================================================================
// Legacy Error Mapper (to be removed after migration)
// ============================================================================

#[deprecated(note = "Error mapping is now handled directly in AliyunProvider")]
pub struct AliyunErrorMapper;

// ============================================================================
// Aliyun Adapter Implementation
// ============================================================================

/// Aliyun Provider for DashScope API
///
/// Direct implementation of the Provider trait for Aliyun's custom API.
#[derive(Debug, Clone)]
pub struct AliyunProvider {
    name: Arc<str>,
    base_url: Arc<str>,
    api_key: Arc<str>,
    client: reqwest::Client,
}

impl AliyunProvider {
    /// Create new Aliyun provider with API key and default base URL
    pub fn new(api_key: &str) -> Self {
        Self {
            name: Arc::from("aliyun"),
            base_url: Arc::from("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation"),
            api_key: Arc::from(api_key),
            client: reqwest::Client::new(),
        }
    }

    /// Create new Aliyun provider with API key and custom base URL
    pub fn with_url(api_key: &str, base_url: &str) -> Self {
        Self {
            name: Arc::from("aliyun"),
            base_url: Arc::from(base_url),
            api_key: Arc::from(api_key),
            client: reqwest::Client::new(),
        }
    }

    /// Send HTTP POST request to Aliyun API
    async fn post_request<T: Serialize, R: serde::de::DeserializeOwned>(
        &self,
        request_body: &T,
    ) -> Result<R, LlmConnectorError> {
        let response = self
            .client
            .post(&*self.base_url)
            .header("Authorization", format!("Bearer {}", &self.api_key))
            .header("Content-Type", "application/json")
            .json(request_body)
            .send()
            .await
            .map_err(LlmConnectorError::from)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body: Value = response.json().await.unwrap_or_default();
            return Err(self.map_http_error(status, body));
        }

        let text = response
            .text()
            .await
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;

        serde_json::from_str::<R>(&text)
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))
    }

    /// Map HTTP errors to LlmConnectorError
    fn map_http_error(&self, status: u16, body: Value) -> LlmConnectorError {
        let error_message = body["error"]["message"]
            .as_str()
            .or_else(|| body["message"].as_str())
            .unwrap_or("Unknown Aliyun error");

        let error_code = body["error"]["code"]
            .as_str()
            .or_else(|| body["code"].as_str())
            .unwrap_or("unknown");

        match status {
            400 => LlmConnectorError::InvalidRequest(format!(
                "Aliyun: {} ({})",
                error_message, error_code
            )),
            401 => LlmConnectorError::AuthenticationError(format!(
                "Aliyun: {} ({})",
                error_message, error_code
            )),
            403 => LlmConnectorError::PermissionError(format!(
                "Aliyun: {} ({})",
                error_message, error_code
            )),
            429 => LlmConnectorError::RateLimitError(format!(
                "Aliyun: {} ({})",
                error_message, error_code
            )),
            500..=599 => LlmConnectorError::ServerError(format!(
                "Aliyun HTTP {}: {} ({})",
                status, error_message, error_code
            )),
            _ => LlmConnectorError::ProviderError(format!(
                "Aliyun HTTP {}: {} ({})",
                status, error_message, error_code
            )),
        }
    }

    /// Build Aliyun-specific request from generic request
    fn build_request(&self, request: &Request, stream: bool) -> AliyunRequest {
        let messages = request
            .messages
            .iter()
            .map(|msg| AliyunMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect();

        let parameters = AliyunParameters {
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            seed: None, // Aliyun-specific field, could be configurable
            result_format: Some("message".to_string()),
            incremental_output: if stream { Some(true) } else { None },
        };

        AliyunRequest {
            model: request.model.clone(),
            input: AliyunInput { messages },
            parameters: Some(parameters),
        }
    }

    /// Parse Aliyun response to generic response
    fn parse_response(&self, response: AliyunResponse) -> Response {
        // Convenience: capture first choice content before moving choices
        let first_content = response
            .output
            .choices
            .get(0)
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Response {
            id: response.request_id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: "qwen".to_string(), // Aliyun doesn't return model in response
            choices: response
                .output
                .choices
                .into_iter()
                .enumerate()
                .map(|(index, choice)| crate::types::Choice {
                    index: index as u32,
                    message: crate::types::Message {
                        role: parse_role(&choice.message.role),
                        content: choice.message.content,
                        name: None,
                        tool_calls: None,
                        tool_call_id: None,
                        ..Default::default()
                    },
                    finish_reason: Some(choice.finish_reason),
                    logprobs: None,
                })
                .collect(),
            content: first_content,
            usage: Some(Usage {
                prompt_tokens: response.usage.input_tokens as u32,
                completion_tokens: response.usage.output_tokens as u32,
                total_tokens: (response.usage.input_tokens + response.usage.output_tokens) as u32,
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        }
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response(&self, response: AliyunStreamResponse) -> StreamingResponse {
        // Convenience: capture first chunk content before moving choices
        let first_chunk_content = response
            .output
            .choices
            .get(0)
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        StreamingResponse {
            id: response.request_id,
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: "qwen".to_string(),
            choices: response
                .output
                .choices
                .into_iter()
                .enumerate()
                .map(|(index, choice)| crate::types::StreamingChoice {
                    index: index as u32,
                    delta: crate::types::Delta {
                        role: Some(parse_role(&choice.message.role)),
                        content: Some(choice.message.content),
                        tool_calls: None,
                        reasoning_content: None,
                        ..Default::default()
                    },
                    finish_reason: choice.finish_reason,
                    logprobs: None,
                })
                .collect(),
            content: first_chunk_content,
            reasoning_content: None,
            usage: response.usage.map(|usage| Usage {
                prompt_tokens: usage.input_tokens as u32,
                completion_tokens: usage.output_tokens as u32,
                total_tokens: (usage.input_tokens + usage.output_tokens) as u32,
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        }
    }
}

impl Default for AliyunProvider {
    fn default() -> Self {
        Self::new("")
    }
}

#[async_trait]
impl Provider for AliyunProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn chat(&self, request: &Request) -> Result<Response, LlmConnectorError> {
        let ali_request = self.build_request(request, false);
        let ali_response = self.post_request::<AliyunRequest, AliyunResponse>(&ali_request).await?;

        Ok(self.parse_response(ali_response))
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &Request) -> Result<ChatStream, LlmConnectorError> {
        use futures_util::stream;

        let ali_request = self.build_request(request, true);
        let ali_response = self.post_request::<AliyunRequest, AliyunStreamResponse>(&ali_request).await?;

        let streaming_response = self.parse_stream_response(ali_response);

        // Convert single response to a stream with one chunk
        let single_chunk_stream = stream::once(async { Ok(streaming_response) });
        Ok(Box::pin(single_chunk_stream))
    }

    async fn fetch_models(&self) -> Result<Vec<String>, LlmConnectorError> {
        // Aliyun doesn't have a public models endpoint
        Err(LlmConnectorError::UnsupportedOperation(
            "Aliyun does not support model listing".to_string()
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ============================================================================
// Legacy Compatibility (to be removed after migration)
// ============================================================================

// ============================================================================
// Legacy Compatibility Layer (to be removed after migration)
// ============================================================================

use crate::protocols::{ProviderAdapter, ErrorMapper};

/// Legacy AliyunProtocol for backward compatibility
#[deprecated(note = "Use AliyunProvider instead")]
#[derive(Debug, Clone)]
pub struct AliyunProtocol {
    base_url: Arc<str>,
}

#[allow(deprecated)]
impl AliyunProtocol {
    /// Create new Aliyun protocol with API key
    pub fn new(_api_key: &str) -> Self {
        Self {
            base_url: Arc::from("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation"),
        }
    }

    /// Create new Aliyun protocol with custom API key and base URL
    pub fn with_url(_api_key: &str, base_url: &str) -> Self {
        Self {
            base_url: Arc::from(base_url),
        }
    }
}

#[allow(deprecated)]
#[async_trait]
impl ProviderAdapter for AliyunProtocol {
    type RequestType = AliyunRequest;
    type ResponseType = AliyunResponse;
    #[cfg(feature = "streaming")]
    type StreamResponseType = AliyunStreamResponse;
    type ErrorMapperType = AliyunErrorMapper;

    fn name(&self) -> &str {
        "aliyun"
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        base_url.as_deref().unwrap_or(&self.base_url).to_string()
    }

    fn models_endpoint_url(&self, _base_url: &Option<String>) -> Option<String> {
        None // Aliyun doesn't have models endpoint
    }

    fn build_request_data(&self, request: &crate::types::ChatRequest, stream: bool) -> Self::RequestType {
        let messages = request
            .messages
            .iter()
            .map(|msg| AliyunMessage {
                role: match msg.role {
                    crate::types::Role::System => "system".to_string(),
                    crate::types::Role::User => "user".to_string(),
                    crate::types::Role::Assistant => "assistant".to_string(),
                    crate::types::Role::Tool => "tool".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect();

        let parameters = AliyunParameters {
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            seed: None,
            result_format: Some("message".to_string()),
            incremental_output: if stream { Some(true) } else { None },
        };

        AliyunRequest {
            model: request.model.clone(),
            input: AliyunInput { messages },
            parameters: Some(parameters),
        }
    }

    fn parse_response_data(&self, response: Self::ResponseType) -> crate::types::ChatResponse {
        let first_content = response
            .output
            .choices
            .get(0)
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        crate::types::ChatResponse {
            id: response.request_id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: "qwen".to_string(),
            choices: response
                .output
                .choices
                .into_iter()
                .enumerate()
                .map(|(index, choice)| crate::types::Choice {
                    index: index as u32,
                    message: crate::types::Message {
                        role: parse_role(&choice.message.role),
                        content: choice.message.content,
                        name: None,
                        tool_calls: None,
                        tool_call_id: None,
                        ..Default::default()
                    },
                    finish_reason: Some(choice.finish_reason),
                    logprobs: None,
                })
                .collect(),
            content: first_content,
            usage: Some(Usage {
                prompt_tokens: response.usage.input_tokens as u32,
                completion_tokens: response.usage.output_tokens as u32,
                total_tokens: (response.usage.input_tokens + response.usage.output_tokens) as u32,
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        }
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> crate::types::StreamingResponse {
        let first_chunk_content = response
            .output
            .choices
            .get(0)
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        crate::types::StreamingResponse {
            id: response.request_id,
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: "qwen".to_string(),
            choices: response
                .output
                .choices
                .into_iter()
                .enumerate()
                .map(|(index, choice)| crate::types::StreamingChoice {
                    index: index as u32,
                    delta: crate::types::Delta {
                        role: Some(parse_role(&choice.message.role)),
                        content: Some(choice.message.content),
                        tool_calls: None,
                        reasoning_content: None,
                        ..Default::default()
                    },
                    finish_reason: choice.finish_reason,
                    logprobs: None,
                })
                .collect(),
            content: first_chunk_content,
            reasoning_content: None,
            usage: response.usage.map(|usage| Usage {
                prompt_tokens: usage.input_tokens as u32,
                completion_tokens: usage.output_tokens as u32,
                total_tokens: (usage.input_tokens + usage.output_tokens) as u32,
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        }
    }
}

#[allow(deprecated)]
impl ErrorMapper for AliyunErrorMapper {
    fn map_http_error(status: u16, body: Value) -> LlmConnectorError {
        let error_message = body["error"]["message"]
            .as_str()
            .or_else(|| body["message"].as_str())
            .unwrap_or("Unknown Aliyun error");

        let error_code = body["error"]["code"]
            .as_str()
            .or_else(|| body["code"].as_str())
            .unwrap_or("unknown");

        match status {
            400 => LlmConnectorError::InvalidRequest(format!(
                "Aliyun: {} ({})",
                error_message, error_code
            )),
            401 => LlmConnectorError::AuthenticationError(format!(
                "Aliyun: {} ({})",
                error_message, error_code
            )),
            403 => LlmConnectorError::PermissionError(format!(
                "Aliyun: {} ({})",
                error_message, error_code
            )),
            429 => LlmConnectorError::RateLimitError(format!(
                "Aliyun: {} ({})",
                error_message, error_code
            )),
            500..=599 => LlmConnectorError::ServerError(format!(
                "Aliyun HTTP {}: {} ({})",
                status, error_message, error_code
            )),
            _ => LlmConnectorError::ProviderError(format!(
                "Aliyun HTTP {}: {} ({})",
                status, error_message, error_code
            )),
        }
    }

    fn map_network_error(error: reqwest::Error) -> LlmConnectorError {
        if error.is_timeout() {
            LlmConnectorError::TimeoutError(format!("Aliyun: {}", error))
        } else if error.is_connect() {
            LlmConnectorError::ConnectionError(format!("Aliyun: {}", error))
        } else {
            LlmConnectorError::NetworkError(format!("Aliyun: {}", error))
        }
    }

    fn is_retriable_error(error: &LlmConnectorError) -> bool {
        matches!(
            error,
            LlmConnectorError::RateLimitError(_)
                | LlmConnectorError::ServerError(_)
                | LlmConnectorError::TimeoutError(_)
                | LlmConnectorError::ConnectionError(_)
        )
    }
}

// ============================================================================
// Convenience Functions and Type Aliases
// ============================================================================

/// Create an Aliyun provider
pub fn aliyun(api_key: &str) -> AliyunProvider {
    AliyunProvider::new(api_key)
}

/// Create an Aliyun provider with custom base URL
pub fn aliyun_with_url(api_key: &str, base_url: &str) -> AliyunProvider {
    AliyunProvider::with_url(api_key, base_url)
}

/// Bridge implementation: Implement old Provider trait for AliyunProvider
/// This allows gradual migration from the old architecture to the new one
#[async_trait]
impl crate::protocols::Provider for AliyunProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn chat(&self, request: &crate::types::ChatRequest) -> Result<crate::types::ChatResponse, LlmConnectorError> {
        Provider::chat(self, request).await
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &crate::types::ChatRequest) -> Result<crate::types::ChatStream, LlmConnectorError> {
        Provider::chat_stream(self, request).await
    }

    async fn fetch_models(&self) -> Result<Vec<String>, LlmConnectorError> {
        Provider::fetch_models(self).await
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

