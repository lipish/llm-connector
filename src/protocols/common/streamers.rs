//! Common Streaming Interpreters

use crate::error::LlmConnectorError;
use crate::types::{Delta, Role, StreamingChoice, StreamingResponse, Usage};
#[cfg(feature = "streaming")]
use futures_util::StreamExt;
use serde_json::Value;

#[cfg(feature = "streaming")]
pub fn map_sse_json_stream<F>(response: reqwest::Response, mapper: F) -> crate::types::ChatStream
where
    F: Fn(String) -> Result<Option<StreamingResponse>, LlmConnectorError> + Send + Sync + 'static,
{
    let events_stream = crate::sse::create_text_stream(response, crate::sse::StreamFormat::Sse);
    let response_stream = events_stream.filter_map(move |result| {
        let mapped = match result {
            Ok(json_str) => match mapper(json_str) {
                Ok(Some(response)) => Some(Ok(response)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            },
            Err(e) => Some(Err(e)),
        };
        std::future::ready(mapped)
    });

    Box::pin(response_stream)
}

/// Anthropic Event interpretation
pub fn interpret_anthropic_event(
    event: &Value,
    message_id: &str,
) -> Result<Option<StreamingResponse>, LlmConnectorError> {
    let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match event_type {
        "content_block_delta" => {
            if let Some(text) = event
                .get("delta")
                .and_then(|d| d.get("text"))
                .and_then(|t| t.as_str())
            {
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
            let stop_reason = event
                .get("delta")
                .and_then(|d| d.get("stop_reason"))
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());
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
                created: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
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
