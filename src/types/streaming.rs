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
    pub object: String,

    /// Unix timestamp of creation
    pub created: u64,

    /// Model used for the completion
    pub model: String,

    /// List of streaming choices
    pub choices: Vec<StreamingChoice>,

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
}
