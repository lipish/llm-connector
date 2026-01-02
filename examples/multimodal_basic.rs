//! Multi-modal Content Basic Example
//!
//! Demonstrates how to use MessageBlock to send text and images.

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role, MessageBlock}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Multi-modal Content Example");
    println!("{}", "=".repeat(80));

    // Example 1: Text-only message (backward compatible)
    println!("\n📝 Example 1: Text-only message");
    println!("{}", "-".repeat(80));
    
    let message = Message::text(Role::User, "Hello, world!");
    println!("Create text-only message:");
    println!("  role: {:?}", message.role);
    println!("  content blocks: {}", message.content.len());
    println!("  text: {}", message.content_as_text());

    // Example 2: Multi-modal message (text + image URL)
    println!("\n\n🖼️  Example 2: Multi-modal message (text + image URL)");
    println!("{}", "-".repeat(80));
    
    let message = Message::new(
        Role::User,
        vec![
            MessageBlock::text("What's in this image?"),
            MessageBlock::image_url("https://example.com/image.jpg"),
        ],
    );
    
    println!("Create multi-modal message:");
    println!("  role: {:?}", message.role);
    println!("  content blocks: {}", message.content.len());
    println!("  has images: {}", message.has_images());
    println!("  is text only: {}", message.is_text_only());

    // Example 3: Base64 image
    println!("\n\n📷 Example 3: Base64 image");
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
    
    println!("Create Base64 image message:");
    println!("  content blocks: {}", message.content.len());
    println!("  block 0: text");
    println!("  block 1: image (base64)");

    // Example 4: Multiple images
    println!("\n\n🖼️🖼️  Example 4: Multiple images");
    println!("{}", "-".repeat(80));
    
    let message = Message::new(
        Role::User,
        vec![
            MessageBlock::text("Compare these two images"),
            MessageBlock::image_url("https://example.com/image1.jpg"),
            MessageBlock::image_url("https://example.com/image2.jpg"),
        ],
    );
    
    println!("Create multi-image message:");
    println!("  content blocks: {}", message.content.len());
    for (i, block) in message.content.iter().enumerate() {
        if block.is_text() {
            println!("  block {}: text - {}", i, block.as_text().unwrap());
        } else if block.is_image() {
            println!("  block {}: image", i);
        }
    }

    // Example 5: Real API call (requires API key)
    println!("\n\n🚀 Example 5: Real API call");
    println!("{}", "-".repeat(80));
    
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        println!("Using OpenAI API...");
        
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
                println!("✅ Response succeeded:");
                println!("  {}", response.content);
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }
    } else {
        println!("⚠️  OPENAI_API_KEY is not set; skipping API call");
        println!("   Set it via: export OPENAI_API_KEY=your-key");
    }

    // Example 6: Convenience constructors
    println!("\n\n⚡ Example 6: Convenience constructors");
    println!("{}", "-".repeat(80));
    
    let system_msg = Message::system("You are a helpful assistant.");
    let user_msg = Message::user("Hello!");
    let assistant_msg = Message::assistant("Hi! How can I help you?");
    
    println!("Create messages:");
    println!("  system: {}", system_msg.content_as_text());
    println!("  user: {}", user_msg.content_as_text());
    println!("  assistant: {}", assistant_msg.content_as_text());

    println!("\n{}", "=".repeat(80));
    println!("✅ Multi-modal content example completed!");
    println!("{}", "=".repeat(80));

    println!("\n📚 Summary:");
    println!("   1. Text-only: Message::text(role, \"text\")");
    println!("   2. Multi-modal: Message::new(role, vec![MessageBlock::text(...), MessageBlock::image_url(...)])");
    println!("   3. Base64 image: MessageBlock::image_base64(media_type, data)");
    println!("   4. Image URL: MessageBlock::image_url(url)");
    println!("   5. Convenience: Message::system(), Message::user(), Message::assistant()");

    Ok(())
}
