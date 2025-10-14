//! Streaming types for chat completions

use super::request::{Role, ToolCall};
use super::response::Usage;
use serde::{Deserialize, Serialize};

#[cfg(feature = "streaming")]
use futures_util::Stream;
#[cfg(feature = "streaming")]
use std::pin::Pin;

/// Type alias for chat completion streams
#[cfg(feature = "streaming")]
pub type ChatStream =
    Pin<Box<dyn Stream<Item = Result<StreamingResponse, crate::error::LlmConnectorError>> + Send>>;

/// Streaming chat completion response chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingResponse {
    /// Unique identifier for the completion
    pub id: String,

    /// Object type (always "chat.completion.chunk")
    #[serde(default)]
    pub object: String,

    /// Unix timestamp of creation
    pub created: u64,

    /// Model used for the completion
    pub model: String,

    /// List of streaming choices
    pub choices: Vec<StreamingChoice>,

    /// Convenience field: current chunk content (from first choice delta)
    #[serde(default)]
    pub content: String,

    /// Convenience field: provider-specific reasoning content (e.g., GLM)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,

    /// Usage statistics (only in final chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,

    /// System fingerprint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// A streaming completion choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingChoice {
    /// Index of the choice
    pub index: u32,

    /// The delta (incremental content)
    pub delta: Delta,

    /// Reason for finishing (only in final chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,

    /// Log probabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,
}

/// Delta content for streaming
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Delta {
    /// Role of the message (only in first chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,

    /// Incremental content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Tool calls (for function calling)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    /// Reasoning content (for o1 models)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,

    /// Reasoning (Qwen/DeepSeek/OpenAI o1 通用键)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,

    /// Thought (OpenAI o1 键)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<String>,

    /// Thinking (Anthropic 键)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
}

impl Delta {
    /// Convenience: get the first available reasoning-like content in delta
    pub fn reasoning_any(&self) -> Option<&str> {
        self.reasoning_content
            .as_deref()
            .or(self.reasoning.as_deref())
            .or(self.thought.as_deref())
            .or(self.thinking.as_deref())
    }

    /// Provider-agnostic post-processor: populate reasoning synonyms from raw JSON
    pub fn populate_reasoning_from_json(&mut self, raw: &serde_json::Value) {
        fn collect_synonyms(val: &serde_json::Value, acc: &mut std::collections::HashMap<String, String>) {
            match val {
                serde_json::Value::Array(arr) => {
                    for v in arr { collect_synonyms(v, acc); }
                }
                serde_json::Value::Object(map) => {
                    for (k, v) in map {
                        let key = k.to_ascii_lowercase();
                        if let serde_json::Value::String(s) = v {
                            match key.as_str() {
                                "reasoning_content" | "reasoning" | "thought" | "thinking" => {
                                    acc.entry(key).or_insert_with(|| s.clone());
                                }
                                _ => {}
                            }
                        }
                        collect_synonyms(v, acc);
                    }
                }
                _ => {}
            }
        }

        let mut found = std::collections::HashMap::<String, String>::new();
        collect_synonyms(raw, &mut found);

        if self.reasoning_content.is_none() {
            if let Some(v) = found.get("reasoning_content") { self.reasoning_content = Some(v.clone()); }
        }
        if self.reasoning.is_none() {
            if let Some(v) = found.get("reasoning") { self.reasoning = Some(v.clone()); }
        }
        if self.thought.is_none() {
            if let Some(v) = found.get("thought") { self.thought = Some(v.clone()); }
        }
        if self.thinking.is_none() {
            if let Some(v) = found.get("thinking") { self.thinking = Some(v.clone()); }
        }
    }
}

impl StreamingResponse {
    /// Provider-agnostic post-processor: populate reasoning synonyms into deltas
    pub fn populate_reasoning_synonyms(&mut self, raw: &serde_json::Value) {
        for choice in &mut self.choices {
            choice.delta.populate_reasoning_from_json(raw);
        }
        if self.reasoning_content.is_none() {
            if let Some(reason) = self
                .choices
                .iter()
                .find_map(|c| c.delta.reasoning_any().map(|s| s.to_string()))
            {
                self.reasoning_content = Some(reason);
            }
        }
    }

    /// Convenience: get current chunk content as Option<&str>
    /// Returns None when the convenience `content` field is empty
    pub fn get_content(&self) -> Option<&str> {
        if self.content.is_empty() { None } else { Some(&self.content) }
    }
}
