//! Retry middleware for automatic request retrying with exponential backoff
//!
//! This module provides a robust retry mechanism that automatically retries
//! failed requests based on configurable policies.

use crate::config::RetryConfig;
use crate::error::LlmConnectorError;
use std::future::Future;
use std::time::Duration;

/// Retry middleware that handles automatic retrying with exponential backoff
#[derive(Debug, Clone, Default)]
pub struct RetryMiddleware {
    config: RetryConfig,
}

impl RetryMiddleware {
    /// Create a new retry middleware with the given configuration
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute a function with retry logic
    ///
    /// # Arguments
    ///
    /// * `operation` - The async operation to execute
    ///
    /// # Returns
    ///
    /// The result of the operation, or an error if all retries are exhausted
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use llm_connector::middleware::RetryMiddleware;
    /// use llm_connector::config::RetryConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let retry = RetryMiddleware::new(RetryConfig::default());
    ///
    /// let result = retry.execute(|| async {
    ///     // Your async operation here
    ///     Ok::<_, llm_connector::error::LlmConnectorError>(())
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute<F, Fut, T>(&self, mut operation: F) -> Result<T, LlmConnectorError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, LlmConnectorError>>,
    {
        let mut attempts = 0;
        let mut backoff_ms = self.config.initial_backoff_ms;

        loop {
            match operation().await {
                Ok(result) => {
                    if attempts > 0 {
                        log::info!("Request succeeded after {} retries", attempts);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    attempts += 1;

                    // Check if we should retry
                    if !self.is_retriable(&e) {
                        log::debug!("Error is not retriable: {:?}", e);
                        return Err(e);
                    }

                    if attempts > self.config.max_retries {
                        log::warn!(
                            "Max retries ({}) exhausted for retriable error",
                            self.config.max_retries
                        );
                        return Err(LlmConnectorError::MaxRetriesExceeded(format!(
                            "Failed after {} attempts: {}",
                            attempts, e
                        )));
                    }

                    // Calculate backoff with jitter
                    let jitter = (rand::random::<f32>() * 0.3 - 0.15) * backoff_ms as f32;
                    let actual_backoff = (backoff_ms as f32 + jitter).max(0.0) as u64;
                    let capped_backoff = actual_backoff.min(self.config.max_backoff_ms);

                    log::info!(
                        "Retry attempt {}/{} after {}ms (error: {})",
                        attempts,
                        self.config.max_retries,
                        capped_backoff,
                        e
                    );

                    // Wait before retrying
                    tokio::time::sleep(Duration::from_millis(capped_backoff)).await;

                    // Update backoff for next iteration
                    backoff_ms = ((backoff_ms as f32 * self.config.backoff_multiplier) as u64)
                        .min(self.config.max_backoff_ms);
                }
            }
        }
    }

    /// Check if an error is retriable
    fn is_retriable(&self, error: &LlmConnectorError) -> bool {
        matches!(
            error,
            LlmConnectorError::RateLimitError(_)
                | LlmConnectorError::ServerError(_)
                | LlmConnectorError::TimeoutError(_)
                | LlmConnectorError::ConnectionError(_)
                | LlmConnectorError::NetworkError(_)
        )
    }

    /// Get the retry configuration
    pub fn config(&self) -> &RetryConfig {
        &self.config
    }
}

/// Retry policy builder for more flexible configuration
#[derive(Debug, Clone)]
pub struct RetryPolicyBuilder {
    max_retries: u32,
    initial_backoff_ms: u64,
    backoff_multiplier: f32,
    max_backoff_ms: u64,
}

impl RetryPolicyBuilder {
    /// Create a new retry policy builder with default values
    pub fn new() -> Self {
        let default = RetryConfig::default();
        Self {
            max_retries: default.max_retries,
            initial_backoff_ms: default.initial_backoff_ms,
            backoff_multiplier: default.backoff_multiplier,
            max_backoff_ms: default.max_backoff_ms,
        }
    }

    /// Set the maximum number of retries
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the initial backoff delay in milliseconds
    pub fn initial_backoff_ms(mut self, ms: u64) -> Self {
        self.initial_backoff_ms = ms;
        self
    }

    /// Set the backoff multiplier for exponential backoff
    pub fn backoff_multiplier(mut self, multiplier: f32) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Set the maximum backoff delay in milliseconds
    pub fn max_backoff_ms(mut self, ms: u64) -> Self {
        self.max_backoff_ms = ms;
        self
    }

    /// Build the retry configuration
    pub fn build(self) -> RetryConfig {
        RetryConfig {
            max_retries: self.max_retries,
            initial_backoff_ms: self.initial_backoff_ms,
            backoff_multiplier: self.backoff_multiplier,
            max_backoff_ms: self.max_backoff_ms,
        }
    }

    /// Build and create a retry middleware
    pub fn build_middleware(self) -> RetryMiddleware {
        RetryMiddleware::new(self.build())
    }
}

impl Default for RetryPolicyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let retry = RetryMiddleware::default();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry
            .execute(|| {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, LlmConnectorError>("success")
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let config = RetryConfig {
            max_retries: 3,
            initial_backoff_ms: 10,
            backoff_multiplier: 2.0,
            max_backoff_ms: 1000,
        };
        let retry = RetryMiddleware::new(config);
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry
            .execute(|| {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(LlmConnectorError::ServerError(
                            "temporary error".to_string(),
                        ))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_max_retries_exceeded() {
        let config = RetryConfig {
            max_retries: 2,
            initial_backoff_ms: 10,
            backoff_multiplier: 2.0,
            max_backoff_ms: 1000,
        };
        let retry = RetryMiddleware::new(config);
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry
            .execute(|| {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>(LlmConnectorError::ServerError("always fails".to_string()))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_non_retriable_error() {
        let retry = RetryMiddleware::default();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry
            .execute(|| {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>(LlmConnectorError::InvalidRequest("bad request".to_string()))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // No retries
    }

    #[test]
    fn test_retry_policy_builder() {
        let config = RetryPolicyBuilder::new()
            .max_retries(5)
            .initial_backoff_ms(500)
            .backoff_multiplier(3.0)
            .max_backoff_ms(10000)
            .build();

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_backoff_ms, 500);
        assert_eq!(config.backoff_multiplier, 3.0);
        assert_eq!(config.max_backoff_ms, 10000);
    }
}
