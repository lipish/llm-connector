//! Streaming types for chat completions

use super::request::{Role, ToolCall};
use super::response::Usage;
use serde::{Deserialize, Serialize};

#[cfg(feature = "streaming")]
use futures_util::Stream;
#[cfg(feature = "streaming")]
use std::pin::Pin;

// ============================================================================
// Streaming Format Configuration
// ============================================================================

/// Streaming output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamingFormat {
    /// OpenAI-compatible format (default)
    #[serde(rename = "openai")]
    OpenAI,
    /// Ollama-compatible format
    #[serde(rename = "ollama")]
    Ollama,
}

impl Default for StreamingFormat {
    fn default() -> Self {
        Self::OpenAI
    }
}

/// Streaming configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Output format for streaming responses
    pub format: StreamingFormat,
    /// Whether to include usage statistics in final chunk
    pub include_usage: bool,
    /// Whether to include reasoning content (for providers that support it)
    pub include_reasoning: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            format: StreamingFormat::OpenAI,
            include_usage: true,
            include_reasoning: true,
        }
    }
}

/// Type alias for chat completion streams
#[cfg(feature = "streaming")]
pub type ChatStream =
    Pin<Box<dyn Stream<Item = Result<StreamingResponse, crate::error::LlmConnectorError>> + Send>>;

/// Type alias for Ollama format chat completion streams
#[cfg(feature = "streaming")]
pub type OllamaChatStream =
    Pin<Box<dyn Stream<Item = Result<OllamaStreamChunk, crate::error::LlmConnectorError>> + Send>>;

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

impl Default for StreamingResponse {
    fn default() -> Self {
        Self {
            id: String::new(),
            object: "chat.completion.chunk".to_string(),
            created: 0,
            model: String::new(),
            choices: Vec::new(),
            content: String::new(),
            reasoning_content: None,
            usage: None,
            system_fingerprint: None,
        }
    }
}

// ============================================================================
// Ollama Format Streaming Types
// ============================================================================

/// Ollama-compatible streaming response chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaStreamChunk {
    /// Model name
    pub model: String,

    /// Creation timestamp in RFC3339 format
    pub created_at: String,

    /// Message content
    pub message: OllamaMessage,

    /// Whether this is the final chunk
    pub done: bool,

    /// Total duration in nanoseconds (only in final chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,

    /// Load duration in nanoseconds (only in final chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_duration: Option<u64>,

    /// Number of tokens in the prompt (only in final chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<u32>,

    /// Time spent evaluating the prompt in nanoseconds (only in final chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_duration: Option<u64>,

    /// Number of tokens generated (only in final chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<u32>,

    /// Time spent generating tokens in nanoseconds (only in final chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_duration: Option<u64>,
}

/// Ollama message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaMessage {
    /// Message role
    pub role: String,

    /// Message content
    pub content: String,
}

impl OllamaStreamChunk {
    /// Create a new streaming chunk
    pub fn new(model: String, content: String, done: bool) -> Self {
        Self {
            model,
            created_at: chrono::Utc::now().to_rfc3339(),
            message: OllamaMessage {
                role: "assistant".to_string(),
                content,
            },
            done,
            total_duration: None,
            load_duration: None,
            prompt_eval_count: None,
            prompt_eval_duration: None,
            eval_count: None,
            eval_duration: None,
        }
    }

    /// Create a final chunk with usage statistics
    pub fn final_chunk(model: String, usage: Option<&Usage>) -> Self {
        let mut chunk = Self::new(model, String::new(), true);

        if let Some(usage) = usage {
            // Convert token counts to Ollama format
            chunk.prompt_eval_count = Some(usage.prompt_tokens);
            chunk.eval_count = Some(usage.completion_tokens);

            // Estimate durations (Ollama uses nanoseconds)
            // These are rough estimates since we don't have actual timing data
            chunk.prompt_eval_duration = Some((usage.prompt_tokens as u64) * 1_000_000); // 1ms per token
            chunk.eval_duration = Some((usage.completion_tokens as u64) * 10_000_000); // 10ms per token
            chunk.total_duration = Some(
                chunk.prompt_eval_duration.unwrap_or(0) + chunk.eval_duration.unwrap_or(0)
            );
        }

        chunk
    }

    /// Convert from OpenAI streaming response to Ollama format
    pub fn from_openai_chunk(openai_chunk: &StreamingResponse, done: bool) -> Self {
        let content = if !openai_chunk.content.is_empty() {
            openai_chunk.content.clone()
        } else {
            openai_chunk.choices.get(0)
                .and_then(|choice| choice.delta.content.clone())
                .unwrap_or_default()
        };

        let mut chunk = Self::new(openai_chunk.model.clone(), content, done);

        // If this is the final chunk and we have usage data, include it
        if done {
            if let Some(usage) = &openai_chunk.usage {
                chunk.prompt_eval_count = Some(usage.prompt_tokens);
                chunk.eval_count = Some(usage.completion_tokens);

                // Estimate durations
                chunk.prompt_eval_duration = Some((usage.prompt_tokens as u64) * 1_000_000);
                chunk.eval_duration = Some((usage.completion_tokens as u64) * 10_000_000);
                chunk.total_duration = Some(
                    chunk.prompt_eval_duration.unwrap_or(0) + chunk.eval_duration.unwrap_or(0)
                );
            }
        }

        chunk
    }
}

// ============================================================================
// Format Conversion Utilities
// ============================================================================

/// Convert OpenAI streaming response to specified format
pub fn convert_streaming_format(
    openai_chunk: &StreamingResponse,
    format: StreamingFormat,
    is_final: bool,
) -> Result<String, serde_json::Error> {
    match format {
        StreamingFormat::OpenAI => {
            serde_json::to_string(openai_chunk)
        }
        StreamingFormat::Ollama => {
            let ollama_chunk = OllamaStreamChunk::from_openai_chunk(openai_chunk, is_final);
            serde_json::to_string(&ollama_chunk)
        }
    }
}

/// Create a final Ollama chunk with done: true
pub fn create_final_ollama_chunk(model: &str, usage: Option<&Usage>) -> String {
    let final_chunk = OllamaStreamChunk::final_chunk(model.to_string(), usage);
    serde_json::to_string(&final_chunk).unwrap_or_else(|_| {
        // Fallback minimal final chunk
        format!(
            r#"{{"model":"{}","created_at":"{}","message":{{"role":"assistant","content":""}},"done":true}}"#,
            model,
            chrono::Utc::now().to_rfc3339()
        )
    })
}
