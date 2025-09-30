//! Configuration management for llm-connector

use serde::{Deserialize, Serialize};
use std::env;

/// Main configuration for llm-connector
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

/// Configuration for a specific provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API key for the provider
    pub api_key: String,
    
    /// Base URL for the provider API
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    
    /// Request timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,

    /// Proxy to use for requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
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
            config.openai = Some(ProviderConfig {
                api_key,
                base_url: env::var("OPENAI_BASE_URL").ok(),
                timeout_ms: env::var("OPENAI_TIMEOUT_MS")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                proxy: env::var("OPENAI_PROXY").ok(),
            });
        }
        
        // Anthropic
        if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
            config.anthropic = Some(ProviderConfig {
                api_key,
                base_url: env::var("ANTHROPIC_BASE_URL").ok(),
                timeout_ms: env::var("ANTHROPIC_TIMEOUT_MS")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                proxy: env::var("ANTHROPIC_PROXY").ok(),
            });
        }
        
        // DeepSeek
        if let Ok(api_key) = env::var("DEEPSEEK_API_KEY") {
            config.deepseek = Some(ProviderConfig {
                api_key,
                base_url: env::var("DEEPSEEK_BASE_URL").ok(),
                timeout_ms: env::var("DEEPSEEK_TIMEOUT_MS")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                proxy: env::var("DEEPSEEK_PROXY").ok(),
            });
        }
        
        // Zhipu (GLM)
        if let (Ok(api_key), _) = (
            env::var("ZHIPU_API_KEY").or_else(|_| env::var("GLM_API_KEY")),
            (),
        ) {
            config.zhipu = Some(ProviderConfig {
                api_key,
                base_url: env::var("ZHIPU_BASE_URL")
                    .or_else(|_| env::var("GLM_BASE_URL"))
                    .ok(),
                timeout_ms: env::var("ZHIPU_TIMEOUT_MS")
                    .or_else(|_| env::var("GLM_TIMEOUT_MS"))
                    .ok()
                    .and_then(|s| s.parse().ok()),
                proxy: env::var("ZHIPU_PROXY").or_else(|_| env::var("GLM_PROXY")).ok(),
            });
        }
        
        // Aliyun (Alibaba)
        if let (Ok(api_key), _) = (
            env::var("ALIYUN_API_KEY").or_else(|_| env::var("ALIBABA_QWEN_API_KEY")),
            (),
        ) {
            config.aliyun = Some(ProviderConfig {
                api_key,
                base_url: env::var("ALIYUN_BASE_URL")
                    .or_else(|_| env::var("ALIBABA_QWEN_BASE_URL"))
                    .ok(),
                timeout_ms: env::var("ALIYUN_TIMEOUT_MS")
                    .or_else(|_| env::var("ALIBABA_QWEN_TIMEOUT_MS"))
                    .ok()
                    .and_then(|s| s.parse().ok()),
                proxy: env::var("ALIYUN_PROXY").or_else(|_| env::var("ALIBABA_QWEN_PROXY")).ok(),
            });
        }
        
        // Kimi (Moonshot)
        if let (Ok(api_key), _) = (
            env::var("KIMI_API_KEY").or_else(|_| env::var("MOONSHOT_API_KEY")),
            (),
        ) {
            config.kimi = Some(ProviderConfig {
                api_key,
                base_url: env::var("KIMI_BASE_URL")
                    .or_else(|_| env::var("MOONSHOT_BASE_URL"))
                    .ok(),
                timeout_ms: env::var("KIMI_TIMEOUT_MS")
                    .or_else(|_| env::var("MOONSHOT_TIMEOUT_MS"))
                    .ok()
                    .and_then(|s| s.parse().ok()),
                proxy: env::var("KIMI_PROXY").or_else(|_| env::var("MOONSHOT_PROXY")).ok(),
            });
        }
        
        config
    }
    
    /// Get provider configuration by name
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        match name {
            "openai" => self.openai.as_ref(),
            "anthropic" => self.anthropic.as_ref(),
            "deepseek" => self.deepseek.as_ref(),
            "glm" | "zhipu" => self.zhipu.as_ref(),
            "aliyun" | "alibaba" => self.aliyun.as_ref(),
            "kimi" | "moonshot" => self.kimi.as_ref(),
            _ => None,
        }
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

impl ProviderConfig {
    /// Get the default base URL for a provider
    pub fn default_base_url(provider: &str) -> Option<String> {
        match provider {
            "openai" => Some("https://api.openai.com/v1".to_string()),
            "anthropic" => Some("https://api.anthropic.com".to_string()),
            "deepseek" => Some("https://api.deepseek.com/v1".to_string()),
            "glm" | "zhipu" => Some("https://open.bigmodel.cn/api/paas/v4".to_string()),
            "aliyun" | "alibaba" => Some("https://dashscope.aliyuncs.com/compatible-mode/v1".to_string()),
            "kimi" | "moonshot" => Some("https://api.moonshot.cn/v1".to_string()),
            _ => None,
        }
    }
    
    /// Get the effective base URL (configured or default)
    pub fn effective_base_url(&self, provider: &str) -> Option<String> {
        self.base_url.clone().or_else(|| Self::default_base_url(provider))
    }
    
    /// Get the effective timeout (configured or default)
    pub fn effective_timeout_ms(&self) -> u64 {
        self.timeout_ms.unwrap_or(30000) // 30 seconds default
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: None,
            timeout_ms: None,
            proxy: None,
        }
    }
}
