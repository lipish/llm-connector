//! Integration tests for OpenAI protocol

use llm_connector::{LlmClient, ChatRequest, Message};
use llm_connector::types::{Role, MessageBlock};

#[test]
fn test_openai_client_creation() {
    let client = LlmClient::openai("test-key").unwrap();
    assert_eq!(client.provider_name(), "openai");
}

#[test]
fn test_openai_compatible_client_creation() {
    let client = LlmClient::openai_with_base_url("test-key", "https://api.example.com/v1").unwrap();
    assert_eq!(client.provider_name(), "openai");
}

#[test]
fn test_chat_request_creation() {
    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![
            Message {
                role: Role::User,
                content: vec![MessageBlock::text("Hello")],
                name: None,
                tool_calls: None,
                tool_call_id: None,
                ..Default::default()
            }
        ],
        temperature: Some(0.7),
        max_tokens: Some(100),
        ..Default::default()
    };

    assert_eq!(request.model, "gpt-4");
    assert_eq!(request.messages.len(), 1);
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.max_tokens, Some(100));
}

#[test]
fn test_message_helper_user() {
    let message = Message::user("Hello");
    assert_eq!(message.role, Role::User);
    assert_eq!(message.content_as_text(), "Hello");
}

#[test]
fn test_message_helper_assistant() {
    let message = Message::assistant("Hi there");
    assert_eq!(message.role, Role::Assistant);
    assert_eq!(message.content_as_text(), "Hi there");
}

#[test]
fn test_message_helper_system() {
    let message = Message::system("You are a helpful assistant");
    assert_eq!(message.role, Role::System);
    assert_eq!(message.content_as_text(), "You are a helpful assistant");
}

#[test]
fn test_chat_request_with_multiple_messages() {
    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![
            Message::system("You are a helpful assistant"),
            Message::user("What is 2+2?"),
            Message::assistant("4"),
            Message::user("What is 3+3?"),
        ],
        ..Default::default()
    };

    assert_eq!(request.messages.len(), 4);
    assert_eq!(request.messages[0].role, Role::System);
    assert_eq!(request.messages[1].role, Role::User);
    assert_eq!(request.messages[2].role, Role::Assistant);
    assert_eq!(request.messages[3].role, Role::User);
}

#[test]
fn test_any_model_name_accepted() {
    // Test that any model name can be used (no hardcoded restrictions)
    let models = vec![
        "gpt-4",
        "gpt-4-turbo",
        "gpt-3.5-turbo",
        "gpt-4o",
        "o1-preview",
        "o1-mini",
        "custom-model-123",
        "any-model-name",
    ];

    for model_name in models {
        let request = ChatRequest {
            model: model_name.to_string(),
            messages: vec![Message::user("test")],
            ..Default::default()
        };
        assert_eq!(request.model, model_name);
    }
}

#[test]
fn test_default_chat_request() {
    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::user("test")],
        ..Default::default()
    };

    assert_eq!(request.temperature, None);
    assert_eq!(request.max_tokens, None);
    assert_eq!(request.top_p, None);
    assert_eq!(request.stream, None);
}

#[test]
fn test_chat_request_with_all_parameters() {
    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::user("test")],
        temperature: Some(0.8),
        max_tokens: Some(500),
        top_p: Some(0.9),
        stream: Some(false),
        ..Default::default()
    };

    assert_eq!(request.temperature, Some(0.8));
    assert_eq!(request.max_tokens, Some(500));
    assert_eq!(request.top_p, Some(0.9));
    assert_eq!(request.stream, Some(false));
}

#[tokio::test]
async fn test_fetch_models_with_invalid_key() {
    let client = LlmClient::openai("invalid-key").unwrap();
    
    // This should fail with authentication error
    let result = client.models().await;
    assert!(result.is_err(), "Should fail with invalid API key");
}

#[tokio::test]
async fn test_fetch_models_with_custom_url() {
    let client = LlmClient::openai_with_base_url("test-key", "https://invalid.example.com/v1").unwrap();
    
    // This should fail with connection error
    let result = client.models().await;
    assert!(result.is_err(), "Should fail with invalid URL");
}

