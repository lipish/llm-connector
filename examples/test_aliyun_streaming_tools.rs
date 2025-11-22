use llm_connector::{LlmClient, types::{ChatRequest, Message, Role, Tool, Function, ToolChoice}};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 启用调试输出
    std::env::set_var("LLM_DEBUG_REQUEST_RAW", "1");
    std::env::set_var("LLM_DEBUG_RESPONSE_RAW", "1");
    std::env::set_var("LLM_DEBUG_STREAM_RAW", "1");
    
    let api_key = "sk-17cb8a1feec2440bad2c5a73d7d08af2";
    
    let client = LlmClient::aliyun(api_key)?;
    
    // Define tools
    let tools = vec![Tool {
        tool_type: "function".to_string(),
        function: Function {
            name: "get_weather".to_string(),
            description: Some("Get weather information for a city".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "City name, e.g. Beijing, Shanghai"
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
    }];
    
    println!("=== Test 1: Non-streaming tool_calls ===\n");

    let request = ChatRequest {
        model: "qwen-coder-plus".to_string(),
        messages: vec![Message::text(Role::User, "What's the weather in Beijing?")],
        tools: Some(tools.clone()),
        tool_choice: Some(ToolChoice::required()),
        stream: Some(false),
        ..Default::default()
    };

    println!("📤 Sending non-streaming request...\n");

    match client.chat(&request).await {
        Ok(response) => {
            println!("\n📥 Non-streaming response:");
            println!("  - content: {}", response.content);
            println!("  - finish_reason: {:?}", response.choices.first().and_then(|c| c.finish_reason.as_ref()));

            if let Some(choice) = response.choices.first() {
                if let Some(tool_calls) = &choice.message.tool_calls {
                    println!("\n✅ Triggered {} tool calls:", tool_calls.len());
                    for (i, call) in tool_calls.iter().enumerate() {
                        println!("\n  Tool call #{}:", i + 1);
                        println!("    - ID: {}", call.id);
                        println!("    - Type: {}", call.call_type);
                        println!("    - Function: {}", call.function.name);
                        println!("    - Arguments: {}", call.function.arguments);
                    }
                } else {
                    println!("\n⚠️  No tool calls triggered");
                }
            }
        }
        Err(e) => {
            println!("❌ Non-streaming request failed: {}", e);
        }
    }

    println!("\n\n=== Test 2: Streaming tool_calls ===\n");
    
    #[cfg(feature = "streaming")]
    {
        use futures_util::StreamExt;
        
        let request_stream = ChatRequest {
            model: "qwen-coder-plus".to_string(),
            messages: vec![Message::text(Role::User, "What's the weather in Shanghai?")],
            tools: Some(tools),
            tool_choice: Some(ToolChoice::required()),
            stream: Some(true),
            ..Default::default()
        };

        println!("📤 Sending streaming request...\n");

        match client.chat_stream(&request_stream).await {
            Ok(mut stream) => {
                println!("📥 Streaming response chunks:\n");

                let mut chunk_count = 0;
                let mut tool_calls_count = 0;

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(chunk) => {
                            chunk_count += 1;

                            println!("--- Chunk #{} ---", chunk_count);
                            println!("  content: {:?}", chunk.content);

                            if let Some(choice) = chunk.choices.first() {
                                println!("  delta.content: {:?}", choice.delta.content);
                                println!("  delta.tool_calls: {:?}", choice.delta.tool_calls);
                                println!("  finish_reason: {:?}", choice.finish_reason);

                                if let Some(tool_calls) = &choice.delta.tool_calls {
                                    tool_calls_count += tool_calls.len();
                                    println!("  ⚠️  Found tool_calls! Count: {}", tool_calls.len());
                                    for (i, call) in tool_calls.iter().enumerate() {
                                        println!("    [{}] id={}, type={}, name={}, args={}",
                                            i, call.id, call.call_type, call.function.name, call.function.arguments);
                                    }
                                }
                            }
                            println!();
                        }
                        Err(e) => {
                            println!("❌ Chunk parse error: {}", e);
                        }
                    }
                }

                println!("\n📊 Statistics:");
                println!("  - Total chunks: {}", chunk_count);
                println!("  - Total tool_calls occurrences: {}", tool_calls_count);

                if tool_calls_count > 1 {
                    println!("\n⚠️  Warning: tool_calls appeared in multiple chunks, may cause duplicate execution!");
                }
            }
            Err(e) => {
                println!("❌ Streaming request failed: {}", e);
            }
        }
    }

    #[cfg(not(feature = "streaming"))]
    {
        println!("⚠️  Need to enable streaming feature to test streaming responses");
    }
    
    Ok(())
}

