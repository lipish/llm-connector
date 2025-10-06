//! Configuration management for LLM providers
//!
//! This module provides a unified configuration system for all LLM providers.
//!
//! # Configuration Methods
//!
//! ## Method 1: Direct API Key (Recommended)
//!
//! The simplest way to configure a provider:
//!
//! ```rust
//! use llm_connector::config::ProviderConfig;
//!
//! let config = ProviderConfig::new("your-api-key");
//! ```
//!
//! ## Method 2: Environment Variables
//!
//! For development convenience:
//!
//! ```rust
//! use std::env;
//! use llm_connector::config::ProviderConfig;
//!
//! let api_key = env::var("DEEPSEEK_API_KEY").unwrap();
//! let config = ProviderConfig::new(&api_key);
//! ```
//!
//! ## Method 3: Advanced Configuration
//!
//! For custom settings:
//!
//! ```rust
//! use llm_connector::config::{ProviderConfig, RetryConfig};
//!
//! let config = ProviderConfig::new("your-api-key")
//!     .with_base_url("https://api.example.com/v1")
//!     .with_timeout_ms(30000)
//!     .with_retry(RetryConfig {
//!         max_retries: 3,
//!         initial_backoff_ms: 1000,
//!         backoff_multiplier: 2.0,
//!         max_backoff_ms: 30000,
//!     })
//!     .with_header("X-Custom-Header", "value");
//! ```
//!
//! ## Method 4: YAML Config File (Optional)
//!
//! For multi-provider applications:
//!
//! ```rust,no_run
//! use llm_connector::config::RegistryConfig;
//! use llm_connector::registry::ProviderRegistry;
//!
//! // Load from YAML file
//! let config = RegistryConfig::from_yaml_file("config.yaml").unwrap();
//! let registry = ProviderRegistry::from_config(config).unwrap();
//!
//! // Get providers
//! let deepseek = registry.get("deepseek").unwrap();
//! let claude = registry.get("claude").unwrap();
//! ```
//!
//! **Note**: YAML config is optional and only recommended for complex multi-provider scenarios.

pub mod loader;
pub mod provider;

// Re-export main types
pub use loader::{ProviderConfigEntry, RegistryConfig};
pub use provider::{ProviderConfig, RetryConfig, SharedProviderConfig};

use serde::{Deserialize, Serialize};
use std::env;

/// Legacy main configuration for llm-connector
///
/// This structure is kept for backward compatibility.
/// For new code, use `RegistryConfig` instead.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// OpenAI configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openai: Option<ProviderConfig>,

    /// Anthropic configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anthropic: Option<ProviderConfig>,

    /// DeepSeek configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deepseek: Option<ProviderConfig>,

    /// Zhipu (GLM) configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zhipu: Option<ProviderConfig>,

    /// Aliyun (Alibaba) configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliyun: Option<ProviderConfig>,

    /// Moonshot configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moonshot: Option<ProviderConfig>,

    /// VolcEngine (Doubao) configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volcengine: Option<ProviderConfig>,

    /// LongCat configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longcat: Option<ProviderConfig>,
}

impl Config {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Config::default();

        // OpenAI
        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
            config.openai = Some(
                ProviderConfig::new(api_key)
                    .with_base_url(env::var("OPENAI_BASE_URL").unwrap_or_default())
                    .with_timeout_ms(
                        env::var("OPENAI_TIMEOUT_MS")
                            .ok()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(30000),
                    ),
            );
        }

        // DeepSeek
        if let Ok(api_key) = env::var("DEEPSEEK_API_KEY") {
            config.deepseek = Some(
                ProviderConfig::new(api_key)
                    .with_base_url(env::var("DEEPSEEK_BASE_URL").unwrap_or_default())
                    .with_timeout_ms(
                        env::var("DEEPSEEK_TIMEOUT_MS")
                            .ok()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(30000),
                    ),
            );
        }

        // Zhipu
        if let Ok(api_key) = env::var("ZHIPU_API_KEY").or_else(|_| env::var("GLM_API_KEY")) {
            config.zhipu = Some(
                ProviderConfig::new(api_key)
                    .with_base_url(env::var("ZHIPU_BASE_URL").unwrap_or_default())
                    .with_timeout_ms(
                        env::var("ZHIPU_TIMEOUT_MS")
                            .ok()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(30000),
                    ),
            );
        }

        // Aliyun
        if let Ok(api_key) = env::var("ALIYUN_API_KEY").or_else(|_| env::var("DASHSCOPE_API_KEY")) {
            config.aliyun = Some(
                ProviderConfig::new(api_key)
                    .with_base_url(env::var("ALIYUN_BASE_URL").unwrap_or_default())
                    .with_timeout_ms(
                        env::var("ALIYUN_TIMEOUT_MS")
                            .ok()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(30000),
                    ),
            );
        }

        config
    }

    /// List all configured providers
    pub fn list_providers(&self) -> Vec<String> {
        let mut providers = Vec::new();
        if self.openai.is_some() {
            providers.push("openai".to_string());
        }
        if self.anthropic.is_some() {
            providers.push("anthropic".to_string());
        }
        if self.deepseek.is_some() {
            providers.push("deepseek".to_string());
        }
        if self.zhipu.is_some() {
            providers.push("zhipu".to_string());
        }
        if self.aliyun.is_some() {
            providers.push("aliyun".to_string());
        }
        if self.moonshot.is_some() {
            providers.push("moonshot".to_string());
        }
        if self.volcengine.is_some() {
            providers.push("volcengine".to_string());
        }
        if self.longcat.is_some() {
            providers.push("longcat".to_string());
        }
        providers
    }
}
