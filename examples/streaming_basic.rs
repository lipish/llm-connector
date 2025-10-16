//! 流式响应基础示例
//!
//! 展示如何使用流式响应功能，实时接收AI回复
//!
//! 运行方式: cargo run --example streaming_basic --features streaming

#[cfg(feature = "streaming")]
use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
#[cfg(feature = "streaming")]
use futures_util::StreamExt;

#[cfg(feature = "streaming")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 流式响应基础示例\n");

    // 从环境变量选择provider
    let provider = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "ollama".to_string());
    
    let client = match provider.as_str() {
        "openai" => {
            let api_key = std::env::var("OPENAI_API_KEY")
                .expect("请设置 OPENAI_API_KEY 环境变量");
            LlmClient::openai(&api_key)?
        }
        "zhipu" => {
            let api_key = std::env::var("ZHIPU_API_KEY")
                .expect("请设置 ZHIPU_API_KEY 环境变量");
            LlmClient::zhipu(&api_key)?
        }
        "ollama" => {
            LlmClient::ollama()?
        }
        _ => {
            println!("❌ 不支持的provider: {}", provider);
            println!("💡 支持的provider: openai, zhipu, ollama");
            println!("   设置环境变量: export LLM_PROVIDER=ollama");
            std::process::exit(1);
        }
    };

    // 选择合适的模型
    let model = match provider.as_str() {
        "openai" => "gpt-3.5-turbo".to_string(),
        "zhipu" => "glm-4".to_string(),
        "ollama" => std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama2".to_string()),
        _ => unreachable!(),
    };

    // 构建聊天请求
    let request = ChatRequest {
        model: model.clone(),
        messages: vec![
            Message {
                role: Role::User,
                content: "请写一首关于编程的短诗，要有创意和幽默感。".to_string(),
                ..Default::default()
            }
        ],
        max_tokens: Some(300),
        temperature: Some(0.8),
        ..Default::default()
    };

    println!("🚀 开始流式对话...");
    println!("🔧 Provider: {}", provider);
    println!("📝 模型: {}", model);
    println!("💬 消息: {}", request.messages[0].content);
    println!();
    println!("📡 流式响应:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 发送流式请求
    match client.chat_stream(&request).await {
        Ok(mut stream) => {
            let mut full_content = String::new();
            
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if let Some(content) = chunk.get_content() {
                            print!("{}", content);
                            full_content.push_str(&content);
                            std::io::Write::flush(&mut std::io::stdout()).unwrap();
                        }
                    }
                    Err(e) => {
                        println!("\n❌ 流式响应错误: {}", e);
                        break;
                    }
                }
            }
            
            println!();
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("✅ 流式响应完成");
            println!("📊 总字符数: {}", full_content.len());
        }
        Err(e) => {
            println!("❌ 流式请求失败: {}", e);
            println!();
            println!("💡 请检查:");
            println!("  1. API密钥是否正确设置");
            println!("  2. 网络连接是否正常");
            println!("  3. 模型是否支持流式响应");
        }
    }

    Ok(())
}

#[cfg(not(feature = "streaming"))]
fn main() {
    println!("❌ 此示例需要启用 streaming 功能");
    println!("请使用: cargo run --example streaming_basic --features streaming");
}
