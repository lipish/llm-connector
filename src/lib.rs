//! # llm-connector
//!
//! Next-generation Rust library for LLM protocol abstraction.
//!
//! Supports 5 protocols: OpenAI, Anthropic, Aliyun, Zhipu, Ollama.
//! Clean architecture with clear Protocol/Provider separation.
//!
//! ## Quick Start
//!
//! ### OpenAI Protocol
//! ```rust,no_run
//! use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // OpenAI
//!     let client = LlmClient::openai("sk-...")?;
//!
//!     let request = ChatRequest {
//!         model: "gpt-4".to_string(),
//!         messages: vec![Message {
//!             role: Role::User,
//!             content: "Hello!".to_string(),
//!             ..Default::default()
//!         }],
//!         ..Default::default()
//!     };
//!
//!     let response = client.chat(&request).await?;
//!     println!("Response: {}", response.content);
//!     Ok(())
//! }
//! ```
//!
//! ### Anthropic Protocol
//! ```rust,no_run
//! use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = LlmClient::anthropic("sk-ant-...")?;
//!     let request = ChatRequest {
//!         model: "claude-3-5-sonnet-20241022".to_string(),
//!         messages: vec![Message {
//!             role: Role::User,
//!             content: "Hello!".to_string(),
//!             ..Default::default()
//!         }],
//!         ..Default::default()
//!     };
//!
//!     let response = client.chat(&request).await?;
//!     println!("Response: {}", response.content);
//!     Ok(())
//! }
//! ```
//!
//! ### Aliyun Protocol (DashScope)
//! ```rust,no_run
//! use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = LlmClient::aliyun("sk-...")?;
//!     let request = ChatRequest {
//!         model: "qwen-turbo".to_string(),
//!         messages: vec![Message {
//!             role: Role::User,
//!             content: "Hello!".to_string(),
//!             ..Default::default()
//!         }],
//!         ..Default::default()
//!     };
//!
//!     let response = client.chat(&request).await?;
//!     println!("Response: {}", response.content);
//!     Ok(())
//! }
//! ```
//!
//! ### Ollama Protocol (Local)
//! ```rust,no_run
//! use llm_connector::{LlmClient, Provider, types::{ChatRequest, Message, Role}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Default: localhost:11434
//!     let client = LlmClient::ollama()?;
//!
//!     // Custom URL
//!     let client = LlmClient::ollama_with_base_url("http://192.168.1.100:11434")?;
//!
//!     let request = ChatRequest {
//!         model: "llama3.2".to_string(),
//!         messages: vec![Message {
//!             role: Role::User,
//!             content: "Hello!".to_string(),
//!             ..Default::default()
//!         }],
//!         ..Default::default()
//!     };
//!
//!     let response = client.chat(&request).await?;
//!     println!("Response: {}", response.content);
//!
//!     // Ollama special features
//!     if let Some(ollama) = client.as_ollama() {
//!         let models = ollama.models().await?;
//!         println!("Available models: {:?}", models);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Installation
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! llm-connector = "0.2"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Optional features:
//! ```toml
//! llm-connector = { version = "0.2", features = ["streaming"] }
//! ```

// Core modules (V2 Architecture - Default)
pub mod client;
pub mod config;
pub mod core;
pub mod error;
pub mod protocols;
pub mod providers;
pub mod types;

// Server-Sent Events (SSE) utilities
pub mod sse;

// V1 Architecture (Legacy - for backward compatibility)
#[cfg(feature = "v1-legacy")]
pub mod v1;

// Re-exports for convenience (V2 Architecture)
pub use client::LlmClient;
pub use config::ProviderConfig;
pub use error::LlmConnectorError;
pub use types::{ChatRequest, ChatResponse, Choice, Message, Usage, Role};

// Re-export core traits
pub use core::{Protocol, Provider, GenericProvider, HttpClient};

// Re-export protocols
// 导出标准协议
pub use protocols::{OpenAIProtocol, AnthropicProtocol};

// 导出私有协议（从 providers 中）
pub use providers::{AliyunProtocol, ZhipuProtocol};

// Re-export providers
pub use providers::{
    OpenAIProvider, AliyunProvider, AnthropicProvider, ZhipuProvider, OllamaProvider,
    // Convenience functions
    openai, aliyun, anthropic, zhipu, ollama,
};

#[cfg(feature = "streaming")]
pub use types::{
    ChatStream, Delta, StreamingChoice, StreamingResponse,
    StreamingFormat, StreamingConfig, OllamaStreamChunk, OllamaMessage, OllamaChatStream,
    StreamFormat, StreamChunk, UniversalChatStream
};

