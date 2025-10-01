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

use crate::config::{ProviderConfig, RegistryConfig};
use crate::error::LlmConnectorError;
use crate::protocols::core::{GenericProvider, Provider, ProviderAdapter};
use std::collections::HashMap;

/// Unified provider registry for managing all LLM providers
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn Provider>>,
    configs: HashMap<String, ProviderConfig>,
}

impl ProviderRegistry {
    /// Create a new empty provider registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    /// Create a provider registry from a configuration
    pub fn from_config(config: RegistryConfig) -> Result<Self, LlmConnectorError> {
        let mut registry = Self::new();

        for (name, entry) in config.providers {
            // Extract the config from the entry
            let internal_config = entry.config;

            // Register the provider based on protocol
            match entry.protocol.as_str() {
                "aliyun" => {
                    registry.register(
                        &name,
                        internal_config,
                        crate::protocols::aliyun::aliyun(),
                    )?;
                }
                "openai" => {
                    // Determine which OpenAI provider based on name
                    match name.as_str() {
                        "deepseek" => registry.register(
                            &name,
                            internal_config,
                            crate::protocols::openai::deepseek(),
                        )?,
                        "zhipu" => registry.register(
                            &name,
                            internal_config,
                            crate::protocols::openai::zhipu(),
                        )?,
                        "moonshot" => registry.register(
                            &name,
                            internal_config,
                            crate::protocols::openai::moonshot(),
                        )?,
                        "volcengine" => registry.register(
                            &name,
                            internal_config,
                            crate::protocols::openai::volcengine(),
                        )?,
                        "tencent" => registry.register(
                            &name,
                            internal_config,
                            crate::protocols::openai::tencent(),
                        )?,
                        "minimax" => registry.register(
                            &name,
                            internal_config,
                            crate::protocols::openai::minimax(),
                        )?,
                        "stepfun" => registry.register(
                            &name,
                            internal_config,
                            crate::protocols::openai::stepfun(),
                        )?,
                        "longcat" => registry.register(
                            &name,
                            internal_config,
                            crate::protocols::openai::longcat(),
                        )?,
                        _ => registry.register(
                            &name,
                            internal_config,
                            crate::protocols::openai::deepseek(),
                        )?, // default
                    }
                }
                "anthropic" => {
                    registry.register(
                        &name,
                        internal_config,
                        crate::protocols::anthropic::anthropic(),
                    )?;
                }
                _ => {
                    return Err(LlmConnectorError::ConfigError(format!(
                        "Unknown provider: {}",
                        name
                    )));
                }
            }
        }

        Ok(registry)
    }

    /// Register a provider with its configuration
    pub fn register<T>(
        &mut self,
        name: &str,
        config: ProviderConfig,
        adapter: T,
    ) -> Result<(), LlmConnectorError>
    where
        T: ProviderAdapter + 'static,
    {
        let provider = GenericProvider::new(config.clone(), adapter)?;
        self.providers.insert(name.to_string(), Box::new(provider));
        self.configs.insert(name.to_string(), config);
        Ok(())
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<&dyn Provider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    /// Check if a provider is registered
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    /// Get all registered provider names
    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.keys().map(|k| k.as_str()).collect()
    }

    /// Get provider configuration
    pub fn get_config(&self, name: &str) -> Option<&ProviderConfig> {
        self.configs.get(name)
    }

    /// Update provider configuration
    pub fn update_config(
        &mut self,
        name: &str,
        config: ProviderConfig,
    ) -> Result<(), LlmConnectorError> {
        if let Some(provider_config) = self.configs.get_mut(name) {
            *provider_config = config;
            Ok(())
        } else {
            Err(LlmConnectorError::ConfigError(format!(
                "Provider '{}' not found",
                name
            )))
        }
    }

    /// Remove a provider from the registry
    pub fn remove_provider(&mut self, name: &str) -> Option<Box<dyn Provider>> {
        self.configs.remove(name);
        self.providers.remove(name)
    }

    /// Get the number of registered providers
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for creating pre-configured provider registries
pub struct ProviderRegistryBuilder {
    registry: ProviderRegistry,
}

impl ProviderRegistryBuilder {
    pub fn new() -> Self {
        Self {
            registry: ProviderRegistry::new(),
        }
    }

    /// Add a provider to the registry
    pub fn with_provider<T>(
        mut self,
        name: &str,
        config: ProviderConfig,
        adapter: T,
    ) -> Result<Self, LlmConnectorError>
    where
        T: ProviderAdapter + 'static,
    {
        self.registry.register(name, config, adapter)?;
        Ok(self)
    }

    /// Build the final registry
    pub fn build(self) -> ProviderRegistry {
        self.registry
    }

    /// Build from a configuration
    pub fn from_config(mut self, config: RegistryConfig) -> Result<Self, LlmConnectorError> {
        for (name, entry) in config.providers {
            // Use the provider_config directly
            let internal_config = entry.config;

            // Register the provider based on name
            match name.as_str() {
                "aliyun" => {
                    self.registry.register(
                        &name,
                        internal_config,
                        crate::protocols::aliyun::aliyun(),
                    )?;
                }
                "deepseek" => {
                    self.registry.register(
                        &name,
                        internal_config,
                        crate::protocols::openai::deepseek(),
                    )?;
                }
                "zhipu" => {
                    self.registry.register(
                        &name,
                        internal_config,
                        crate::protocols::openai::zhipu(),
                    )?;
                }
                _ => {
                    return Err(LlmConnectorError::ConfigError(format!(
                        "Unknown provider: {}",
                        name
                    )));
                }
            }
        }

        Ok(self)
    }
}

impl Default for ProviderRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::core::{ErrorMapper, ProviderAdapter};
    use crate::types::{ChatRequest, ChatResponse};
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};

    // Mock adapter for testing
    #[derive(Clone)]
    struct MockAdapter;

    #[async_trait]
    impl ProviderAdapter for MockAdapter {
        type RequestType = MockRequest;
        type ResponseType = MockResponse;
        #[cfg(feature = "streaming")]
        type StreamResponseType = MockResponse;
        type ErrorMapperType = MockErrorMapper;

        fn name(&self) -> &str {
            "mock"
        }

        fn supported_models(&self) -> Vec<String> {
            vec!["mock-model".to_string()]
        }

        fn endpoint_url(&self, _base_url: &Option<String>) -> String {
            "https://mock.api/chat".to_string()
        }

        fn build_request_data(&self, _request: &ChatRequest, _stream: bool) -> Self::RequestType {
            MockRequest {
                model: "mock-model".to_string(),
            }
        }

        fn parse_response_data(&self, _response: Self::ResponseType) -> ChatResponse {
            ChatResponse::default()
        }

        #[cfg(feature = "streaming")]
        fn parse_stream_response_data(
            &self,
            _response: Self::StreamResponseType,
        ) -> crate::types::StreamingResponse {
            crate::types::StreamingResponse::default()
        }
    }

    #[derive(Serialize)]
    struct MockRequest {
        model: String,
    }

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct MockResponse {
        id: String,
    }

    struct MockErrorMapper;

    impl ErrorMapper for MockErrorMapper {
        fn map_http_error(_status: u16, _body: serde_json::Value) -> LlmConnectorError {
            LlmConnectorError::ProviderError("mock error".to_string())
        }

        fn map_network_error(_error: reqwest::Error) -> LlmConnectorError {
            LlmConnectorError::NetworkError("mock network error".to_string())
        }

        fn is_retriable_error(_error: &LlmConnectorError) -> bool {
            false
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = ProviderRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_provider_registration() {
        let mut registry = ProviderRegistry::new();
        let config = ProviderConfig::new("test-key").with_timeout_ms(5000);

        let result = registry.register("mock", config.clone(), MockAdapter);
        assert!(result.is_ok());
        assert_eq!(registry.len(), 1);
        assert!(registry.has_provider("mock"));
    }

    #[test]
    fn test_provider_retrieval() {
        let mut registry = ProviderRegistry::new();
        let config = ProviderConfig::new("test-key").with_timeout_ms(5000);

        registry
            .register("mock", config.clone(), MockAdapter)
            .unwrap();

        assert!(registry.get_provider("mock").is_some());
        assert!(registry.get_provider("nonexistent").is_none());
    }

    #[test]
    fn test_provider_removal() {
        let mut registry = ProviderRegistry::new();
        let config = ProviderConfig::new("test-key").with_timeout_ms(5000);

        registry
            .register("mock", config.clone(), MockAdapter)
            .unwrap();
        assert_eq!(registry.len(), 1);

        registry.remove_provider("mock");
        assert_eq!(registry.len(), 0);
        assert!(!registry.has_provider("mock"));
    }

    #[test]
    fn test_registry_builder() {
        let config = ProviderConfig::new("test-key").with_timeout_ms(5000);

        let registry = ProviderRegistryBuilder::new()
            .with_provider("mock", config, MockAdapter)
            .unwrap()
            .build();

        assert_eq!(registry.len(), 1);
        assert!(registry.has_provider("mock"));
    }
}
