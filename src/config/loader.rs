//! Configuration loading from external files
//!
//! This module provides utilities for loading provider configurations
//! from various file formats (JSON, TOML, YAML).

use super::provider::ProviderConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Registry configuration containing multiple providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Map of provider name to configuration
    pub providers: HashMap<String, ProviderConfigEntry>,
}

/// Provider configuration entry in registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigEntry {
    /// Protocol to use (openai, anthropic, aliyun)
    pub protocol: String,

    /// Provider configuration
    #[serde(flatten)]
    pub config: ProviderConfig,
}

impl RegistryConfig {
    /// Create a new empty registry configuration
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Add a provider configuration
    pub fn add_provider(
        mut self,
        name: impl Into<String>,
        protocol: impl Into<String>,
        config: ProviderConfig,
    ) -> Self {
        self.providers.insert(
            name.into(),
            ProviderConfigEntry {
                protocol: protocol.into(),
                config,
            },
        );
        self
    }

    /// Load configuration from a JSON file
    #[cfg(feature = "config")]
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, LlmConnectorError> {
        let content = fs::read_to_string(path).map_err(|e| {
            LlmConnectorError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        serde_json::from_str(&content)
            .map_err(|e| LlmConnectorError::ConfigError(format!("Invalid JSON config: {}", e)))
    }

    /// Load configuration from a TOML file
    #[cfg(all(feature = "config", feature = "toml"))]
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self, LlmConnectorError> {
        let content = fs::read_to_string(path).map_err(|e| {
            LlmConnectorError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        toml::from_str(&content)
            .map_err(|e| LlmConnectorError::ConfigError(format!("Invalid TOML config: {}", e)))
    }

    /// Load configuration from a YAML file
    #[cfg(all(feature = "config", feature = "yaml"))]
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self, LlmConnectorError> {
        let content = fs::read_to_string(path).map_err(|e| {
            LlmConnectorError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        serde_yaml::from_str(&content)
            .map_err(|e| LlmConnectorError::ConfigError(format!("Invalid YAML config: {}", e)))
    }

    /// Load configuration from a YAML file
    ///
    /// This is the recommended way to load configuration for multi-provider scenarios.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_connector::config::RegistryConfig;
    ///
    /// // Load from YAML file
    /// let config = RegistryConfig::from_yaml_file("config.yaml")?;
    /// # Ok::<(), llm_connector::error::LlmConnectorError>(())
    /// ```
    #[cfg(all(feature = "config", feature = "yaml"))]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, LlmConnectorError> {
        Self::from_yaml_file(path)
    }

    /// Save configuration to a JSON file
    #[cfg(feature = "config")]
    pub fn to_json_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LlmConnectorError> {
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            LlmConnectorError::ConfigError(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(path, content).map_err(|e| {
            LlmConnectorError::ConfigError(format!("Failed to write config file: {}", e))
        })
    }

    /// Get a provider configuration by name
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfigEntry> {
        self.providers.get(name)
    }

    /// Get all provider names
    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_config_builder() {
        let config = RegistryConfig::new()
            .add_provider("deepseek", "openai", ProviderConfig::new("test-key-1"))
            .add_provider("claude", "anthropic", ProviderConfig::new("test-key-2"));

        assert_eq!(config.providers.len(), 2);
        assert!(config.get_provider("deepseek").is_some());
        assert!(config.get_provider("claude").is_some());
    }

    #[test]
    fn test_provider_names() {
        let config = RegistryConfig::new()
            .add_provider("deepseek", "openai", ProviderConfig::new("key1"))
            .add_provider("claude", "anthropic", ProviderConfig::new("key2"));

        let names = config.provider_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"deepseek"));
        assert!(names.contains(&"claude"));
    }

    #[cfg(feature = "config")]
    #[test]
    fn test_json_serialization() {
        let config =
            RegistryConfig::new().add_provider("test", "openai", ProviderConfig::new("test-key"));

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RegistryConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.providers.len(), 1);
        assert!(deserialized.get_provider("test").is_some());
    }
}
