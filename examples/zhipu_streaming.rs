// Enable streaming feature for rust-analyzer
// See: https://zed.dev/docs/languages/rust

use futures_util::StreamExt;
use llm_connector::{
    types::{ChatRequest, Message},
    LlmClient,
};

#[cfg(feature = "streaming")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量读取 API Key
    // Zhipu 官方文档端点（paas v4）：https://open.bigmodel.cn/api/paas/v4
    let api_key = std::env::var("ZHIPU_API_KEY").expect("请设置环境变量 ZHIPU_API_KEY");

    // 使用 Zhipu 协议（默认使用官方 paas/v4 端点）
    let client = LlmClient::zhipu(&api_key);

    // 模型名称
    let model = "glm-4-flash";

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![Message::user("请写三首关于春天的五言诗")],
        max_tokens: Some(200),
        ..Default::default()
    };

    println!("🚀 开始 Zhipu 流式响应示例 (model={model})\n");
    let mut stream = client.chat_stream(&request).await?;

    let mut full_text = String::new();
    while let Some(item) = stream.next().await {
        match item {
            Ok(chunk) => {
                if let Some(content) = chunk.get_content() {
                    print!("{}", content);
                    full_text.push_str(content);
                    use std::io::{self, Write};
                    io::stdout().flush().ok();
                }

                if let Some(fr) = chunk
                    .choices
                    .first()
                    .and_then(|c| c.finish_reason.as_deref())
                {
                    if fr == "stop" {
                        println!("\n\n✅ 流式响应完成！");
                        if let Some(usage) = chunk.usage {
                            println!(
                                "📊 使用统计: prompt={}, completion={}, total={}",
                                usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                            );
                        }
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("\n❌ 流式响应错误: {}", e);
                break;
            }
        }
    }

    println!("\n\n📝 完整文本:\n{}", full_text);
    Ok(())
}

#[cfg(not(feature = "streaming"))]
fn main() {
    println!("❌ 需要启用 'streaming' 功能才能运行此示例");
    println!("   请使用: cargo run --example zhipu_streaming --features streaming");
}
