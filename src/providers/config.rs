//! Provider configuration management for loading from external files

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::LlmConnectorError;

/// Configuration for a single provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub timeout_ms: Option<u64>,
    pub proxy: Option<String>,
    pub parser_type: Option<String>,
    pub max_retries: Option<u32>,
    pub retry_backoff_ms: Option<u64>,
    pub supported_models: Vec<String>,
}

/// Registry configuration containing multiple providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub providers: HashMap<String, ProviderConfig>,
}

impl RegistryConfig {
    /// Load configuration from a TOML file
    #[cfg(feature = "config")]
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self, LlmConnectorError> {
        let content = fs::read_to_string(path).map_err(|e| {
            LlmConnectorError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        // TOML support requires the 'config' feature and toml dependency
        #[cfg(feature = "toml")]
        {
            toml::from_str(&content)
                .map_err(|e| LlmConnectorError::ConfigError(format!("Invalid TOML config: {}", e)))
        }

        #[cfg(not(feature = "toml"))]
        {
            Err(LlmConnectorError::ConfigError(
                "TOML support not enabled. Enable 'toml' feature.".to_string()
            ))
        }
    }

    /// Load configuration from a JSON file
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, LlmConnectorError> {
        let content = fs::read_to_string(path).map_err(|e| {
            LlmConnectorError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        serde_json::from_str(&content)
            .map_err(|e| LlmConnectorError::ConfigError(format!("Invalid JSON config: {}", e)))
    }

    /// Save configuration to a TOML file
    #[cfg(feature = "config")]
    pub fn save_to_toml_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LlmConnectorError> {
        #[cfg(feature = "toml")]
        {
            let content = toml::to_string_pretty(self).map_err(|e| {
                LlmConnectorError::ConfigError(format!("Failed to serialize config: {}", e))
            })?;

            fs::write(path, content).map_err(|e| {
                LlmConnectorError::ConfigError(format!("Failed to write config file: {}", e))
            })
        }

        #[cfg(not(feature = "toml"))]
        {
            Err(LlmConnectorError::ConfigError(
                "TOML support not enabled. Enable 'toml' feature.".to_string()
            ))
        }
    }

    /// Save configuration to a JSON file
    pub fn save_to_json_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LlmConnectorError> {
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            LlmConnectorError::ConfigError(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(path, content).map_err(|e| {
            LlmConnectorError::ConfigError(format!("Failed to write config file: {}", e))
        })
    }

    /// Get a provider configuration by name
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }

    /// Add or update a provider configuration
    pub fn set_provider(&mut self, name: String, config: ProviderConfig) {
        self.providers.insert(name, config);
    }

    /// Remove a provider configuration
    pub fn remove_provider(&mut self, name: &str) -> Option<ProviderConfig> {
        self.providers.remove(name)
    }

    /// Get all provider names
    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.keys().map(|k| k.as_str()).collect()
    }

    /// Check if a provider exists
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    /// Create a default configuration with example providers
    pub fn default_config() -> Self {
        let mut providers = HashMap::new();

        // Example Aliyun configuration
        providers.insert("aliyun".to_string(), ProviderConfig {
            name: "aliyun".to_string(),
            api_key: "your-aliyun-api-key".to_string(),
            base_url: Some("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation".to_string()),
            timeout_ms: Some(30000),
            proxy: None,
            parser_type: Some("sse".to_string()),
            max_retries: Some(3),
            retry_backoff_ms: Some(1000),
            supported_models: vec!["qwen-turbo".to_string(), "qwen-plus".to_string()],
        });

        // Example DeepSeek configuration
        providers.insert(
            "deepseek".to_string(),
            ProviderConfig {
                name: "deepseek".to_string(),
                api_key: "your-deepseek-api-key".to_string(),
                base_url: Some("https://api.deepseek.com/v1".to_string()),
                timeout_ms: Some(30000),
                proxy: None,
                parser_type: Some("sse".to_string()),
                max_retries: Some(3),
                retry_backoff_ms: Some(1000),
                supported_models: vec!["deepseek-chat".to_string(), "deepseek-coder".to_string()],
            },
        );

        // Example Zhipu configuration
        providers.insert(
            "zhipu".to_string(),
            ProviderConfig {
                name: "zhipu".to_string(),
                api_key: "your-zhipu-api-key".to_string(),
                base_url: Some("https://open.bigmodel.cn/api/paas/v4".to_string()),
                timeout_ms: Some(30000),
                proxy: None,
                parser_type: Some("sse".to_string()),
                max_retries: Some(3),
                retry_backoff_ms: Some(1000),
                supported_models: vec!["glm-4".to_string(), "glm-3-turbo".to_string()],
            },
        );

        Self { providers }
    }

    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut providers = HashMap::new();

        // Load Aliyun from env
        if let (Ok(api_key), Ok(base_url)) = (
            std::env::var("ALIYUN_API_KEY"),
            std::env::var("ALIYUN_BASE_URL"),
        ) {
            providers.insert(
                "aliyun".to_string(),
                ProviderConfig {
                    name: "aliyun".to_string(),
                    api_key,
                    base_url: Some(base_url),
                    timeout_ms: Some(30000),
                    proxy: None,
                    parser_type: Some("sse".to_string()),
                    max_retries: Some(3),
                    retry_backoff_ms: Some(1000),
                    supported_models: vec!["qwen-turbo".to_string(), "qwen-plus".to_string()],
                },
            );
        }

        // Load DeepSeek from env
        if let (Ok(api_key), Ok(base_url)) = (
            std::env::var("DEEPSEEK_API_KEY"),
            std::env::var("DEEPSEEK_BASE_URL"),
        ) {
            providers.insert(
                "deepseek".to_string(),
                ProviderConfig {
                    name: "deepseek".to_string(),
                    api_key,
                    base_url: Some(base_url),
                    timeout_ms: Some(30000),
                    proxy: None,
                    parser_type: Some("sse".to_string()),
                    max_retries: Some(3),
                    retry_backoff_ms: Some(1000),
                    supported_models: vec![
                        "deepseek-chat".to_string(),
                        "deepseek-coder".to_string(),
                    ],
                },
            );
        }

        // Load Zhipu from env
        if let (Ok(api_key), Ok(base_url)) = (
            std::env::var("ZHIPU_API_KEY"),
            std::env::var("ZHIPU_BASE_URL"),
        ) {
            providers.insert(
                "zhipu".to_string(),
                ProviderConfig {
                    name: "zhipu".to_string(),
                    api_key,
                    base_url: Some(base_url),
                    timeout_ms: Some(30000),
                    proxy: None,
                    parser_type: Some("sse".to_string()),
                    max_retries: Some(3),
                    retry_backoff_ms: Some(1000),
                    supported_models: vec!["glm-4".to_string(), "glm-3-turbo".to_string()],
                },
            );
        }

        Self { providers }
    }
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RegistryConfig::default();
        assert!(config.has_provider("aliyun"));
        assert!(config.has_provider("deepseek"));
        assert!(config.has_provider("zhipu"));
    }

    #[test]
    fn test_json_serialization() {
        let config = RegistryConfig::default();

        // Serialize to JSON string
        let json_str = serde_json::to_string_pretty(&config).unwrap();

        // Deserialize from JSON string
        let loaded_config: RegistryConfig = serde_json::from_str(&json_str).unwrap();

        assert_eq!(config.provider_names().len(), loaded_config.provider_names().len());
    }

    #[test]
    fn test_provider_management() {
        let mut config = RegistryConfig::default();

        // Test adding a new provider
        let new_provider = ProviderConfig {
            name: "test".to_string(),
            api_key: "test-key".to_string(),
            base_url: Some("https://test.api".to_string()),
            timeout_ms: Some(5000),
            proxy: None,
            parser_type: Some("sse".to_string()),
            max_retries: Some(3),
            retry_backoff_ms: Some(1000),
            supported_models: vec!["test-model".to_string()],
        };

        config.set_provider("test".to_string(), new_provider);
        assert!(config.has_provider("test"));

        // Test removing a provider
        config.remove_provider("test");
        assert!(!config.has_provider("test"));
    }
}
