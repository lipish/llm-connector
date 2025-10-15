use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "tencent-native"))]
    {
        println!("❌ 此示例需要启用 tencent-native 功能");
        println!("请使用: cargo run --example hunyuan_native_basic --features tencent-native");
        return Ok(());
    }

    #[cfg(feature = "tencent-native")]
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
            messages: vec![Message::user("请简要介绍一下腾讯混元大模型的特点，使用原生API调用。")],
            max_tokens: Some(256),
            ..Default::default()
        };

        println!("🚀 腾讯混元原生API非流式连接测试 (model={})\n", request.model);
        println!("🔐 使用TC3-HMAC-SHA256签名认证");
        println!("🌍 地域: {}\n", region.as_deref().unwrap_or("ap-beijing"));

        match client.chat(&request).await {
            Ok(resp) => {
                println!("✅ 成功，输出：\n{}", resp.choices[0].message.content);
                if let Some(usage) = resp.usage {
                    println!("\n📊 Token 使用情况:");
                    println!("  输入 tokens: {}", usage.prompt_tokens);
                    println!("  输出 tokens: {}", usage.completion_tokens);
                    println!("  总计 tokens: {}", usage.total_tokens);
                }
                println!("\n🆔 请求ID: {}", resp.id);
            }
            Err(e) => {
                println!("❌ 失败：{}", e);
                println!("\n💡 请检查：");
                println!("  1. TENCENT_SECRET_ID 和 TENCENT_SECRET_KEY 是否正确");
                println!("  2. 账户是否有混元大模型的访问权限");
                println!("  3. 网络连接是否正常");
            }
        }
    }

    Ok(())
}
