//! Middleware for request/response processing
//!
//! This module provides middleware components for enhancing request handling:
//! - Retry logic with exponential backoff
//! - Request/response logging
//! - Metrics collection
//! - Custom interceptors

pub mod interceptor;
pub mod logging;
pub mod metrics;
pub mod retry;

// Re-export main types
pub use interceptor::{
    HeaderInterceptor, Interceptor, InterceptorChain, SanitizationInterceptor,
    ValidationInterceptor,
};
pub use logging::LoggingMiddleware;
pub use metrics::{Metrics, MetricsMiddleware, MetricsSnapshot};
pub use retry::{RetryMiddleware, RetryPolicyBuilder};
