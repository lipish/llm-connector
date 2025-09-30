//! Metrics middleware for collecting performance and usage statistics
//!
//! This module provides middleware for collecting metrics about LLM API usage.

use crate::types::{ChatRequest, ChatResponse};
use crate::error::LlmConnectorError;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Metrics collector for LLM API usage
#[derive(Debug, Clone)]
pub struct MetricsMiddleware {
    metrics: Arc<Metrics>,
}

/// Metrics data structure
#[derive(Debug)]
pub struct Metrics {
    // Request counters
    requests_total: AtomicU64,
    requests_success: AtomicU64,
    requests_failed: AtomicU64,
    
    // Token counters
    tokens_prompt: AtomicU64,
    tokens_completion: AtomicU64,
    tokens_total: AtomicU64,
    
    // Timing (in milliseconds)
    total_duration_ms: AtomicU64,
    min_duration_ms: AtomicU64,
    max_duration_ms: AtomicU64,
    
    // Error counters by type
    rate_limit_errors: AtomicU64,
    server_errors: AtomicU64,
    timeout_errors: AtomicU64,
    auth_errors: AtomicU64,
    other_errors: AtomicU64,
}

impl Metrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            requests_success: AtomicU64::new(0),
            requests_failed: AtomicU64::new(0),
            tokens_prompt: AtomicU64::new(0),
            tokens_completion: AtomicU64::new(0),
            tokens_total: AtomicU64::new(0),
            total_duration_ms: AtomicU64::new(0),
            min_duration_ms: AtomicU64::new(u64::MAX),
            max_duration_ms: AtomicU64::new(0),
            rate_limit_errors: AtomicU64::new(0),
            server_errors: AtomicU64::new(0),
            timeout_errors: AtomicU64::new(0),
            auth_errors: AtomicU64::new(0),
            other_errors: AtomicU64::new(0),
        }
    }

    /// Record a successful request
    pub fn record_success(&self, response: &ChatResponse, duration: Duration) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.requests_success.fetch_add(1, Ordering::Relaxed);
        
        // Record token usage
        if let Some(usage) = &response.usage {
            self.tokens_prompt.fetch_add(usage.prompt_tokens as u64, Ordering::Relaxed);
            self.tokens_completion.fetch_add(usage.completion_tokens as u64, Ordering::Relaxed);
            self.tokens_total.fetch_add(usage.total_tokens as u64, Ordering::Relaxed);
        }
        
        // Record timing
        let duration_ms = duration.as_millis() as u64;
        self.total_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
        
        // Update min duration
        self.min_duration_ms.fetch_min(duration_ms, Ordering::Relaxed);
        
        // Update max duration
        self.max_duration_ms.fetch_max(duration_ms, Ordering::Relaxed);
    }

    /// Record a failed request
    pub fn record_failure(&self, error: &LlmConnectorError, duration: Duration) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.requests_failed.fetch_add(1, Ordering::Relaxed);
        
        // Record timing
        let duration_ms = duration.as_millis() as u64;
        self.total_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
        
        // Categorize error
        match error {
            LlmConnectorError::RateLimitError(_) => {
                self.rate_limit_errors.fetch_add(1, Ordering::Relaxed);
            }
            LlmConnectorError::ServerError(_) => {
                self.server_errors.fetch_add(1, Ordering::Relaxed);
            }
            LlmConnectorError::TimeoutError(_) => {
                self.timeout_errors.fetch_add(1, Ordering::Relaxed);
            }
            LlmConnectorError::AuthenticationError(_) => {
                self.auth_errors.fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                self.other_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let total = self.requests_total.load(Ordering::Relaxed);
        let success = self.requests_success.load(Ordering::Relaxed);
        let failed = self.requests_failed.load(Ordering::Relaxed);
        let total_duration = self.total_duration_ms.load(Ordering::Relaxed);
        
        MetricsSnapshot {
            requests_total: total,
            requests_success: success,
            requests_failed: failed,
            success_rate: if total > 0 {
                (success as f64 / total as f64) * 100.0
            } else {
                0.0
            },
            tokens_prompt: self.tokens_prompt.load(Ordering::Relaxed),
            tokens_completion: self.tokens_completion.load(Ordering::Relaxed),
            tokens_total: self.tokens_total.load(Ordering::Relaxed),
            avg_duration_ms: if total > 0 {
                total_duration / total
            } else {
                0
            },
            min_duration_ms: {
                let min = self.min_duration_ms.load(Ordering::Relaxed);
                if min == u64::MAX { 0 } else { min }
            },
            max_duration_ms: self.max_duration_ms.load(Ordering::Relaxed),
            rate_limit_errors: self.rate_limit_errors.load(Ordering::Relaxed),
            server_errors: self.server_errors.load(Ordering::Relaxed),
            timeout_errors: self.timeout_errors.load(Ordering::Relaxed),
            auth_errors: self.auth_errors.load(Ordering::Relaxed),
            other_errors: self.other_errors.load(Ordering::Relaxed),
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.requests_total.store(0, Ordering::Relaxed);
        self.requests_success.store(0, Ordering::Relaxed);
        self.requests_failed.store(0, Ordering::Relaxed);
        self.tokens_prompt.store(0, Ordering::Relaxed);
        self.tokens_completion.store(0, Ordering::Relaxed);
        self.tokens_total.store(0, Ordering::Relaxed);
        self.total_duration_ms.store(0, Ordering::Relaxed);
        self.min_duration_ms.store(u64::MAX, Ordering::Relaxed);
        self.max_duration_ms.store(0, Ordering::Relaxed);
        self.rate_limit_errors.store(0, Ordering::Relaxed);
        self.server_errors.store(0, Ordering::Relaxed);
        self.timeout_errors.store(0, Ordering::Relaxed);
        self.auth_errors.store(0, Ordering::Relaxed);
        self.other_errors.store(0, Ordering::Relaxed);
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_failed: u64,
    pub success_rate: f64,
    pub tokens_prompt: u64,
    pub tokens_completion: u64,
    pub tokens_total: u64,
    pub avg_duration_ms: u64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub rate_limit_errors: u64,
    pub server_errors: u64,
    pub timeout_errors: u64,
    pub auth_errors: u64,
    pub other_errors: u64,
}

impl MetricsSnapshot {
    /// Format metrics as a human-readable string
    pub fn format(&self) -> String {
        format!(
            "Requests: {} total ({} success, {} failed, {:.2}% success rate)\n\
             Tokens: {} total ({} prompt, {} completion)\n\
             Duration: {} avg, {} min, {} max (ms)\n\
             Errors: {} rate limit, {} server, {} timeout, {} auth, {} other",
            self.requests_total,
            self.requests_success,
            self.requests_failed,
            self.success_rate,
            self.tokens_total,
            self.tokens_prompt,
            self.tokens_completion,
            self.avg_duration_ms,
            self.min_duration_ms,
            self.max_duration_ms,
            self.rate_limit_errors,
            self.server_errors,
            self.timeout_errors,
            self.auth_errors,
            self.other_errors
        )
    }
}

impl MetricsMiddleware {
    /// Create a new metrics middleware
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Metrics::new()),
        }
    }

    /// Get a reference to the metrics
    pub fn metrics(&self) -> &Arc<Metrics> {
        &self.metrics
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.metrics.reset();
    }

    /// Execute a request with metrics collection
    pub async fn execute<F, Fut>(
        &self,
        operation: F,
    ) -> Result<ChatResponse, LlmConnectorError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<ChatResponse, LlmConnectorError>>,
    {
        let start = Instant::now();
        
        match operation().await {
            Ok(response) => {
                let duration = start.elapsed();
                self.metrics.record_success(&response, duration);
                Ok(response)
            }
            Err(error) => {
                let duration = start.elapsed();
                self.metrics.record_failure(&error, duration);
                Err(error)
            }
        }
    }
}

impl Default for MetricsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Message, Choice, Usage};

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        let snapshot = metrics.snapshot();
        
        assert_eq!(snapshot.requests_total, 0);
        assert_eq!(snapshot.requests_success, 0);
        assert_eq!(snapshot.requests_failed, 0);
    }

    #[test]
    fn test_record_success() {
        let metrics = Metrics::new();
        
        let response = ChatResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: "test".to_string(),
            choices: vec![],
            usage: Some(Usage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        };
        
        metrics.record_success(&response, Duration::from_millis(100));
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.requests_total, 1);
        assert_eq!(snapshot.requests_success, 1);
        assert_eq!(snapshot.tokens_total, 30);
    }

    #[test]
    fn test_record_failure() {
        let metrics = Metrics::new();
        
        let error = LlmConnectorError::RateLimitError("test".to_string());
        metrics.record_failure(&error, Duration::from_millis(50));
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.requests_total, 1);
        assert_eq!(snapshot.requests_failed, 1);
        assert_eq!(snapshot.rate_limit_errors, 1);
    }
}
