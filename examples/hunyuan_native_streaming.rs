use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "tencent-native"))]
    {
        println!("❌ 此示例需要启用 tencent-native 功能");
        println!("请使用: cargo run --example hunyuan_native_streaming --features \"tencent-native,streaming\"");
        return Ok(());
    }

    #[cfg(not(feature = "streaming"))]
    {
        println!("❌ 此示例需要启用 streaming 功能");
        println!("请使用: cargo run --example hunyuan_native_streaming --features \"tencent-native,streaming\"");
        return Ok(());
    }

    #[cfg(all(feature = "tencent-native", feature = "streaming"))]
    {
        // 腾讯云 SecretId 和 SecretKey
        let secret_id = std::env::var("TENCENT_SECRET_ID")
            .expect("请设置环境变量 TENCENT_SECRET_ID");
        let secret_key = std::env::var("TENCENT_SECRET_KEY")
            .expect("请设置环境变量 TENCENT_SECRET_KEY");

        // 可选：指定地域，默认为 ap-beijing
        let region = std::env::var("TENCENT_REGION").ok();

        let client = LlmClient::hunyuan_native(&secret_id, &secret_key, region.as_deref());

        let model = std::env::var("HUNYUAN_MODEL").unwrap_or_else(|_| "hunyuan-lite".to_string());
        let request = ChatRequest {
            model,
            messages: vec![Message::user("请详细介绍一下人工智能的发展历程，包括重要的里程碑事件。使用腾讯混元原生API。")],
            max_tokens: Some(512),
            ..Default::default()
        };

        println!("🚀 腾讯混元原生API流式连接测试 (model={})\n", request.model);
        println!("🔐 使用TC3-HMAC-SHA256签名认证");
        println!("🌍 地域: {}", region.as_deref().unwrap_or("ap-beijing"));
        println!("💬 AI 回复：\n");

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
                println!("\n💡 请检查：");
                println!("  1. TENCENT_SECRET_ID 和 TENCENT_SECRET_KEY 是否正确");
                println!("  2. 账户是否有混元大模型的访问权限");
                println!("  3. 网络连接是否正常");
                println!("  4. 是否启用了流式响应功能");
            }
        }
    }

    Ok(())
}
