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

use crate::error::LlmConnectorError;
use crate::protocols::core::{ErrorMapper, ProviderAdapter};
use crate::types::{ChatRequest, ChatResponse, Choice, Message, Role, Usage};
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

#[derive(Deserialize, Debug)]
pub struct OllamaResponseMessage {
    role: String,
    content: String,
}

// ============================================================================
// Ollama Error Mapper
// ============================================================================

pub struct OllamaErrorMapper;

impl ErrorMapper for OllamaErrorMapper {
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
// Ollama Protocol Adapter
// ============================================================================

/// Ollama Protocol implementation for local LLM servers
#[derive(Debug, Clone)]
pub struct OllamaProtocol {
    base_url: Arc<str>,
    supported_models: Arc<[String]>,
}

impl OllamaProtocol {
    /// Create new Ollama protocol with default localhost:11434
    pub fn new() -> Self {
        Self {
            base_url: Arc::from("http://localhost:11434"),
            supported_models: Arc::from(vec![
                "llama3.2".to_string(),
                "llama3.1".to_string(),
                "llama3".to_string(),
                "llama2".to_string(),
                "mistral".to_string(),
                "mixtral".to_string(),
                "qwen2.5".to_string(),
            ]),
        }
    }

    /// Create new Ollama protocol with custom URL
    pub fn with_url(base_url: &str) -> Self {
        Self {
            base_url: Arc::from(base_url),
            supported_models: Arc::from(vec![]), // Empty - users specify models in requests
        }
    }
}

#[async_trait]
impl ProviderAdapter for OllamaProtocol {
    type RequestType = OllamaRequest;
    type ResponseType = OllamaResponse;
    type ErrorMapperType = OllamaErrorMapper;

    fn name(&self) -> &str {
        "ollama"
    }

    fn supported_models(&self) -> Vec<String> {
        self.supported_models.to_vec()
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        let base = base_url.as_deref().unwrap_or(&self.base_url);
        format!("{}/api/chat", base)
    }

    fn build_request_data(&self, request: &ChatRequest, _stream: bool) -> Self::RequestType {
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
            stream: None, // Not implemented in this minimal version
            options: Some(OllamaOptions {
                temperature: request.temperature,
                top_p: request.top_p,
                num_predict: request.max_tokens,
            }),
        }
    }

    fn parse_response_data(&self, response: Self::ResponseType) -> ChatResponse {
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
                },
                finish_reason: if response.done {
                    Some("stop".to_string())
                } else {
                    Some("length".to_string())
                },
                logprobs: None,
            }],
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
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Create an Ollama protocol adapter
pub fn ollama() -> OllamaProtocol {
    OllamaProtocol::new()
}

/// Ollama provider type alias
pub type OllamaProvider = crate::protocols::core::GenericProvider<OllamaProtocol>;
