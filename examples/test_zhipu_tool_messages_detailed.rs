/// 详细测试智谱 GLM 在包含 Role::Tool 消息时的流式响应
/// 这个测试会输出详细的请求和响应信息，帮助诊断问题

#[cfg(feature = "streaming")]
use futures_util::StreamExt;
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role, Tool, Function}};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 需要启用 'streaming' 功能");
        println!("运行: cargo run --example test_zhipu_tool_messages_detailed --features streaming");
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
                name: "get_weather".to_string(),
                description: Some("获取指定城市的天气信息".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "城市名称，例如：北京、上海"
                        }
                    },
                    "required": ["location"]
                }),
            },
        }];
        
        println!("🔍 详细测试智谱 GLM 流式响应 - Role::Tool 消息场景");
        println!("{}", "=".repeat(80));

        let model = "glm-4.6";
        println!("\n📝 测试模型: {}", model);
        
        // ========================================================================
        // 场景 1: 第一轮请求（触发工具调用）
        // ========================================================================
        println!("\n{}", "─".repeat(80));
        println!("✅ 场景 1: 第一轮请求（无 Role::Tool 消息）");
        println!("{}", "─".repeat(80));
        
        let request1 = ChatRequest {
            model: model.to_string(),
            messages: vec![Message {
                role: Role::User,
                content: "上海的天气怎么样？".to_string(),
                ..Default::default()
            }],
            tools: Some(tools.clone()),
            stream: Some(true),
            ..Default::default()
        };
        
        println!("\n📤 请求信息:");
        println!("   messages count: {}", request1.messages.len());
        for (i, msg) in request1.messages.iter().enumerate() {
            println!("   [{}] role: {:?}, content: {:?}", i, msg.role, 
                msg.content.chars().take(50).collect::<String>());
        }
        println!("   stream: {:?}", request1.stream);
        println!("   tools: {} 个", request1.tools.as_ref().map(|t| t.len()).unwrap_or(0));
        
        let mut stream1 = client.chat_stream(&request1).await?;
        let mut chunk_count1 = 0;
        let mut content1 = String::new();
        let mut has_tool_calls = false;
        let mut tool_call_id = String::new();
        let mut tool_call_args = String::new();
        
        println!("\n📥 流式响应:");
        while let Some(chunk) = stream1.next().await {
            match chunk {
                Ok(response) => {
                    chunk_count1 += 1;
                    
                    // 详细输出每个 chunk
                    if chunk_count1 <= 5 || chunk_count1 % 10 == 0 {
                        println!("   [Chunk {}] id: {}, choices: {}", 
                            chunk_count1, 
                            response.id,
                            response.choices.len()
                        );
                    }
                    
                    if let Some(delta_content) = response.get_content() {
                        content1.push_str(delta_content);
                        if chunk_count1 <= 3 {
                            println!("      content: {:?}", delta_content);
                        }
                    }
                    
                    // 检查工具调用
                    if let Some(choice) = response.choices.first() {
                        if let Some(tool_calls) = &choice.delta.tool_calls {
                            has_tool_calls = true;
                            if let Some(call) = tool_calls.first() {
                                if !call.id.is_empty() {
                                    tool_call_id = call.id.clone();
                                    println!("      tool_call_id: {}", tool_call_id);
                                }
                                if !call.function.arguments.is_empty() {
                                    tool_call_args.push_str(&call.function.arguments);
                                }
                            }
                        }
                        
                        if let Some(reason) = &choice.finish_reason {
                            println!("   [Chunk {}] finish_reason: {}", chunk_count1, reason);
                            break;
                        }
                    }
                }
                Err(e) => {
                    println!("   ❌ 错误: {}", e);
                    break;
                }
            }
        }
        
        println!("\n📊 场景 1 统计:");
        println!("   总流式块数: {}", chunk_count1);
        println!("   内容长度: {} 字符", content1.len());
        println!("   有工具调用: {}", has_tool_calls);
        println!("   tool_call_id: {}", tool_call_id);
        println!("   tool_call_args: {}", tool_call_args);
        
        if !has_tool_calls || tool_call_id.is_empty() {
            println!("\n⚠️  第一轮未触发工具调用，无法继续测试场景 2");
            return Ok(());
        }
        
        // ========================================================================
        // 场景 2: 第二轮请求（包含 Role::Tool 消息）
        // ========================================================================
        println!("\n{}", "─".repeat(80));
        println!("⚠️  场景 2: 第二轮请求（包含 Role::Tool 消息）");
        println!("{}", "─".repeat(80));
        
        let request2 = ChatRequest {
            model: model.to_string(),
            messages: vec![
                Message {
                    role: Role::User,
                    content: "上海的天气怎么样？".to_string(),
                    ..Default::default()
                },
                Message {
                    role: Role::Assistant,
                    content: String::new(),
                    tool_calls: Some(vec![llm_connector::types::ToolCall {
                        id: tool_call_id.clone(),
                        call_type: "function".to_string(),
                        function: llm_connector::types::FunctionCall {
                            name: "get_weather".to_string(),
                            arguments: r#"{"location":"上海"}"#.to_string(),
                        },
                    }]),
                    ..Default::default()
                },
                Message {
                    role: Role::Tool,
                    content: r#"{"temperature": 22, "condition": "晴天", "humidity": 65}"#.to_string(),
                    tool_call_id: Some(tool_call_id.clone()),
                    name: Some("get_weather".to_string()),
                    ..Default::default()
                },
            ],
            tools: Some(tools.clone()),
            stream: Some(true),
            ..Default::default()
        };
        
        println!("\n📤 请求信息:");
        println!("   messages count: {}", request2.messages.len());
        for (i, msg) in request2.messages.iter().enumerate() {
            println!("   [{}] role: {:?}, content_len: {}, tool_calls: {}, tool_call_id: {:?}", 
                i, 
                msg.role, 
                msg.content.len(),
                msg.tool_calls.as_ref().map(|t| t.len()).unwrap_or(0),
                msg.tool_call_id
            );
        }
        println!("   stream: {:?}", request2.stream);
        println!("   tools: {} 个", request2.tools.as_ref().map(|t| t.len()).unwrap_or(0));
        
        let mut stream2 = client.chat_stream(&request2).await?;
        let mut chunk_count2 = 0;
        let mut content2 = String::new();
        let mut empty_chunks = 0;
        
        println!("\n📥 流式响应:");
        while let Some(chunk) = stream2.next().await {
            match chunk {
                Ok(response) => {
                    chunk_count2 += 1;
                    
                    // 详细输出每个 chunk
                    if chunk_count2 <= 10 || chunk_count2 % 10 == 0 {
                        println!("   [Chunk {}] id: {}, choices: {}", 
                            chunk_count2, 
                            response.id,
                            response.choices.len()
                        );
                    }
                    
                    if let Some(delta_content) = response.get_content() {
                        if delta_content.is_empty() {
                            empty_chunks += 1;
                            if chunk_count2 <= 5 {
                                println!("      ⚠️  Chunk has no content");
                            }
                        } else {
                            content2.push_str(delta_content);
                            if chunk_count2 <= 10 {
                                println!("      content: {:?}", delta_content);
                            }
                        }
                    } else {
                        empty_chunks += 1;
                        if chunk_count2 <= 5 {
                            println!("      ⚠️  Chunk has no delta content");
                        }
                    }
                    
                    if let Some(choice) = response.choices.first() {
                        if let Some(reason) = &choice.finish_reason {
                            println!("   [Chunk {}] finish_reason: {}", chunk_count2, reason);
                            break;
                        }
                    }
                }
                Err(e) => {
                    println!("   ❌ 错误: {}", e);
                    break;
                }
            }
        }
        
        println!("\n📊 场景 2 统计:");
        println!("   总流式块数: {}", chunk_count2);
        println!("   空块数量: {}", empty_chunks);
        println!("   内容长度: {} 字符", content2.len());
        
        if !content2.is_empty() {
            println!("   内容预览: {}...", 
                content2.chars().take(100).collect::<String>());
        } else {
            println!("   ⚠️  内容为空！");
        }
        
        // ========================================================================
        // 对比分析
        // ========================================================================
        println!("\n{}", "=".repeat(80));
        println!("📊 对比分析:");
        println!("{}", "=".repeat(80));
        println!("   场景 1（无 Tool）: {} 块, {} 字符", chunk_count1, content1.len());
        println!("   场景 2（有 Tool）: {} 块, {} 字符 (空块: {})", chunk_count2, content2.len(), empty_chunks);
        
        if content2.is_empty() {
            println!("\n   ❌ 问题确认: 包含 Role::Tool 时流式返回空内容！");
            println!("   这证实了智谱 GLM 在流式模式下不能正确处理 tool messages");
        } else if chunk_count2 == 1 && chunk_count1 > 10 {
            println!("\n   ⚠️  可能的问题: 流式块数量显著减少");
            println!("   可能被强制切换为非流式模式");
        } else if chunk_count2 > 10 && content2.len() > 50 {
            println!("\n   ✅ 正常: 包含 Role::Tool 时流式响应正常");
            println!("   智谱 GLM 可以正确处理 tool messages");
        } else {
            println!("\n   ⚠️  结果不确定，需要进一步分析");
        }
        
        println!("\n{}", "=".repeat(80));
        println!("🎯 测试完成！");
    }
    
    Ok(())
}

