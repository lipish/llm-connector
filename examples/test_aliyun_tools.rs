//! Test Aliyun DashScope tools support
//!
//! This example demonstrates tool calling with Aliyun DashScope API.
//! 
//! Run with:
//! ```bash
//! cargo run --example test_aliyun_tools --features streaming
//! ```

use llm_connector::{LlmClient, types::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Aliyun DashScope Tools Support\n");
    
    // Get API key from environment or use default
    let api_key = std::env::var("ALIYUN_API_KEY")
        .unwrap_or_else(|_| {
            println!("⚠️  ALIYUN_API_KEY not set, using placeholder");
            "your-api-key-here".to_string()
        });
    
    if api_key == "your-api-key-here" {
        println!("❌ Please set ALIYUN_API_KEY environment variable");
        println!("   export ALIYUN_API_KEY=your-actual-api-key");
        return Ok(());
    }
    
    // Test 1: Non-streaming with tools
    println!("📝 Test 1: Non-streaming tool call");
    println!("===================================");
    test_non_streaming_tools(&api_key).await?;
    
    println!("\n");
    
    // Test 2: Streaming with tools
    println!("📝 Test 2: Streaming tool call");
    println!("===============================");
    test_streaming_tools(&api_key).await?;
    
    Ok(())
}

async fn test_non_streaming_tools(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::aliyun(api_key)?;
    
    // Define a weather tool
    let tools = vec![
        Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "get_weather".to_string(),
                description: Some("Get current weather information for a city".to_string()),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "City name, e.g., Beijing, Shanghai"
                        },
                        "unit": {
                            "type": "string",
                            "enum": ["celsius", "fahrenheit"],
                            "description": "Temperature unit"
                        }
                    },
                    "required": ["location"]
                }),
            },
        },
    ];
    
    let request = ChatRequest {
        model: "qwen-plus".to_string(),
        messages: vec![
            Message::text(Role::User, "What's the weather like in Beijing today?")
        ],
        tools: Some(tools),
        tool_choice: Some(ToolChoice::Mode("auto".to_string())),
        max_tokens: Some(1000),
        ..Default::default()
    };
    
    println!("📤 Sending request with tools...");
    
    match client.chat(&request).await {
        Ok(response) => {
            println!("✅ Response received");
            println!("📊 Model: {}", response.model);
            println!("📊 Finish reason: {:?}", response.choices[0].finish_reason);
            
            // Check for tool calls
            if let Some(tool_calls) = &response.choices[0].message.tool_calls {
                println!("\n🔧 Tool Calls:");
                for (i, tool_call) in tool_calls.iter().enumerate() {
                    println!("  {}. ID: {}", i + 1, tool_call.id);
                    println!("     Function: {}", tool_call.function.name);
                    println!("     Arguments: {}", tool_call.function.arguments);
                }
            } else {
                println!("\n💬 Response: {}", response.content);
            }
            
            if let Some(usage) = &response.usage {
                println!("\n📊 Usage:");
                println!("   Prompt tokens: {}", usage.prompt_tokens);
                println!("   Completion tokens: {}", usage.completion_tokens);
                println!("   Total tokens: {}", usage.total_tokens);
            }
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

#[cfg(feature = "streaming")]
async fn test_streaming_tools(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    use futures_util::StreamExt;
    
    let client = LlmClient::aliyun(api_key)?;
    
    // Define a calculator tool
    let tools = vec![
        Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "calculate".to_string(),
                description: Some("Perform mathematical calculations".to_string()),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "Mathematical expression to evaluate"
                        }
                    },
                    "required": ["expression"]
                }),
            },
        },
    ];
    
    let request = ChatRequest {
        model: "qwen-plus".to_string(),
        messages: vec![
            Message::text(Role::User, "Calculate 123 * 456")
        ],
        tools: Some(tools),
        tool_choice: Some(ToolChoice::Mode("auto".to_string())),
        stream: Some(true),
        ..Default::default()
    };
    
    println!("📤 Sending streaming request with tools...");
    
    match client.chat_stream(&request).await {
        Ok(mut stream) => {
            println!("✅ Stream created");
            
            let mut chunk_count = 0;
            let mut has_tool_calls = false;
            
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        chunk_count += 1;
                        
                        if let Some(content) = chunk.get_content() {
                            print!("{}", content);
                        }
                        
                        if let Some(tool_calls) = &chunk.choices[0].delta.tool_calls {
                            has_tool_calls = true;
                            println!("\n🔧 Tool calls in chunk {}:", chunk_count);
                            for tool_call in tool_calls {
                                println!("   Function: {}", tool_call.function.name);
                                println!("   Arguments: {}", tool_call.function.arguments);
                            }
                        }
                        
                        if let Some(finish_reason) = &chunk.choices[0].finish_reason {
                            println!("\n📊 Finish reason: {}", finish_reason);
                        }
                    }
                    Err(e) => {
                        println!("\n❌ Stream error: {}", e);
                        return Err(e.into());
                    }
                }
            }
            
            println!("\n\n✅ Stream completed");
            println!("📊 Total chunks: {}", chunk_count);
            println!("📊 Has tool calls: {}", has_tool_calls);
        }
        Err(e) => {
            println!("❌ Failed to create stream: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

#[cfg(not(feature = "streaming"))]
async fn test_streaming_tools(_api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚠️  Streaming feature not enabled");
    println!("   Run with: cargo run --example test_aliyun_tools --features streaming");
    Ok(())
}

