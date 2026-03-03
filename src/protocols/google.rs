//! Google Gemini Protocol Implementation
//!
//! This module provides the Google Gemini API protocol.

use crate::core::Protocol;
use crate::error::LlmConnectorError;
use crate::types::{
    ChatRequest, ChatResponse, Choice, EmbedRequest, EmbedResponse, EmbeddingData, Message, Role,
    Usage,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct GoogleProtocol;

impl GoogleProtocol {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Protocol for GoogleProtocol {
    type Request = GoogleRequest;
    type Response = GoogleResponse;

    fn name(&self) -> &str {
        "google"
    }

    fn chat_endpoint(&self, base_url: &str, model: &str) -> String {
        format!("{}/models/{}:generateContent", base_url.trim_end_matches('/'), model)
    }

    #[cfg(feature = "streaming")]
    fn chat_stream_endpoint(&self, base_url: &str, model: &str) -> String {
        format!("{}/models/{}:streamGenerateContent?alt=sse", base_url.trim_end_matches('/'), model)
    }

    fn models_endpoint(&self, base_url: &str) -> Option<String> {
        Some(format!("{}/models", base_url.trim_end_matches('/')))
    }

    fn embed_endpoint(&self, base_url: &str, model: &str) -> Option<String> {
        Some(format!("{}/models/{}:batchEmbedContents", base_url.trim_end_matches('/'), model))
    }

    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        Ok(GoogleRequest::from(request))
    }

    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        let google_response: GoogleResponse =
            serde_json::from_str(response).map_err(LlmConnectorError::JsonError)?;
        Ok(google_response.into())
    }

    fn parse_models(&self, response: &str) -> Result<Vec<String>, LlmConnectorError> {
        let models_response: GoogleModelsResponse =
            serde_json::from_str(response).map_err(LlmConnectorError::JsonError)?;

        Ok(models_response
            .models
            .into_iter()
            .map(|m| m.name.replace("models/", ""))
            .collect())
    }

    fn build_embed_request(
        &self,
        request: &EmbedRequest,
    ) -> Result<serde_json::Value, LlmConnectorError> {
        let requests: Vec<GoogleEmbedRequest> = request
            .input
            .iter()
            .map(|text| GoogleEmbedRequest {
                model: format!("models/{}", request.model),
                content: GoogleContent {
                    role: String::new(),
                    parts: vec![GooglePart {
                        text: text.clone(),
                    }],
                },
            })
            .collect();

        let req_body = GoogleBatchEmbedRequest { requests };
        serde_json::to_value(req_body).map_err(LlmConnectorError::JsonError)
    }

    fn parse_embed_response(&self, response: &str) -> Result<EmbedResponse, LlmConnectorError> {
        let google_response: GoogleBatchEmbedResponse =
            serde_json::from_str(response).map_err(LlmConnectorError::JsonError)?;

        let mut data = Vec::new();
        if let Some(embeddings) = google_response.embeddings {
            for (index, emb) in embeddings.into_iter().enumerate() {
                data.push(EmbeddingData {
                    object: "embedding".to_string(),
                    embedding: emb.values,
                    index: index as u32,
                });
            }
        }

        // We don't have model info in the raw response usually, but we can pass it if we want.
        // Actually Protocol trait doesn't receive the model here.
        // But EmbedResponse needs it.
        Ok(EmbedResponse {
            object: "list".to_string(),
            data,
            model: "google".to_string(), // Placeholder or we need to adjust trait
            usage: Usage::default(),
        })
    }

    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError {
        LlmConnectorError::ProviderError(format!("Google API error: {} - {}", status, body))
    }

    #[cfg(feature = "streaming")]
    async fn parse_stream_response(
        &self,
        response: reqwest::Response,
    ) -> Result<crate::types::ChatStream, LlmConnectorError> {
        use crate::sse::sse_events;
        use crate::types::{Delta, StreamingChoice, StreamingResponse};
        use futures_util::StreamExt;

        // Gemini streaming returns SSE events where each `data:` payload is a JSON object
        // similar to the non-streaming response schema.
        
        // Note: GenericProvider doesn't pass model to this method yet.
        // Let's assume we might need to update Protocol trait for this too if we want perfect model mapping in stream.
        
        let stream = sse_events(response)
            .scan(false, move |sent_role, event_result| {
                let mapped: Result<Option<StreamingResponse>, LlmConnectorError> =
                    match event_result {
                        Ok(json_str) => {
                            if json_str.trim().is_empty() {
                                Ok(None)
                            } else {
                                let google_resp: GoogleResponse =
                                    match serde_json::from_str(&json_str) {
                                        Ok(v) => v,
                                        Err(e) => {
                                            return std::future::ready(Some(Err(
                                                LlmConnectorError::JsonError(e),
                                            )));
                                        }
                                    };

                                // Extract incremental text (if present)
                                let (content, finish_reason) = google_resp
                                    .candidates
                                    .as_ref()
                                    .and_then(|c| c.first())
                                    .map(|candidate| {
                                        let text = candidate
                                            .content
                                            .as_ref()
                                            .and_then(|c| c.parts.first())
                                            .map(|p| p.text.clone())
                                            .unwrap_or_default();
                                        (text, candidate.finish_reason.clone())
                                    })
                                    .unwrap_or_default();

                                // Some events may contain only metadata; skip empty content unless finish_reason/usage exists.
                                let usage = google_resp.usage_metadata.map(|u| Usage {
                                    prompt_tokens: u.prompt_token_count.unwrap_or(0),
                                    completion_tokens: u.candidates_token_count.unwrap_or(0)
                                        + u.thoughts_token_count.unwrap_or(0),
                                    total_tokens: u.total_token_count.unwrap_or(0),
                                    ..Default::default()
                                });

                                if content.is_empty() && finish_reason.is_none() && usage.is_none()
                                {
                                    Ok(None)
                                } else {
                                    let role = if !*sent_role {
                                        *sent_role = true;
                                        Some(Role::Assistant)
                                    } else {
                                        None
                                    };

                                    Ok(Some(StreamingResponse {
                                        id: "google".to_string(),
                                        object: "chat.completion.chunk".to_string(),
                                        created: chrono::Utc::now().timestamp() as u64,
                                        model: "google".to_string(),
                                        choices: vec![StreamingChoice {
                                            index: 0,
                                            delta: Delta {
                                                role,
                                                content: if content.is_empty() {
                                                    None
                                                } else {
                                                    Some(content.clone())
                                                },
                                                ..Default::default()
                                            },
                                            finish_reason,
                                            logprobs: None,
                                        }],
                                        content,
                                        usage,
                                        ..Default::default()
                                    }))
                                }
                            }
                        }
                        Err(e) => Err(e),
                    };

                std::future::ready(Some(mapped))
            })
            .filter_map(|x| async move {
                match x {
                    Ok(Some(v)) => Some(Ok(v)),
                    Ok(None) => None,
                    Err(e) => Some(Err(e)),
                }
            });

        Ok(Box::pin(stream))
    }
}

// ============================================================================
// Google API Types
// ============================================================================

#[derive(Serialize)]
pub struct GoogleRequest {
    pub contents: Vec<GoogleContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GoogleGenerationConfig>,
}

impl From<&ChatRequest> for GoogleRequest {
    fn from(req: &ChatRequest) -> Self {
        let contents = req
            .messages
            .iter()
            .map(|msg| {
                GoogleContent {
                    role: match msg.role {
                        Role::User => "user".to_string(),
                        Role::Assistant => "model".to_string(),
                        Role::System => "user".to_string(), 
                        _ => "user".to_string(),
                    },
                    parts: vec![GooglePart {
                        text: msg.content_as_text(),
                    }],
                }
            })
            .collect();

        GoogleRequest {
            contents,
            generation_config: Some(GoogleGenerationConfig {
                temperature: req.temperature,
                top_p: req.top_p,
                max_output_tokens: req.max_tokens,
            }),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GoogleContent {
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub parts: Vec<GooglePart>,
}

#[derive(Serialize, Deserialize)]
pub struct GooglePart {
    #[serde(default)]
    pub text: String,
}

#[derive(Serialize)]
pub struct GoogleGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
}

#[derive(Deserialize)]
pub struct GoogleResponse {
    pub candidates: Option<Vec<GoogleCandidate>>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: Option<GoogleUsageMetadata>,
}

#[derive(Deserialize)]
pub struct GoogleCandidate {
    pub content: Option<GoogleContent>,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
}

#[derive(Deserialize)]
pub struct GoogleUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: Option<u32>,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: Option<u32>,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: Option<u32>,
    #[serde(rename = "thoughtsTokenCount")]
    pub thoughts_token_count: Option<u32>,
}

impl From<GoogleResponse> for ChatResponse {
    fn from(value: GoogleResponse) -> Self {
        let choice = if let Some(candidates) = value.candidates {
            if let Some(candidate) = candidates.into_iter().next() {
                let content = candidate
                    .content
                    .and_then(|c| c.parts.into_iter().next())
                    .map(|p| p.text)
                    .unwrap_or_default();

                Choice {
                    index: 0,
                    message: Message::assistant(&content),
                    finish_reason: candidate.finish_reason,
                    logprobs: None,
                }
            } else {
                Choice {
                    index: 0,
                    message: Message::assistant(""),
                    finish_reason: None,
                    logprobs: None,
                }
            }
        } else {
            Choice {
                index: 0,
                message: Message::assistant(""),
                finish_reason: None,
                logprobs: None,
            }
        };

        let usage = value.usage_metadata.map(|u| Usage {
            prompt_tokens: u.prompt_token_count.unwrap_or(0),
            completion_tokens: u.candidates_token_count.unwrap_or(0)
                + u.thoughts_token_count.unwrap_or(0),
            total_tokens: u.total_token_count.unwrap_or(0),
            ..Default::default()
        });

        ChatResponse {
            id: "google".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: "google".to_string(),
            choices: vec![choice.clone()],
            content: choice.message.content_as_text(),
            reasoning_content: None,
            usage,
            system_fingerprint: None,
        }
    }
}

#[derive(Deserialize)]
pub struct GoogleModelsResponse {
    pub models: Vec<GoogleModel>,
}

#[derive(Deserialize)]
pub struct GoogleModel {
    pub name: String,
}

#[derive(Serialize)]
pub struct GoogleBatchEmbedRequest {
    pub requests: Vec<GoogleEmbedRequest>,
}

#[derive(Serialize)]
pub struct GoogleEmbedRequest {
    pub model: String,
    pub content: GoogleContent,
}

#[derive(Deserialize)]
pub struct GoogleBatchEmbedResponse {
    pub embeddings: Option<Vec<GoogleEmbedding>>,
}

#[derive(Deserialize)]
pub struct GoogleEmbedding {
    pub values: Vec<f32>,
}
