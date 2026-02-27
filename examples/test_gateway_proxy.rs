use futures::StreamExt;
use llm_connector::{
    LlmClient,
    types::{ChatRequest, Function, Message, ReasoningEffort, ResponseFormat, Tool},
};
use serde_json::json;
use std::env;
use std::error::Error;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load .env file manually if present
    if let Ok(content) = std::fs::read_to_string(".env") {
        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');
                if std::env::var(key).is_err() {
                    // Safe to set env var in single-threaded test context
                    unsafe {
                        std::env::set_var(key, value);
                    }
                }
            }
        }
    }

    // 1. Configuration
    // Use the URL from the image as default, but allow override
    let base_url = env::var("GATEWAY_BASE_URL")
        .unwrap_or_else(|_| "http://123.129.219.111:3000/v1".to_string());

    let api_key =
        env::var("GATEWAY_API_KEY").expect("Please set GATEWAY_API_KEY environment variable");

    println!("--- Gateway Test Configuration ---");
    println!("Base URL: {}", base_url);
    println!(
        "API Key: {}...",
        &api_key.chars().take(6).collect::<String>()
    );
    println!("--------------------------------\n");

    // Create client (using OpenAI compatible mode as the gateway exposes /v1/chat/completions)
    // Use builder to set a longer timeout (Claude can be slow)
    // Note: builder.openai_compatible takes (api_key, service_name), base_url is set separately
    let client = LlmClient::builder()
        .base_url(&base_url)
        .openai_compatible(&api_key, "gateway-proxy")
        .timeout(300) // Increase timeout to 300 seconds
        .build()?;

    // Test 1: OpenAI GPT-4o (Non-Reasoning)
    println!("--- Test 1: OpenAI GPT-4o (Standard) ---");
    let req_gpt4o =
        ChatRequest::new("gpt-4o").add_message(Message::user("Hello! Say this is a test."));

    match client.chat(&req_gpt4o).await {
        Ok(res) => {
            println!("✅ Success!");
            println!("Model: {}", res.model);
            println!("Content: {}", res.content);
        }
        Err(e) => println!("❌ Failed: {}", e),
    }
    println!();

    // Test 2: Claude 3.5 Sonnet (Standard) via Gateway
    // Using the model name from the image list
    println!("--- Test 2: Claude 3.5 Sonnet (Standard) ---");
    let req_claude = ChatRequest::new("claude-sonnet-4-5-20250929")
        .add_message(Message::user("What is the capital of France?"));

    match client.chat(&req_claude).await {
        Ok(res) => {
            println!("✅ Success!");
            println!("Model: {}", res.model);
            println!("Content: {}", res.content);
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
            // Print error details if available
            if let llm_connector::error::LlmConnectorError::ApiError(msg) = &e {
                println!("Details: {}", msg);
            }
        }
    }
    println!();

    // Test 3: Claude 3.5 Sonnet (Non-Streaming, Long Timeout)
    println!("--- Test 3: Claude 3.5 Sonnet (Non-Streaming) ---");
    // Use the name from the screenshot, but add reasoning_effort
    let req_claude_thinking = ChatRequest::new("claude-sonnet-4-5-20250929")
        .add_message(Message::user("How many R's are in Strawberry?"))
        .with_reasoning_effort(ReasoningEffort::Medium)
        .with_max_tokens(4096); // Ensure enough tokens for thinking

    println!("Sending request...");
    let start = std::time::Instant::now();

    match client.chat(&req_claude_thinking).await {
        Ok(res) => {
            let duration = start.elapsed();
            println!("✅ Success! (Took {:?})", duration);
            println!("Model: {}", res.model);

            // Check if reasoning content is present in standard response
            if let Some(reasoning) = res.reasoning_content {
                println!("[Thinking Process]:\n{}", reasoning);
            } else {
                println!("[No explicit reasoning content field returned]");
                // Check if it's maybe embedded in content?
                if res.content.contains("<thinking>") {
                    println!("[Thinking found in content]");
                }
            }
            println!("[Final Answer]:\n{}", res.content);
        }
        Err(e) => {
            let duration = start.elapsed();
            println!("❌ Failed after {:?}: {}", duration, e);
            if let llm_connector::error::LlmConnectorError::ApiError(msg) = &e {
                println!("Details: {}", msg);
            }
        }
    }

    // Test 4: Claude 3.5 Sonnet (Streaming + Thinking) via Gateway
    println!("--- Test 4: Claude 3.5 Sonnet (Streaming + Thinking) ---");
    let req_claude_stream = ChatRequest::new("claude-sonnet-4-5-20250929-thinking")
        .add_message(Message::user("How many R's are in Strawberry?"))
        .with_reasoning_effort(ReasoningEffort::Medium);

    match client.chat_stream(&req_claude_stream).await {
        Ok(stream) => {
            println!("✅ Connection Established!");
            print!("Stream Content: ");
            let mut stream = stream;
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        // Print thinking content if available
                        if let Some(thinking) = &chunk.reasoning_content {
                            print!("[Thinking: {}]", thinking);
                        }
                        print!("{}", chunk.content);
                        std::io::stdout().flush().unwrap();
                    }
                    Err(e) => {
                        println!("\n❌ Stream Error: {}", e);
                        break;
                    }
                }
            }
            println!("\nStream Finished.");
        }
        Err(e) => {
            println!("❌ Connection Failed: {}", e);
            if let llm_connector::error::LlmConnectorError::ApiError(msg) = &e {
                println!("Details: {}", msg);
            }
        }
    }

    // Test 5: OpenAI o3-mini (Reasoning Effort)
    println!("--- Test 5: OpenAI o3-mini (Reasoning Effort) ---");
    let req_o3 = ChatRequest::new("o3-mini")
        .add_message(Message::user(
            "Solve this: If A=5, B=A+2, what is A+B? Explain step by step.",
        ))
        .with_reasoning_effort(ReasoningEffort::High);

    match client.chat(&req_o3).await {
        Ok(res) => {
            println!("✅ Success!");
            println!("Model: {}", res.model);
            println!("Content: {}", res.content);
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
            if let llm_connector::error::LlmConnectorError::ApiError(msg) = &e {
                println!("Details: {}", msg);
            }
        }
    }
    println!();

    // Test 6: OpenAI Function Calling
    println!("--- Test 6: OpenAI Function Calling ---");
    let weather_tool = Tool {
        tool_type: "function".to_string(),
        function: Function {
            name: "get_weather".to_string(),
            description: Some("Get current weather".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                },
                "required": ["location"]
            }),
        },
    };

    let req_tool = ChatRequest::new("gpt-4o")
        .add_message(Message::user("What's the weather in Tokyo?"))
        .with_tools(vec![weather_tool]); // Use with_tools instead of add_tool

    match client.chat(&req_tool).await {
        Ok(res) => {
            println!("✅ Success!");
            let calls = res.tool_calls();
            if !calls.is_empty() {
                for call in calls {
                    println!(
                        "Tool Call: {} ({})",
                        call.function.name, call.function.arguments
                    );
                }
            } else {
                println!("⚠️ No tool calls returned (Unexpected)");
            }
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
            if let llm_connector::error::LlmConnectorError::ApiError(msg) = &e {
                println!("Details: {}", msg);
            }
        }
    }
    println!();

    // Test 7: OpenAI JSON Mode
    println!("--- Test 7: OpenAI JSON Mode ---");
    let req_json = ChatRequest::new("gpt-4o")
        .add_message(Message::system("You are a JSON generator."))
        .add_message(Message::user(
            "Generate a JSON object with keys 'name' and 'age' for a person.",
        ))
        .with_response_format(ResponseFormat::json_object());

    match client.chat(&req_json).await {
        Ok(res) => {
            println!("✅ Success!");
            println!("Content: {}", res.content);
            // Verify it's valid JSON
            if serde_json::from_str::<serde_json::Value>(&res.content).is_ok() {
                println!("✅ Valid JSON parsed");
            } else {
                println!("❌ Invalid JSON");
            }
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
            if let llm_connector::error::LlmConnectorError::ApiError(msg) = &e {
                println!("Details: {}", msg);
            }
        }
    }

    Ok(())
}
