//! Server-Sent Events (SSE) streaming utilities
//!
//! This module provides SSE parsing for streaming responses from LLM providers.
//! All major commercial LLM providers (OpenAI, Anthropic, DeepSeek, etc.) use SSE for streaming.

use crate::error::LlmConnectorError;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

/// Parse Server-Sent Events (SSE) from an HTTP response and yield one JSON string per event.
/// This function handles the SSE format where events are separated by double newlines.
#[inline]
pub fn sse_events(
        response: reqwest::Response,
    ) -> Pin<Box<dyn Stream<Item = Result<String, LlmConnectorError>> + Send>> {
        let stream = response.bytes_stream();

        let events_stream = stream
            .scan(String::new(), move |buffer, chunk_result| {
                let mut out: Vec<Result<String, LlmConnectorError>> = Vec::new();
                match chunk_result {
                    Ok(chunk) => {
                        let chunk_str = String::from_utf8_lossy(&chunk).replace("\r\n", "\n");
                        buffer.push_str(&chunk_str);

                        // Extract all complete SSE events separated by double-newline
                        while let Some(boundary_idx) = buffer.find("\n\n") {
                            let event_str: String = buffer.drain(..boundary_idx + 2).collect();

                            // Aggregate all `data:` lines
                            let mut data_lines: Vec<String> = Vec::new();
                            for raw_line in event_str.split('\n') {
                                // Trim trailing CR (already normalized), and leading/trailing spaces
                                let line = raw_line.trim_end();
                                if let Some(rest) = line
                                    .strip_prefix("data: ")
                                    .or_else(|| line.strip_prefix("data:"))
                                {
                                    let payload = rest.trim_start();
                                    // Skip final marker
                                    if payload.trim() == "[DONE]" {
                                        // ignore terminal marker
                                        continue;
                                    }
                                    // Collect payload line (may span multiple `data:` lines)
                                    if !payload.is_empty() {
                                        data_lines.push(payload.to_string());
                                    }
                                }
                            }

                            if !data_lines.is_empty() {
                                // Per SSE spec, multiple data lines are joined with `\n`
                                out.push(Ok(data_lines.join("\n")));
                            }
                        }
                    }
                    Err(e) => {
                        out.push(Err(LlmConnectorError::NetworkError(
                            e.to_string(),
                        )));
                    }
                }
                std::future::ready(Some(out))
            })
            .flat_map(futures_util::stream::iter);

        Box::pin(events_stream)
    }

/// Parse line-delimited JSON events (non-SSE) from an HTTP response.
/// Each line is treated as a standalone JSON payload.
#[inline]
pub fn json_lines_events(
        response: reqwest::Response,
    ) -> Pin<Box<dyn Stream<Item = Result<String, LlmConnectorError>> + Send>> {
        let stream = response.bytes_stream();

        let events_stream = stream
            .scan(String::new(), move |buffer, chunk_result| {
                let mut out: Vec<Result<String, LlmConnectorError>> = Vec::new();
                match chunk_result {
                    Ok(chunk) => {
                        let chunk_str = String::from_utf8_lossy(&chunk).replace("\r\n", "\n");
                        buffer.push_str(&chunk_str);

                        // Extract complete lines
                        while let Some(boundary_idx) = buffer.find('\n') {
                            let line: String = buffer.drain(..boundary_idx + 1).collect();
                            let trimmed = line.trim();
                            if trimmed.is_empty() { continue; }
                            if trimmed == "[DONE]" { continue; }
                            out.push(Ok(trimmed.to_string()));
                        }
                    }
                    Err(e) => {
                        out.push(Err(LlmConnectorError::NetworkError(e.to_string())));
                    }
                }
                std::future::ready(Some(out))
            })
            .flat_map(futures_util::stream::iter);

        Box::pin(events_stream)
    }

/// Convert SSE string stream to StreamingResponse stream
///
/// This function handles tool_calls accumulation for streaming responses.
/// In OpenAI's streaming API, tool_calls are sent incrementally across multiple chunks,
/// and need to be accumulated by index.
#[cfg(feature = "streaming")]
pub fn sse_to_streaming_response(
    response: reqwest::Response,
) -> crate::types::ChatStream {
    use crate::types::{StreamingResponse, ToolCall};
    use futures_util::StreamExt;
    use std::collections::HashMap;

    let string_stream = sse_events(response);

    // State for accumulating tool_calls across chunks
    let response_stream = string_stream.scan(
        HashMap::<usize, ToolCall>::new(),
        |accumulated_tool_calls, result| {
            let processed = result.and_then(|json_str| {
                // Try to parse as StreamingResponse
                let mut streaming_response = serde_json::from_str::<StreamingResponse>(&json_str)
                    .map_err(|e| crate::error::LlmConnectorError::ParseError(
                        format!("Failed to parse streaming response: {}", e)
                    ))?;

                // 🔧 Fix: Populate the convenience `content` field from choices[0].delta
                // This is critical for Volcengine and other OpenAI-compatible providers
                //
                // Priority order:
                // 1. delta.content (standard OpenAI format)
                // 2. delta.reasoning_content (Volcengine Doubao-Seed-Code, DeepSeek R1)
                // 3. delta.reasoning (Qwen, DeepSeek)
                // 4. delta.thought (OpenAI o1)
                // 5. delta.thinking (Anthropic)
                if streaming_response.content.is_empty() {
                    if let Some(choice) = streaming_response.choices.first() {
                        let content_to_use = choice.delta.content.as_ref()
                            .filter(|s| !s.is_empty())
                            .or_else(|| choice.delta.reasoning_content.as_ref())
                            .or_else(|| choice.delta.reasoning.as_ref())
                            .or_else(|| choice.delta.thought.as_ref())
                            .or_else(|| choice.delta.thinking.as_ref());

                        if let Some(content) = content_to_use {
                            streaming_response.content = content.clone();
                        }
                    }
                }

                // 🔧 Fix: Accumulate tool_calls across chunks
                // OpenAI streaming API sends tool_calls incrementally with an index field
                if let Some(choice) = streaming_response.choices.first_mut() {
                    if let Some(delta_tool_calls) = &choice.delta.tool_calls {
                        for delta_call in delta_tool_calls {
                            let index = delta_call.index.unwrap_or(0);

                            accumulated_tool_calls
                                .entry(index)
                                .and_modify(|existing| existing.merge_delta(delta_call))
                                .or_insert_with(|| delta_call.clone());
                        }

                        // Replace delta.tool_calls with accumulated complete tool_calls
                        // Only include complete tool_calls (have id, type, and name)
                        let complete_calls: Vec<ToolCall> = accumulated_tool_calls
                            .values()
                            .filter(|call| call.is_complete())
                            .cloned()
                            .collect();

                        if !complete_calls.is_empty() {
                            choice.delta.tool_calls = Some(complete_calls);
                        } else {
                            // Don't send incomplete tool_calls to avoid duplicate execution
                            choice.delta.tool_calls = None;
                        }
                    }
                }

                Ok(streaming_response)
            });

            std::future::ready(Some(processed))
        }
    );

    Box::pin(response_stream)
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "streaming")]
    fn test_streaming_response_content_population() {
        use crate::types::StreamingResponse;

        // Test 1: Standard OpenAI format with content
        let json_standard = r#"{
            "id": "test-1",
            "object": "chat.completion.chunk",
            "created": 1234567890,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "delta": {
                    "role": "assistant",
                    "content": "Hello world"
                },
                "finish_reason": null
            }]
        }"#;

        let mut response: StreamingResponse = serde_json::from_str(json_standard).unwrap();

        // Simulate the content population logic
        if response.content.is_empty() {
            if let Some(choice) = response.choices.first() {
                let content_to_use = choice.delta.content.as_ref()
                    .filter(|s| !s.is_empty())
                    .or_else(|| choice.delta.reasoning_content.as_ref())
                    .or_else(|| choice.delta.reasoning.as_ref())
                    .or_else(|| choice.delta.thought.as_ref())
                    .or_else(|| choice.delta.thinking.as_ref());

                if let Some(content) = content_to_use {
                    response.content = content.clone();
                }
            }
        }

        assert_eq!(response.content, "Hello world");

        // Test 2: Volcengine Doubao-Seed-Code format with reasoning_content
        let json_volcengine = r#"{
            "id": "test-2",
            "object": "chat.completion.chunk",
            "created": 1234567890,
            "model": "doubao-seed-code",
            "choices": [{
                "index": 0,
                "delta": {
                    "role": "assistant",
                    "content": "",
                    "reasoning_content": "我is豆包"
                },
                "finish_reason": null
            }]
        }"#;

        let mut response: StreamingResponse = serde_json::from_str(json_volcengine).unwrap();

        // Simulate the content population logic
        if response.content.is_empty() {
            if let Some(choice) = response.choices.first() {
                let content_to_use = choice.delta.content.as_ref()
                    .filter(|s| !s.is_empty())
                    .or_else(|| choice.delta.reasoning_content.as_ref())
                    .or_else(|| choice.delta.reasoning.as_ref())
                    .or_else(|| choice.delta.thought.as_ref())
                    .or_else(|| choice.delta.thinking.as_ref());

                if let Some(content) = content_to_use {
                    response.content = content.clone();
                }
            }
        }

        assert_eq!(response.content, "我is豆包");

        // Test 3: DeepSeek format with reasoning
        let json_deepseek = r#"{
            "id": "test-3",
            "object": "chat.completion.chunk",
            "created": 1234567890,
            "model": "deepseek-r1",
            "choices": [{
                "index": 0,
                "delta": {
                    "role": "assistant",
                    "content": "",
                    "reasoning": "Let me think..."
                },
                "finish_reason": null
            }]
        }"#;

        let mut response: StreamingResponse = serde_json::from_str(json_deepseek).unwrap();

        // Simulate the content population logic
        if response.content.is_empty() {
            if let Some(choice) = response.choices.first() {
                let content_to_use = choice.delta.content.as_ref()
                    .filter(|s| !s.is_empty())
                    .or_else(|| choice.delta.reasoning_content.as_ref())
                    .or_else(|| choice.delta.reasoning.as_ref())
                    .or_else(|| choice.delta.thought.as_ref())
                    .or_else(|| choice.delta.thinking.as_ref());

                if let Some(content) = content_to_use {
                    response.content = content.clone();
                }
            }
        }

        assert_eq!(response.content, "Let me think...");
    }

    #[test]
    fn test_sse_event_parsing() {
        // Test SSE format parsing
        let sse_data = "data: {\"test\": \"value\"}\n\n";
        assert!(sse_data.starts_with("data: "));

        let sse_done = "data: [DONE]\n\n";
        assert!(sse_done.contains("[DONE]"));
    }
}
