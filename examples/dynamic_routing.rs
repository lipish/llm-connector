use llm_connector::core::{EnvVarResolver, ServiceResolver};
use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message},
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Dynamic Service Resolution ---");

    // 1. Setup Resolver
    let resolver = EnvVarResolver::new()
        .with_mapping("gpt", "OPENAI_API_KEY")
        .with_mapping("claude", "ANTHROPIC_API_KEY");

    // 2. Resolve target (simulated)
    let model_name = "claude-3-opus-20240229";
    let target = resolver.resolve(model_name).await?;

    println!("Resolved Target:");
    println!("  Model: {}", target.model);
    println!("  Key Found: {}", target.api_key.is_some());

    // 3. Use in request (Gateway Mode)
    // Create a generic client (or use connection pool)
    // We use a dummy key here because we'll override it per-request
    let client = LlmClient::openai("sk-placeholder")?;

    let mut request =
        ChatRequest::new(&target.model).add_message(Message::user("Hello via dynamic routing!"));

    // Apply overrides
    if let Some(key) = target.api_key {
        request = request.with_api_key(key);
    }
    if let Some(endpoint) = target.endpoint {
        request = request.with_base_url(endpoint);
    }

    // Send request (only if we have a real key)
    if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        let response = client.chat(&request).await?;
        println!("Response: {}", response.content);
    } else {
        println!("Skipping actual request (no API key in env)");
    }

    Ok(())
}
