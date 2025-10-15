use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 此示例需要启用 streaming 功能");
        println!("请使用: cargo run --example hunyuan_streaming --features streaming");
        return Ok(());
    }

    #[cfg(feature = "streaming")]
    {
        // 腾讯混元 OpenAI 兼容端点
        let api_key = std::env::var("HUNYUAN_API_KEY")
            .expect("请设置环境变量 HUNYUAN_API_KEY");

        let client = LlmClient::hunyuan(&api_key);

        let model = std::env::var("HUNYUAN_MODEL").unwrap_or_else(|_| "hunyuan-lite".to_string());
        let request = ChatRequest {
            model,
            messages: vec![Message::user("请详细介绍一下人工智能的发展历程，包括重要的里程碑事件。")],
            max_tokens: Some(512),
            ..Default::default()
        };

        println!("🚀 腾讯混元流式连接测试 (model={})\n", request.model);
        println!("💬 AI 回复：");

        match client.chat_stream(&request).await {
            Ok(mut stream) => {
                let mut full_content = String::new();
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(response) => {
                            if !response.content.is_empty() {
                                print!("{}", response.content);
                                full_content.push_str(&response.content);
                                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                            }
                        }
                        Err(e) => {
                            println!("\n❌ 流式响应错误：{}", e);
                            break;
                        }
                    }
                }
                println!("\n\n✅ 流式响应完成");
                println!("📝 完整内容长度: {} 字符", full_content.len());
            }
            Err(e) => {
                println!("❌ 失败：{}", e);
            }
        }
    }

    Ok(())
}
