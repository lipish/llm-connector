//! 调试 LongCat Anthropic 流式响应格式

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("LONGCAT_API_KEY")
        .expect("LONGCAT_API_KEY environment variable not set");

    println!("🔍 调试 LongCat Anthropic 流式响应格式\n");

    // 创建客户端
    let client = LlmClient::longcat_anthropic(&api_key)?;

    // 创建请求
    let request = ChatRequest {
        model: "LongCat-Flash-Chat".to_string(),
        messages: vec![Message::text(Role::User, "你好")],
        stream: Some(true),
        max_tokens: Some(50),
        ..Default::default()
    };

    println!("📤 发送流式请求...\n");

    // 直接使用 reqwest 发送请求，查看原始响应
    let url = "https://api.longcat.chat/anthropic/v1/messages";
    let client_http = reqwest::Client::new();
    
    let response = client_http
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "LongCat-Flash-Chat",
            "messages": [{"role": "user", "content": "你好"}],
            "stream": true,
            "max_tokens": 50
        }))
        .send()
        .await?;

    println!("📥 响应状态: {}\n", response.status());
    println!("📋 响应头:");
    for (key, value) in response.headers() {
        println!("   {}: {:?}", key, value);
    }
    println!();

    // 读取原始流
    use futures_util::StreamExt;
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut event_count = 0;

    println!("📡 原始 SSE 事件:\n");
    println!("{}", "=".repeat(80));

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                let chunk_str = String::from_utf8_lossy(&bytes);
                buffer.push_str(&chunk_str);

                // 查找完整的事件（以 \n\n 分隔）
                while let Some(idx) = buffer.find("\n\n") {
                    let event = buffer.drain(..idx + 2).collect::<String>();
                    event_count += 1;

                    println!("事件 #{}", event_count);
                    println!("{}", "-".repeat(80));
                    
                    // 提取 data: 行
                    for line in event.lines() {
                        if let Some(data) = line.strip_prefix("data: ").or_else(|| line.strip_prefix("data:")) {
                            let data = data.trim();
                            if data == "[DONE]" {
                                println!("📌 [DONE] 标记");
                            } else if !data.is_empty() {
                                println!("📦 JSON 数据:");
                                // 尝试格式化 JSON
                                match serde_json::from_str::<serde_json::Value>(data) {
                                    Ok(json) => {
                                        println!("{}", serde_json::to_string_pretty(&json)?);
                                    }
                                    Err(e) => {
                                        println!("❌ JSON 解析失败: {}", e);
                                        println!("原始数据: {}", data);
                                    }
                                }
                            }
                        }
                    }
                    println!("{}", "=".repeat(80));
                    println!();

                    // 只显示前 5 个事件
                    if event_count >= 5 {
                        println!("... (省略剩余事件)\n");
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ 读取流错误: {}", e);
                break;
            }
        }
    }

    println!("\n✅ 总共收到 {} 个事件", event_count);

    Ok(())
}

