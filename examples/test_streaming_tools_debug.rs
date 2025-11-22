use llm_connector::{LlmClient, types::{ChatRequest, Message, Role, Tool, Function}};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 Moonshot，它支持 OpenAI 兼容的 tool_calls
    let api_key = "sk-5ipahcLR7y73YfOE5Tlkq39cpcIIcbLcOKlI7G69x7DtVw4b";

    // 使用 Moonshot
    let client = LlmClient::moonshot(api_key)?;
    let model = "moonshot-v1-8k";
    
    // 定义工具
    let tools = vec![Tool {
        tool_type: "function".to_string(),
        function: Function {
            name: "get_weather".to_string(),
            description: Some("Get the current weather in a given location".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city name, e.g. San Francisco"
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
    
    println!("=== 测试流式 tool_calls 是否重复 ===\n");
    
    #[cfg(feature = "streaming")]
    {
        use futures_util::StreamExt;
        
        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![Message::text(Role::User, "What's the weather like in Beijing?")],
            tools: Some(tools),
            stream: Some(true),
            ..Default::default()
        };
        
        println!("📤 发送流式请求...\n");
        
        match client.chat_stream(&request).await {
            Ok(mut stream) => {
                println!("📥 流式响应 chunks:\n");
                
                let mut chunk_count = 0;
                let mut tool_calls_appearances = Vec::new();
                
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(chunk) => {
                            chunk_count += 1;
                            
                            if let Some(choice) = chunk.choices.first() {
                                if let Some(tool_calls) = &choice.delta.tool_calls {
                                    println!("--- Chunk #{} ---", chunk_count);
                                    println!("  ⚠️  发现 tool_calls! 数量: {}", tool_calls.len());
                                    
                                    for (i, call) in tool_calls.iter().enumerate() {
                                        println!("    [{}] Tool Call:", i);
                                        println!("        id: {}", call.id);
                                        println!("        type: {}", call.call_type);
                                        println!("        function.name: {}", call.function.name);
                                        println!("        function.arguments: {}", call.function.arguments);
                                        
                                        tool_calls_appearances.push((chunk_count, call.clone()));
                                    }
                                    println!();
                                }
                                
                                if choice.finish_reason.is_some() {
                                    println!("--- Chunk #{} (Final) ---", chunk_count);
                                    println!("  finish_reason: {:?}", choice.finish_reason);
                                    println!();
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Chunk 解析错误: {}", e);
                        }
                    }
                }
                
                println!("\n📊 统计分析:");
                println!("  - 总 chunk 数: {}", chunk_count);
                println!("  - tool_calls 出现次数: {}", tool_calls_appearances.len());
                
                if tool_calls_appearances.is_empty() {
                    println!("\n⚠️  没有检测到 tool_calls");
                } else if tool_calls_appearances.len() == 1 {
                    println!("\n✅ tool_calls 只出现一次（正常）");
                } else {
                    println!("\n⚠️  警告: tool_calls 在多个 chunk 中出现!");
                    println!("\n详细信息:");
                    
                    // 按 tool_call.id 分组
                    use std::collections::HashMap;
                    let mut by_id: HashMap<String, Vec<usize>> = HashMap::new();
                    
                    for (chunk_num, call) in &tool_calls_appearances {
                        by_id.entry(call.id.clone())
                            .or_insert_with(Vec::new)
                            .push(*chunk_num);
                    }
                    
                    for (id, chunks) in &by_id {
                        if chunks.len() > 1 {
                            println!("  - Tool Call ID '{}' 出现在 {} 个 chunks: {:?}", 
                                id, chunks.len(), chunks);
                            println!("    ❌ 这会导致重复执行!");
                        } else {
                            println!("  - Tool Call ID '{}' 出现在 chunk {}", id, chunks[0]);
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ 流式请求失败: {}", e);
            }
        }
    }
    
    #[cfg(not(feature = "streaming"))]
    {
        println!("⚠️  需要启用 streaming feature 才能测试");
    }
    
    Ok(())
}

