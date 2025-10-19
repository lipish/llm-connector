/// 调试阿里云 DashScope 响应解析
/// 
/// 验证是否存在"响应内容为空"的问题

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("ALIYUN_API_KEY")
        .expect("请设置环境变量 ALIYUN_API_KEY");
    
    let client = LlmClient::aliyun(&api_key)?;
    
    println!("🔍 调试阿里云 DashScope 响应解析");
    println!("{}", "=".repeat(80));
    
    // 测试多个场景
    let test_cases = vec![
        ("简单问候", "你好"),
        ("长回答", "请详细介绍一下人工智能的发展历史"),
        ("代码生成", "用 Rust 写一个 Hello World 程序"),
        ("数学问题", "1+1等于几？"),
    ];
    
    for (name, prompt) in test_cases {
        println!("\n{}", "-".repeat(80));
        println!("📝 测试场景: {}", name);
        println!("{}", "-".repeat(80));
        
        let request = ChatRequest {
            model: "qwen-turbo".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: prompt.to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        
        println!("\n📤 发送请求:");
        println!("   Model: qwen-turbo");
        println!("   Prompt: {}", prompt);
        
        match client.chat(&request).await {
            Ok(response) => {
                println!("\n✅ 请求成功");
                println!("\n📥 响应:");
                println!("   Model: {}", response.model);
                println!("   Content length: {} 字符", response.content.len());
                
                if response.content.is_empty() {
                    println!("\n   ❌ 内容为空！");
                    println!("   这是一个 bug，需要检查响应解析逻辑。");
                } else {
                    println!("\n   ✅ 内容正常");
                    println!("   Content preview: {}...", 
                        response.content.chars().take(100).collect::<String>());
                }
                
                // 检查其他字段
                if let Some(usage) = &response.usage {
                    println!("\n   Usage:");
                    println!("      prompt_tokens: {}", usage.prompt_tokens);
                    println!("      completion_tokens: {}", usage.completion_tokens);
                    println!("      total_tokens: {}", usage.total_tokens);
                }
            }
            Err(e) => {
                println!("\n❌ 请求失败: {}", e);
                println!("   错误类型: {:?}", e);
            }
        }
    }
    
    println!("\n{}", "=".repeat(80));
    println!("📊 总结:");
    println!("{}", "=".repeat(80));
    println!("\n如果所有测试都返回了内容，说明 llm-connector 可以正确解析 Aliyun 响应。");
    println!("如果有测试返回空内容，说明存在响应解析问题。");
    
    println!("\n{}", "=".repeat(80));
    
    Ok(())
}

