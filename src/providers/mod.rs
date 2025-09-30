//! Provider implementations for different LLM services
//!
//! # Architecture
//!
//! This module uses the Adapter pattern to minimize code duplication:
//!
//! - **ProviderAdapter**: Protocol-specific logic (request/response conversion)
//! - **GenericProvider<T>**: Generic implementation of Provider trait
//! - **No per-provider wrapper classes**: Eliminates redundant code
//!
//! # Usage
//!
//! ## Direct Usage
//!
//! ```rust,no_run
//! # use llm_connector::providers::{GenericProvider, DeepSeekAdapter};
//! # use llm_connector::config::ProviderConfig;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ProviderConfig {
//!     api_key: "your-api-key".to_string(),
//!     base_url: None,
//!     timeout_ms: None,
//!     proxy: None,
//! };
//!
//! let provider = GenericProvider::new(config, DeepSeekAdapter)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Using Registry (Recommended)
//!
//! ```rust,no_run
//! # use llm_connector::providers::{ProviderRegistry, DeepSeekAdapter};
//! # use llm_connector::config::ProviderConfig;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let config = ProviderConfig {
//! #     api_key: "your-api-key".to_string(),
//! #     base_url: None,
//! #     timeout_ms: None,
//! #     proxy: None,
//! # };
//! let mut registry = ProviderRegistry::new();
//! registry.register("deepseek", config, DeepSeekAdapter)?;
//!
//! let provider = registry.get_provider("deepseek").unwrap();
//! # Ok(())
//! # }
//! ```
//!
//! # Adding a New Provider
//!
//! To add a new provider, you only need to:
//!
//! 1. Add an Adapter in `adapters.rs`:
//!
//! ```rust,ignore
//! pub struct NewProviderAdapter;
//!
//! impl ProviderAdapter for NewProviderAdapter {
//!     // Implement protocol conversion
//! }
//! ```
//!
//! 2. Re-export it in this module
//!
//! **No need to create a new .rs file!**

mod adapters;
pub mod base; // Make base public to access utils
mod config;
mod errors;
mod generic;
// mod macros; // Temporarily disabled - requires separate proc-macro crate
mod parser;
mod registry;
mod traits;
mod transport;

// Re-export the Provider trait and utilities
pub use base::Provider;

// Re-export common components
pub use errors::ErrorMapper;
pub use parser::ParserType;
pub use traits::ProviderAdapter;
pub use transport::HttpTransport;

// Re-export generic provider
pub use generic::GenericProvider;

// Re-export registry
pub use registry::{ProviderRegistry, ProviderRegistryBuilder};

// Re-export adapter implementations
pub use adapters::{AliyunAdapter, DeepSeekAdapter, ZhipuAdapter};

// Re-export configuration
pub use config::RegistryConfig;

/// Create a pre-configured provider registry with all available providers
///
/// # Example
///
/// ```rust,no_run
/// # use llm_connector::providers::{create_default_registry, DeepSeekAdapter};
/// # use llm_connector::config::ProviderConfig;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut registry = create_default_registry()?;
///
/// // Register providers with their adapters
/// let config = ProviderConfig {
///     api_key: "your-api-key".to_string(),
///     base_url: None,
///     timeout_ms: None,
///     proxy: None,
/// };
///
/// registry.register("deepseek", config, DeepSeekAdapter)?;
/// # Ok(())
/// # }
/// ```
pub fn create_default_registry() -> Result<ProviderRegistry, crate::error::LlmConnectorError> {
    Ok(ProviderRegistry::new())
}

/// Get a provider by name from a registry
pub fn get_provider<'a>(registry: &'a ProviderRegistry, name: &str) -> Option<&'a dyn Provider> {
    registry.get_provider(name)
}

/// Create a provider registry from a configuration file
///
/// # Example
///
/// ```rust,no_run
/// # use llm_connector::providers::create_registry_from_toml_file;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let registry = create_registry_from_toml_file("config.toml")?;
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "config")]
pub fn create_registry_from_toml_file<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<ProviderRegistry, crate::error::LlmConnectorError> {
    let config = crate::providers::config::RegistryConfig::from_toml_file(path)?;
    ProviderRegistry::from_config(config)
}

/// Create a provider registry from a configuration
pub fn create_registry_from_config(
    config: RegistryConfig,
) -> Result<ProviderRegistry, crate::error::LlmConnectorError> {
    ProviderRegistry::from_config(config)
}
