//! Protocol and Provider tests
//!
//! Tests for protocol implementations and provider adapters.

use llm_connector::{
    AnthropicProtocol, OpenAIProtocol, Protocol, ZhipuProtocol,
    types::{ChatRequest, Message, MessageBlock, Role},
};

#[test]
fn test_protocol_creation() {
    // Test OpenAI protocol creation
    let openai_protocol = OpenAIProtocol::new("test-key");
    assert_eq!(openai_protocol.name(), "openai");

    // Test Anthropic protocol creation
    let anthropic_protocol = AnthropicProtocol::new("sk-ant-test");
    assert_eq!(anthropic_protocol.name(), "anthropic");

    // Test Zhipu protocol creation
    let zhipu_protocol = ZhipuProtocol::new("test-key");
    assert_eq!(zhipu_protocol.name(), "zhipu");
}

#[test]
fn test_protocol_endpoints() {
    let openai_protocol = OpenAIProtocol::new("test-key");
    let base_url = "https://api.openai.com";
    assert!(
        openai_protocol
            .chat_endpoint(base_url)
            .contains("chat/completions")
    );

    let anthropic_protocol = AnthropicProtocol::new("sk-ant-test");
    let base_url = "https://api.anthropic.com";
    assert!(
        anthropic_protocol
            .chat_endpoint(base_url)
            .contains("messages")
    );

    let zhipu_protocol = ZhipuProtocol::new("test-key");
    assert!(
        zhipu_protocol
            .chat_endpoint("https://open.bigmodel.cn")
            .contains("chat/completions")
    );
}

#[test]
fn test_basic_request_building() {
    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: vec![MessageBlock::text("Hello, world!")],
            ..Default::default()
        }],
        max_tokens: Some(100),
        ..Default::default()
    };

    // Test that protocols can build requests without panicking
    let openai_protocol = OpenAIProtocol::new("test-key");
    let _openai_request = openai_protocol.build_request(&request);

    let anthropic_protocol = AnthropicProtocol::new("sk-ant-test");
    let _anthropic_request = anthropic_protocol.build_request(&request);

    let zhipu_protocol = ZhipuProtocol::new("test-key");
    let _zhipu_request = zhipu_protocol.build_request(&request);
}
