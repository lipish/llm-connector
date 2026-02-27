use llm_connector::{LlmClient, types::{ChatRequest, Message, Role, MessageBlock}};
use std::error::Error;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // This example demonstrates the file upload helper
    // Note: You need a valid image file to run this
    let image_path = "test_image.jpg";
    if !Path::new(image_path).exists() {
        println!("Skipping example: {} not found", image_path);
        return Ok(());
    }

    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        let client = LlmClient::openai(&key)?;
        
        // 1. Create message with file upload (one-liner)
        let request = ChatRequest::new("gpt-4o")
            .add_message(Message::user("What's in this image?"))
            .add_message_block(MessageBlock::from_file_path(image_path)?);
            
        let response = client.chat(&request).await?;
        println!("Analysis: {}", response.content);
    }

    Ok(())
}
