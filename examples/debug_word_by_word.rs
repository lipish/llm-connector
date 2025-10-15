use futures_util::StreamExt;
use llm_connector::{
    types::{ChatRequest, Message},
    LlmClient,
};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("ZHIPU_API_KEY").expect("请设置环境变量 ZHIPU_API_KEY");

    println!("🔍 逐字流式响应调试示例");

    let client = LlmClient::zhipu(&api_key);
    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: vec![Message::user(
            "请详细介绍春天的美景，包括花朵、鸟鸣、和风等元素",
        )],
        max_tokens: Some(100),
        ..Default::default()
    };

    println!("开始逐字流式请求...\n");
    let mut stream = client.chat_stream(&request).await?;
    let mut char_count = 0;
    let mut full_text = String::new();

    while let Some(item) = stream.next().await {
        match item {
            Ok(chunk) => {
                if let Some(content) = chunk.get_content() {
                    if !content.is_empty() {
                        print!("📦 块 {}: '{}'", char_count + 1, content);
                        io::stdout().flush().ok();

                        // 逐个字符显示
                        for ch in content.chars() {
                            print!("{}", ch);
                            io::stdout().flush().ok();
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }

                        println!(); // 换行
                        full_text.push_str(content);
                        char_count += 1;
                    }
                }

                // 检查是否完成
                if let Some(finish_reason) = chunk
                    .choices
                    .first()
                    .and_then(|c| c.finish_reason.as_deref())
                {
                    if finish_reason == "stop" {
                        println!("\n✅ 流式响应完成！");
                        if let Some(usage) = chunk.usage {
                            println!(
                                "📊 使用统计: prompt={}, completion={}, total={}",
                                usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                            );
                        }
                        println!("🔢 总块数: {}", char_count);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ 流式响应错误: {}", e);
                break;
            }
        }
    }

    println!("\n📝 完整文本: {}", full_text);
    println!(
        "🎯 平均每块字符数: {:.1}",
        full_text.len() as f64 / char_count as f64
    );

    Ok(())
}
