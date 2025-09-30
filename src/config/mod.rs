//! Configuration management for LLM providers
//!
//! This module provides a unified configuration system for all LLM providers.
//! It supports:
//! - Provider-specific configuration
//! - Retry policies
//! - Custom headers
//! - Loading from files (JSON, TOML, YAML)
//!
//! # Examples
//!
//! ## Creating a simple configuration
//!
//! ```rust
//! use llm_connector::config::ProviderConfig;
//!
//! let config = ProviderConfig::new("your-api-key")
//!     .with_timeout_ms(5000)
//!     .with_retry(Default::default());
//! ```
//!
//! ## Loading from a file
//!
//! ```rust,no_run
//! use llm_connector::config::RegistryConfig;
//!
//! # #[cfg(feature = "config")]
//! let config = RegistryConfig::from_file("config.json").unwrap();
//! ```

pub mod provider;
pub mod loader;

// Re-export main types
pub use provider::{ProviderConfig, RetryConfig, SharedProviderConfig};
pub use loader::{RegistryConfig, ProviderConfigEntry};

use serde::{Deserialize, Serialize};
use std::env;

/// Legacy main configuration for llm-connector
///
/// This structure is kept for backward compatibility.
/// For new code, use `RegistryConfig` instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Kimi (Moonshot) configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kimi: Option<ProviderConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            openai: None,
            anthropic: None,
            deepseek: None,
            zhipu: None,
            aliyun: None,
            kimi: None,
        }
    }
}

impl Config {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Config::default();

        // OpenAI
        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
            config.openai = Some(ProviderConfig::new(api_key)
                .with_base_url(env::var("OPENAI_BASE_URL").unwrap_or_default())
                .with_timeout_ms(
                    env::var("OPENAI_TIMEOUT_MS")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(30000)
                ));
        }

        // DeepSeek
        if let Ok(api_key) = env::var("DEEPSEEK_API_KEY") {
            config.deepseek = Some(ProviderConfig::new(api_key)
                .with_base_url(env::var("DEEPSEEK_BASE_URL").unwrap_or_default())
                .with_timeout_ms(
                    env::var("DEEPSEEK_TIMEOUT_MS")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(30000)
                ));
        }

        // Zhipu
        if let Ok(api_key) = env::var("ZHIPU_API_KEY").or_else(|_| env::var("GLM_API_KEY")) {
            config.zhipu = Some(ProviderConfig::new(api_key)
                .with_base_url(env::var("ZHIPU_BASE_URL").unwrap_or_default())
                .with_timeout_ms(
                    env::var("ZHIPU_TIMEOUT_MS")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(30000)
                ));
        }

        // Aliyun
        if let Ok(api_key) = env::var("ALIYUN_API_KEY").or_else(|_| env::var("DASHSCOPE_API_KEY")) {
            config.aliyun = Some(ProviderConfig::new(api_key)
                .with_base_url(env::var("ALIYUN_BASE_URL").unwrap_or_default())
                .with_timeout_ms(
                    env::var("ALIYUN_TIMEOUT_MS")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(30000)
                ));
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
        if self.kimi.is_some() {
            providers.push("kimi".to_string());
        }
        providers
    }
}
