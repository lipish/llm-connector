//! Common Request Assembly Logic

use {
    crate::{
        error::LlmConnectorError,
        types::{DocumentSource, ImageSource, Message, MessageBlock, Role},
    },
    serde_json::{Value, json, to_value},
};

/// Convert a MessageBlock to OpenAI-compatible JSON format.
/// Anthropic-format blocks (Image with Base64/Url source) are converted to
/// OpenAI-format image_url blocks with data URIs.
fn block_to_openai(block: &MessageBlock) -> Value {
    match block {
        MessageBlock::Text { text } => {
            json!({"type": "text", "text": text})
        }
        MessageBlock::Image { source } => match source {
            ImageSource::Base64 { media_type, data } => {
                let data_url = format!("data:{};base64,{}", media_type, data);
                json!({
                    "type": "image_url",
                    "image_url": {
                        "url": data_url
                    }
                })
            }
            ImageSource::Url { url } => {
                json!({
                    "type": "image_url",
                    "image_url": {
                        "url": url
                    }
                })
            }
        },
        MessageBlock::ImageUrl { image_url: _ } => to_value(block).unwrap_or_else(|_| json!(block)),
        MessageBlock::Document { source } => match source {
            DocumentSource::Base64 { media_type, data } => {
                json!({
                    "type": "text",
                    "text": format!("[Document: {} (base64, {} bytes)]", media_type, data.len())
                })
            }
        },
        MessageBlock::DocumentUrl { document_url } => {
            json!({
                "type": "text",
                "text": format!("[Document: {}]", document_url.url)
            })
        }
    }
}

/// Generic message converter for OpenAI-compatible protocols
pub fn openai_message_converter(messages: &[Message]) -> Vec<Value> {
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
                json!(msg.content[0].as_text().unwrap())
            } else {
                // Convert each block to OpenAI-compatible format
                let blocks = msg.content.iter().map(block_to_openai).collect::<Vec<_>>();
                json!(blocks)
            };

            let mut map = serde_json::Map::new();
            map.insert("role".to_string(), json!(role));
            map.insert("content".to_string(), content);

            if let Some(ref tc) = msg.tool_calls {
                map.insert("tool_calls".to_string(), json!(tc));
            }
            if let Some(ref id) = msg.tool_call_id {
                map.insert("tool_call_id".to_string(), json!(id));
            }
            if let Some(ref name) = msg.name {
                map.insert("name".to_string(), json!(name));
            }
            if let Some(ref rc) = msg.reasoning_content {
                map.insert("reasoning_content".to_string(), json!(rc));
            }

            Value::Object(map)
        })
        .collect()
}

/// Downgrade message content for providers that only support text content
pub fn openai_message_converter_downgrade(
    messages: &[Message],
) -> Result<Vec<Value>, LlmConnectorError> {
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

            let content = json!(content_str);

            let mut map = serde_json::Map::new();
            map.insert("role".to_string(), json!(role));
            map.insert("content".to_string(), content);

            if let Some(ref tc) = msg.tool_calls {
                map.insert("tool_calls".to_string(), json!(tc));
            }
            if let Some(ref id) = msg.tool_call_id {
                map.insert("tool_call_id".to_string(), json!(id));
            }
            if let Some(ref name) = msg.name {
                map.insert("name".to_string(), json!(name));
            }
            if let Some(ref rc) = msg.reasoning_content {
                map.insert("reasoning_content".to_string(), json!(rc));
            }

            Ok(Value::Object(map))
        })
        .collect()
}
