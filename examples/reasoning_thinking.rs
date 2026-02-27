use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message, ReasoningEffort},
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Claude 3.7 Sonnet (Thinking Mode)
    // Note: Requires ANTHROPIC_API_KEY environment variable
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        println!("--- Claude 3.7 Thinking Mode ---");
        let client = LlmClient::anthropic(&key)?;

        let request = ChatRequest::new("claude-3-7-sonnet-20250219")
            .add_message(Message::user("How many R's are in the word Strawberry?"))
            .with_thinking_budget(2048) // Enable thinking with 2k tokens
            .with_max_tokens(4096); // Total tokens > thinking budget

        let response = client.chat(&request).await?;

        // Show thinking process if available
        if let Some(thinking) = response.reasoning_content {
            println!("\n[Thinking Process]:\n{}", thinking);
        }
        println!("\n[Final Answer]:\n{}", response.content);
    }

    // 2. OpenAI o1/o3 (Reasoning Effort)
    // Note: Requires OPENAI_API_KEY environment variable
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        println!("\n--- OpenAI o3-mini Reasoning ---");
        let client = LlmClient::openai(&key)?;

        let request = ChatRequest::new("o3-mini")
            .add_message(Message::user("Solve this logic puzzle..."))
            .with_reasoning_effort(ReasoningEffort::Medium); // Set effort level

        let response = client.chat(&request).await?;
        println!("\n[Answer]:\n{}", response.content);
    }

    Ok(())
}
