//! Core traits and abstractions for llm-connector
//!
//! This module provides the fundamental abstractions for the architecture:
//! - Provider: Service implementation interface
//! - Protocol: Protocol specification interface
//! - HttpTransport: HTTP communication layer
//! - ErrorMapper: Error mapping utilities

pub mod provider;
pub mod protocol;
pub mod http;
pub mod error;

// Re-export core traits
pub use provider::Provider;
pub use protocol::Protocol;
pub use http::HttpTransport;
pub use error::ErrorMapper;