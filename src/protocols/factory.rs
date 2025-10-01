//! Protocol factory for dynamic protocol creation
//!
//! This module provides a factory pattern for creating protocol adapters dynamically
//! based on configuration, without hardcoding provider names.
//!
//! # Purpose
//!
//! The factory pattern allows the `ProviderRegistry` to create providers from YAML
//! configuration files without knowing the specific provider implementations at compile time.
//!
//! # Architecture
//!
//! ## Factory Trait
//!
//! The `ProtocolFactory` trait defines the interface for creating protocol adapters:
//! - `protocol_name()` - Returns the protocol identifier (e.g., "openai", "anthropic")
//! - `supported_providers()` - Lists all providers using this protocol
//! - `create_adapter()` - Creates a protocol adapter instance
//!
//! ## Built-in Factories
//!
//! - **`OpenAIProtocolFactory`** - Creates OpenAI-compatible adapters
//!   - Supports: DeepSeek, Zhipu, Moonshot, VolcEngine, Tencent, MiniMax, StepFun, LongCat
//!
//! - **`AnthropicProtocolFactory`** - Creates Anthropic adapters
//!   - Supports: Claude (Anthropic)
//!
//! - **`AliyunProtocolFactory`** - Creates Aliyun adapters
//!   - Supports: Qwen (Aliyun DashScope)
//!
//! ## Factory Registry
//!
//! The `ProtocolFactoryRegistry` manages all protocol factories and provides:
//! - Automatic registration of built-in factories
//! - Dynamic provider creation from configuration
//! - Protocol lookup by provider name
//!
//! # Example: Using with YAML Config
//!
//! ```yaml
//! # config.yaml
//! providers:
//!   deepseek:
//!     protocol: openai
//!     api_key: sk-xxx
//!   claude:
//!     protocol: anthropic
//!     api_key: sk-ant-xxx
//!   qwen:
//!     protocol: aliyun
//!     api_key: sk-xxx
//! ```
//!
//! ```rust,no_run
//! use llm_connector::config::RegistryConfig;
//! use llm_connector::registry::ProviderRegistry;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load configuration
//! let config = RegistryConfig::from_yaml_file("config.yaml")?;
//!
//! // Create registry (uses factories internally)
//! let registry = ProviderRegistry::from_config(config)?;
//!
//! // Get providers (created by factories)
//! let deepseek = registry.get("deepseek").unwrap();
//! let claude = registry.get("claude").unwrap();
//! let qwen = registry.get("qwen").unwrap();
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Custom Factory
//!
//! ```rust
//! use llm_connector::protocols::factory::{ProtocolFactory, ProtocolFactoryRegistry};
//! use llm_connector::config::ProviderConfig;
//! use llm_connector::error::LlmConnectorError;
//!
//! struct MyCustomFactory;
//!
//! impl ProtocolFactory for MyCustomFactory {
//!     fn protocol_name(&self) -> &str {
//!         "custom"
//!     }
//!
//!     fn supported_providers(&self) -> Vec<&str> {
//!         vec!["my-provider"]
//!     }
//!
//!     fn create_adapter(
//!         &self,
//!         provider_name: &str,
//!         config: &ProviderConfig,
//!     ) -> Result<Box<dyn std::any::Any + Send>, LlmConnectorError> {
//!         // Create your custom adapter
//!         todo!()
//!     }
//! }
//!
//! // Register custom factory
//! let mut registry = ProtocolFactoryRegistry::new();
//! registry.register(Box::new(MyCustomFactory));
//! ```

use crate::config::ProviderConfig;
use crate::error::LlmConnectorError;
use crate::protocols::{AliyunProtocol, AnthropicProtocol};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Trait for protocol factories
///
/// Implement this trait to create custom protocol factories that can
/// dynamically create protocol adapters based on configuration.
pub trait ProtocolFactory: Send + Sync {
    /// Get the protocol name (e.g., "openai", "anthropic")
    fn protocol_name(&self) -> &str;

    /// Get the list of provider names that use this protocol
    fn supported_providers(&self) -> Vec<&str>;

    /// Create a protocol adapter instance
    fn create_adapter(
        &self,
        provider_name: &str,
        config: &ProviderConfig,
    ) -> Result<Box<dyn std::any::Any + Send>, LlmConnectorError>;

    /// Check if this factory supports a given provider
    fn supports_provider(&self, provider_name: &str) -> bool {
        self.supported_providers().contains(&provider_name)
    }
}

/// Factory for OpenAI protocol
#[derive(Debug, Clone)]
pub struct OpenAIProtocolFactory;

impl ProtocolFactory for OpenAIProtocolFactory {
    fn protocol_name(&self) -> &str {
        "openai"
    }

    fn supported_providers(&self) -> Vec<&str> {
        vec![
            "deepseek",
            "zhipu",
            "moonshot",
            "volcengine",
            "tencent",
            "minimax",
            "stepfun",
            "longcat",
        ]
    }

    fn create_adapter(
        &self,
        provider_name: &str,
        _config: &ProviderConfig,
    ) -> Result<Box<dyn std::any::Any + Send>, LlmConnectorError> {
        let adapter = match provider_name {
            "deepseek" => crate::protocols::openai::deepseek(),
            "zhipu" => crate::protocols::openai::zhipu(),
            "moonshot" => crate::protocols::openai::moonshot(),
            "volcengine" => crate::protocols::openai::volcengine(),
            "tencent" => crate::protocols::openai::tencent(),
            "minimax" => crate::protocols::openai::minimax(),
            "stepfun" => crate::protocols::openai::stepfun(),
            "longcat" => crate::protocols::openai::longcat(),
            _ => {
                return Err(LlmConnectorError::UnsupportedModel(format!(
                    "Unknown OpenAI-compatible provider: {}",
                    provider_name
                )))
            }
        };

        Ok(Box::new(adapter))
    }
}

/// Factory for Anthropic protocol
#[derive(Debug, Clone)]
pub struct AnthropicProtocolFactory;

impl ProtocolFactory for AnthropicProtocolFactory {
    fn protocol_name(&self) -> &str {
        "anthropic"
    }

    fn supported_providers(&self) -> Vec<&str> {
        vec!["anthropic", "claude"]
    }

    fn create_adapter(
        &self,
        _provider_name: &str,
        config: &ProviderConfig,
    ) -> Result<Box<dyn std::any::Any + Send>, LlmConnectorError> {
        let adapter = AnthropicProtocol::new(config.base_url.as_deref());
        Ok(Box::new(adapter))
    }
}

/// Factory for Aliyun protocol
#[derive(Debug, Clone)]
pub struct AliyunProtocolFactory;

impl ProtocolFactory for AliyunProtocolFactory {
    fn protocol_name(&self) -> &str {
        "aliyun"
    }

    fn supported_providers(&self) -> Vec<&str> {
        vec!["aliyun", "dashscope", "qwen"]
    }

    fn create_adapter(
        &self,
        _provider_name: &str,
        config: &ProviderConfig,
    ) -> Result<Box<dyn std::any::Any + Send>, LlmConnectorError> {
        let adapter = AliyunProtocol::new(config.base_url.as_deref());
        Ok(Box::new(adapter))
    }
}

/// Protocol factory registry
///
/// Manages all registered protocol factories and provides methods to
/// create protocol adapters dynamically.
#[derive(Clone)]
pub struct ProtocolFactoryRegistry {
    factories: Arc<RwLock<HashMap<String, Arc<dyn ProtocolFactory>>>>,
    provider_to_protocol: Arc<RwLock<HashMap<String, String>>>,
}

impl ProtocolFactoryRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            factories: Arc::new(RwLock::new(HashMap::new())),
            provider_to_protocol: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a registry with default factories
    pub fn with_defaults() -> Self {
        let registry = Self::new();
        registry.register_default_factories();
        registry
    }

    /// Register default protocol factories
    pub fn register_default_factories(&self) {
        self.register(Arc::new(OpenAIProtocolFactory));
        self.register(Arc::new(AnthropicProtocolFactory));
        self.register(Arc::new(AliyunProtocolFactory));
    }

    /// Register a protocol factory
    pub fn register(&self, factory: Arc<dyn ProtocolFactory>) {
        let protocol_name = factory.protocol_name().to_string();

        // Register factory
        self.factories
            .write()
            .unwrap()
            .insert(protocol_name.clone(), factory.clone());

        // Register provider mappings
        let mut provider_map = self.provider_to_protocol.write().unwrap();
        for provider in factory.supported_providers() {
            provider_map.insert(provider.to_string(), protocol_name.clone());
        }
    }

    /// Get a factory by protocol name
    pub fn get_factory(&self, protocol_name: &str) -> Option<Arc<dyn ProtocolFactory>> {
        self.factories.read().unwrap().get(protocol_name).cloned()
    }

    /// Get the protocol name for a provider
    pub fn get_protocol_for_provider(&self, provider_name: &str) -> Option<String> {
        self.provider_to_protocol
            .read()
            .unwrap()
            .get(provider_name)
            .cloned()
    }

    /// Create a protocol adapter for a provider
    pub fn create_for_provider(
        &self,
        provider_name: &str,
        config: &ProviderConfig,
    ) -> Result<Box<dyn std::any::Any + Send>, LlmConnectorError> {
        // Find the protocol for this provider
        let protocol_name = self
            .get_protocol_for_provider(provider_name)
            .ok_or_else(|| {
                LlmConnectorError::UnsupportedModel(format!("Unknown provider: {}", provider_name))
            })?;

        // Get the factory
        let factory = self.get_factory(&protocol_name).ok_or_else(|| {
            LlmConnectorError::ProviderError(format!(
                "No factory registered for protocol: {}",
                protocol_name
            ))
        })?;

        // Create the adapter
        factory.create_adapter(provider_name, config)
    }

    /// List all registered protocols
    pub fn list_protocols(&self) -> Vec<String> {
        self.factories.read().unwrap().keys().cloned().collect()
    }

    /// List all supported providers
    pub fn list_providers(&self) -> Vec<String> {
        self.provider_to_protocol
            .read()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }

    /// Get all providers for a protocol
    pub fn get_providers_for_protocol(&self, protocol_name: &str) -> Vec<String> {
        self.provider_to_protocol
            .read()
            .unwrap()
            .iter()
            .filter(|(_, proto)| proto.as_str() == protocol_name)
            .map(|(provider, _)| provider.clone())
            .collect()
    }
}

impl Default for ProtocolFactoryRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_factory() {
        let factory = OpenAIProtocolFactory;
        assert_eq!(factory.protocol_name(), "openai");
        assert!(factory.supports_provider("deepseek"));
        assert!(factory.supports_provider("zhipu"));
        assert!(!factory.supports_provider("claude"));
    }

    #[test]
    fn test_anthropic_factory() {
        let factory = AnthropicProtocolFactory;
        assert_eq!(factory.protocol_name(), "anthropic");
        assert!(factory.supports_provider("anthropic"));
        assert!(factory.supports_provider("claude"));
        assert!(!factory.supports_provider("deepseek"));
    }

    #[test]
    fn test_registry() {
        let registry = ProtocolFactoryRegistry::with_defaults();

        // Test protocol lookup
        assert!(registry.get_factory("openai").is_some());
        assert!(registry.get_factory("anthropic").is_some());
        assert!(registry.get_factory("aliyun").is_some());

        // Test provider lookup
        assert_eq!(
            registry.get_protocol_for_provider("deepseek"),
            Some("openai".to_string())
        );
        assert_eq!(
            registry.get_protocol_for_provider("claude"),
            Some("anthropic".to_string())
        );
        assert_eq!(
            registry.get_protocol_for_provider("qwen"),
            Some("aliyun".to_string())
        );
    }

    #[test]
    fn test_list_protocols() {
        let registry = ProtocolFactoryRegistry::with_defaults();
        let protocols = registry.list_protocols();

        assert!(protocols.contains(&"openai".to_string()));
        assert!(protocols.contains(&"anthropic".to_string()));
        assert!(protocols.contains(&"aliyun".to_string()));
    }

    #[test]
    fn test_list_providers() {
        let registry = ProtocolFactoryRegistry::with_defaults();
        let providers = registry.list_providers();

        assert!(providers.contains(&"deepseek".to_string()));
        assert!(providers.contains(&"claude".to_string()));
        assert!(providers.contains(&"qwen".to_string()));
    }
}
