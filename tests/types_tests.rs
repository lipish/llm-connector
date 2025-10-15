//! Tests for type definitions and serialization

use llm_connector::types::{ChatRequest, ChatResponse, Message, Role, Choice, Usage};
use serde_json;

#[test]
fn test_role_serialization() {
    assert_eq!(serde_json::to_string(&Role::User).unwrap(), r#""user""#);
    assert_eq!(serde_json::to_string(&Role::Assistant).unwrap(), r#""assistant""#);
    assert_eq!(serde_json::to_string(&Role::System).unwrap(), r#""system""#);
}

#[test]
fn test_role_deserialization() {
    assert_eq!(serde_json::from_str::<Role>(r#""user""#).unwrap(), Role::User);
    assert_eq!(serde_json::from_str::<Role>(r#""assistant""#).unwrap(), Role::Assistant);
    assert_eq!(serde_json::from_str::<Role>(r#""system""#).unwrap(), Role::System);
}

#[test]
fn test_message_serialization() {
    let message = Message {
        role: Role::User,
        content: "Hello".to_string(),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        ..Default::default()
    };

    let json = serde_json::to_value(&message).unwrap();
    assert_eq!(json["role"], "user");
    assert_eq!(json["content"], "Hello");
}

#[test]
fn test_message_deserialization() {
    let json = r#"{"role":"user","content":"Hello"}"#;
    let message: Message = serde_json::from_str(json).unwrap();
    
    assert_eq!(message.role, Role::User);
    assert_eq!(message.content, "Hello");
    assert_eq!(message.name, None);
}

#[test]
fn test_message_with_name() {
    let message = Message {
        role: Role::User,
        content: "Hello".to_string(),
        name: Some("John".to_string()),
        tool_calls: None,
        tool_call_id: None,
        ..Default::default()
    };

    let json = serde_json::to_value(&message).unwrap();
    assert_eq!(json["name"], "John");
}

#[test]
fn test_chat_request_serialization() {
    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::user("Hello")],
        temperature: Some(0.7),
        max_tokens: Some(100),
        ..Default::default()
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["model"], "gpt-4");
    // Use approximate comparison for floating point
    assert!((json["temperature"].as_f64().unwrap() - 0.7).abs() < 0.01);
    assert_eq!(json["max_tokens"], 100);
    assert!(json["messages"].is_array());
}

#[test]
fn test_chat_request_default() {
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
fn test_usage_serialization() {
    let usage = Usage {
        prompt_tokens: 10,
        completion_tokens: 20,
        total_tokens: 30,
        prompt_cache_hit_tokens: None,
        prompt_cache_miss_tokens: None,
        prompt_tokens_details: None,
        completion_tokens_details: None,
    };

    let json = serde_json::to_value(&usage).unwrap();
    assert_eq!(json["prompt_tokens"], 10);
    assert_eq!(json["completion_tokens"], 20);
    assert_eq!(json["total_tokens"], 30);
}

#[test]
fn test_usage_deserialization() {
    let json = r#"{"prompt_tokens":10,"completion_tokens":20,"total_tokens":30}"#;
    let usage: Usage = serde_json::from_str(json).unwrap();
    
    assert_eq!(usage.prompt_tokens, 10);
    assert_eq!(usage.completion_tokens, 20);
    assert_eq!(usage.total_tokens, 30);
}

#[test]
fn test_choice_deserialization() {
    let json = r#"{
        "index": 0,
        "message": {
            "role": "assistant",
            "content": "Hello!"
        },
        "finish_reason": "stop"
    }"#;
    
    let choice: Choice = serde_json::from_str(json).unwrap();
    assert_eq!(choice.index, 0);
    assert_eq!(choice.message.role, Role::Assistant);
    assert_eq!(choice.message.content, "Hello!");
    assert_eq!(choice.finish_reason, Some("stop".to_string()));
}

#[test]
fn test_chat_response_deserialization() {
    let json = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello!"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 20,
            "total_tokens": 30
        }
    }"#;
    
    let response: ChatResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.id, "chatcmpl-123");
    assert_eq!(response.model, "gpt-4");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].message.content, "Hello!");
    assert!(response.usage.is_some());
    assert_eq!(response.usage.unwrap().total_tokens, 30);
}

#[test]
fn test_message_helper_functions() {
    let user_msg = Message::user("Hello");
    assert_eq!(user_msg.role, Role::User);
    assert_eq!(user_msg.content, "Hello");

    let assistant_msg = Message::assistant("Hi");
    assert_eq!(assistant_msg.role, Role::Assistant);
    assert_eq!(assistant_msg.content, "Hi");

    let system_msg = Message::system("You are helpful");
    assert_eq!(system_msg.role, Role::System);
    assert_eq!(system_msg.content, "You are helpful");
}

#[test]
fn test_chat_request_with_multiple_messages() {
    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
            Message::assistant("Hi"),
        ],
        ..Default::default()
    };

    assert_eq!(request.messages.len(), 3);
    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["messages"].as_array().unwrap().len(), 3);
}

#[test]
fn test_types_are_send() {
    fn assert_send<T: Send>() {}
    assert_send::<ChatRequest>();
    assert_send::<ChatResponse>();
    assert_send::<Message>();
    assert_send::<Role>();
}

#[test]
fn test_types_are_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<ChatRequest>();
    assert_sync::<ChatResponse>();
    assert_sync::<Message>();
    assert_sync::<Role>();
}

#[test]
fn test_types_are_clone() {
    let message = Message::user("test");
    let cloned = message.clone();
    assert_eq!(message.content, cloned.content);

    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::user("test")],
        ..Default::default()
    };
    let cloned_request = request.clone();
    assert_eq!(request.model, cloned_request.model);
}

#[test]
fn test_message_populate_reasoning_from_json() {
    let mut message = Message::assistant("result");
    let raw = serde_json::json!({
        "choices": [{
            "message": {
                "reasoning": "chain-of-thought",
                "thinking": "anthropic-thought",
                "thought": "o1-thought",
                "reasoning_content": "glm-reasoning"
            }
        }]
    });
    message.populate_reasoning_from_json(&raw);
    assert_eq!(message.reasoning.as_deref(), Some("chain-of-thought"));
    assert_eq!(message.thinking.as_deref(), Some("anthropic-thought"));
    assert_eq!(message.thought.as_deref(), Some("o1-thought"));
    assert_eq!(message.reasoning_content.as_deref(), Some("glm-reasoning"));
}

#[cfg(feature = "streaming")]
#[test]
fn test_delta_populate_reasoning_from_json() {
    let mut delta = llm_connector::types::Delta::default();
    let raw = serde_json::json!({
        "delta": {
            "thinking": "hidden",
            "reasoning": "explanation"
        }
    });
    delta.populate_reasoning_from_json(&raw);
    assert_eq!(delta.thinking.as_deref(), Some("hidden"));
    assert_eq!(delta.reasoning.as_deref(), Some("explanation"));
}

