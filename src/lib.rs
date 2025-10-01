//! # llm-connector
//!
//! A lightweight Rust library for protocol adaptation across multiple LLM providers.
//!
//! This library focuses solely on converting between different LLM provider APIs
//! and providing a unified OpenAI-compatible interface.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use llm_connector::{Client, ChatRequest, Message};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize client from environment variables
//!     let client = Client::from_env();
//!
//!     // Create a chat request
//!     let request = ChatRequest {
//!         model: "openai/gpt-4".to_string(),
//!         messages: vec![
//!             Message::user("Hello, how are you?")
//!         ],
//!         ..Default::default()
//!     };
//!
//!     // Send request
//!     let response = client.chat(request).await?;
//!     println!("Response: {}", response.choices[0].message.content);
//!
//!     Ok(())
//! }
//! ```

// Core modules
pub mod client;
pub mod config;
pub mod error;
pub mod middleware;
pub mod registry;
pub mod types;

// Provider implementations
pub mod protocols;
// Legacy compatibility - will be deprecated in v0.2.0
pub mod providers {
    //! Legacy module - use `protocols` instead
    //! This module will be deprecated in v0.2.0
    pub use crate::protocols::*;
}

// Server-Sent Events (SSE) utilities
pub mod sse;

// Re-exports for convenience
pub use client::Client;
pub use config::{Config, ProviderConfig, RegistryConfig, RetryConfig};
pub use error::LlmConnectorError;
pub use types::{ChatRequest, ChatResponse, Choice, Message, Usage};

#[cfg(feature = "streaming")]
pub use types::{ChatStream, Delta, StreamingChoice, StreamingResponse};

// Provider trait
pub use providers::Provider;
