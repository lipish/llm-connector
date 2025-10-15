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
pub mod zhipu;

// Re-export core components
pub use core::{ErrorMapper, GenericProvider, HttpTransport, Provider, ProviderAdapter};

// Re-export new Protocol architecture
pub use crate::core::{Protocol, ErrorMapper as ProtocolErrorMapper};
pub use crate::core::provider::{Provider as NewProvider, ProtocolProvider};

// Re-export protocols (both old and new for compatibility)
pub use aliyun::{AliyunProvider, aliyun, aliyun_with_url};
pub use anthropic::AnthropicProtocol;
pub use ollama::{OllamaProvider, ollama, ollama_with_url};
pub use openai::OpenAIProtocol;
pub use zhipu::{ZhipuProtocol, zhipu, zhipu_default, zhipu_with_timeout};

// Re-export configuration
pub use crate::config::ProviderConfig;