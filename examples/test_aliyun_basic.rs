/// 测试阿里云 DashScope 基础功能
/// 
/// 验证修复后的 Content-Type 头部问题

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("ALIYUN_API_KEY")
        .expect("请设置环境变量 ALIYUN_API_KEY");
    
    let client = LlmClient::aliyun(&api_key)?;
    
    println!("🧪 测试阿里云 DashScope 基础功能");
    println!("{}", "=".repeat(80));
    
    let request = ChatRequest {
        model: "qwen-turbo".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "你好，请用一句话介绍你自己。".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    
    println!("\n📤 发送请求...");
    println!("   Model: qwen-turbo");
    println!("   Message: 你好，请用一句话介绍你自己。");
    
    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ 请求成功！");
            println!("\n📥 响应:");
            println!("   Model: {}", response.model);
            println!("   Content: {}", response.content);
            
            if !response.content.is_empty() {
                println!("\n🎉 阿里云 DashScope 工作正常！");
                println!("   Content-Type 头部问题已修复。");
            }
        }
        Err(e) => {
            println!("\n❌ 请求失败: {}", e);
            println!("\n如果错误是 'Content-Type/Accept application/json,application/json is not supported'");
            println!("说明 Content-Type 头部仍然重复，需要进一步检查。");
            return Err(e.into());
        }
    }
    
    println!("\n{}", "=".repeat(80));
    
    Ok(())
}

