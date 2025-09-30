//! Logging middleware for request/response tracking
//!
//! This module provides middleware for logging LLM API requests and responses.

use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse};
use std::time::Instant;

/// Logging middleware for tracking requests and responses
#[derive(Debug, Clone)]
pub struct LoggingMiddleware {
    /// Whether to log request bodies
    log_request_body: bool,
    /// Whether to log response bodies
    log_response_body: bool,
    /// Whether to log timing information
    log_timing: bool,
    /// Whether to log token usage
    log_usage: bool,
}

impl LoggingMiddleware {
    /// Create a new logging middleware with default settings
    pub fn new() -> Self {
        Self {
            log_request_body: true,
            log_response_body: true,
            log_timing: true,
            log_usage: true,
        }
    }

    /// Create a minimal logging middleware (only errors and timing)
    pub fn minimal() -> Self {
        Self {
            log_request_body: false,
            log_response_body: false,
            log_timing: true,
            log_usage: false,
        }
    }

    /// Set whether to log request bodies
    pub fn with_request_body(mut self, enabled: bool) -> Self {
        self.log_request_body = enabled;
        self
    }

    /// Set whether to log response bodies
    pub fn with_response_body(mut self, enabled: bool) -> Self {
        self.log_response_body = enabled;
        self
    }

    /// Set whether to log timing information
    pub fn with_timing(mut self, enabled: bool) -> Self {
        self.log_timing = enabled;
        self
    }

    /// Set whether to log token usage
    pub fn with_usage(mut self, enabled: bool) -> Self {
        self.log_usage = enabled;
        self
    }

    /// Log before sending a request
    pub fn log_request(&self, provider: &str, request: &ChatRequest) {
        log::info!("Sending request to provider: {}", provider);

        if self.log_request_body {
            log::debug!("Request model: {}", request.model);
            log::debug!("Request messages: {} messages", request.messages.len());

            if let Some(max_tokens) = request.max_tokens {
                log::debug!("Request max_tokens: {}", max_tokens);
            }

            if let Some(temperature) = request.temperature {
                log::debug!("Request temperature: {}", temperature);
            }
        }
    }

    /// Log after receiving a response
    pub fn log_response(
        &self,
        provider: &str,
        response: &ChatResponse,
        duration: std::time::Duration,
    ) {
        log::info!("Received response from provider: {}", provider);

        if self.log_timing {
            log::info!("Request duration: {:?}", duration);
        }

        if self.log_response_body {
            log::debug!("Response ID: {}", response.id);
            log::debug!("Response model: {}", response.model);
            log::debug!("Response choices: {}", response.choices.len());

            if let Some(first_choice) = response.choices.first() {
                let content_preview = if first_choice.message.content.len() > 100 {
                    format!("{}...", &first_choice.message.content[..100])
                } else {
                    first_choice.message.content.clone()
                };
                log::debug!("Response content preview: {}", content_preview);

                if let Some(finish_reason) = &first_choice.finish_reason {
                    log::debug!("Finish reason: {}", finish_reason);
                }
            }
        }

        if self.log_usage {
            if let Some(usage) = &response.usage {
                log::info!(
                    "Token usage - Prompt: {}, Completion: {}, Total: {}",
                    usage.prompt_tokens,
                    usage.completion_tokens,
                    usage.total_tokens
                );
            }
        }
    }

    /// Log an error
    pub fn log_error(
        &self,
        provider: &str,
        error: &LlmConnectorError,
        duration: std::time::Duration,
    ) {
        log::error!(
            "Request to {} failed after {:?}: {}",
            provider,
            duration,
            error
        );
    }

    /// Execute a request with logging
    pub async fn execute<F, Fut>(
        &self,
        provider: &str,
        request: &ChatRequest,
        operation: F,
    ) -> Result<ChatResponse, LlmConnectorError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<ChatResponse, LlmConnectorError>>,
    {
        let start = Instant::now();

        self.log_request(provider, request);

        match operation().await {
            Ok(response) => {
                let duration = start.elapsed();
                self.log_response(provider, &response, duration);
                Ok(response)
            }
            Err(error) => {
                let duration = start.elapsed();
                self.log_error(provider, &error, duration);
                Err(error)
            }
        }
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Choice, Message, Usage};

    #[test]
    fn test_logging_middleware_creation() {
        let logger = LoggingMiddleware::new();
        assert!(logger.log_request_body);
        assert!(logger.log_response_body);
        assert!(logger.log_timing);
        assert!(logger.log_usage);
    }

    #[test]
    fn test_minimal_logging() {
        let logger = LoggingMiddleware::minimal();
        assert!(!logger.log_request_body);
        assert!(!logger.log_response_body);
        assert!(logger.log_timing);
        assert!(!logger.log_usage);
    }

    #[test]
    fn test_builder_pattern() {
        let logger = LoggingMiddleware::new()
            .with_request_body(false)
            .with_response_body(false)
            .with_timing(true)
            .with_usage(true);

        assert!(!logger.log_request_body);
        assert!(!logger.log_response_body);
        assert!(logger.log_timing);
        assert!(logger.log_usage);
    }

    #[tokio::test]
    async fn test_execute_with_logging() {
        let logger = LoggingMiddleware::minimal();

        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }],
            max_tokens: Some(100),
            temperature: Some(0.7),
            top_p: None,
            stop: None,
            tools: None,
            tool_choice: None,
            frequency_penalty: None,
            logit_bias: None,
            presence_penalty: None,
            response_format: None,
            seed: None,
            user: None,
            stream: None,
        };

        let result = logger
            .execute("test-provider", &request, || async {
                Ok(ChatResponse {
                    id: "test-id".to_string(),
                    object: "chat.completion".to_string(),
                    created: 0,
                    model: "test-model".to_string(),
                    choices: vec![Choice {
                        index: 0,
                        message: Message {
                            role: "assistant".to_string(),
                            content: "Hello!".to_string(),
                            name: None,
                            tool_calls: None,
                            tool_call_id: None,
                        },
                        finish_reason: Some("stop".to_string()),
                        logprobs: None,
                    }],
                    usage: Some(Usage {
                        prompt_tokens: 10,
                        completion_tokens: 5,
                        total_tokens: 15,
                        prompt_cache_hit_tokens: None,
                        prompt_cache_miss_tokens: None,
                        prompt_tokens_details: None,
                        completion_tokens_details: None,
                    }),
                    system_fingerprint: None,
                })
            })
            .await;

        assert!(result.is_ok());
    }
}
