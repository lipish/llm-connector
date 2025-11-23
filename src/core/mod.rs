//! V2 Architecture Core Module
//!
//! This module contains all core components of V2 architecture：
//! - Unified trait definitions (Protocol, Provider)
//! - HTTP client implementation
//! - Generic provider implementation

pub mod traits;
pub mod client;
pub mod builder;
pub mod configurable;

// Re-export core types
pub use traits::{Protocol, Provider, GenericProvider};
pub use client::HttpClient;
pub use builder::ProviderBuilder;
pub use configurable::{ConfigurableProtocol, ProtocolConfig, EndpointConfig, AuthConfig};
