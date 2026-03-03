//! Shared Utilities for OpenAI-Compatible Protocols
//!
//! Many LLM providers (Zhipu, Aliyun, DeepSeek, etc.) adopt an API structure
//! that is highly compatible with the official OpenAI specification.
//! This module provides shared parsing and conversion utilities to reduce
//! boilerplate across different protocol implementations.

use crate::error::LlmConnectorError;
use crate::types::{ChatResponse, EmbedResponse, EmbeddingData, Usage};
use serde::Deserialize;

// ============================================================================
// Standard OpenAI Compatible Response Types (Internal Parsing)
// ============================================================================

#[derive(Deserialize, Debug)]
pub struct OpenAICompatibleResponse {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<u64>,
    pub model: Option<String>,
    pub choices: Option<Vec<OpenAICompatibleChoice>>,
    pub usage: Option<OpenAICompatibleUsage>,
    pub system_fingerprint: Option<String>,

    // Potentially proprietary fields we ignore or handle specially
    #[serde(default)]
    pub request_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAICompatibleChoice {
    pub index: Option<u32>,
    pub message: Option<OpenAICompatibleMessage>,
    pub delta: Option<OpenAICompatibleMessage>, // For streaming
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OpenAICompatibleMessage {
    #[allow(dead_code)]
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<serde_json::Value>,
    // DeepSeek reasoning content Extension
    pub reasoning_content: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAICompatibleUsage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
    #[serde(default)]
    pub prompt_cache_hit_tokens: Option<u32>,
    #[serde(default)]
    pub prompt_cache_miss_tokens: Option<u32>,
}

// ============================================================================
// Shared Parsers
// ============================================================================

/// Parse a standard OpenAI-compatible JSON response into a ChatResponse
pub fn parse_openai_compatible_chat_response(
    response: &str,
    provider_name: &str,
) -> Result<ChatResponse, LlmConnectorError> {
    let raw: OpenAICompatibleResponse = serde_json::from_str(response)
        .map_err(|e| LlmConnectorError::ParseError(format!("{}: {}", provider_name, e)))?;

    // Extract usage
    let usage = raw.usage.map(|u| Usage {
        prompt_tokens: u.prompt_tokens.unwrap_or(0),
        completion_tokens: u.completion_tokens.unwrap_or(0),
        total_tokens: u.total_tokens.unwrap_or(0),
        prompt_cache_hit_tokens: u.prompt_cache_hit_tokens,
        prompt_cache_miss_tokens: u.prompt_cache_miss_tokens,
        ..Default::default()
    });

    // Extract choices
    let mut mapped_choices = Vec::new();
    let mut main_content = String::new();
    let mut main_reasoning = None;

    if let Some(choices) = raw.choices {
        for choice in choices {
            let msg_source = choice.message.or(choice.delta); // Support standard and delta
            if let Some(msg) = msg_source {
                let mut content_str = msg.content.unwrap_or_default();
                let mut reasoning_str = msg.reasoning_content;

                // Handle providers like Minimax that embed thinking in <think> tags
                if reasoning_str.is_none()
                    && content_str.contains("<think>")
                    && let Some(start_idx) = content_str.find("<think>")
                    && let Some(end_idx) = content_str.find("</think>")
                {
                    let extracted_reasoning = content_str[start_idx + 7..end_idx].to_string();
                    reasoning_str = Some(extracted_reasoning);

                    // Remove the <think>...</think> block from the content
                    let mut new_content = content_str[..start_idx].to_string();
                    new_content.push_str(&content_str[end_idx + 8..]);
                    content_str = new_content.trim().to_string();
                }

                // Keep the first choice's content as main
                if choice.index.unwrap_or(0) == 0 {
                    main_content = content_str.clone();
                    main_reasoning = reasoning_str.clone();
                }

                // Parse Tool Calls
                let mut mapped_tool_calls = None;
                if let Some(tc_val) = msg.tool_calls {
                    // Try to parse tool calls if they exist
                    if let Ok(calls) =
                        serde_json::from_value::<Vec<crate::types::ToolCall>>(tc_val.clone())
                    {
                        mapped_tool_calls = Some(calls);
                    }
                }

                let mut final_message = if let Some(tc) = mapped_tool_calls {
                    crate::types::Message::assistant_with_tool_calls(tc)
                } else {
                    crate::types::Message::assistant(&content_str)
                };

                final_message.reasoning_content = reasoning_str;

                mapped_choices.push(crate::types::Choice {
                    index: choice.index.unwrap_or(0),
                    message: final_message,
                    finish_reason: choice.finish_reason,
                    logprobs: None, // Simplified for now
                });
            }
        }
    }

    Ok(ChatResponse {
        id: raw.id.unwrap_or_else(|| raw.request_id.unwrap_or_default()),
        object: raw.object.unwrap_or_else(|| "chat.completion".to_string()),
        created: raw.created.unwrap_or(0),
        model: raw.model.unwrap_or_default(),
        choices: mapped_choices,
        content: main_content,
        reasoning_content: main_reasoning,
        usage,
        system_fingerprint: raw.system_fingerprint,
    })
}

// ============================================================================
// Standard OpenAI Compatible Embedding Types
// ============================================================================

#[derive(Deserialize, Debug)]
pub struct OpenAICompatibleEmbedResponse {
    pub object: Option<String>,
    pub data: Option<Vec<OpenAICompatibleEmbedData>>,
    pub model: Option<String>,
    pub usage: Option<OpenAICompatibleUsage>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAICompatibleEmbedData {
    pub object: Option<String>,
    pub embedding: Vec<f32>,
    pub index: u32,
}

/// Parse a standard OpenAI-compatible embedding JSON response into an EmbedResponse
pub fn parse_openai_compatible_embed_response(
    response: &str,
    provider_name: &str,
) -> Result<EmbedResponse, LlmConnectorError> {
    let raw: OpenAICompatibleEmbedResponse = serde_json::from_str(response)
        .map_err(|e| LlmConnectorError::ParseError(format!("{}: {}", provider_name, e)))?;

    // Extract usage
    let usage = raw
        .usage
        .map(|u| Usage {
            prompt_tokens: u.prompt_tokens.unwrap_or(0),
            completion_tokens: u.completion_tokens.unwrap_or(0),
            total_tokens: u.total_tokens.unwrap_or(0),
            prompt_cache_hit_tokens: u.prompt_cache_hit_tokens,
            prompt_cache_miss_tokens: u.prompt_cache_miss_tokens,
            ..Default::default()
        })
        .unwrap_or_default();

    // Extract embeddings data
    let mut data = Vec::new();
    if let Some(raw_data) = raw.data {
        for item in raw_data {
            data.push(EmbeddingData {
                object: item.object.unwrap_or_else(|| "embedding".to_string()),
                embedding: item.embedding,
                index: item.index,
            });
        }
    }

    Ok(EmbedResponse {
        object: raw.object.unwrap_or_else(|| "list".to_string()),
        data,
        model: raw.model.unwrap_or_else(|| "unknown".to_string()),
        usage,
    })
}
