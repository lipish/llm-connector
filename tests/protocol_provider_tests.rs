//! Unit tests for Protocol implementations
//!
//! These tests focus on the Protocol implementations without making real API calls.

use llm_connector::{
    core::Protocol,
    protocols::{AnthropicProtocol, OpenAIProtocol, ZhipuProtocol, ProviderAdapter},
    types::{ChatRequest, Message},
};

#[test]
fn test_protocol_creation() {
    // Test OpenAI protocol creation
    let openai_protocol = OpenAIProtocol::new("test-key");
    assert_eq!(openai_protocol.name(), "openai");

    // Test Anthropic protocol creation
    let anthropic_protocol = AnthropicProtocol::new("test-key");
    assert_eq!(anthropic_protocol.name(), "anthropic");

    // Test Zhipu protocol creation
    let zhipu_protocol = ZhipuProtocol::new();
    assert_eq!(zhipu_protocol.name(), "zhipu");
}

#[test]
fn test_protocol_endpoints() {
    let openai_protocol = OpenAIProtocol::new("test-key");
    let base_url = Some("https://api.openai.com/v1".to_string());
    assert!(openai_protocol.endpoint_url(&base_url).contains("chat/completions"));

    let anthropic_protocol = AnthropicProtocol::new("test-key");
    let base_url = Some("https://api.anthropic.com".to_string());
    assert!(anthropic_protocol.endpoint_url(&base_url).contains("messages"));

    let zhipu_protocol = ZhipuProtocol::new();
    assert!(zhipu_protocol.endpoint("https://open.bigmodel.cn/api/paas/v4").contains("chat/completions"));
}

#[test]
fn test_basic_request_building() {
    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![Message::user("Hello, world!")],
        max_tokens: Some(100),
        ..Default::default()
    };

    // Test that protocols can build requests without panicking
    let openai_protocol = OpenAIProtocol::new("test-key");
    let _openai_request = openai_protocol.build_request_data(&request, false);

    let anthropic_protocol = AnthropicProtocol::new("test-key");
    let _anthropic_request = anthropic_protocol.build_request_data(&request, false);

    let zhipu_protocol = ZhipuProtocol::new();
    let _zhipu_request = zhipu_protocol.build_request(&request, false);
}
