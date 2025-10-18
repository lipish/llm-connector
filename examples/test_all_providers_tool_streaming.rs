#[cfg(feature = "streaming")]
use futures_util::StreamExt;
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role, Tool, Function}};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 需要启用 'streaming' 功能");
        return Ok(());
    }
    
    #[cfg(feature = "streaming")]
    {
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
        
        let mut providers: Vec<(&str, LlmClient, &str)> = Vec::new();
        
        if let Ok(key) = std::env::var("DEEPSEEK_API_KEY") {
            providers.push(("DeepSeek", LlmClient::openai_with_base_url(&key, "https://api.deepseek.com")?, "deepseek-chat"));
        }
        
        if let Ok(key) = std::env::var("ZHIPU_API_KEY") {
            providers.push(("Zhipu GLM-4", LlmClient::zhipu(&key)?, "glm-4"));
            providers.push(("Zhipu GLM-4.5", LlmClient::zhipu(&key)?, "glm-4.5"));
            providers.push(("Zhipu GLM-4-Flash", LlmClient::zhipu(&key)?, "glm-4-flash"));
        }
        
        if let Ok(key) = std::env::var("ALIYUN_API_KEY") {
            providers.push(("Aliyun Qwen", LlmClient::aliyun(&key)?, "qwen-plus"));
        }
        
        let mut results: HashMap<String, TestResult> = HashMap::new();
        
        for (name, client, model) in providers {
            println!("\n{}", "=".repeat(70));
            println!("🧪 测试 Provider: {}", name);
            println!("   Model: {}", model);
            println!("{}\n", "=".repeat(70));
            
            let result = test_provider(&client, model, &tools).await;
            
            match result {
                Ok(test_result) => {
                    results.insert(name.to_string(), test_result);
                    println!("\n✅ {} 测试完成", name);
                }
                Err(e) => {
                    println!("\n❌ {} 测试失败: {}", name, e);
                    results.insert(name.to_string(), TestResult::error());
                }
            }
        }
        
        println!("\n\n{}", "=".repeat(70));
        println!("📊 测试结果汇总");
        println!("{}\n", "=".repeat(70));
        
        println!("{:<20} | {:^15} | {:^15} | {:^15}", "Provider", "第一轮流式", "第二轮流式", "需要修复");
        println!("{}", "-".repeat(70));
        
        for (name, result) in results.iter() {
            let round1_status = if result.round1_chunks > 1 { "✅ 支持" } else if result.round1_chunks == 1 { "⚠️ 单块" } else { "❌ 失败" };
            let round2_status = if result.round2_chunks > 1 { "✅ 支持" } else if result.round2_chunks == 1 { "⚠️ 单块" } else { "❌ 失败" };
            let needs_fix = if result.round2_chunks == 1 && result.round1_chunks > 1 { "⚠️ 需要" } else if result.round2_chunks == 0 { "❌ 异常" } else { "✅ 不需要" };
            
            println!("{:<20} | {:^15} | {:^15} | {:^15}", name, round1_status, round2_status, needs_fix);
        }
        
        println!("\n💡 说明:");
        println!("  - 第一轮流式: 不包含 Role::Tool 的请求");
        println!("  - 第二轮流式: 包含 Role::Tool 的请求");
        println!("  - ✅ 支持: 收到多个流式块");
        println!("  - ⚠️ 单块: 只收到1个块（可能被强制切换为非流式）");
        println!("  - ⚠️ 需要: 第一轮支持流式，但第二轮被强制切换（说明需要 workaround）");
    }
    
    Ok(())
}

#[derive(Debug)]
struct TestResult {
    round1_chunks: usize,
    round1_has_tool_calls: bool,
    round2_chunks: usize,
    round2_has_content: bool,
}

impl TestResult {
    fn error() -> Self {
        Self {
            round1_chunks: 0,
            round1_has_tool_calls: false,
            round2_chunks: 0,
            round2_has_content: false,
        }
    }
}

#[cfg(feature = "streaming")]
async fn test_provider(
    client: &LlmClient,
    model: &str,
    tools: &[Tool],
) -> Result<TestResult, Box<dyn std::error::Error>> {
    
    println!("📝 第一轮请求（触发工具调用）");
    
    let request1 = ChatRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "What's the current weather in San Francisco? Use the get_weather function.".to_string(),
            ..Default::default()
        }],
        tools: Some(tools.to_vec()),
        ..Default::default()
    };
    
    let mut stream1 = client.chat_stream(&request1).await?;
    let mut round1_chunks = 0;
    let mut tool_calls_buffer = Vec::new();
    let mut content_buffer = String::new();
    
    while let Some(chunk) = stream1.next().await {
        round1_chunks += 1;
        match chunk {
            Ok(response) => {
                if let Some(content) = response.get_content() {
                    content_buffer.push_str(content);
                }
                
                if let Some(choice) = response.choices.first() {
                    if let Some(calls) = &choice.delta.tool_calls {
                        tool_calls_buffer.extend(calls.clone());
                    }
                    
                    if let Some(_reason) = &choice.finish_reason {
                        break;
                    }
                }
            }
            Err(e) => {
                println!("⚠️ 第一轮流式错误: {}", e);
                break;
            }
        }
    }
    
    println!("  - 收到 {} 个流式块", round1_chunks);
    println!("  - 工具调用: {}", if !tool_calls_buffer.is_empty() { "✅ 有" } else { "❌ 无" });
    
    if !content_buffer.is_empty() {
        println!("  - 响应内容预览: {}", 
            if content_buffer.len() > 100 { 
                format!("{}...", &content_buffer[..100]) 
            } else { 
                content_buffer.clone() 
            }
        );
    }
    
    if tool_calls_buffer.is_empty() {
        println!("\n⚠️ 跳过第二轮测试（未触发工具调用）");
        return Ok(TestResult {
            round1_chunks,
            round1_has_tool_calls: false,
            round2_chunks: 0,
            round2_has_content: false,
        });
    }
    
    println!("\n📝 第二轮请求（包含 Role::Tool 结果）");
    
    let first_call = &tool_calls_buffer[0];
    
    let request2 = ChatRequest {
        model: model.to_string(),
        messages: vec![
            Message {
                role: Role::User,
                content: "What's the current weather in San Francisco? Use the get_weather function.".to_string(),
                ..Default::default()
            },
            Message {
                role: Role::Assistant,
                content: String::new(),
                tool_calls: Some(vec![first_call.clone()]),
                ..Default::default()
            },
            Message {
                role: Role::Tool,
                content: r#"{"location":"San Francisco","temperature":18,"unit":"celsius","condition":"sunny","humidity":65}"#.to_string(),
                tool_call_id: Some(first_call.id.clone()),
                name: Some(first_call.function.name.clone()),
                ..Default::default()
            },
        ],
        tools: Some(tools.to_vec()),
        ..Default::default()
    };
    
    let mut stream2 = client.chat_stream(&request2).await?;
    let mut round2_chunks = 0;
    let mut round2_content = String::new();
    
    while let Some(chunk) = stream2.next().await {
        round2_chunks += 1;
        match chunk {
            Ok(response) => {
                if let Some(content) = response.get_content() {
                    round2_content.push_str(content);
                }
                
                if let Some(choice) = response.choices.first() {
                    if let Some(_reason) = &choice.finish_reason {
                        break;
                    }
                }
            }
            Err(e) => {
                println!("⚠️ 第二轮流式错误: {}", e);
                break;
            }
        }
    }
    
    println!("  - 收到 {} 个流式块", round2_chunks);
    println!("  - 响应内容: {}", if !round2_content.is_empty() { "✅ 有" } else { "❌ 无" });
    
    Ok(TestResult {
        round1_chunks,
        round1_has_tool_calls: true,
        round2_chunks,
        round2_has_content: !round2_content.is_empty(),
    })
}
