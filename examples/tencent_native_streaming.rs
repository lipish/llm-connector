use llm_connector::{LlmClient, ChatRequest, Message};
use std::io::Write;
#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "tencent"))]
    {
        println!("❌ This example requires enabling the 'tencent' feature");
        println!("Please run: cargo run --example tencent_native_streaming --features tencent");
        return Ok(());
    }

    #[cfg(feature = "tencent")]
    {
        // Tencent Hunyuan Secret credentials
        let secret_id = std::env::var("TENCENT_SECRET_ID")
            .expect("Please set environment variable TENCENT_SECRET_ID");
        let secret_key = std::env::var("TENCENT_SECRET_KEY")
            .expect("Please set environment variable TENCENT_SECRET_KEY");

        let client = LlmClient::tencent(&secret_id, &secret_key)?;

        let model = "hunyuan-standard"; // or hunyuan-lite, hunyuan-pro

        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![Message::user("Please write a short poem about the ocean.")],
            max_tokens: Some(256),
            stream: Some(true),
            ..Default::default()
        };

        println!("🚀 Tencent Hunyuan Native API v3 Streaming Test (model={})\n", request.model);

        #[cfg(feature = "streaming")]
        {
            let mut stream = client.chat_stream(&request).await?;
            let mut full_response = String::new();

            print!("Response: ");
            std::io::stdout().flush()?;

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        print!("{}", chunk.content);
                        std::io::stdout().flush()?;
                        full_response.push_str(&chunk.content);
                    }
                    Err(e) => {
                        println!("\n❌ Stream Error: {}", e);
                        break;
                    }
                }
            }
            println!("\n\n✅ Full response received.");
        }
        
        #[cfg(not(feature = "streaming"))]
        {
             println!("❌ Streaming feature is not enabled. Please enable 'streaming' feature.");
        }
    }

    Ok(())
}
