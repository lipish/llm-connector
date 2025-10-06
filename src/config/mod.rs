//! Configuration management for LLM providers
//!
//! This module provides simple configuration for LLM providers.
//!
//! # Direct API Key Configuration
//!
//! The simplest way to configure a provider:
//!
//! ```rust
//! use llm_connector::config::ProviderConfig;
//!
//! let config = ProviderConfig::new("your-api-key");
//! ```
//!
//! # Advanced Configuration
//!
//! For custom settings:
//!
//! ```rust
//! use llm_connector::config::{ProviderConfig, RetryConfig};
//!
//! let config = ProviderConfig::new("your-api-key")
//!     .with_base_url("https://api.example.com/v1")
//!     .with_timeout_ms(30000)
//!     .with_retry(RetryConfig {
//!         max_retries: 3,
//!         initial_backoff_ms: 1000,
//!         backoff_multiplier: 2.0,
//!         max_backoff_ms: 30000,
//!     })
//!     .with_header("X-Custom-Header", "value");
//! ```

pub mod provider;

// Re-export main types
pub use provider::{ProviderConfig, RetryConfig, SharedProviderConfig};