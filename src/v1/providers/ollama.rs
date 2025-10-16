//! Ollama Protocol Implementation
//!
//! This module implements the Ollama API protocol for local LLM servers.
//!
//! # Supported Providers
//!
//! - **Ollama (Local)** - `ollama()` - Llama3.2, Llama3.1, Mistral, Qwen, etc.
//!
//! # Protocol Differences
//!
//! Ollama uses a custom protocol that differs from OpenAI and Anthropic:
//!
//! ## 1. Endpoint
//! - Ollama: `POST /api/chat`
//! - OpenAI: `POST /v1/chat/completions`
//!
//! ## 2. Request Structure
//! - **No API key**: Local authentication only
//! - **Model field": Required, model must be pulled first
//! - **Options**: Separate options object for generation parameters
//!
//! ## 3. Response Structure
//! - **Done flag**: Indicates if generation is complete
//! - **Timing info**: Includes duration and evaluation metrics
//! - **Usage**: Different token counting approach
//!
//! # Request Format
//!
//! ```json
//! {
//!   "model": "llama3.2",
//!   "messages": [
//!     {"role": "user", "content": "Hello"}
//!   ],
//!   "stream": false,
//!   "options": {
//!     "temperature": 0.7,
//!     "num_predict": 1000
//!   }
//! }
//! ```
//!
//! # Response Format
//!
//! ```json
//! {
//!   "model": "llama3.2",
//!   "created_at": "2024-01-01T00:00:00Z",
//!   "message": {
//!     "role": "assistant",
//!     "content": "Hello! How can I help you?"
//!   },
//!   "done": true,
//!   "eval_count": 20,
//!   "eval_duration": 1000000
//! }
//! ```

use crate::v1::protocols::core::Provider;
use crate::v1::core::protocol::ProtocolError;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, Choice, Message, Role, Usage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[cfg(feature = "streaming")]
use crate::types::{ChatStream, StreamingResponse};

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

// ============================================================================
// Ollama-specific Request/Response Structures
// ============================================================================

/// Ollama chat request
#[derive(Serialize, Debug)]
pub struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Serialize, Debug)]
pub struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Debug)]
pub struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
}

/// Ollama chat response
#[derive(Deserialize, Debug)]
pub struct OllamaResponse {
    model: String,
    #[serde(default)]
    _created_at: String,
    message: OllamaResponseMessage,
    done: bool,
    #[serde(default)]
    eval_count: Option<u32>,
}

/// Ollama model information
#[derive(Deserialize, Debug, Clone)]
pub struct OllamaModel {
    pub name: String,
    pub model: String,
    pub modified_at: String,
    pub size: Option<u64>,
    pub digest: Option<String>,
    pub details: Option<OllamaModelDetails>,
    pub expires_at: Option<String>,
}

/// Ollama model details
#[derive(Deserialize, Debug, Clone)]
pub struct OllamaModelDetails {
    pub format: Option<String>,
    pub family: Option<String>,
    pub families: Option<Vec<String>>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

/// Ollama models list response
#[derive(Deserialize, Debug)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModel>,
}

/// Ollama model pull/push request
#[derive(Serialize, Debug)]
pub struct OllamaModelRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// Ollama model delete request
#[derive(Serialize, Debug)]
pub struct OllamaModelDeleteRequest {
    pub name: String,
}

/// Ollama model operation response (for pull/push progress)
#[derive(Deserialize, Debug)]
pub struct OllamaModelProgressResponse {
    pub status: String,
    #[serde(default)]
    pub digest: Option<String>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub completed: Option<u64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OllamaResponseMessage {
    role: String,
    content: String,
}

/// Ollama streaming response line (JSON)
#[cfg(feature = "streaming")]
#[derive(Deserialize, Debug, Clone)]
pub struct OllamaStreamResponse {
    pub model: String,
    #[serde(default)]
    pub created_at: Option<String>,
    pub message: OllamaResponseMessage,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub done_reason: Option<String>,
    #[serde(default)]
    pub eval_count: Option<u32>,
}

// ============================================================================
// Ollama Error Mapper
// ============================================================================

pub struct OllamaErrorMapper;

impl ProtocolError for OllamaErrorMapper {
    fn map_http_error(status: u16, body: Value) -> LlmConnectorError {
        let error_message = body["error"]
            .as_str()
            .or_else(|| body["message"].as_str())
            .unwrap_or("Unknown Ollama error");

        match status {
            400 => LlmConnectorError::InvalidRequest(format!(
                "Ollama: {}",
                error_message
            )),
            404 => LlmConnectorError::InvalidRequest(format!(
                "Ollama: Model not found. Make sure to pull the model first with 'ollama pull <model>'"
            )),
            500 => LlmConnectorError::ServerError(format!(
                "Ollama: Server error. Is Ollama running on localhost:11434?"
            )),
            429 => LlmConnectorError::RateLimitError(format!(
                "Ollama: {}",
                error_message
            )),
            _ => LlmConnectorError::ProviderError(format!(
                "Ollama HTTP {}: {}",
                status, error_message
            )),
        }
    }

    fn map_network_error(error: reqwest::Error) -> LlmConnectorError {
        if error.is_timeout() {
            LlmConnectorError::TimeoutError(format!("Ollama: {}", error))
        } else if error.is_connect() {
            LlmConnectorError::ConnectionError(format!(
                "Ollama: Cannot connect to Ollama server. Is it running on localhost:11434?"
            ))
        } else {
            LlmConnectorError::NetworkError(format!("Ollama: {}", error))
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
// Ollama Provider
// ============================================================================

/// Ollama Provider implementation for local LLM servers
#[derive(Debug, Clone)]
pub struct OllamaProvider {
    name: Arc<str>,
    base_url: Arc<str>,
    client: reqwest::Client,
}

impl OllamaProvider {
    /// Create new Ollama provider with default localhost:11434
    pub fn new() -> Self {
        Self {
            name: Arc::from("ollama"),
            base_url: Arc::from("http://localhost:11434"),
            client: reqwest::Client::builder()
                .no_proxy()  // 绕过代理以避免 502 错误
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Create new Ollama provider with custom URL
    pub fn with_url(base_url: &str) -> Self {
        Self {
            name: Arc::from("ollama"),
            base_url: Arc::from(base_url),
            client: reqwest::Client::builder()
                .no_proxy()  // 绕过代理以避免 502 错误
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// List all available models
    pub async fn list_models(&self) -> Result<Vec<String>, LlmConnectorError> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| OllamaErrorMapper::map_network_error(e))?;

        if response.status().is_success() {
            let models_response: OllamaModelsResponse = response.json().await
                .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;

            Ok(models_response.models.into_iter().map(|m| m.name).collect())
        } else {
            let status = response.status().as_u16();
            let body = response.json().await.unwrap_or_default();
            Err(OllamaErrorMapper::map_http_error(status, body))
        }
    }

    /// Pull a model from Ollama registry
    pub async fn pull_model(&self, model_name: &str) -> Result<(), LlmConnectorError> {
        let url = format!("{}/api/pull", self.base_url);

        let request = OllamaModelRequest {
            name: model_name.to_string(),
            insecure: None,
            stream: Some(false),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| OllamaErrorMapper::map_network_error(e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let body = response.json().await.unwrap_or_default();
            Err(OllamaErrorMapper::map_http_error(status, body))
        }
    }

    /// Push a model to Ollama registry
    pub async fn push_model(&self, model_name: &str) -> Result<(), LlmConnectorError> {
        let url = format!("{}/api/push", self.base_url);

        let request = OllamaModelRequest {
            name: model_name.to_string(),
            insecure: None,
            stream: Some(false),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| OllamaErrorMapper::map_network_error(e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let body = response.json().await.unwrap_or_default();
            Err(OllamaErrorMapper::map_http_error(status, body))
        }
    }

    /// Delete a model
    pub async fn delete_model(&self, model_name: &str) -> Result<(), LlmConnectorError> {
        let url = format!("{}/api/delete", self.base_url);

        let request = OllamaModelDeleteRequest {
            name: model_name.to_string(),
        };

        let response = self.client
            .delete(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| OllamaErrorMapper::map_network_error(e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let body = response.json().await.unwrap_or_default();
            Err(OllamaErrorMapper::map_http_error(status, body))
        }
    }

    /// Get model details
    pub async fn show_model(&self, model_name: &str) -> Result<OllamaModel, LlmConnectorError> {
        let url = format!("{}/api/show", self.base_url);

        let request = serde_json::json!({
            "name": model_name
        });

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| OllamaErrorMapper::map_network_error(e))?;

        if response.status().is_success() {
            let model_info: OllamaModel = response.json().await
                .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;
            Ok(model_info)
        } else {
            let status = response.status().as_u16();
            let body = response.json().await.unwrap_or_default();
            Err(OllamaErrorMapper::map_http_error(status, body))
        }
    }

    /// Build Ollama request from generic request
    fn build_request(&self, request: &ChatRequest, stream: bool) -> OllamaRequest {
        let messages = request
            .messages
            .iter()
            .map(|msg| OllamaMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect();

        OllamaRequest {
            model: request.model.clone(),
            messages,
            stream: Some(stream),
            options: Some(OllamaOptions {
                temperature: request.temperature,
                top_p: request.top_p,
                num_predict: request.max_tokens,
            }),
        }
    }

    /// Parse Ollama response to generic response
    fn parse_response(&self, response: OllamaResponse) -> ChatResponse {
        let first_content = response.message.content.clone();
        ChatResponse {
            id: format!("ollama-{}", response.model),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: response.model,
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: parse_role(&response.message.role),
                    content: response.message.content,
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    ..Default::default()
                },
                finish_reason: if response.done {
                    Some("stop".to_string())
                } else {
                    Some("length".to_string())
                },
                logprobs: None,
            }],
            content: first_content,
            usage: Some(Usage {
                prompt_tokens: 0, // Ollama doesn't provide prompt tokens
                completion_tokens: response.eval_count.unwrap_or(0),
                total_tokens: response.eval_count.unwrap_or(0),
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        }
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response(&self, response: OllamaStreamResponse) -> StreamingResponse {
        let content = response.message.content.clone();
        let role = parse_role(&response.message.role);
        StreamingResponse {
            id: format!("ollama-{}", response.model),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: response.model,
            choices: vec![crate::types::StreamingChoice {
                index: 0,
                delta: crate::types::Delta {
                    role: Some(role),
                    content: Some(content.clone()),
                    tool_calls: None,
                    reasoning_content: None,
                    ..Default::default()
                },
                finish_reason: if response.done { response.done_reason.clone().or(Some("stop".to_string())) } else { None },
                logprobs: None,
            }],
            content,
            reasoning_content: None,
            usage: None,
            system_fingerprint: None,
        }
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        let ollama_request = self.build_request(request, false);
        let url = format!("{}/api/chat", self.base_url);

        let response = self.client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| OllamaErrorMapper::map_network_error(e))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body: Value = response.json().await.unwrap_or_default();
            return Err(OllamaErrorMapper::map_http_error(status, body));
        }

        let text = response
            .text()
            .await
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;

        let ollama_response: OllamaResponse = serde_json::from_str(&text)
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;

        Ok(self.parse_response(ollama_response))
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        use futures_util::stream;

        let ollama_request = self.build_request(request, true);
        let url = format!("{}/api/chat", self.base_url);

        let response = self.client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| OllamaErrorMapper::map_network_error(e))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body: Value = response.json().await.unwrap_or_default();
            return Err(OllamaErrorMapper::map_http_error(status, body));
        }

        // For now, return a single response stream for Ollama
        let text = response
            .text()
            .await
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;

        let ollama_response: OllamaResponse = serde_json::from_str(&text)
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;

        let streaming_response = self.parse_stream_response(OllamaStreamResponse {
            model: ollama_response.model,
            created_at: Some(ollama_response._created_at),
            message: ollama_response.message,
            done: ollama_response.done,
            done_reason: None,
            eval_count: ollama_response.eval_count,
        });

        let single_chunk_stream = stream::once(async { Ok(streaming_response) });
        Ok(Box::pin(single_chunk_stream))
    }

    async fn fetch_models(&self) -> Result<Vec<String>, LlmConnectorError> {
        self.list_models().await
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}



// ============================================================================
// Convenience Functions
// ============================================================================

/// Create an Ollama provider
pub fn ollama() -> OllamaProvider {
    OllamaProvider::new()
}

/// Create an Ollama provider with custom URL
pub fn ollama_with_url(base_url: &str) -> OllamaProvider {
    OllamaProvider::with_url(base_url)
}
