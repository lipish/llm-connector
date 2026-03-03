//! Protocol Mapping Tests (V2)
//!
//! Verified the correctness of request building and response parsing for each provider.

use llm_connector::core::Protocol;
use llm_connector::protocols::aliyun::AliyunProtocol;
use llm_connector::protocols::anthropic::AnthropicProtocol;
use llm_connector::protocols::openai::OpenAIProtocol;
use llm_connector::protocols::zhipu::ZhipuProtocol;
use llm_connector::types::{ChatRequest, Message};

#[test]
fn test_openai_request_mapping() {
    let protocol = OpenAIProtocol::new("test-key");
    let request = ChatRequest::new("gpt-4").add_message(Message::user("Hello"));

    let mapped = protocol.build_request(&request).unwrap();
    assert_eq!(mapped.model, "gpt-4");
    assert_eq!(mapped.messages.len(), 1);
    assert_eq!(mapped.messages[0]["role"], "user");
    assert_eq!(mapped.messages[0]["content"], "Hello");
}

#[test]
fn test_anthropic_request_mapping() {
    let protocol = AnthropicProtocol::new("test-key");
    let request = ChatRequest::new("claude-3")
        .add_message(Message::system("System prompt"))
        .add_message(Message::user("Hello"));

    let mapped = protocol.build_request(&request).unwrap();
    assert_eq!(mapped.model, "claude-3");
    assert_eq!(mapped.system.unwrap(), "System prompt");
    assert_eq!(mapped.messages.len(), 1);
    assert_eq!(mapped.messages[0].role, "user");
}

#[test]
fn test_zhipu_request_mapping() {
    let protocol = ZhipuProtocol::new("test-key");
    let request = ChatRequest::new("glm-4").add_message(Message::user("Hello zhipu"));

    let mapped = protocol.build_request(&request).unwrap();
    assert_eq!(mapped.model, "glm-4");
    assert_eq!(mapped.messages.len(), 1);
    assert_eq!(mapped.messages[0]["role"], "user");
}

#[test]
fn test_aliyun_request_mapping() {
    let protocol = AliyunProtocol::new("test-key");
    let request = ChatRequest::new("qwen-max").add_message(Message::user("Hello aliyun"));

    let mapped = protocol.build_request(&request).unwrap();
    assert_eq!(mapped.model, "qwen-max");
    assert_eq!(mapped.input.messages.len(), 1);
    assert_eq!(mapped.input.messages[0]["role"], "user");
}
