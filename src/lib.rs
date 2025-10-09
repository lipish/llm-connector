//! # llm-connector
//!
//! Minimal Rust library for LLM protocol abstraction.
//!
//! Supports 4 protocols: OpenAI, Anthropic, Aliyun, Ollama.
//! No complex configuration - just pick a protocol and start chatting.
//!
//! ## Quick Start
//!
//! ### OpenAI Protocol
//! ```rust,no_run
//! use llm_connector::{LlmClient, ChatRequest, Message};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // OpenAI (default base URL)
//!     let client = LlmClient::openai("sk-...", None);
//!
//!     let request = ChatRequest {
//!         model: "gpt-4".to_string(),
//!         messages: vec![Message::user("Hello!")],
//!         ..Default::default()
//!     };
//!
//!     let response = client.chat(&request).await?;
//!     println!("Response: {}", response.choices[0].message.content);
//!     Ok(())
//! }
//! ```
//!
//! ### Anthropic Protocol
//! ```rust,ignore
//! use llm_connector::{LlmClient, ChatRequest, Message};
//!
//! let client = LlmClient::anthropic("sk-ant-...");
//! let request = ChatRequest {
//!     model: "claude-3-5-sonnet-20241022".to_string(),
//!     messages: vec![Message::user("Hello!")],
//!     ..Default::default()
//! };
//!
//! let response = client.chat(&request).await?;
//! println!("Response: {}", response.choices[0].message.content);
//! ```
//!
//! ### Aliyun Protocol (DashScope)
//! ```rust,ignore
//! use llm_connector::{LlmClient, ChatRequest, Message};
//!
//! let client = LlmClient::aliyun("sk-...");
//! let request = ChatRequest {
//!     model: "qwen-turbo".to_string(),
//!     messages: vec![Message::user("Hello!")],
//!     ..Default::default()
//! };
//!
//! let response = client.chat(&request).await?;
//! println!("Response: {}", response.choices[0].message.content);
//! ```
//!
//! ### Ollama Protocol (Local)
//! ```rust,ignore
//! use llm_connector::{LlmClient, ChatRequest, Message};
//!
//! // Default: localhost:11434
//! let client = LlmClient::ollama(None);
//!
//! // Custom URL
//! let client = LlmClient::ollama(Some("http://192.168.1.100:11434"));
//!
//! let request = ChatRequest {
//!     model: "llama3.2".to_string(),
//!     messages: vec![Message::user("Hello!")],
//!     ..Default::default()
//! };
//!
//! let response = client.chat(&request).await?;
//! println!("Response: {}", response.choices[0].message.content);
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

// Core modules
pub mod client;
pub mod config;
pub mod error;
pub mod protocols;
pub mod types;
pub mod ollama;


// Server-Sent Events (SSE) utilities
pub mod sse;

// Re-exports for convenience
pub use client::LlmClient;
pub use config::ProviderConfig;
pub use error::LlmConnectorError;
pub use types::{ChatRequest, ChatResponse, Choice, Message, Usage};

#[cfg(feature = "streaming")]
pub use types::{ChatStream, Delta, StreamingChoice, StreamingResponse};

