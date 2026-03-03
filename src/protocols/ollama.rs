//! Ollama Protocol Implementation
//!
//! This module provides the Ollama API protocol.

use crate::core::Protocol;
use crate::error::LlmConnectorError;
use crate::types::{
    ChatRequest, ChatResponse, Choice, EmbedRequest, EmbedResponse, EmbeddingData, Message, Role,
    Usage,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct OllamaProtocol;

impl OllamaProtocol {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Protocol for OllamaProtocol {
    type Request = OllamaChatRequest;
    type Response = OllamaChatResponse;

    fn name(&self) -> &str {
        "ollama"
    }

    fn chat_endpoint(&self, base_url: &str, _model: &str) -> String {
        format!("{}/api/chat", base_url.trim_end_matches('/'))
    }

    fn models_endpoint(&self, base_url: &str) -> Option<String> {
        Some(format!("{}/api/tags", base_url.trim_end_matches('/')))
    }

    fn embed_endpoint(&self, base_url: &str, _model: &str) -> Option<String> {
        Some(format!("{}/api/embed", base_url.trim_end_matches('/')))
    }

    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        Ok(OllamaChatRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|msg| OllamaMessage {
                    role: match msg.role {
                        Role::User => "user".to_string(),
                        Role::Assistant => "assistant".to_string(),
                        Role::System => "system".to_string(),
                        Role::Tool => "user".to_string(),
                    },
                    content: msg.content_as_text(),
                })
                .collect(),
            stream: request.stream,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens.map(|t| t as i32),
                top_p: request.top_p,
            }),
        })
    }

    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        let ollama_response: OllamaChatResponse =
            serde_json::from_str(response).map_err(LlmConnectorError::JsonError)?;

        let content = ollama_response.message.content.clone();
        let choices = vec![Choice {
            index: 0,
            message: Message::assistant(&content),
            finish_reason: Some("stop".to_string()),
            logprobs: None,
        }];

        Ok(ChatResponse {
            id: "ollama".to_string(),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: ollama_response.model,
            choices,
            content,
            reasoning_content: None,
            usage: None,
            system_fingerprint: None,
        })
    }

    fn parse_models(&self, response: &str) -> Result<Vec<String>, LlmConnectorError> {
        let models_response: OllamaModelsResponse =
            serde_json::from_str(response).map_err(LlmConnectorError::JsonError)?;
        Ok(models_response.models.into_iter().map(|m| m.name).collect())
    }

    fn build_embed_request(
        &self,
        request: &EmbedRequest,
    ) -> Result<serde_json::Value, LlmConnectorError> {
        Ok(serde_json::json!({
            "model": request.model,
            "input": request.input,
        }))
    }

    fn parse_embed_response(&self, response: &str) -> Result<EmbedResponse, LlmConnectorError> {
        let embed_response: serde_json::Value =
            serde_json::from_str(response).map_err(LlmConnectorError::JsonError)?;

        let embeddings = embed_response
            .get("embeddings")
            .and_then(|e| e.as_array())
            .ok_or_else(|| LlmConnectorError::ParseError("Missing embeddings field".to_string()))?;

        let mut data = Vec::new();
        for (index, emb) in embeddings.iter().enumerate() {
            let vec = emb
                .as_array()
                .ok_or_else(|| {
                    LlmConnectorError::ParseError("Invalid embedding format".to_string())
                })?
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect();
            data.push(EmbeddingData {
                object: "embedding".to_string(),
                embedding: vec,
                index: index as u32,
            });
        }

        let usage = Usage {
            prompt_tokens: embed_response
                .get("prompt_eval_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            completion_tokens: 0,
            total_tokens: embed_response
                .get("prompt_eval_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            ..Default::default()
        };

        // Note: model is not easily available here, but we can use a placeholder
        Ok(EmbedResponse {
            object: "list".to_string(),
            data,
            model: "ollama".to_string(),
            usage,
        })
    }

    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError {
        LlmConnectorError::ProviderError(format!("Ollama API error: {} - {}", status, body))
    }

    #[cfg(feature = "streaming")]
    async fn parse_stream_response(
        &self,
        response: reqwest::Response,
    ) -> Result<crate::types::ChatStream, LlmConnectorError> {
        // Ollama uses JSONL format instead of SSE for its native /api/chat endpoint
        // However, our sse_to_streaming_response might need adjustment for JSONL or we use a custom parser.
        // Actually, Ollama's /api/chat with stream: true returns a sequence of JSON objects.
        
        Ok(crate::sse::sse_to_streaming_response(response))
    }
}

// ============================================================================
// Ollama API Types
// ============================================================================

#[derive(Serialize, Debug)]
pub struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<OllamaMessage>,
    pub stream: Option<bool>,
    pub options: Option<OllamaOptions>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Debug)]
pub struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct OllamaChatResponse {
    pub model: String,
    pub message: OllamaResponseMessage,
    pub done: bool,
}

#[derive(Deserialize, Debug)]
pub struct OllamaResponseMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModel>,
}

#[derive(Deserialize, Debug)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}
