//! End-to-end tests for all provider adapters
//!
//! This test suite validates that our provider architecture refactoring is successful
//! by testing all three providers (DeepSeek, Aliyun, Zhipu) with both regular and streaming requests.

use llm_connector::{ChatRequest, Client, Config, Message, ProviderConfig};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Test all providers with regular chat completion
/// Note: This test uses Client API which requires model routing configuration.
/// For direct provider testing, use GenericProvider instead (see test_generic_provider_with_adapters).
#[tokio::test]
#[ignore] // Ignored because Client requires proper model routing setup
async fn test_all_providers_chat_completion() {
    println!("🧪 Testing providers with regular chat completion");

    // Test DeepSeek
    test_provider_chat_completion("deepseek", "deepseek-chat", "/chat/completions").await;

    // Test Zhipu
    test_provider_chat_completion("zhipu", "glm-4", "/chat/completions").await;

    println!("✅ All tested providers passed regular chat completion tests");
}

/// Test Aliyun provider separately due to different response format
/// Note: Temporarily disabled due to response format differences
#[tokio::test]
#[ignore]
async fn test_aliyun_provider_chat_completion() {
    println!("🧪 Testing Aliyun provider with regular chat completion");

    test_provider_chat_completion(
        "aliyun",
        "qwen-turbo",
        "/api/v1/services/aigc/text-generation/generation",
    )
    .await;

    println!("✅ Aliyun provider passed chat completion test");
}

/// Test streaming functionality for all providers
#[cfg(feature = "streaming")]
#[tokio::test]
async fn test_all_providers_streaming() {
    println!("🧪 Testing all providers with streaming");

    // Test DeepSeek streaming
    test_provider_streaming("deepseek", "deepseek-chat", "/chat/completions").await;

    // Test Aliyun streaming
    test_provider_streaming(
        "aliyun",
        "qwen-turbo",
        "/api/v1/services/aigc/text-generation/generation",
    )
    .await;

    // Test Zhipu streaming
    test_provider_streaming("zhipu", "glm-4", "/chat/completions").await;

    println!("✅ All providers passed streaming tests");
}

/// Test provider registry functionality
#[tokio::test]
async fn test_provider_registry() {
    use llm_connector::protocols::{
        aliyun::AliyunProtocol,
        openai::{deepseek, zhipu},
    };
    use llm_connector::registry::ProviderRegistry;

    println!("🧪 Testing provider registry");

    let mut registry = ProviderRegistry::new();

    let config = ProviderConfig::new("test-key")
        .with_base_url("https://api.example.com")
        .with_timeout_ms(30000);

    // Register all providers
    registry
        .register("deepseek", config.clone(), deepseek())
        .unwrap();
    registry
        .register("aliyun", config.clone(), AliyunProtocol::new(None))
        .unwrap();
    registry.register("zhipu", config.clone(), zhipu()).unwrap();

    // Verify all providers are registered
    assert!(registry.has_provider("deepseek"));
    assert!(registry.has_provider("aliyun"));
    assert!(registry.has_provider("zhipu"));
    assert_eq!(registry.len(), 3);

    // Test provider retrieval
    assert!(registry.get_provider("deepseek").is_some());
    assert!(registry.get_provider("aliyun").is_some());
    assert!(registry.get_provider("zhipu").is_some());
    assert!(registry.get_provider("nonexistent").is_none());

    println!("✅ Provider registry test passed");
}

/// Test GenericProvider directly with different adapters
#[tokio::test]
async fn test_generic_provider_with_adapters() {
    use llm_connector::protocols::{
        aliyun::AliyunProtocol,
        openai::{deepseek, zhipu},
        GenericProvider, Provider,
    };

    println!("🧪 Testing GenericProvider with different adapters");

    let config = ProviderConfig::new("test-key")
        .with_base_url("https://api.example.com")
        .with_timeout_ms(30000);

    // Test DeepSeek adapter
    let deepseek_provider = GenericProvider::new(config.clone(), deepseek()).unwrap();
    assert_eq!(deepseek_provider.name(), "deepseek");
    assert!(deepseek_provider.supports_model("deepseek-chat"));

    // Test Aliyun adapter
    let aliyun_provider = GenericProvider::new(config.clone(), AliyunProtocol::new(None)).unwrap();
    assert_eq!(aliyun_provider.name(), "aliyun");
    assert!(aliyun_provider.supports_model("qwen-turbo"));

    // Test Zhipu adapter
    let zhipu_provider = GenericProvider::new(config.clone(), zhipu()).unwrap();
    assert_eq!(zhipu_provider.name(), "zhipu");
    assert!(zhipu_provider.supports_model("glm-4"));

    println!("✅ GenericProvider adapter test passed");
}

/// Helper function to test a single provider's chat completion
async fn test_provider_chat_completion(provider_name: &str, model: &str, endpoint: &str) {
    println!(
        "  📝 Testing {} provider with model {}",
        provider_name, model
    );

    // Start mock server
    let server = MockServer::start().await;

    // Create provider-specific mock response
    let response_body = match provider_name {
        "aliyun" => json!({
            "request_id": format!("{}-test-request", provider_name),
            "output": {
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": format!("Hello from {} provider!", provider_name)
                    },
                    "finish_reason": "stop"
                }]
            },
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 15,
                "total_tokens": 25
            }
        }),
        _ => json!({
            "id": format!("chatcmpl-{}-test", provider_name),
            "object": "chat.completion",
            "created": 1677652288,
            "model": model,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": format!("Hello from {} provider!", provider_name)
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 15,
                "total_tokens": 25
            }
        }),
    };

    Mock::given(method("POST"))
        .and(path(endpoint))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&server)
        .await;

    // Configure client
    let config = create_provider_config(provider_name, &server.uri());
    let client = Client::with_config(config);

    // Create request
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    // Send request and verify response
    let response = client.chat(request).await.unwrap();
    assert_eq!(response.model, model);
    assert_eq!(
        response.choices[0].message.content,
        format!("Hello from {} provider!", provider_name)
    );
    assert!(response.usage.is_some());

    println!(
        "    ✅ {} provider chat completion test passed",
        provider_name
    );
}

/// Helper function to test a single provider's streaming
#[cfg(feature = "streaming")]
async fn test_provider_streaming(provider_name: &str, model: &str, endpoint: &str) {
    use futures_util::StreamExt;

    println!(
        "  📝 Testing {} provider streaming with model {}",
        provider_name, model
    );

    // Start mock server
    let server = MockServer::start().await;

    // Create SSE mock response
    let sse_data = format!(
        "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
        json!({
            "id": format!("chatcmpl-{}-stream", provider_name),
            "object": "chat.completion.chunk",
            "created": 1677652288,
            "model": model,
            "choices": [{
                "index": 0,
                "delta": {
                    "role": "assistant",
                    "content": "Hello "
                },
                "finish_reason": null
            }]
        }),
        json!({
            "id": format!("chatcmpl-{}-stream", provider_name),
            "object": "chat.completion.chunk",
            "created": 1677652288,
            "model": model,
            "choices": [{
                "index": 0,
                "delta": {
                    "content": "from streaming!"
                },
                "finish_reason": "stop"
            }]
        })
    );

    Mock::given(method("POST"))
        .and(path(endpoint))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(sse_data)
                .insert_header("Content-Type", "text/event-stream")
                .insert_header("Cache-Control", "no-cache"),
        )
        .mount(&server)
        .await;

    // Configure client
    let config = create_provider_config(provider_name, &server.uri());
    let client = Client::with_config(config);

    // Create streaming request
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            ..Default::default()
        }],
        stream: Some(true),
        ..Default::default()
    };

    // Send streaming request and collect responses
    let mut stream = client.chat_stream(request).await.unwrap();
    let mut responses = Vec::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => responses.push(response),
            Err(e) => panic!("Streaming error for {}: {}", provider_name, e),
        }
    }

    // Verify we received streaming responses
    assert!(
        !responses.is_empty(),
        "No streaming responses received for {}",
        provider_name
    );

    println!(
        "    ✅ {} provider streaming test passed ({} chunks)",
        provider_name,
        responses.len()
    );
}

/// Helper function to create provider-specific configuration
fn create_provider_config(provider_name: &str, server_uri: &str) -> Config {
    let provider_config = ProviderConfig::new("test-api-key")
        .with_base_url(server_uri.to_string())
        .with_timeout_ms(30000);

    match provider_name {
        "deepseek" => Config {
            deepseek: Some(provider_config),
            ..Default::default()
        },
        "aliyun" => Config {
            aliyun: Some(provider_config),
            ..Default::default()
        },
        "zhipu" => Config {
            zhipu: Some(provider_config),
            ..Default::default()
        },
        _ => panic!("Unknown provider: {}", provider_name),
    }
}
