//! Utility functions and helpers

use crate::error::LlmConnectorError;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

/// Streaming utilities
pub mod streaming {
    use super::*;

    /// Parse Server-Sent Events (SSE) from an HTTP response and yield one JSON string per event.
    /// This function handles the SSE format where events are separated by double newlines.
    #[inline]
    pub fn sse_events(
        response: reqwest::Response,
    ) -> Pin<Box<dyn Stream<Item = Result<String, crate::error::LlmConnectorError>> + Send>> {
        let stream = response.bytes_stream();

        let events_stream = stream
            .scan(String::new(), move |buffer, chunk_result| {
                let mut out: Vec<Result<String, crate::error::LlmConnectorError>> = Vec::new();
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
                                if let Some(rest) = line.strip_prefix("data: ")
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
                        out.push(Err(crate::error::LlmConnectorError::NetworkError(
                            e.to_string(),
                        )));
                    }
                }
                std::future::ready(Some(out))
            })
            .flat_map(futures_util::stream::iter);

        Box::pin(events_stream)
    }

    /// Parse NDJSON (Newline Delimited JSON) from an HTTP response and yield one JSON string per line.
    /// Empty lines are ignored; CRLF will be normalized to LF to ensure consistent line splitting.
    #[inline]
    pub fn ndjson_events(
        response: reqwest::Response,
    ) -> Pin<Box<dyn Stream<Item = Result<String, crate::error::LlmConnectorError>> + Send>> {
        let stream = response.bytes_stream();

        let lines_stream = stream
            .scan(String::new(), move |buffer, chunk_result| {
                let mut out: Vec<Result<String, crate::error::LlmConnectorError>> = Vec::new();
                match chunk_result {
                    Ok(chunk) => {
                        let chunk_str = String::from_utf8_lossy(&chunk).replace("\r\n", "\n");
                        buffer.push_str(&chunk_str);
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line_str: String = buffer.drain(..=newline_pos).collect();
                            let line = line_str.trim();
                            if !line.is_empty() {
                                out.push(Ok(line.to_string()));
                            }
                        }
                    }
                    Err(e) => {
                        out.push(Err(crate::error::LlmConnectorError::NetworkError(
                            e.to_string(),
                        )));
                    }
                }
                std::future::ready(Some(out))
            })
            .flat_map(futures_util::stream::iter);

        Box::pin(lines_stream)
    }
}

/// Validate a chat request
pub fn validate_chat_request(request: &crate::types::ChatRequest) -> Result<(), LlmConnectorError> {
    if request.messages.is_empty() {
        return Err(LlmConnectorError::InvalidRequest(
            "Messages cannot be empty".to_string(),
        ));
    }
    Ok(())
}

/// Clean model name by removing provider prefix
pub fn clean_model_name(model: &str) -> &str {
    if let Some(idx) = model.find('/') {
        &model[idx + 1..]
    } else {
        model
    }
}

/// Detect provider from model name
pub fn detect_provider_from_model(model: &str) -> Option<&str> {
    if let Some(idx) = model.find('/') {
        Some(&model[..idx])
    } else {
        None
    }
}
