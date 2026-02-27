use llm_connector::types::{ChatRequest, Message, MessageBlock, Role};

#[test]
fn test_add_message_block() {
    let mut req = ChatRequest::new("gpt-4");
    
    // First block creates new user message
    req = req.add_message_block(MessageBlock::text("Hello"));
    assert_eq!(req.messages.len(), 1);
    assert_eq!(req.messages[0].role, Role::User);
    assert_eq!(req.messages[0].content.len(), 1);

    // Second block appends to existing user message
    req = req.add_message_block(MessageBlock::text("World"));
    assert_eq!(req.messages.len(), 1);
    assert_eq!(req.messages[0].content.len(), 2);

    // Add assistant message
    req = req.add_message(Message::assistant("Hi"));
    assert_eq!(req.messages.len(), 2);

    // Third block creates NEW user message (since last was assistant)
    req = req.add_message_block(MessageBlock::text("New Question"));
    assert_eq!(req.messages.len(), 3);
    assert_eq!(req.messages[2].role, Role::User);
}
