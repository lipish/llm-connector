//! Google Gemini Service Provider Implementation
//!
//! This module provides Google Gemini service implementation.
//! Since Google API uses model-specific endpoints, we implement Provider directly instead of using GenericProvider.

use crate::core::{HttpClient, Provider};
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, Choice, Message, Role, Usage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

#[cfg(feature = "streaming")]
use crate::sse::sse_events;

#[cfg(feature = "streaming")]
use crate::types::{Delta, StreamingChoice, StreamingResponse};

#[cfg(feature = "streaming")]
use futures_util::StreamExt;

/// Google Gemini Service Provider
#[derive(Clone, Debug)]
pub struct GoogleProvider {
    client: HttpClient,
}

impl GoogleProvider {
    /// Create new Google provider
    pub fn new(api_key: &str) -> Result<Self, LlmConnectorError> {
        Self::with_config(api_key, None, None, None)
    }

    /// Create Google provider with custom configuration
    pub fn with_config(
        api_key: &str,
        base_url: Option<&str>,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        let base_url = base_url.unwrap_or("https://generativelanguage.googleapis.com/v1beta");
        let client = HttpClient::with_config(base_url, timeout_secs, proxy)?
            .with_header("x-goog-api-key".to_string(), api_key.to_string());

        Ok(Self { client })
    }
}

#[async_trait]
impl Provider for GoogleProvider {
    fn name(&self) -> &str {
        "google"
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        let url = format!(
            "{}/models/{}:generateContent",
            self.client.base_url(),
            request.model
        );

        let google_request = GoogleRequest::from(request);

        let response = self.client.post(&url, &google_request).await?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;

        if !status.is_success() {
            return Err(LlmConnectorError::ProviderError(format!(
                "Google API error: {} - {}",
                status, text
            )));
        }

        // DEBUG: Print raw response if parsing fails or content is empty
        // println!("DEBUG: Google Response: {}", text);

        let google_response: GoogleResponse =
            serde_json::from_str(&text).map_err(|e| LlmConnectorError::JsonError(e))?;

        Ok(google_response.into())
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        let url = format!(
            "{}/models/{}:streamGenerateContent?alt=sse",
            self.client.base_url(),
            request.model
        );

        let google_request = GoogleRequest::from(request);

        let response = self.client.stream(&url, &google_request).await?;
        let status = response.status();

        if !status.is_success() {
            let text = response
                .text()
                .await
                .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;
            return Err(LlmConnectorError::ProviderError(format!(
                "Google API error: {} - {}",
                status, text
            )));
        }

        // Gemini streaming returns SSE events where each `data:` payload is a JSON object
        // similar to the non-streaming response schema.
        let model = request.model.clone();

        // Track whether we already emitted the role in the first chunk.
        let stream = sse_events(response)
            .scan(false, move |sent_role, event_result| {
                let model = model.clone();

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
                                    prompt_cache_hit_tokens: None,
                                    prompt_cache_miss_tokens: None,
                                    prompt_tokens_details: None,
                                    completion_tokens_details: None,
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
                                        model: model.clone(),
                                        choices: vec![StreamingChoice {
                                            index: 0,
                                            delta: Delta {
                                                role,
                                                content: if content.is_empty() {
                                                    None
                                                } else {
                                                    Some(content.clone())
                                                },
                                                tool_calls: None,
                                                reasoning_content: None,
                                                reasoning: None,
                                                thought: None,
                                                thinking: None,
                                            },
                                            finish_reason,
                                            logprobs: None,
                                        }],
                                        content,
                                        usage,
                                        reasoning_content: None,
                                        system_fingerprint: None,
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

    async fn models(&self) -> Result<Vec<String>, LlmConnectorError> {
        let url = format!("{}/models", self.client.base_url());

        let response = self.client.get(&url).await?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;

        if !status.is_success() {
            return Err(LlmConnectorError::ProviderError(format!(
                "Google API error: {} - {}",
                status, text
            )));
        }

        let models_response: GoogleModelsResponse =
            serde_json::from_str(&text).map_err(|e| LlmConnectorError::JsonError(e))?;

        Ok(models_response
            .models
            .into_iter()
            .map(|m| m.name.replace("models/", ""))
            .collect())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// Google API Types
// ============================================================================

#[derive(Serialize)]
struct GoogleRequest {
    contents: Vec<GoogleContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GoogleGenerationConfig>,
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
                        Role::System => "user".to_string(), // Google doesn't have system role in v1beta, usually mapped to user or handled differently.
                        // Actually v1beta supports system instructions now, but let's stick to simple mapping for now.
                        // Or map system to "user" with some prefix?
                        // Newer Gemini models support "system" role? No, usually "user".
                        // But there is `system_instruction` field in request.
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
                max_output_tokens: req.max_tokens.map(|t| t as u32),
            }),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GoogleContent {
    #[serde(default)]
    role: String,
    #[serde(default)]
    parts: Vec<GooglePart>,
}

#[derive(Serialize, Deserialize)]
struct GooglePart {
    #[serde(default)]
    text: String,
}

#[derive(Serialize)]
struct GoogleGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct GoogleResponse {
    candidates: Option<Vec<GoogleCandidate>>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GoogleUsageMetadata>,
}

#[derive(Deserialize)]
struct GoogleCandidate {
    content: Option<GoogleContent>,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct GoogleUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<u32>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<u32>,
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<u32>,
    #[serde(rename = "thoughtsTokenCount")]
    thoughts_token_count: Option<u32>,
}

impl Into<ChatResponse> for GoogleResponse {
    fn into(self) -> ChatResponse {
        let choice = if let Some(candidates) = self.candidates {
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

        let usage = self.usage_metadata.map(|u| Usage {
            prompt_tokens: u.prompt_token_count.unwrap_or(0),
            completion_tokens: u.candidates_token_count.unwrap_or(0)
                + u.thoughts_token_count.unwrap_or(0),
            total_tokens: u.total_token_count.unwrap_or(0),
            prompt_cache_hit_tokens: None,
            prompt_cache_miss_tokens: None,
            prompt_tokens_details: None,
            completion_tokens_details: None,
        });

        ChatResponse {
            id: "google".to_string(), // Google doesn't return ID?
            object: "chat.completion".to_string(),
            created: 0,
            model: "google".to_string(), // Should be passed from request?
            choices: vec![choice.clone()],
            content: choice.message.content_as_text(),
            reasoning_content: None,
            usage,
            system_fingerprint: None,
        }
    }
}

#[derive(Deserialize)]
struct GoogleModelsResponse {
    models: Vec<GoogleModel>,
}

#[derive(Deserialize)]
struct GoogleModel {
    name: String,
}

// Public factory functions
pub fn google(api_key: &str) -> Result<GoogleProvider, LlmConnectorError> {
    GoogleProvider::new(api_key)
}

pub fn google_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<GoogleProvider, LlmConnectorError> {
    GoogleProvider::with_config(api_key, base_url, timeout_secs, proxy)
}
