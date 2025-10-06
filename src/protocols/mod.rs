//! Minimal Protocol Implementation
//!
//! This library focuses on protocol abstraction rather than individual providers.
//! Only 4 protocols are supported, making the API minimal and easy to use.
//!
//! # Supported Protocols
//!
//! ## 1. OpenAI Protocol
//! Used by OpenAI's API.
//!
//! ## 2. Anthropic Protocol
//! Used by Anthropic's Claude models.
//!
//! ## 3. Aliyun Protocol
//! Custom protocol used by Aliyun DashScope (Qwen models).
//!
//! ## 4. Ollama Protocol
//! Local LLM server protocol. No API key required.

pub mod aliyun;
pub mod anthropic;
pub mod core;
pub mod ollama;
pub mod openai;

// Re-export core components
pub use core::{ErrorMapper, GenericProvider, HttpTransport, Provider, ProviderAdapter};

// Re-export protocols
pub use aliyun::AliyunProtocol;
pub use anthropic::AnthropicProtocol;
pub use ollama::OllamaProtocol;
pub use openai::OpenAIProtocol;

// Re-export configuration
pub use crate::config::ProviderConfig;