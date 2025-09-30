//! Middleware for request/response processing
//!
//! This module provides middleware components for enhancing request handling:
//! - Retry logic with exponential backoff
//! - Request/response logging
//! - Metrics collection
//! - Custom interceptors

pub mod retry;
pub mod logging;
pub mod metrics;
pub mod interceptor;

// Re-export main types
pub use retry::{RetryMiddleware, RetryPolicyBuilder};
pub use logging::LoggingMiddleware;
pub use metrics::{MetricsMiddleware, Metrics, MetricsSnapshot};
pub use interceptor::{
    Interceptor, InterceptorChain,
    HeaderInterceptor, ValidationInterceptor, SanitizationInterceptor,
};
