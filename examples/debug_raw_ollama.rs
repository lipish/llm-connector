use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 原始 Ollama HTTP 请求调试");

    // 手动构建 Ollama 请求格式
    let ollama_req = serde_json::json!({
        "model": "glm-4-flash",
        "messages": [
            {"role": "user", "content": "Say hello"}
        ],
        "stream": false,
        "options": {
            "num_predict": 50
        }
    });

    println!("\n🔧 Ollama 请求:");
    println!("  URL: http://localhost:11434/api/chat");
    println!("  请求体: {}", serde_json::to_string_pretty(&ollama_req)?);

    // 测试 HTTP 请求
    println!("\n🌐 发送 HTTP 请求...");
    let start = Instant::now();

    let http_client = reqwest::Client::builder()
        .user_agent("llm-connector/0.3.7")
        .no_proxy()  // 绕过代理
        .build()?;
    match http_client
        .post("http://localhost:11434/api/chat")
        .json(&ollama_req)
        .send()
        .await
    {
        Ok(response) => {
            println!("✅ HTTP 响应: {:?}", response.status());
            println!("  响应时间: {:?}", start.elapsed());

            // 读取响应体
            let body = response.text().await?;
            println!("📄 响应体: {}", body);

            // 尝试解析
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    println!("✅ JSON 解析成功");
                    println!("  结构: {}", serde_json::to_string_pretty(&json)?);
                }
                Err(e) => {
                    println!("❌ JSON 解析失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ HTTP 请求失败: {}", e);
        }
    }

    // 测试 llm-connector
    println!("\n💬 测试 llm-connector LlmClient...");
    let start = Instant::now();

    // 添加必要的导入
    use llm_connector::{LlmClient, types::{ChatRequest, Message}};

    let client = LlmClient::ollama(Some("http://localhost:11434"));
    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: vec![Message::user("Say hello")],
        max_tokens: Some(50),
        ..Default::default()
    };

    match client.chat(&request).await {
        Ok(response) => {
            println!("✅ LlmClient 响应成功 ({:?})", start.elapsed());
            println!("  内容: {}", response.choices[0].message.content);
        }
        Err(e) => {
            println!("❌ LlmClient 响应失败 ({:?}): {}", start.elapsed(), e);
        }
    }

    Ok(())
}