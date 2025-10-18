/// 测试智谱 GLM 在包含 Role::Tool 消息时的流式响应问题
///
/// 验证场景：
/// 1. 第一轮请求（无 Tool 消息）- 应该正常流式返回
/// 2. 第二轮请求（包含 Role::Tool 消息）- 检查是否返回空内容

#[cfg(feature = "streaming")]
use futures_util::StreamExt;
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role, Tool, Function}};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 需要启用 'streaming' 功能");
        println!("运行: cargo run --example test_zhipu_tool_streaming_issue --features streaming");
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
        
        println!("🧪 测试智谱 GLM 流式响应问题");
        println!("{}", "=".repeat(70));

        // 测试三个模型
        let models = vec!["glm-4-flash", "glm-4", "glm-4.5"];

        for model in models {
            println!("\n📝 测试模型: {}", model);
            println!("{}", "-".repeat(70));
            
            // 场景 1: 第一轮请求（无 Tool 消息）
            println!("\n✅ 场景 1: 第一轮请求（无 Role::Tool 消息）");
            let request1 = ChatRequest {
                model: model.to_string(),
                messages: vec![Message {
                    role: Role::User,
                    content: "上海的天气怎么样？".to_string(),
                    ..Default::default()
                }],
                tools: Some(tools.clone()),
                ..Default::default()
            };
            
            let mut stream1 = client.chat_stream(&request1).await?;
            let mut chunk_count1 = 0;
            let mut content1 = String::new();
            let mut has_tool_calls = false;
            let mut tool_call_id = String::new();
            
            while let Some(chunk) = stream1.next().await {
                match chunk {
                    Ok(response) => {
                        chunk_count1 += 1;
                        
                        if let Some(delta_content) = response.get_content() {
                            content1.push_str(delta_content);
                        }
                        
                        // 检查是否有工具调用
                        if let Some(choice) = response.choices.first() {
                            if let Some(tool_calls) = &choice.delta.tool_calls {
                                has_tool_calls = true;
                                if let Some(call) = tool_calls.first() {
                                    if !call.id.is_empty() {
                                        tool_call_id = call.id.clone();
                                    }
                                }
                            }
                            
                            if let Some(reason) = &choice.finish_reason {
                                println!("   finish_reason: {}", reason);
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
            
            println!("   收到流式块: {} 个", chunk_count1);
            println!("   内容长度: {} 字符", content1.len());
            println!("   有工具调用: {}", has_tool_calls);
            
            if !content1.is_empty() {
                println!("   内容预览: {}...", 
                    content1.chars().take(50).collect::<String>());
            }
            
            // 场景 2: 第二轮请求（包含 Role::Tool 消息）
            if has_tool_calls && !tool_call_id.is_empty() {
                println!("\n⚠️  场景 2: 第二轮请求（包含 Role::Tool 消息）");
                
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
                    ..Default::default()
                };
                
                let mut stream2 = client.chat_stream(&request2).await?;
                let mut chunk_count2 = 0;
                let mut content2 = String::new();
                
                while let Some(chunk) = stream2.next().await {
                    match chunk {
                        Ok(response) => {
                            chunk_count2 += 1;
                            
                            if let Some(delta_content) = response.get_content() {
                                content2.push_str(delta_content);
                            }
                            
                            if let Some(choice) = response.choices.first() {
                                if let Some(reason) = &choice.finish_reason {
                                    println!("   finish_reason: {}", reason);
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
                
                println!("   收到流式块: {} 个", chunk_count2);
                println!("   内容长度: {} 字符", content2.len());
                
                if !content2.is_empty() {
                    println!("   内容预览: {}...", 
                        content2.chars().take(50).collect::<String>());
                } else {
                    println!("   ⚠️  内容为空！");
                }
                
                // 分析结果
                println!("\n📊 对比分析:");
                println!("   场景 1（无 Tool）: {} 块, {} 字符", chunk_count1, content1.len());
                println!("   场景 2（有 Tool）: {} 块, {} 字符", chunk_count2, content2.len());
                
                if content2.is_empty() && chunk_count2 > 0 {
                    println!("\n   ❌ 问题确认: 包含 Role::Tool 时流式返回空内容！");
                } else if chunk_count2 == 1 && chunk_count1 > 10 {
                    println!("\n   ⚠️  可能的问题: 流式块数量显著减少（可能被强制切换为非流式）");
                } else if !content2.is_empty() {
                    println!("\n   ✅ 正常: 包含 Role::Tool 时流式响应正常");
                }
            } else {
                println!("\n   ⚠️  跳过场景 2: 第一轮未触发工具调用");
            }

            println!("\n{}", "=".repeat(70));
        }
        
        println!("\n🎯 测试完成！");
    }
    
    Ok(())
}

