//! Provider implementations
//!
//! This module contains provider-specific implementations for various LLM services.
//! Most providers use the `GenericProvider<A>` architecture from `protocols/core.rs`.
//!
//! ## Architecture Pattern
//! Most providers follow this pattern:
//! 1. Implement `ProviderAdapter` trait for protocol-specific logic
//! 2. Use `GenericProvider<YourAdapter>` for the actual provider implementation
//! 3. Export convenience functions for easy client creation
//!
//! ## Supported Providers
//! - **Aliyun**: DashScope API with custom request/response format
//! - **Zhipu**: GLM API with OpenAI-compatible mode
//! - **Tencent**: Hunyuan API with TC3-HMAC-SHA256 authentication
//! - **Ollama**: Local server with custom provider implementation

pub mod aliyun;
pub mod zhipu;
pub mod ollama;

#[cfg(feature = "tencent")]
pub mod tencent;

// Re-export provider types and functions
pub use aliyun::{AliyunProvider, aliyun};
pub use zhipu::{zhipu, zhipu_default, zhipu_with_timeout};
pub use ollama::{OllamaProvider, ollama, ollama_with_url};

#[cfg(feature = "tencent")]
pub use tencent::{TencentProvider, tencent, tencent_with_timeout};
