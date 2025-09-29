//! Utility functions and helpers

// Placeholder for future utility modules

#[cfg(feature = "streaming")]
pub mod streaming {
    use futures_util::{Stream, StreamExt};
    use std::pin::Pin;

    /// Parse Server-Sent Events (SSE) from an HTTP response and yield the `data:` payload lines.
    /// This utility handles buffering across network chunks and ignores the terminal `[DONE]` marker.
    #[inline]
    pub fn sse_data_lines(
        response: reqwest::Response,
    ) -> Pin<Box<dyn Stream<Item = Result<String, crate::error::LlmConnectorError>> + Send>> {
        let stream = response.bytes_stream();

        let lines_stream = stream
            .scan(String::new(), move |buffer, chunk_result| {
                let mut out: Vec<Result<String, crate::error::LlmConnectorError>> = Vec::new();
                match chunk_result {
                    Ok(chunk) => {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line_str = buffer.drain(..=newline_pos).collect::<String>();
                            let line = line_str.trim();
                            if line.starts_with("data: ") {
                                let payload = &line[6..];
                                if payload.trim() != "[DONE]" {
                                    out.push(Ok(payload.to_string()));
                                }
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

    /// Robust SSE parser that emits aggregated `data:` payload per SSE event.
    ///
    /// Improvements over `sse_data_lines`:
    /// - Splits strictly on SSE event boundaries (double-newline: \n\n), with CRLF normalization
    /// - Aggregates multiple `data:` lines into a single payload, separated by newlines per spec
    /// - Accepts both `data:` and `data: ` prefixes
    /// - Ignores the terminal `[DONE]` event gracefully
    ///
    /// Returns a stream of complete `data` payloads (one per SSE event).
    #[inline]
    pub fn sse_data_events(
        response: reqwest::Response,
    ) -> Pin<Box<dyn Stream<Item = Result<String, crate::error::LlmConnectorError>> + Send>> {
        let stream = response.bytes_stream();

        let events_stream = stream
            .scan(String::new(), move |buffer, chunk_result| {
                let mut out: Vec<Result<String, crate::error::LlmConnectorError>> = Vec::new();
                match chunk_result {
                    Ok(chunk) => {
                        // Normalize CRLF to LF so we can detect \n\n boundaries reliably
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
