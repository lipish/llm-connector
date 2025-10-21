#[cfg(feature = "tencent")]
use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "tencent"))]
    {
        println!("❌ 此示例需要启用 tencent 功能");
        println!("请使用: cargo run --example tencent_basic --features tencent");
        return Ok(());
    }

    #[cfg(feature = "tencent")]
    {
        // 腾讯云混元 API Key (OpenAI 兼容格式)
        let api_key = std::env::var("TENCENT_API_KEY")
            .expect("请设置环境变量 TENCENT_API_KEY (格式: sk-...)");

        let client = LlmClient::tencent(&api_key)?;

        let model = std::env::var("HUNYUAN_MODEL").unwrap_or_else(|_| "hunyuan-lite".to_string());
        let request = ChatRequest {
            model,
            messages: vec![Message::user("请简要介绍一下腾讯混元大模型的特点，使用原生API调用。")],
            max_tokens: Some(256),
            ..Default::default()
        };

        println!("🚀 腾讯混元 OpenAI 兼容 API 非流式连接测试 (model={})\n", request.model);

        match client.chat(&request).await {
            Ok(resp) => {
                println!("✅ 成功，输出：\n{}", resp.choices[0].message.content_as_text());
                println!("\n📊 Token 使用情况:");
                println!("  输入 tokens: {}", resp.prompt_tokens());
                println!("  输出 tokens: {}", resp.completion_tokens());
                println!("  总计 tokens: {}", resp.total_tokens());
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
        Ok(())
    }
}
