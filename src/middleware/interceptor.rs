//! Request/Response interceptor middleware
//!
//! This module provides a flexible interceptor system for modifying requests
//! and responses before and after they are sent to the LLM provider.

use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse};
use async_trait::async_trait;
use std::sync::Arc;

/// Trait for request/response interceptors
///
/// Implement this trait to create custom interceptors that can modify
/// requests before they are sent and responses after they are received.
#[async_trait]
pub trait Interceptor: Send + Sync {
    /// Called before a request is sent
    ///
    /// This method can modify the request or return an error to prevent
    /// the request from being sent.
    async fn before_request(&self, request: &mut ChatRequest) -> Result<(), LlmConnectorError> {
        let _ = request;
        Ok(())
    }

    /// Called after a successful response is received
    ///
    /// This method can modify the response before it is returned to the caller.
    async fn after_response(&self, response: &mut ChatResponse) -> Result<(), LlmConnectorError> {
        let _ = response;
        Ok(())
    }

    /// Called when an error occurs
    ///
    /// This method can transform the error or perform cleanup actions.
    async fn on_error(&self, error: &mut LlmConnectorError) -> Result<(), LlmConnectorError> {
        let _ = error;
        Ok(())
    }
}

/// Interceptor chain that executes multiple interceptors in sequence
#[derive(Clone)]
pub struct InterceptorChain {
    interceptors: Vec<Arc<dyn Interceptor>>,
}

impl InterceptorChain {
    /// Create a new empty interceptor chain
    pub fn new() -> Self {
        Self {
            interceptors: Vec::new(),
        }
    }

    /// Add an interceptor to the chain
    pub fn with_interceptor(mut self, interceptor: Arc<dyn Interceptor>) -> Self {
        self.interceptors.push(interceptor);
        self
    }

    /// Execute all before_request interceptors
    pub async fn before_request(&self, request: &mut ChatRequest) -> Result<(), LlmConnectorError> {
        for interceptor in &self.interceptors {
            interceptor.before_request(request).await?;
        }
        Ok(())
    }

    /// Execute all after_response interceptors
    pub async fn after_response(
        &self,
        response: &mut ChatResponse,
    ) -> Result<(), LlmConnectorError> {
        for interceptor in &self.interceptors {
            interceptor.after_response(response).await?;
        }
        Ok(())
    }

    /// Execute all on_error interceptors
    pub async fn on_error(&self, error: &mut LlmConnectorError) -> Result<(), LlmConnectorError> {
        for interceptor in &self.interceptors {
            interceptor.on_error(error).await?;
        }
        Ok(())
    }

    /// Execute a request with the interceptor chain
    pub async fn execute<F, Fut>(
        &self,
        mut request: ChatRequest,
        operation: F,
    ) -> Result<ChatResponse, LlmConnectorError>
    where
        F: FnOnce(ChatRequest) -> Fut,
        Fut: std::future::Future<Output = Result<ChatResponse, LlmConnectorError>>,
    {
        // Execute before_request interceptors
        self.before_request(&mut request).await?;

        // Execute the operation
        match operation(request).await {
            Ok(mut response) => {
                // Execute after_response interceptors
                self.after_response(&mut response).await?;
                Ok(response)
            }
            Err(mut error) => {
                // Execute on_error interceptors
                let _ = self.on_error(&mut error).await;
                Err(error)
            }
        }
    }
}

impl Default for InterceptorChain {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Built-in Interceptors
// ============================================================================

/// Interceptor that adds custom headers to requests
#[derive(Debug, Clone)]
pub struct HeaderInterceptor {
    headers: std::collections::HashMap<String, String>,
}

impl HeaderInterceptor {
    /// Create a new header interceptor
    pub fn new() -> Self {
        Self {
            headers: std::collections::HashMap::new(),
        }
    }

    /// Add a header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Get the headers
    pub fn headers(&self) -> &std::collections::HashMap<String, String> {
        &self.headers
    }
}

impl Default for HeaderInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Interceptor for HeaderInterceptor {
    async fn before_request(&self, _request: &mut ChatRequest) -> Result<(), LlmConnectorError> {
        // Headers are typically added at the HTTP transport level
        // This is a placeholder for demonstration
        Ok(())
    }
}

/// Interceptor that validates requests
#[derive(Debug, Clone)]
pub struct ValidationInterceptor {
    max_tokens: Option<u32>,
    max_messages: Option<usize>,
}

impl ValidationInterceptor {
    /// Create a new validation interceptor
    pub fn new() -> Self {
        Self {
            max_tokens: None,
            max_messages: None,
        }
    }

    /// Set maximum tokens allowed
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set maximum messages allowed
    pub fn with_max_messages(mut self, max_messages: usize) -> Self {
        self.max_messages = Some(max_messages);
        self
    }
}

impl Default for ValidationInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Interceptor for ValidationInterceptor {
    async fn before_request(&self, request: &mut ChatRequest) -> Result<(), LlmConnectorError> {
        // Validate max tokens
        if let Some(max_tokens) = self.max_tokens {
            if let Some(requested_tokens) = request.max_tokens {
                if requested_tokens > max_tokens {
                    return Err(LlmConnectorError::InvalidRequest(format!(
                        "Requested tokens ({}) exceeds maximum ({})",
                        requested_tokens, max_tokens
                    )));
                }
            }
        }

        // Validate max messages
        if let Some(max_messages) = self.max_messages {
            if request.messages.len() > max_messages {
                return Err(LlmConnectorError::InvalidRequest(format!(
                    "Number of messages ({}) exceeds maximum ({})",
                    request.messages.len(),
                    max_messages
                )));
            }
        }

        Ok(())
    }
}

/// Interceptor that sanitizes responses
#[derive(Debug, Clone)]
pub struct SanitizationInterceptor {
    remove_system_fingerprint: bool,
}

impl SanitizationInterceptor {
    /// Create a new sanitization interceptor
    pub fn new() -> Self {
        Self {
            remove_system_fingerprint: false,
        }
    }

    /// Set whether to remove system fingerprint
    pub fn with_remove_system_fingerprint(mut self, remove: bool) -> Self {
        self.remove_system_fingerprint = remove;
        self
    }
}

impl Default for SanitizationInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Interceptor for SanitizationInterceptor {
    async fn after_response(&self, response: &mut ChatResponse) -> Result<(), LlmConnectorError> {
        if self.remove_system_fingerprint {
            response.system_fingerprint = None;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Message;

    #[tokio::test]
    async fn test_interceptor_chain() {
        let chain = InterceptorChain::new()
            .with_interceptor(Arc::new(ValidationInterceptor::new().with_max_tokens(1000)))
            .with_interceptor(Arc::new(SanitizationInterceptor::new()));

        let request = ChatRequest {
            model: "test".to_string(),
            messages: vec![Message::user("Hello")],
            max_tokens: Some(100),
            temperature: None,
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

        let result = chain
            .execute(request, |req| async move {
                Ok(ChatResponse {
                    id: "test".to_string(),
                    object: "chat.completion".to_string(),
                    created: 0,
                    model: req.model,
                    choices: vec![],
                    usage: None,
                    system_fingerprint: Some("test-fingerprint".to_string()),
                })
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validation_interceptor() {
        let interceptor = ValidationInterceptor::new().with_max_tokens(100);

        let mut request = ChatRequest {
            model: "test".to_string(),
            messages: vec![],
            max_tokens: Some(200),
            temperature: None,
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

        let result = interceptor.before_request(&mut request).await;
        assert!(result.is_err());
    }
}
