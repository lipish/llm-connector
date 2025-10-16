//! Protocol Implementations and Core Architecture
//!
//! This module contains the current architecture implementation:
//!
//! ## Core Architecture (protocols/core.rs)
//! - `GenericProvider<A>`: Universal provider implementation
//! - `ProviderAdapter`: Protocol adaptation interface
//! - `HttpTransport`: HTTP communication layer
//! - `ErrorMapper`: Error handling utilities
//!
//! ## Pure Protocol Adapters
//! - `OpenAIProtocol`: OpenAI API specification adapter
//! - `AnthropicProtocol`: Anthropic API specification adapter
//!
//! Note: This module contains both the core architecture implementation
//! and pure protocol adapters. The core architecture is used by most
//! providers in the `providers/` directory.

pub mod anthropic;
pub mod core;
pub mod openai;

// Re-export core components (current architecture implementation)
pub use core::{ErrorMapper, GenericProvider, HttpTransport, Provider, ProviderAdapter};

// Re-export pure protocols
pub use anthropic::AnthropicProtocol;
pub use openai::OpenAIProtocol;

// Re-export configuration
pub use crate::config::ProviderConfig;