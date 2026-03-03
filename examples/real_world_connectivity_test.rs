use dotenvy::dotenv;
use llm_connector::{LlmClient, Message};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let proxy_url = env::var("PROXY_BASE_URL").unwrap_or_else(|_| "http://123.129.219.111:3000/v1".to_string());
    let proxy_key = env::var("PROXY_API_KEY").unwrap_or_default();

    println!("--- Testing OpenAI (via Proxy) ---");
    test_openai(&proxy_url, &proxy_key).await?;

    println!("\n--- Testing Anthropic (via Proxy) ---");
    test_anthropic(&proxy_url, &proxy_key).await?;

    if let Ok(zhipu_key) = env::var("ZHIPU_API_KEY") {
        if !zhipu_key.contains("your_") {
            println!("\n--- Testing Zhipu Native ---");
            test_zhipu(&zhipu_key).await?;
        }
    }

    if let Ok(aliyun_key) = env::var("ALIYUN_API_KEY") {
        if !aliyun_key.contains("your_") {
            println!("\n--- Testing Aliyun Native ---");
            test_aliyun(&aliyun_key).await?;
        }
    }

    Ok(())
}

async fn test_openai(url: &str, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::openai(key, url)?;
    let request = llm_connector::ChatRequest::new("gpt-4o")
        .add_message(Message::user("Hello, who are you?"));

    let response = client.chat(&request).await?;
    println!("Response: {}", response.content);
    Ok(())
}

async fn test_anthropic(url: &str, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Note: The proxy might expect OpenAI format even for Claude models, 
    // or it might pass through the Anthropic protocol. 
    // Usually these proxies are OpenAI-compatible gateways.
    // If the proxy is OpenAI-compatible, we should use LlmClient::openai even for Claude models.
    
    let client = LlmClient::openai(key, url)?;
    let request = llm_connector::ChatRequest::new("claude-sonnet-4-20250514")
        .add_message(Message::user("Hello Claude, what's your version?"));

    let response = client.chat(&request).await?;
    println!("Response: {}", response.content);
    Ok(())
}

async fn test_zhipu(key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::zhipu(key, "https://open.bigmodel.cn/api/paas/v4")?;
    let request = llm_connector::ChatRequest::new("glm-4")
        .add_message(Message::user("Hello Zhipu, are you there?"));

    let response = client.chat(&request).await?;
    println!("Response: {}", response.content);
    Ok(())
}

async fn test_aliyun(key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::aliyun(key, "https://dashscope.aliyuncs.com")?;
    let request = llm_connector::ChatRequest::new("qwen-max")
        .add_message(Message::user("Hello Qwen, say something nice."));

    let response = client.chat(&request).await?;
    println!("Response: {}", response.content);
    Ok(())
}
