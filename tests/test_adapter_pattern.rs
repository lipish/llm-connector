//! Test the adapter pattern implementation
//! 
//! This test validates that our adapter pattern refactoring works correctly
//! and that all adapters implement the ProviderAdapter trait properly.

use llm_connector::providers::{
    ProviderAdapter, AliyunAdapter, GenericProvider, Provider,
    openai::{deepseek, zhipu},
};
use llm_connector::types::{ChatRequest, Message};
use llm_connector::config::ProviderConfig;

mod common;

#[test]
fn test_adapter_trait_implementation() {
    println!("🧪 Testing adapter trait implementations");
    
    // Test DeepSeek adapter
    let deepseek = deepseek();
    assert_eq!(deepseek.name(), "deepseek");
    assert!(deepseek.supported_models().contains(&"deepseek-chat".to_string()));
    assert!(deepseek.endpoint_url(&None).contains("deepseek"));

    // Test Aliyun adapter
    let aliyun = AliyunAdapter;
    assert_eq!(aliyun.name(), "aliyun");
    assert!(aliyun.supported_models().contains(&"qwen-turbo".to_string()));
    assert!(aliyun.endpoint_url(&None).contains("dashscope"));

    // Test Zhipu adapter
    let zhipu = zhipu();
    assert_eq!(zhipu.name(), "zhipu");
    assert!(zhipu.supported_models().contains(&"glm-4".to_string()));
    assert!(zhipu.endpoint_url(&None).contains("bigmodel"));
    
    println!("✅ Adapter trait implementation test passed");
}

#[test]
fn test_adapter_request_building() {
    println!("🧪 Testing adapter request building");
    
    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
                ..Default::default()
            },
            Message {
                role: "user".to_string(),
                content: "Hello, how are you?".to_string(),
                ..Default::default()
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(100),
        ..Default::default()
    };
    
    // Test DeepSeek adapter request building
    let deepseek = DeepSeekAdapter;
    let deepseek_request = deepseek.build_request_data(&request, false);
    // The request should be serializable (we can't inspect the exact structure without exposing internals)
    let _serialized = serde_json::to_value(deepseek_request).unwrap();
    
    // Test Aliyun adapter request building
    let aliyun = AliyunAdapter;
    let aliyun_request = aliyun.build_request_data(&request, false);
    let _serialized = serde_json::to_value(aliyun_request).unwrap();
    
    // Test Zhipu adapter request building
    let zhipu = ZhipuAdapter;
    let zhipu_request = zhipu.build_request_data(&request, false);
    let _serialized = serde_json::to_value(zhipu_request).unwrap();
    
    println!("✅ Adapter request building test passed");
}

#[test]
fn test_adapter_streaming_request_building() {
    println!("🧪 Testing adapter streaming request building");
    
    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                ..Default::default()
            },
        ],
        stream: Some(true),
        ..Default::default()
    };
    
    // Test streaming request building for all adapters
    let deepseek = DeepSeekAdapter;
    let deepseek_stream_request = deepseek.build_request_data(&request, true);
    let _serialized = serde_json::to_value(deepseek_stream_request).unwrap();
    
    let aliyun = AliyunAdapter;
    let aliyun_stream_request = aliyun.build_request_data(&request, true);
    let _serialized = serde_json::to_value(aliyun_stream_request).unwrap();
    
    let zhipu = ZhipuAdapter;
    let zhipu_stream_request = zhipu.build_request_data(&request, true);
    let _serialized = serde_json::to_value(zhipu_stream_request).unwrap();
    
    println!("✅ Adapter streaming request building test passed");
}

#[test]
fn test_adapter_endpoint_url_customization() {
    println!("🧪 Testing adapter endpoint URL customization");
    
    // Test default URLs
    let deepseek = DeepSeekAdapter;
    let default_url = deepseek.endpoint_url(&None);
    assert!(default_url.contains("deepseek"));
    
    // Test custom URLs
    let custom_base = Some("https://custom.api.com".to_string());
    let custom_url = deepseek.endpoint_url(&custom_base);
    assert!(custom_url.contains("custom.api.com"));
    
    // Test for all adapters
    let aliyun = AliyunAdapter;
    assert!(aliyun.endpoint_url(&None).contains("dashscope"));
    assert!(aliyun.endpoint_url(&custom_base).contains("custom.api.com"));
    
    let zhipu = ZhipuAdapter;
    assert!(zhipu.endpoint_url(&None).contains("bigmodel"));
    assert!(zhipu.endpoint_url(&custom_base).contains("custom.api.com"));
    
    println!("✅ Adapter endpoint URL customization test passed");
}

#[test]
fn test_adapter_model_support() {
    println!("🧪 Testing adapter model support");
    
    // Test DeepSeek models
    let deepseek = DeepSeekAdapter;
    let deepseek_models = deepseek.supported_models();
    assert!(deepseek_models.contains(&"deepseek-chat".to_string()));
    assert!(deepseek_models.contains(&"deepseek-coder".to_string()));
    
    // Test Aliyun models
    let aliyun = AliyunAdapter;
    let aliyun_models = aliyun.supported_models();
    assert!(aliyun_models.contains(&"qwen-turbo".to_string()));
    assert!(aliyun_models.contains(&"qwen-plus".to_string()));
    
    // Test Zhipu models
    let zhipu = ZhipuAdapter;
    let zhipu_models = zhipu.supported_models();
    assert!(zhipu_models.contains(&"glm-4".to_string()));
    assert!(zhipu_models.contains(&"glm-3-turbo".to_string()));
    
    println!("✅ Adapter model support test passed");
}

#[test]
fn test_generic_provider_with_different_adapters() {
    println!("🧪 Testing GenericProvider with different adapters");
    
    let config = ProviderConfig {
        api_key: "test-key".to_string(),
        base_url: Some("https://api.test.com".to_string()),
        timeout_ms: Some(30000),
        proxy: None,
    };
    
    // Test GenericProvider with DeepSeek adapter
    let deepseek_provider = GenericProvider::new(config.clone(), deepseek()).unwrap();
    assert_eq!(deepseek_provider.name(), "deepseek");
    assert!(deepseek_provider.supports_model("deepseek-chat"));
    assert!(!deepseek_provider.supports_model("gpt-4"));

    // Test GenericProvider with Aliyun adapter
    let aliyun_provider = GenericProvider::new(config.clone(), AliyunAdapter).unwrap();
    assert_eq!(aliyun_provider.name(), "aliyun");
    assert!(aliyun_provider.supports_model("qwen-turbo"));
    assert!(!aliyun_provider.supports_model("deepseek-chat"));

    // Test GenericProvider with Zhipu adapter
    let zhipu_provider = GenericProvider::new(config.clone(), zhipu()).unwrap();
    assert_eq!(zhipu_provider.name(), "zhipu");
    assert!(zhipu_provider.supports_model("glm-4"));
    assert!(!zhipu_provider.supports_model("qwen-turbo"));
    
    println!("✅ GenericProvider with different adapters test passed");
}

#[test]
fn test_adapter_cloning() {
    println!("🧪 Testing adapter cloning");
    
    // Test that all adapters can be cloned (required by ProviderAdapter trait)
    let deepseek = DeepSeekAdapter;
    let deepseek_clone = deepseek.clone();
    assert_eq!(deepseek.name(), deepseek_clone.name());
    
    let aliyun = AliyunAdapter;
    let aliyun_clone = aliyun.clone();
    assert_eq!(aliyun.name(), aliyun_clone.name());
    
    let zhipu = ZhipuAdapter;
    let zhipu_clone = zhipu.clone();
    assert_eq!(zhipu.name(), zhipu_clone.name());
    
    println!("✅ Adapter cloning test passed");
}

#[test]
fn test_adapter_send_sync() {
    println!("🧪 Testing adapter Send + Sync traits");
    
    // Test that adapters implement Send + Sync (required for async usage)
    fn assert_send_sync<T: Send + Sync>() {}
    
    assert_send_sync::<DeepSeekAdapter>();
    assert_send_sync::<AliyunAdapter>();
    assert_send_sync::<ZhipuAdapter>();
    
    println!("✅ Adapter Send + Sync test passed");
}

#[test]
fn test_adapter_static_lifetime() {
    println!("🧪 Testing adapter 'static lifetime");
    
    // Test that adapters have 'static lifetime (required by ProviderAdapter trait)
    fn assert_static<T: 'static>() {}
    
    assert_static::<DeepSeekAdapter>();
    assert_static::<AliyunAdapter>();
    assert_static::<ZhipuAdapter>();
    
    println!("✅ Adapter 'static lifetime test passed");
}

#[test]
fn test_adapter_debug_display() {
    println!("🧪 Testing adapter Debug trait");
    
    // Test that adapters implement Debug (useful for debugging)
    let deepseek = DeepSeekAdapter;
    let debug_str = format!("{:?}", deepseek);
    assert!(debug_str.contains("DeepSeekAdapter"));
    
    let aliyun = AliyunAdapter;
    let debug_str = format!("{:?}", aliyun);
    assert!(debug_str.contains("AliyunAdapter"));
    
    let zhipu = ZhipuAdapter;
    let debug_str = format!("{:?}", zhipu);
    assert!(debug_str.contains("ZhipuAdapter"));
    
    println!("✅ Adapter Debug trait test passed");
}
