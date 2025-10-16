//! Ollama基础示例
//!
//! 展示如何使用本地Ollama服务进行基本的聊天对话
//!
//! 运行方式: cargo run --example ollama_basic

use llm_connector::{LlmClient, types::{ChatRequest, Message}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Ollama本地模型基础聊天示例\n");

    // 创建Ollama客户端 (默认连接到 http://localhost:11434)
    let client = LlmClient::ollama().unwrap();

    // 获取可用模型列表
    println!("🔍 获取可用模型列表...");
    match client.models().await {
        Ok(models) => {
            if models.is_empty() {
                println!("❌ 没有找到可用的模型");
                println!("💡 请先下载模型，例如:");
                println!("   ollama pull llama2");
                println!("   ollama pull qwen:7b");
                return Ok(());
            }
            
            println!("✅ 找到 {} 个可用模型:", models.len());
            for (i, model) in models.iter().enumerate() {
                println!("  {}. {}", i + 1, model);
            }
            println!();
        }
        Err(e) => {
            println!("❌ 获取模型列表失败: {}", e);
            println!("💡 请检查:");
            println!("  1. Ollama服务是否正在运行 (ollama serve)");
            println!("  2. 服务地址是否正确 (默认: http://localhost:11434)");
            return Ok(());
        }
    }

    // 使用第一个可用模型或默认模型
    let model = std::env::var("OLLAMA_MODEL")
        .unwrap_or_else(|_| "llama2".to_string());

    // 构建聊天请求
    let request = ChatRequest {
        model: model.clone(),
        messages: vec![
            Message::user("请简要介绍一下你自己，以及你能帮助我做什么。")
        ],
        max_tokens: Some(200),
        temperature: Some(0.7),
        ..Default::default()
    };

    println!("🚀 发送请求到Ollama...");
    println!("📝 模型: {}", request.model);
    println!("💬 消息: {}", request.messages[0].content);
    println!();

    // 发送请求
    match client.chat(&request).await {
        Ok(response) => {
            println!("✅ 成功收到回复:");
            println!("{}", response.content);
            println!();
            println!("📊 Token使用情况:");
            println!("  输入: {} tokens", response.prompt_tokens());
            println!("  输出: {} tokens", response.completion_tokens());
            println!("  总计: {} tokens", response.total_tokens());
        }
        Err(e) => {
            println!("❌ 请求失败: {}", e);
            println!();
            println!("💡 请检查:");
            println!("  1. Ollama服务是否正在运行");
            println!("  2. 模型 '{}' 是否已下载", model);
            println!("  3. 网络连接是否正常");
            println!();
            println!("🔧 常用命令:");
            println!("  ollama serve          # 启动Ollama服务");
            println!("  ollama pull {}   # 下载模型", model);
            println!("  ollama list           # 查看已下载的模型");
        }
    }

    Ok(())
}
