use llm_connector::{LlmClient, types::EmbedRequest};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_content = fs::read_to_string(".env").unwrap_or_default();
    let mut keys = std::collections::HashMap::new();
    for line in env_content.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            keys.insert(k.trim(), v.trim());
        }
    }

    println!("--- LLM-Connector Embedding Real-World Test ---");

    // 1. OpenAI 测试
    if let Some(&key) = keys.get("OPENAI_API_KEY") {
        if !key.contains("your_") && !key.is_empty() {
            println!("\n[Testing OpenAI]...");
            let base_url = keys.get("OPENAI_BASE_URL").map(|&s| s);
            
            let client = if let Some(url) = base_url {
                println!("Using Custom URL: {}", url);
                LlmClient::openai(key, url)?
            } else {
                LlmClient::openai(key, llm_connector::endpoints::OPENAI_API_V1)?
            };

            let req = EmbedRequest {
                model: "text-embedding-3-small".to_string(),
                input: vec!["Hello from Rust!".to_string()],
                ..Default::default()
            };
            match client.embed(&req).await {
                Ok(resp) => println!("✅ OpenAI Success: Embedding size {}", resp.data[0].embedding.len()),
                Err(e) => println!("❌ OpenAI Error: {}", e),
            }
        }
    }

    // 2. Zhipu 测试 (通常不需要自定义 URL)
    if let Some(&key) = keys.get("ZHIPU_API_KEY") {
        if !key.contains("your_") && !key.is_empty() {
            println!("\n[Testing Zhipu]...");
            let client = LlmClient::zhipu(key, llm_connector::endpoints::ZHIPU_CN_V4)?;
            let req = EmbedRequest {
                model: "embedding-2".to_string(),
                input: vec!["你好，来自 Rust 的测试！".to_string()],
                ..Default::default()
            };
            match client.embed(&req).await {
                Ok(resp) => println!("✅ Zhipu Success: Embedding size {}", resp.data[0].embedding.len()),
                Err(e) => println!("❌ Zhipu Error: {}", e),
            }
        }
    }

    // 3. Aliyun 测试
    if let Some(&key) = keys.get("ALIYUN_API_KEY") {
        if !key.contains("your_") && !key.is_empty() {
            println!("\n[Testing Aliyun]...");
            let client = LlmClient::aliyun(key, llm_connector::endpoints::ALIYUN_DASHSCOPE_V1)?;
            let req = EmbedRequest {
                model: "text-embedding-v1".to_string(),
                input: vec!["阿里云 Embedding 测试".to_string()],
                ..Default::default()
            };
            match client.embed(&req).await {
                Ok(resp) => println!("✅ Aliyun Success: Embedding size {}", resp.data[0].embedding.len()),
                Err(e) => println!("❌ Aliyun Error: {}", e),
            }
        }
    }

    println!("\n--- Test Finished ---");
    Ok(())
}
