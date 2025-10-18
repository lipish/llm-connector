/// 调试智谱流式响应中的 tool_calls 解析
/// 
/// 这个示例用于验证智谱 API 返回的 tool_calls 是否能被正确解析

#[cfg(feature = "streaming")]
use {
    futures_util::StreamExt,
    llm_connector::{LlmClient, types::{ChatRequest, Message, Role, Tool, Function}},
    serde_json::json,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 需要启用 'streaming' 功能");
        println!("运行: cargo run --example debug_zhipu_streaming_tool_calls --features streaming");
        return Ok(());
    }
    
    #[cfg(feature = "streaming")]
    {
        let api_key = std::env::var("ZHIPU_API_KEY")
            .expect("请设置环境变量 ZHIPU_API_KEY");
        
        let client = LlmClient::zhipu(&api_key)?;
        
        // 定义工具
        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "list_files".to_string(),
                description: Some("列出指定目录下的文件".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "目录路径"
                        }
                    },
                    "required": ["path"]
                }),
            },
        }];
        
        println!("🔍 调试智谱流式响应中的 tool_calls 解析");
        println!("{}", "=".repeat(80));
        
        let request = ChatRequest {
            model: "glm-4.6".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: "请列出当前目录的文件".to_string(),
                ..Default::default()
            }],
            tools: Some(tools),
            stream: Some(true),
            ..Default::default()
        };
        
        println!("\n📤 发送请求...");
        println!("   Model: glm-4.6");
        println!("   Stream: true");
        println!("   Tools: 1 个");
        
        let mut stream = client.chat_stream(&request).await?;
        let mut chunk_count = 0;
        let mut has_tool_calls = false;
        let mut tool_calls_chunks = Vec::new();
        
        println!("\n📥 接收流式响应:");
        println!("{}", "-".repeat(80));
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(response) => {
                    chunk_count += 1;
                    
                    // 检查是否有 tool_calls
                    if let Some(choice) = response.choices.first() {
                        if let Some(ref tool_calls) = choice.delta.tool_calls {
                            has_tool_calls = true;
                            tool_calls_chunks.push((chunk_count, tool_calls.clone()));
                            
                            println!("\n✅ [Chunk {}] 发现 tool_calls!", chunk_count);
                            println!("   tool_calls 数量: {}", tool_calls.len());
                            
                            for (i, call) in tool_calls.iter().enumerate() {
                                println!("   [{}] id: {}", i, call.id);
                                println!("       type: {}", call.call_type);
                                println!("       function.name: {}", call.function.name);
                                println!("       function.arguments: {}", call.function.arguments);
                            }
                        }
                        
                        // 检查 delta.content
                        if let Some(ref content) = choice.delta.content {
                            if !content.is_empty() {
                                println!("[Chunk {}] content: {:?}", chunk_count, content);
                            }
                        }
                        
                        // 检查 finish_reason
                        if let Some(ref reason) = choice.finish_reason {
                            println!("\n🏁 [Chunk {}] finish_reason: {}", chunk_count, reason);
                            break;
                        }
                    }
                }
                Err(e) => {
                    println!("\n❌ 错误: {}", e);
                    break;
                }
            }
        }
        
        println!("\n{}", "=".repeat(80));
        println!("📊 统计信息:");
        println!("   总流式块数: {}", chunk_count);
        println!("   包含 tool_calls 的块数: {}", tool_calls_chunks.len());
        println!("   是否检测到 tool_calls: {}", if has_tool_calls { "✅ 是" } else { "❌ 否" });
        
        if !tool_calls_chunks.is_empty() {
            println!("\n📋 tool_calls 详细信息:");
            for (chunk_num, calls) in &tool_calls_chunks {
                println!("   Chunk {}: {} 个 tool_calls", chunk_num, calls.len());
                for call in calls {
                    println!("      - {}: {}", call.function.name, call.function.arguments);
                }
            }
        } else {
            println!("\n⚠️  警告: 没有检测到任何 tool_calls!");
            println!("   这可能表示:");
            println!("   1. 模型没有调用工具（不太可能，因为我们明确要求列出文件）");
            println!("   2. llm-connector 没有正确解析 tool_calls（需要修复）");
        }
        
        println!("\n{}", "=".repeat(80));
        
        // 如果没有检测到 tool_calls，建议用户检查原始响应
        if !has_tool_calls {
            println!("\n💡 建议:");
            println!("   1. 运行 tests/test_zhipu_streaming_direct.sh 查看原始 API 响应");
            println!("   2. 检查 llm-connector 的流式解析代码");
            println!("   3. 确认 Delta 结构体的 tool_calls 字段定义");
        }
    }
    
    Ok(())
}

