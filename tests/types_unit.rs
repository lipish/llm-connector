//! Types Unit Tests
//!
//! Verified the core data types and their helpers.

use llm_connector::types::{ChatRequest, Message, MessageBlock, Role};

#[test]
fn test_message_creation() {
    let msg = Message::user("Hello");
    assert_eq!(msg.role, Role::User);
    assert_eq!(msg.content_as_text(), "Hello");

    let sys = Message::system("System");
    assert_eq!(sys.role, Role::System);
    assert_eq!(sys.content_as_text(), "System");
}

#[test]
fn test_chat_request_builder() {
    let req = ChatRequest::new("model")
        .add_message(Message::user("h1"))
        .with_temperature(0.5)
        .with_max_tokens(100)
        .with_stream(true); // Added with_stream

    assert_eq!(req.model, "model");
    assert_eq!(req.messages.len(), 1);
    assert_eq!(req.temperature, Some(0.5));
    assert_eq!(req.max_tokens, Some(100));
    assert_eq!(req.stream, Some(true)); // Added assertion for stream
}

#[test]
fn test_message_blocks() {
    let msg = Message::new(
        Role::User,
        vec![MessageBlock::text("t1"), MessageBlock::text("t2")],
    );
    assert_eq!(msg.content.len(), 2);
    assert_eq!(msg.content_as_text(), "t1\nt2");
}
