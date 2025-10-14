//! Core traits and abstractions for llm-connector
//!
//! This module provides the fundamental abstractions for the two-tier architecture:
//! - Protocols: Pure API specifications (OpenAI, Anthropic)
//! - Providers: Service implementations (Aliyun, Zhipu, Ollama)

pub mod protocol;
pub mod provider;
pub mod http;
pub mod error;

// Re-export core traits
pub use protocol::Protocol;
pub use provider::Provider;
pub use http::HttpTransport;
pub use error::ErrorMapper;