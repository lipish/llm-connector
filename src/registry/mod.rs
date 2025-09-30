//! Provider registry for managing multiple LLM providers
//!
//! This module provides a centralized registry for managing multiple LLM providers.
//! It allows you to register providers and route requests to the appropriate provider
//! based on configuration.
//!
//! ## Features
//!
//! - Centralized provider management
//! - Configuration-driven provider registration
//! - Dynamic provider lookup
//! - Support for multiple providers simultaneously
//!
//! ## Example
//!
//! ```rust,no_run
//! use llm_connector::registry::{ProviderRegistry, ProviderRegistryBuilder};
//! use llm_connector::config::ProviderConfig;
//! use llm_connector::protocols::openai::deepseek;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a registry
//! let mut registry = ProviderRegistry::new();
//!
//! // Register a provider
//! let config = ProviderConfig::new("api-key");
//! registry.register("deepseek", config, deepseek())?;
//!
//! // Get a provider
//! let provider = registry.get_provider("deepseek").unwrap();
//! # Ok(())
//! # }
//! ```

pub mod provider_registry;

// Re-export main types
pub use provider_registry::{ProviderRegistry, ProviderRegistryBuilder};
