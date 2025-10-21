//! 多模态内容基础示例
//!
//! 演示如何使用 MessageBlock 发送文本和图片

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role, MessageBlock}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 多模态内容示例");
    println!("{}", "=".repeat(80));

    // 示例 1: 纯文本消息（向后兼容）
    println!("\n📝 示例 1: 纯文本消息");
    println!("{}", "-".repeat(80));
    
    let message = Message::text(Role::User, "Hello, world!");
    println!("创建纯文本消息:");
    println!("  role: {:?}", message.role);
    println!("  content blocks: {}", message.content.len());
    println!("  text: {}", message.content_as_text());

    // 示例 2: 多模态消息（文本 + 图片 URL）
    println!("\n\n🖼️  示例 2: 多模态消息（文本 + 图片 URL）");
    println!("{}", "-".repeat(80));
    
    let message = Message::new(
        Role::User,
        vec![
            MessageBlock::text("What's in this image?"),
            MessageBlock::image_url("https://example.com/image.jpg"),
        ],
    );
    
    println!("创建多模态消息:");
    println!("  role: {:?}", message.role);
    println!("  content blocks: {}", message.content.len());
    println!("  has images: {}", message.has_images());
    println!("  is text only: {}", message.is_text_only());

    // 示例 3: Base64 图片
    println!("\n\n📷 示例 3: Base64 图片");
    println!("{}", "-".repeat(80));
    
    let message = Message::new(
        Role::User,
        vec![
            MessageBlock::text("Analyze this image"),
            MessageBlock::image_base64(
                "image/jpeg",
                "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
            ),
        ],
    );
    
    println!("创建 Base64 图片消息:");
    println!("  content blocks: {}", message.content.len());
    println!("  block 0: text");
    println!("  block 1: image (base64)");

    // 示例 4: 多张图片
    println!("\n\n🖼️🖼️  示例 4: 多张图片");
    println!("{}", "-".repeat(80));
    
    let message = Message::new(
        Role::User,
        vec![
            MessageBlock::text("Compare these two images"),
            MessageBlock::image_url("https://example.com/image1.jpg"),
            MessageBlock::image_url("https://example.com/image2.jpg"),
        ],
    );
    
    println!("创建多图片消息:");
    println!("  content blocks: {}", message.content.len());
    for (i, block) in message.content.iter().enumerate() {
        if block.is_text() {
            println!("  block {}: text - {}", i, block.as_text().unwrap());
        } else if block.is_image() {
            println!("  block {}: image", i);
        }
    }

    // 示例 5: 实际 API 调用（需要 API key）
    println!("\n\n🚀 示例 5: 实际 API 调用");
    println!("{}", "-".repeat(80));
    
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        println!("使用 OpenAI API...");
        
        let client = LlmClient::openai(&api_key)?;
        
        let request = ChatRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![
                Message::text(Role::User, "Hello! Please introduce yourself in one sentence."),
            ],
            max_tokens: Some(100),
            ..Default::default()
        };
        
        match client.chat(&request).await {
            Ok(response) => {
                println!("✅ 响应成功:");
                println!("  {}", response.content);
            }
            Err(e) => {
                println!("❌ 错误: {}", e);
            }
        }
    } else {
        println!("⚠️  未设置 OPENAI_API_KEY，跳过 API 调用");
        println!("   设置方法: export OPENAI_API_KEY=your-key");
    }

    // 示例 6: 使用便捷构造函数
    println!("\n\n⚡ 示例 6: 便捷构造函数");
    println!("{}", "-".repeat(80));
    
    let system_msg = Message::system("You are a helpful assistant.");
    let user_msg = Message::user("Hello!");
    let assistant_msg = Message::assistant("Hi! How can I help you?");
    
    println!("创建消息:");
    println!("  system: {}", system_msg.content_as_text());
    println!("  user: {}", user_msg.content_as_text());
    println!("  assistant: {}", assistant_msg.content_as_text());

    println!("\n{}", "=".repeat(80));
    println!("✅ 多模态内容示例完成！");
    println!("{}", "=".repeat(80));

    println!("\n📚 总结:");
    println!("   1. 纯文本: Message::text(role, \"text\")");
    println!("   2. 多模态: Message::new(role, vec![MessageBlock::text(...), MessageBlock::image_url(...)])");
    println!("   3. Base64 图片: MessageBlock::image_base64(media_type, data)");
    println!("   4. 图片 URL: MessageBlock::image_url(url)");
    println!("   5. 便捷函数: Message::system(), Message::user(), Message::assistant()");

    Ok(())
}

