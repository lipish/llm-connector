//! Common Request Assembly Logic

use crate::types::{Message, Role};
use crate::error::LlmConnectorError;

/// Generic message converter for OpenAI-compatible protocols
pub fn openai_message_converter(messages: &[Message]) -> Vec<serde_json::Value> {
    messages
        .iter()
        .map(|msg| {
            let role = match msg.role {
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::System => "system",
                Role::Tool => "tool",
            };

            let content = if msg.content.len() == 1 && msg.content[0].is_text() {
                serde_json::json!(msg.content[0].as_text().unwrap())
            } else {
                serde_json::json!(msg.content)
            };

            let mut map = serde_json::Map::new();
            map.insert("role".to_string(), serde_json::json!(role));
            map.insert("content".to_string(), content);

            if let Some(ref tc) = msg.tool_calls {
                map.insert("tool_calls".to_string(), serde_json::json!(tc));
            }
            if let Some(ref id) = msg.tool_call_id {
                map.insert("tool_call_id".to_string(), serde_json::json!(id));
            }
            if let Some(ref name) = msg.name {
                map.insert("name".to_string(), serde_json::json!(name));
            }
            if let Some(ref rc) = msg.reasoning_content {
                map.insert("reasoning_content".to_string(), serde_json::json!(rc));
            }

            serde_json::Value::Object(map)
        })
        .collect()
}

/// Downgrade message content for providers that only support text content
pub fn openai_message_converter_downgrade(messages: &[Message]) -> Result<Vec<serde_json::Value>, LlmConnectorError> {
    messages
        .iter()
        .map(|msg| {
            let role = match msg.role {
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::System => "system",
                Role::Tool => "tool",
            };

            // Downgrade content logic
            let content_str = if msg.content.is_empty() {
                String::new()
            } else {
                let mut text_content = String::new();
                for block in &msg.content {
                    if let Some(text) = block.as_text() {
                        text_content.push_str(text);
                    } else {
                        // Found non-text block (e.g. image), but provider only supports text
                        return Err(LlmConnectorError::InvalidRequest(format!(
                            "Provider does not support complex content blocks (found {:?})",
                            block
                        )));
                    }
                }
                text_content
            };

            let content = serde_json::json!(content_str);

            let mut map = serde_json::Map::new();
            map.insert("role".to_string(), serde_json::json!(role));
            map.insert("content".to_string(), content);

            if let Some(ref tc) = msg.tool_calls {
                map.insert("tool_calls".to_string(), serde_json::json!(tc));
            }
            if let Some(ref id) = msg.tool_call_id {
                map.insert("tool_call_id".to_string(), serde_json::json!(id));
            }
            if let Some(ref name) = msg.name {
                map.insert("name".to_string(), serde_json::json!(name));
            }
            if let Some(ref rc) = msg.reasoning_content {
                map.insert("reasoning_content".to_string(), serde_json::json!(rc));
            }

            Ok(serde_json::Value::Object(map))
        })
        .collect()
}
