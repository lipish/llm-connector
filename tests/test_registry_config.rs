//! Test provider registry configuration functionality

use llm_connector::providers::{ProviderRegistry, RegistryConfig};

mod common;

#[test]
fn test_registry_config_creation() {
    println!("🧪 Testing registry configuration creation");

    let config = RegistryConfig::default_config();

    // Verify default providers are included
    assert!(config.has_provider("aliyun"));
    assert!(config.has_provider("deepseek"));
    assert!(config.has_provider("zhipu"));

    // Verify provider count
    assert_eq!(config.provider_names().len(), 3);

    println!("✅ Registry config creation test passed");
}

#[test]
fn test_registry_from_config() {
    println!("🧪 Testing registry creation from configuration");

    let config = RegistryConfig::default_config();

    // Create registry from config
    let registry = ProviderRegistry::from_config(config).unwrap();

    // Verify all providers are registered
    assert!(registry.has_provider("aliyun"));
    assert!(registry.has_provider("deepseek"));
    assert!(registry.has_provider("zhipu"));
    assert_eq!(registry.len(), 3);

    // Verify providers can be retrieved
    assert!(registry.get_provider("aliyun").is_some());
    assert!(registry.get_provider("deepseek").is_some());
    assert!(registry.get_provider("zhipu").is_some());

    println!("✅ Registry from config test passed");
}

#[test]
fn test_multiple_provider_registration() {
    use llm_connector::providers::{AliyunAdapter, openai::{deepseek, zhipu}};

    println!("🧪 Testing multiple provider registration");

    let mut registry = ProviderRegistry::new();

    let config1 = llm_connector::config::ProviderConfig {
        api_key: "deepseek-key".to_string(),
        base_url: Some("https://api.deepseek.com".to_string()),
        timeout_ms: Some(30000),
        proxy: None,
    };

    let config2 = llm_connector::config::ProviderConfig {
        api_key: "aliyun-key".to_string(),
        base_url: Some("https://dashscope.aliyuncs.com".to_string()),
        timeout_ms: Some(45000),
        proxy: None,
    };

    let config3 = llm_connector::config::ProviderConfig {
        api_key: "zhipu-key".to_string(),
        base_url: Some("https://open.bigmodel.cn".to_string()),
        timeout_ms: Some(60000),
        proxy: None,
    };

    // Register multiple providers
    registry.register("deepseek", config1, deepseek()).unwrap();
    registry.register("aliyun", config2, AliyunAdapter).unwrap();
    registry.register("zhipu", config3, zhipu()).unwrap();

    // Verify all providers are registered
    assert_eq!(registry.len(), 3);
    assert!(registry.has_provider("deepseek"));
    assert!(registry.has_provider("aliyun"));
    assert!(registry.has_provider("zhipu"));

    // Verify provider names
    let provider_names = registry.provider_names();
    assert!(provider_names.contains(&"deepseek"));
    assert!(provider_names.contains(&"aliyun"));
    assert!(provider_names.contains(&"zhipu"));

    // Verify each provider has correct name
    assert_eq!(registry.get_provider("deepseek").unwrap().name(), "deepseek");
    assert_eq!(registry.get_provider("aliyun").unwrap().name(), "aliyun");
    assert_eq!(registry.get_provider("zhipu").unwrap().name(), "zhipu");

    println!("✅ Multiple provider registration test passed");
}


