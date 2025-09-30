//! P0-1: Configuration Separation Demo
//!
//! This example demonstrates the improved configuration architecture where
//! config and registry are separated from protocols.
//!
//! ## Key Improvements
//!
//! 1. **Separated Concerns**: Config is now independent of protocols
//! 2. **Unified Configuration**: Single ProviderConfig type for all providers
//! 3. **Enhanced Features**: Retry config, custom headers, shared config
//! 4. **Better Organization**: Clear module boundaries

use llm_connector::{
    config::{ProviderConfig, RetryConfig, RegistryConfig, SharedProviderConfig},
    error::LlmConnectorError,
};
use std::collections::HashMap;

fn main() -> Result<(), LlmConnectorError> {
    println!("🚀 P0-1: Configuration Separation Demo");
    println!("========================================\n");

    // ============================================================================
    // 1. Basic Configuration
    // ============================================================================
    println!("📋 1. Basic Configuration");
    println!("   Creating a simple provider configuration:");
    
    let basic_config = ProviderConfig::new("my-api-key")
        .with_timeout_ms(5000);
    
    println!("   ✅ API Key: {}", basic_config.api_key);
    println!("   ✅ Timeout: {:?}", basic_config.timeout());
    
    // ============================================================================
    // 2. Advanced Configuration with Retry
    // ============================================================================
    println!("\n📋 2. Advanced Configuration with Retry");
    println!("   Adding retry policy:");
    
    let retry_config = RetryConfig {
        max_retries: 3,
        initial_backoff_ms: 1000,
        backoff_multiplier: 2.0,
        max_backoff_ms: 30000,
    };
    
    let advanced_config = ProviderConfig::new("my-api-key")
        .with_base_url("https://api.example.com")
        .with_timeout_ms(10000)
        .with_retry(retry_config.clone());
    
    println!("   ✅ Base URL: {:?}", advanced_config.base_url);
    println!("   ✅ Retry enabled: {}", advanced_config.retry.is_some());
    println!("   ✅ Max retries: {}", retry_config.max_retries);
    println!("   ✅ Initial backoff: {}ms", retry_config.initial_backoff_ms);
    
    // ============================================================================
    // 3. Custom Headers
    // ============================================================================
    println!("\n📋 3. Custom Headers");
    println!("   Adding custom HTTP headers:");
    
    let mut headers = HashMap::new();
    headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
    headers.insert("X-Request-ID".to_string(), "req-12345".to_string());
    
    let header_config = ProviderConfig::new("my-api-key")
        .with_headers(headers.clone());
    
    println!("   ✅ Headers configured: {}", header_config.headers.is_some());
    if let Some(h) = &header_config.headers {
        for (key, value) in h {
            println!("      - {}: {}", key, value);
        }
    }
    
    // ============================================================================
    // 4. Shared Configuration (Zero-Copy)
    // ============================================================================
    println!("\n📋 4. Shared Configuration (Zero-Copy)");
    println!("   Using Arc for efficient sharing:");
    
    let config = ProviderConfig::new("shared-key")
        .with_timeout_ms(5000);
    
    let shared1 = SharedProviderConfig::new(config.clone());
    let shared2 = shared1.clone();
    let shared3 = shared1.clone();
    
    println!("   ✅ Created 3 shared references");
    println!("   ✅ API Key from shared1: {}", shared1.api_key);
    println!("   ✅ API Key from shared2: {}", shared2.api_key);
    println!("   ✅ API Key from shared3: {}", shared3.api_key);
    println!("   ✅ All references point to the same data (zero-copy)");
    
    // ============================================================================
    // 5. Registry Configuration
    // ============================================================================
    println!("\n📋 5. Registry Configuration");
    println!("   Creating a multi-provider registry:");
    
    let registry = RegistryConfig::new()
        .add_provider(
            "deepseek",
            "openai",
            ProviderConfig::new("deepseek-key")
                .with_base_url("https://api.deepseek.com/v1")
                .with_retry(RetryConfig::default()),
        )
        .add_provider(
            "claude",
            "anthropic",
            ProviderConfig::new("claude-key")
                .with_base_url("https://api.anthropic.com")
                .with_timeout_ms(60000),
        )
        .add_provider(
            "qwen",
            "aliyun",
            ProviderConfig::new("qwen-key")
                .with_timeout_ms(30000),
        );
    
    println!("   ✅ Registered {} providers", registry.providers.len());
    for name in registry.provider_names() {
        if let Some(entry) = registry.get_provider(name) {
            println!("      - {}: protocol={}", name, entry.protocol);
        }
    }
    
    // ============================================================================
    // 6. Builder Pattern
    // ============================================================================
    println!("\n📋 6. Builder Pattern");
    println!("   Fluent configuration building:");
    
    let fluent_config = ProviderConfig::new("api-key")
        .with_base_url("https://api.example.com")
        .with_timeout_ms(15000)
        .with_proxy("http://proxy.example.com:8080")
        .with_header("Authorization", "Bearer token")
        .with_header("X-API-Version", "v1")
        .with_retry(RetryConfig {
            max_retries: 5,
            initial_backoff_ms: 500,
            backoff_multiplier: 1.5,
            max_backoff_ms: 10000,
        })
        .with_max_concurrent_requests(10);
    
    println!("   ✅ Base URL: {:?}", fluent_config.base_url);
    println!("   ✅ Timeout: {}ms", fluent_config.timeout_ms.unwrap());
    println!("   ✅ Proxy: {:?}", fluent_config.proxy);
    println!("   ✅ Headers: {} custom headers", fluent_config.headers.as_ref().map(|h| h.len()).unwrap_or(0));
    println!("   ✅ Retry: enabled");
    println!("   ✅ Max concurrent: {}", fluent_config.max_concurrent_requests.unwrap());
    
    // ============================================================================
    // 7. Configuration Serialization
    // ============================================================================
    println!("\n📋 7. Configuration Serialization");
    println!("   Serializing configuration to JSON:");
    
    let json_config = ProviderConfig::new("test-key")
        .with_base_url("https://api.test.com")
        .with_timeout_ms(5000);
    
    let json = serde_json::to_string_pretty(&json_config)
        .map_err(|e| LlmConnectorError::ConfigError(e.to_string()))?;
    
    println!("   ✅ JSON output:");
    println!("{}", json);
    
    // ============================================================================
    // Summary
    // ============================================================================
    println!("\n🎯 P0-1 Improvements Summary");
    println!("============================");
    println!("✅ Separated config from protocols");
    println!("✅ Unified ProviderConfig type");
    println!("✅ Added RetryConfig for resilience");
    println!("✅ Support for custom headers");
    println!("✅ SharedProviderConfig for zero-copy sharing");
    println!("✅ RegistryConfig for multi-provider management");
    println!("✅ Fluent builder pattern");
    println!("✅ JSON serialization support");
    
    println!("\n📊 Architecture Benefits");
    println!("========================");
    println!("✅ Clear separation of concerns");
    println!("✅ Config independent of protocols");
    println!("✅ Easier to test and maintain");
    println!("✅ More flexible and extensible");
    println!("✅ Better performance with Arc");
    
    println!("\n✨ P0-1 Demo Complete!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_config() {
        let config = ProviderConfig::new("test-key");
        assert_eq!(config.api_key, "test-key");
    }
    
    #[test]
    fn test_retry_config() {
        let retry = RetryConfig::default();
        assert_eq!(retry.max_retries, 3);
        assert_eq!(retry.initial_backoff_ms, 1000);
    }
    
    #[test]
    fn test_shared_config() {
        let config = ProviderConfig::new("test-key");
        let shared1 = SharedProviderConfig::new(config);
        let shared2 = shared1.clone();
        
        assert_eq!(shared1.api_key, shared2.api_key);
    }
    
    #[test]
    fn test_registry_config() {
        let registry = RegistryConfig::new()
            .add_provider("test1", "openai", ProviderConfig::new("key1"))
            .add_provider("test2", "anthropic", ProviderConfig::new("key2"));
        
        assert_eq!(registry.providers.len(), 2);
        assert!(registry.get_provider("test1").is_some());
        assert!(registry.get_provider("test2").is_some());
    }
}
