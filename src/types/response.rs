//! Response types for chat completions

use super::request::Message;
use serde::{Deserialize, Serialize};

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatResponse {
    /// A unique identifier for the chat completion.
    pub id: String,

    /// Object type (always "chat.completion")
    pub object: String,

    /// Unix timestamp of creation
    pub created: u64,

    /// Model used for the completion
    pub model: String,

    /// List of completion choices
    pub choices: Vec<Choice>,

    /// Convenience field: first choice content
    /// Extracted from `choices[0].message.content` if present
    #[serde(default)]
    pub content: String,

    /// Usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,

    /// System fingerprint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// A completion choice
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Choice {
    /// The index of the choice in the list of choices.
    pub index: u32,

    /// The generated message
    pub message: Message,

    /// Reason for finishing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,

    /// Log probabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,

    /// Number of tokens in the completion
    pub completion_tokens: u32,

    /// Total number of tokens
    pub total_tokens: u32,

    /// Number of prompt tokens that hit the cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_hit_tokens: Option<u32>,

    /// Number of prompt tokens that missed the cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_miss_tokens: Option<u32>,

    /// Detailed prompt token usage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,

    /// Detailed completion token usage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

/// Detailed prompt token usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    /// Number of cached tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,
}

/// Detailed completion token usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    /// Number of reasoning tokens (for o1 models)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
}

impl ChatResponse {
    /// Convenience: prompt tokens or 0
    pub fn prompt_tokens(&self) -> u32 {
        self.usage.as_ref().map(|u| u.prompt_tokens).unwrap_or(0)
    }

    /// Convenience: completion tokens or 0
    pub fn completion_tokens(&self) -> u32 {
        self.usage
            .as_ref()
            .map(|u| u.completion_tokens)
            .unwrap_or(0)
    }

    /// Convenience: total tokens or 0
    pub fn total_tokens(&self) -> u32 {
        self.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0)
    }

    /// Convenience: get usage safely as a tuple (prompt, completion, total)
    pub fn get_usage_safe(&self) -> (u32, u32, u32) {
        (self.prompt_tokens(), self.completion_tokens(), self.total_tokens())
    }

    /// Convenience: get first choice content as Option<&str>
    /// Returns None if the convenience `content` field is empty
    pub fn get_content(&self) -> Option<&str> {
        if self.content.is_empty() { None } else { Some(&self.content) }
    }

    /// Provider-agnostic post-processor: populate reasoning synonyms into messages
    pub fn populate_reasoning_synonyms(&mut self, raw: &serde_json::Value) {
        for choice in &mut self.choices {
            choice.message.populate_reasoning_from_json(raw);
        }
    }
}
