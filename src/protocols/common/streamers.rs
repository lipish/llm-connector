//! Common Streaming Interpreters

use crate::error::LlmConnectorError;
use crate::types::{StreamingResponse, Usage, StreamingChoice, Delta, Role};
use serde_json::Value;

/// Anthropic Event interpretation
pub fn interpret_anthropic_event(
    event: &Value,
    message_id: &str,
) -> Result<Option<StreamingResponse>, LlmConnectorError> {
    let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match event_type {
        "content_block_delta" => {
            if let Some(text) = event.get("delta").and_then(|d| d.get("text")).and_then(|t| t.as_str()) {
                Ok(Some(StreamingResponse {
                    id: message_id.to_string(),
                    object: "chat.completion.chunk".to_string(),
                    created: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    model: "anthropic".to_string(),
                    choices: vec![StreamingChoice {
                        index: 0,
                        delta: Delta {
                            role: Some(Role::Assistant),
                            content: Some(text.to_string()),
                            ..Default::default()
                        },
                        finish_reason: None,
                        logprobs: None,
                    }],
                    content: text.to_string(),
                    ..Default::default()
                }))
            } else {
                Ok(None)
            }
        }
        "message_delta" => {
            let stop_reason = event.get("delta").and_then(|d| d.get("stop_reason")).and_then(|s| s.as_str()).map(|s| s.to_string());
            let usage = event.get("usage").map(|u| {
                let in_t = u.get("input_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
                let out_t = u.get("output_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
                Usage {
                    prompt_tokens: in_t,
                    completion_tokens: out_t,
                    total_tokens: in_t + out_t,
                    ..Default::default()
                }
            });

            Ok(Some(StreamingResponse {
                id: message_id.to_string(),
                object: "chat.completion.chunk".to_string(),
                created: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
                model: "anthropic".to_string(),
                choices: vec![StreamingChoice {
                    index: 0,
                    delta: Delta::default(),
                    finish_reason: stop_reason,
                    logprobs: None,
                }],
                usage,
                ..Default::default()
            }))
        }
        _ => Ok(None),
    }
}
