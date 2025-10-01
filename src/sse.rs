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
