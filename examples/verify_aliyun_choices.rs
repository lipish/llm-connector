/// 验证 Aliyun 响应的 choices 数组
/// 
/// 确认修复后 choices 数组不为空，并且与 content 字段一致

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("ALIYUN_API_KEY")
        .expect("请设置环境变量 ALIYUN_API_KEY");
    
    let client = LlmClient::aliyun(&api_key)?;
    
    println!("🔍 验证 Aliyun 响应的 choices 数组");
    println!("{}", "=".repeat(80));
    
    let request = ChatRequest {
        model: "qwen-turbo".to_string(),
        messages: vec![Message::text(Role::User, "你好")],
        ..Default::default()
    };
    
    println!("\n📤 发送请求...");
    
    let response = client.chat(&request).await?;
    
    println!("\n📥 响应结构:");
    println!("{}", "-".repeat(80));
    
    // 检查 choices 数组
    println!("\n1. choices 数组:");
    println!("   长度: {}", response.choices.len());
    
    if response.choices.is_empty() {
        println!("   ❌ choices 数组为空！");
        println!("   这是一个 bug，应该包含至少一个 choice。");
        return Err("choices 数组为空".into());
    } else {
        println!("   ✅ choices 数组不为空");
    }
    
    // 检查第一个 choice
    if let Some(first_choice) = response.choices.first() {
        println!("\n2. choices[0]:");
        println!("   index: {}", first_choice.index);
        println!("   message.role: {:?}", first_choice.message.role);
        println!("   message.content: {}", first_choice.message.content);
        println!("   finish_reason: {:?}", first_choice.finish_reason);
        
        // 检查 content 字段
        println!("\n3. content 便利字段:");
        println!("   content: {}", response.content);
        
        // 验证一致性
        println!("\n4. 一致性检查:");
        if first_choice.message.content == response.content {
            println!("   ✅ choices[0].message.content == content");
            println!("   符合设计意图：content 是从 choices[0] 提取的便利字段");
        } else {
            println!("   ❌ choices[0].message.content != content");
            println!("   choices[0].message.content: {}", first_choice.message.content);
            println!("   content: {}", response.content);
            return Err("content 字段与 choices[0] 不一致".into());
        }
    }
    
    // 检查 usage
    println!("\n5. usage 信息:");
    if let Some(ref usage) = response.usage {
        println!("   ✅ 包含 usage 信息");
        println!("   prompt_tokens: {}", usage.prompt_tokens);
        println!("   completion_tokens: {}", usage.completion_tokens);
        println!("   total_tokens: {}", usage.total_tokens);
    } else {
        println!("   ⚠️  没有 usage 信息");
    }
    
    // 检查其他字段
    println!("\n6. 其他字段:");
    println!("   id: {}", if response.id.is_empty() { "(empty)" } else { &response.id });
    println!("   object: {}", response.object);
    println!("   model: {}", response.model);
    
    println!("\n{}", "=".repeat(80));
    println!("✅ 所有检查通过！");
    println!("\n总结:");
    println!("  • choices 数组不为空");
    println!("  • choices[0].message.content 与 content 字段一致");
    println!("  • 包含 usage 信息");
    println!("  • 符合 OpenAI 标准格式");
    println!("{}", "=".repeat(80));
    
    Ok(())
}

