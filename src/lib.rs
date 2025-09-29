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
//!             Message {
//!                 role: "user".to_string(),
//!                 content: "Hello, how are you?".to_string(),
//!                 ..Default::default()
//!             }
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
pub mod types;

// Provider implementations
pub mod providers;

// Utilities
pub mod utils;

// Re-exports for convenience
pub use client::Client;
pub use config::{Config, ProviderConfig};
pub use error::LlmConnectorError;
pub use types::{
    ChatRequest, ChatResponse, Message, Choice, Usage,
};

#[cfg(feature = "streaming")]
pub use types::{StreamingResponse, StreamingChoice, Delta, ChatStream};

// Provider trait
pub use providers::Provider;
