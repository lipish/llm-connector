//! Test LongCat with OpenAI-compatible and Anthropic-compatible endpoints.
//!
//! Usage:
//! - Set `LONGCAT_API_KEY` in environment.
//! - Optional: override models via `LONGCAT_OPENAI_MODEL` and `LONGCAT_ANTHROPIC_MODEL`.
//! - Run: `cargo run --example longcat_dual`

use llm_connector::{LlmClient, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("LONGCAT_API_KEY")
        .expect("Please set LONGCAT_API_KEY environment variable");

    let openai_model = std::env::var("LONGCAT_OPENAI_MODEL")
        .unwrap_or_else(|_| "gpt-4o-mini".to_string());
    let anthropic_model = std::env::var("LONGCAT_ANTHROPIC_MODEL")
        .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());

    // OpenAI-compatible LongCat client
    let lc_openai = LlmClient::longcat(&api_key, false);
    let req_openai = ChatRequest::new(openai_model.clone())
        .with_messages(vec![Message::user("longcat")])
        .with_max_tokens(64);
    let resp_openai = lc_openai.chat(&req_openai).await?;
    println!(
        "[LongCat OpenAI] model={} -> {}",
        resp_openai.model,
        resp_openai.choices.first().map(|c| c.message.content.clone()).unwrap_or_default()
    );

    // Anthropic-compatible LongCat client
    let lc_anthropic = LlmClient::longcat(&api_key, true);
    let req_anthropic = ChatRequest::new(anthropic_model.clone())
        .with_messages(vec![Message::user("longcat")])
        .with_max_tokens(64);
    let resp_anthropic = lc_anthropic.chat(&req_anthropic).await?;
    println!(
        "[LongCat Anthropic] model={} -> {}",
        resp_anthropic.model,
        resp_anthropic.choices.first().map(|c| c.message.content.clone()).unwrap_or_default()
    );

    Ok(())
}