//! Integration test for wishlist features: tool calling, structured output, error types, usage
use llm_connector::{ChatRequest, LlmClient, Message, ResponseFormat, Tool, ToolChoice};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .expect("Please set DEEPSEEK_API_KEY environment variable");
    let client = LlmClient::deepseek(&api_key)?;
    let model = "deepseek-chat";

    // ========== 1. Tool Calling ==========
    println!("=== Test 1: Tool Calling ===");
    let tools = vec![Tool::function(
        "get_weather",
        Some("Get the current weather for a location".to_string()),
        serde_json::json!({
            "type": "object",
            "properties": {
                "location": { "type": "string", "description": "City name" },
                "unit": { "type": "string", "enum": ["celsius", "fahrenheit"] }
            },
            "required": ["location"]
        }),
    )];

    let request = ChatRequest::new(model)
        .add_message(Message::user("What's the weather in Beijing?"))
        .with_tools(tools)
        .with_tool_choice(ToolChoice::auto());

    let response = client.chat(&request).await?;
    println!("  has_tool_calls: {}", response.has_tool_calls());
    println!("  is_tool_call:   {}", response.is_tool_call());
    println!("  finish_reason:  {:?}", response.finish_reason());
    for tc in response.tool_calls() {
        println!(
            "  tool_call: id={}, fn={}, args={}",
            tc.id, tc.function.name, tc.function.arguments
        );
        let args: serde_json::Value = tc.parse_arguments()?;
        println!("  parsed args: {:?}", args);
    }

    // ========== 2. Structured Output (JSON mode) ==========
    println!("\n=== Test 2: Structured Output (json_object) ===");
    let request = ChatRequest::new(model)
        .add_message(Message::system(
            "You are a helpful assistant that outputs JSON.",
        ))
        .add_message(Message::user(
            "List 3 programming languages with name and year. Output as JSON array.",
        ))
        .with_response_format(ResponseFormat::json_object());

    let response = client.chat(&request).await?;
    println!("  content: {}", response.content);

    // ========== 3. Token Usage ==========
    println!("\n=== Test 3: Token Usage ===");
    let (prompt, completion, total) = response.get_usage_safe();
    println!("  prompt_tokens:     {}", prompt);
    println!("  completion_tokens: {}", completion);
    println!("  total_tokens:      {}", total);

    // ========== 4. Error Type Detection ==========
    println!("\n=== Test 4: Error Type Detection ===");
    // Test with invalid model to trigger error
    let bad_request = ChatRequest::new("nonexistent-model-xyz").add_message(Message::user("Hello"));
    match client.chat(&bad_request).await {
        Ok(_) => println!("  (unexpected success)"),
        Err(e) => {
            println!("  error: {}", e);
            println!("  is_retryable:         {}", e.is_retryable());
            println!("  should_reduce_context: {}", e.should_reduce_context());
            println!("  is_auth_error:        {}", e.is_auth_error());
            println!("  is_rate_limited:      {}", e.is_rate_limited());
            println!("  status_code:          {}", e.status_code());
        }
    }

    println!("\n=== All tests completed ===");
    Ok(())
}
