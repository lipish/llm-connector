/// 测试智谱 GLM-4-Flash 流式响应中的 tool_calls
/// 
/// 对比原始 API 响应和 llm-connector 解析结果

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
        println!("运行: cargo run --example test_zhipu_flash_streaming_tool_calls --features streaming");
        return Ok(());
    }
    
    #[cfg(feature = "streaming")]
    {
        let api_key = std::env::var("ZHIPU_API_KEY")
            .expect("请设置环境变量 ZHIPU_API_KEY");
        
        let client = LlmClient::zhipu(&api_key)?;
        
        // 使用与测试脚本完全相同的配置
        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "list_files".to_string(),
                description: Some("List files in a directory".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    }
                }),
            },
        }];
        
        println!("🧪 测试智谱 GLM-4-Flash 流式 Tool Calls");
        println!("{}", "=".repeat(80));
        println!("\n📋 配置:");
        println!("   Model: glm-4-flash");
        println!("   Message: List files in the current directory");
        println!("   Tools: list_files");
        println!("   Stream: true");
        
        let request = ChatRequest {
            model: "glm-4-flash".to_string(),
            messages: vec![Message::text(Role::User, "List files in the current directory")],
            tools: Some(tools),
            stream: Some(true),
            ..Default::default()
        };
        
        println!("\n📤 发送请求...");
        
        let mut stream = client.chat_stream(&request).await?;
        let mut chunk_count = 0;
        let mut chunks_with_tool_calls = 0;
        let mut all_tool_calls = Vec::new();
        
        println!("\n📥 接收流式响应:");
        println!("{}", "-".repeat(80));
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(response) => {
                    chunk_count += 1;
                    
                    println!("\n[Chunk {}]", chunk_count);
                    println!("  id: {}", response.id);
                    println!("  model: {}", response.model);
                    println!("  choices: {}", response.choices.len());
                    
                    if let Some(choice) = response.choices.first() {
                        println!("  choice[0].index: {}", choice.index);
                        
                        // 检查 delta.role
                        if let Some(ref role) = choice.delta.role {
                            println!("  choice[0].delta.role: {:?}", role);
                        }
                        
                        // 检查 delta.content
                        if let Some(ref content) = choice.delta.content {
                            println!("  choice[0].delta.content: {:?}", content);
                        }
                        
                        // 检查 delta.tool_calls
                        if let Some(ref tool_calls) = choice.delta.tool_calls {
                            chunks_with_tool_calls += 1;
                            println!("  choice[0].delta.tool_calls: {} 个", tool_calls.len());
                            
                            for (i, call) in tool_calls.iter().enumerate() {
                                println!("    [{}] id: {}", i, call.id);
                                println!("        type: {}", call.call_type);
                                println!("        function.name: {}", call.function.name);
                                println!("        function.arguments: {}", call.function.arguments);
                                
                                all_tool_calls.push(call.clone());
                            }
                        }
                        
                        // 检查 finish_reason
                        if let Some(ref reason) = choice.finish_reason {
                            println!("  choice[0].finish_reason: {}", reason);
                        }
                    }
                    
                    // 检查 usage
                    if let Some(ref usage) = response.usage {
                        println!("  usage:");
                        println!("    prompt_tokens: {}", usage.prompt_tokens);
                        println!("    completion_tokens: {}", usage.completion_tokens);
                        println!("    total_tokens: {}", usage.total_tokens);
                    }
                }
                Err(e) => {
                    println!("\n❌ 错误: {}", e);
                    break;
                }
            }
        }
        
        println!("\n{}", "=".repeat(80));
        println!("📊 统计:");
        println!("   总流式块数: {}", chunk_count);
        println!("   包含 tool_calls 的块数: {}", chunks_with_tool_calls);
        println!("   收集到的 tool_calls 总数: {}", all_tool_calls.len());
        
        println!("\n{}", "=".repeat(80));
        println!("🔍 对比分析:");
        println!("{}", "=".repeat(80));
        
        println!("\n原始 API 响应 (来自 tests/test_zhipu_streaming_direct.sh):");
        println!("  - 返回 2 个流式块 + [DONE]");
        println!("  - 第 1 块包含 tool_calls");
        println!("  - 第 2 块包含 usage 和空 content");
        
        println!("\nllm-connector 解析结果:");
        println!("  - 解析了 {} 个流式块", chunk_count);
        println!("  - {} 个块包含 tool_calls", chunks_with_tool_calls);
        println!("  - 收集到 {} 个 tool_calls", all_tool_calls.len());
        
        if all_tool_calls.is_empty() {
            println!("\n❌ 问题确认: llm-connector 没有解析到 tool_calls!");
            println!("   虽然原始 API 返回了 tool_calls，但 llm-connector 没有正确传递。");
        } else {
            println!("\n✅ llm-connector 正确解析了 tool_calls:");
            for (i, call) in all_tool_calls.iter().enumerate() {
                println!("   [{}] {}: {}", i, call.function.name, call.function.arguments);
            }
        }
        
        println!("\n{}", "=".repeat(80));
    }
    
    Ok(())
}

