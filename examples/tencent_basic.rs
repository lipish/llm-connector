#[cfg(feature = "tencent")]
use llm_connector::{
    LlmClient,
    types::{ChatRequest, Message},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "tencent"))]
    {
        println!("❌ This example requires enabling the 'tencent' feature");
        println!("Please run: cargo run --example tencent_basic --features tencent");
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

        let model = std::env::var("HUNYUAN_MODEL").unwrap_or_else(|_| "hunyuan-lite".to_string());
        let request = ChatRequest {
            model: model.clone(),
            messages: vec![Message::user(
                "Please briefly describe the features of Tencent Hunyuan LLMs using the native API.",
            )],
            max_tokens: Some(256),
            ..Default::default()
        };

        println!(
            "🚀 Tencent Hunyuan Native API v3 Test (model={})\n",
            request.model
        );

        match client.chat(&request).await {
            Ok(resp) => {
                println!(
                    "✅ Success, output:\n{}",
                    resp.choices[0].message.content_as_text()
                );
                println!("\n📊 Token usage:");
                println!("  Input tokens: {}", resp.prompt_tokens());
                println!("  Output tokens: {}", resp.completion_tokens());
                println!("  Total tokens: {}", resp.total_tokens());
                println!("\n🆔 Request ID: {}", resp.id);
            }
            Err(e) => {
                println!("❌ Failed: {}", e);
                println!("\n💡 Please check:");
                println!("  1. Whether TENCENT_SECRET_ID and TENCENT_SECRET_KEY are correct");
                println!("  2. Whether the account has access to Hunyuan models");
                println!("  3. Whether your network connection is working");
            }
        }
        Ok(())
    }
}
