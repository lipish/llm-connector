//! Test LongCat (Anthropic protocol) using keys.yaml with only API key.
//!
//! Requirements:
//! - `keys.yaml` contains just the API key (no protocol/base_url/models).
//!   Supported minimal layouts:
//!   1) providers.longcat.api_key: "..."
//!   2) longcat.api_key: "..."
//!   3) api_key: "..." (top level)
//! - Model is hardcoded to `LongCat-Flash-Chat`.
//!
//! Run: `cargo run --example longcat_anthropic`

use llm_connector::{LlmClient, ChatRequest, Message};
use serde_yaml::Value;

fn find_api_key(doc: &Value) -> Option<String> {
    // Try providers.longcat.api_key
    if let Some(v) = doc.get("providers").and_then(|p| p.get("longcat")).and_then(|lc| lc.get("api_key")) {
        if let Some(s) = v.as_str() { return Some(s.to_string()); }
    }
    // Try longcat.api_key
    if let Some(v) = doc.get("longcat").and_then(|lc| lc.get("api_key")) {
        if let Some(s) = v.as_str() { return Some(s.to_string()); }
    }
    // Try top-level api_key
    if let Some(v) = doc.get("api_key") {
        if let Some(s) = v.as_str() { return Some(s.to_string()); }
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load keys.yaml (minimal schema)
    let keys_content = std::fs::read_to_string("keys.yaml")
        .expect("Please provide keys.yaml in the project root");
    let doc: Value = serde_yaml::from_str(&keys_content)
        .expect("Failed to parse keys.yaml");

    // Extract API key
    let api_key = find_api_key(&doc).expect("keys.yaml missing api_key for longcat");

    // Use LongCat's own model (Anthropic-compatible protocol)
    let model = "LongCat-Flash-Chat".to_string();

    println!("📡 LongCat (Anthropic protocol) - minimal keys.yaml");
    println!("   Using model: {}", model);

    // Create LongCat client with Anthropic-compatible endpoint
    let client = LlmClient::longcat(&api_key, true);

    // Build request (Anthropic requires max_tokens)
    let prompt = "Say 'Hello' in one word";
    println!("🗣️ Prompt: {}", prompt);

    let request = ChatRequest::new(model)
        .with_messages(vec![Message::user(prompt)])
        .with_max_tokens(64);

    match client.chat(&request).await {
        Ok(response) => {
            println!("✅ Chat succeeded");
            if let Some(choice) = response.choices.first() {
                println!("💬 Response: {}", choice.message.content);
            }
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }

    Ok(())
}