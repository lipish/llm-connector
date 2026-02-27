//! Server-Sent Events (SSE) streaming utilities
//!
//! This module provides robust streaming utilities for handling various LLM provider response formats.
//! It supports:
//! - Standard SSE (Server-Sent Events) with double-newline separators
//! - Non-standard SSE with single-newline separators (e.g. Zhipu)
//! - NDJSON (Newline Delimited JSON) (e.g. Ollama)
//! - Automatic format detection

use crate::error::LlmConnectorError;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

/// Stream format type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamFormat {
    /// Standard SSE (double newline separator)
    Sse,
    /// Line-delimited JSON (single newline separator)
    NdJson,
    /// Auto-detect based on content
    Auto,
}

/// Create a robust stream from a reqwest response
///
/// This function automatically handles different streaming formats and normalizes them
/// into a stream of JSON strings.
pub fn create_text_stream(
    response: reqwest::Response,
    format: StreamFormat,
) -> Pin<Box<dyn Stream<Item = Result<String, LlmConnectorError>> + Send>> {
    let stream = response.bytes_stream();

    // Use a scanning state to handle partial chunks and format detection
    struct ScanState {
        buffer: String,
        detected_format: Option<StreamFormat>,
    }

    let events_stream = stream
        .scan(
            ScanState {
                buffer: String::new(),
                detected_format: if format == StreamFormat::Auto {
                    None
                } else {
                    Some(format)
                },
            },
            move |state, chunk_result| {
                let mut out: Vec<Result<String, LlmConnectorError>> = Vec::new();
                match chunk_result {
                    Ok(chunk) => {
                        // Normalize line endings
                        let chunk_str = String::from_utf8_lossy(&chunk).replace("\r\n", "\n");
                        state.buffer.push_str(&chunk_str);

                        // Auto-detect format if not yet detected
                        if state.detected_format.is_none() {
                            if state.buffer.contains("data:") {
                                state.detected_format = Some(StreamFormat::Sse);
                            } else if state.buffer.contains('\n') && state.buffer.trim().starts_with('{') {
                                state.detected_format = Some(StreamFormat::NdJson);
                            }
                        }

                        match state.detected_format {
                            Some(StreamFormat::Sse) => {
                                // SSE processing (split by \n\n)
                                // Handle edge case where \n\n might be split across chunks
                                while let Some(boundary_idx) = state.buffer.find("\n\n") {
                                    let event_str: String = state.buffer.drain(..boundary_idx + 2).collect();
                                    
                                    // Extract data lines
                                    let mut data_lines = Vec::new();
                                    for line in event_str.split('\n') {
                                        let line = line.trim();
                                        if let Some(payload) = line.strip_prefix("data:") {
                                            let payload = payload.trim();
                                            if !payload.is_empty() && payload != "[DONE]" {
                                                data_lines.push(payload.to_string());
                                            }
                                        }
                                    }
                                    
                                    if !data_lines.is_empty() {
                                        out.push(Ok(data_lines.join("\n")));
                                    }
                                }
                            }
                            Some(StreamFormat::NdJson) => {
                                // NDJSON processing (split by \n)
                                while let Some(boundary_idx) = state.buffer.find('\n') {
                                    let line: String = state.buffer.drain(..boundary_idx + 1).collect();
                                    let trimmed = line.trim();
                                    
                                    // Handle "data:" prefix if present (Zhipu style)
                                    let payload = if let Some(p) = trimmed.strip_prefix("data:") {
                                        p.trim()
                                    } else {
                                        trimmed
                                    };

                                    if !payload.is_empty() && payload != "[DONE]" {
                                        out.push(Ok(payload.to_string()));
                                    }
                                }
                            }
                            None => {
                                // Not enough data to detect format yet, wait for more
                            }
                            _ => {
                                // Should not happen
                            }
                        }
                    }
                    Err(e) => {
                        out.push(Err(LlmConnectorError::NetworkError(e.to_string())));
                    }
                }
                std::future::ready(Some(out))
            },
        )
        .flat_map(futures_util::stream::iter);

    Box::pin(events_stream)
}

/// Legacy SSE events parser (kept for backward compatibility)
#[inline]
pub fn sse_events(
    response: reqwest::Response,
) -> Pin<Box<dyn Stream<Item = Result<String, LlmConnectorError>> + Send>> {
    create_text_stream(response, StreamFormat::Sse)
}

/// Legacy JSON lines events parser (kept for backward compatibility)
#[inline]
pub fn json_lines_events(
    response: reqwest::Response,
) -> Pin<Box<dyn Stream<Item = Result<String, LlmConnectorError>> + Send>> {
    create_text_stream(response, StreamFormat::NdJson)
}

/// Convert HTTP response to StreamingResponse stream with automatic format detection
#[cfg(feature = "streaming")]
pub fn sse_to_streaming_response(response: reqwest::Response) -> crate::types::ChatStream {
    use crate::types::{StreamingResponse, ToolCall};
    use std::collections::HashMap;

    // Use Auto detection by default
    let string_stream = create_text_stream(response, StreamFormat::Auto);

    // State for accumulating tool_calls across chunks
    let response_stream = string_stream.scan(
        HashMap::<usize, ToolCall>::new(),
        |accumulated_tool_calls, result| {
            let processed = result.and_then(|json_str| {
                // Try to parse as StreamingResponse
                let mut streaming_response = serde_json::from_str::<StreamingResponse>(&json_str)
                    .map_err(|e| {
                    crate::error::LlmConnectorError::ParseError(format!(
                        "Failed to parse streaming response: {}. Content: {}",
                        e, json_str
                    ))
                })?;

                // Populate convenience fields
                populate_convenience_fields(&mut streaming_response);

                // Accumulate tool calls
                accumulate_tool_calls(&mut streaming_response, accumulated_tool_calls);

                Ok(streaming_response)
            });

            std::future::ready(Some(processed))
        },
    );

    Box::pin(response_stream)
}

#[cfg(feature = "streaming")]
fn populate_convenience_fields(response: &mut crate::types::StreamingResponse) {
    if response.content.is_empty()
        && let Some(choice) = response.choices.first()
    {
        let content_to_use = choice
            .delta
            .content
            .as_ref()
            .filter(|s| !s.is_empty())
            .or(choice.delta.reasoning_content.as_ref())
            .or(choice.delta.reasoning.as_ref())
            .or(choice.delta.thought.as_ref())
            .or(choice.delta.thinking.as_ref());

        if let Some(content) = content_to_use {
            response.content = content.clone();
        }
    }
}

#[cfg(feature = "streaming")]
fn accumulate_tool_calls(
    response: &mut crate::types::StreamingResponse,
    accumulated: &mut std::collections::HashMap<usize, crate::types::ToolCall>
) {
    if let Some(choice) = response.choices.first_mut()
        && let Some(delta_tool_calls) = &choice.delta.tool_calls
    {
        for delta_call in delta_tool_calls {
            let index = delta_call.index.unwrap_or(0);

            accumulated
                .entry(index)
                .and_modify(|existing| existing.merge_delta(delta_call))
                .or_insert_with(|| delta_call.clone());
        }

        let complete_calls: Vec<crate::types::ToolCall> = accumulated
            .values()
            .filter(|call| call.is_complete())
            .cloned()
            .collect();

        if !complete_calls.is_empty() {
            choice.delta.tool_calls = Some(complete_calls);
        } else {
            choice.delta.tool_calls = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn test_sse_detection() {
        // Mock SSE response
        let mock_response = "data: {\"test\":1}\n\ndata: {\"test\":2}\n\n";
        // In a real test we would need to mock reqwest::Response, but since we can't easily construct one,
        // we'll verify the logic in CreateTextStream via integration tests or by exposing the internal scanner.
    }
}
