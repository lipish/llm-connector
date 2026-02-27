use llm_connector::core::Protocol;
use llm_connector::protocols::anthropic::AnthropicProtocol;
use llm_connector::protocols::openai::OpenAIProtocol;
use llm_connector::types::{ChatRequest, Message, ReasoningEffort};

#[test]
fn test_anthropic_thinking_budget_default_max_tokens() {
    let protocol = AnthropicProtocol::new("sk-test");

    let request = ChatRequest::new("claude-3-7-sonnet-20250219")
        .add_message(Message::user("test"))
        .with_thinking_budget(16000);

    // Default max_tokens is None, which defaults to 1024 in build_request.
    // 1024 <= 16000, so it should bump to 16000 + 4096 = 20096.

    let req = protocol.build_request(&request).unwrap();
    assert!(req.thinking.is_some());
    let thinking = req.thinking.unwrap();
    assert_eq!(thinking.budget_tokens, 16000);
    assert_eq!(thinking.thinking_type, "enabled");
    assert_eq!(req.max_tokens, 20096);
}

#[test]
fn test_anthropic_thinking_budget_custom_max_tokens() {
    let protocol = AnthropicProtocol::new("sk-test");

    let request = ChatRequest::new("claude-3-7-sonnet-20250219")
        .add_message(Message::user("test"))
        .with_thinking_budget(16000)
        .with_max_tokens(20000); // 20000 > 16000, so it should stay 20000

    let req = protocol.build_request(&request).unwrap();
    assert!(req.thinking.is_some());
    assert_eq!(req.max_tokens, 20000);
}

#[test]
fn test_anthropic_thinking_budget_too_small_max_tokens() {
    let protocol = AnthropicProtocol::new("sk-test");

    let request = ChatRequest::new("claude-3-7-sonnet-20250219")
        .add_message(Message::user("test"))
        .with_thinking_budget(16000)
        .with_max_tokens(10000); // 10000 <= 16000, so it should bump to 16000 + 4096 = 20096

    let req = protocol.build_request(&request).unwrap();
    assert!(req.thinking.is_some());
    assert_eq!(req.max_tokens, 20096);
}

#[test]
fn test_openai_reasoning_effort() {
    let protocol = OpenAIProtocol::new("sk-test");

    let request = ChatRequest::new("o1")
        .add_message(Message::user("test"))
        .with_reasoning_effort(ReasoningEffort::High);

    let req = protocol.build_request(&request).unwrap();
    assert_eq!(req.reasoning_effort, Some(ReasoningEffort::High));
}
