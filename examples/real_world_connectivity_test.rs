use dotenvy::dotenv;
use llm_connector::{LlmClient, Message};
use llm_providers;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let proxy_url =
        env::var("PROXY_BASE_URL").unwrap_or_else(|_| "http://123.129.219.111:3000/v1".to_string());
    let proxy_key = env::var("PROXY_API_KEY").unwrap_or_default();

    println!("--- Testing OpenAI (via Proxy) ---");
    if let Err(e) = test_openai(&proxy_url, &proxy_key).await {
        println!("❌ OpenAI Error: {:?}", e);
    }

    println!("\n--- Testing Anthropic (via Proxy) ---");
    if let Err(e) = test_anthropic(&proxy_url, &proxy_key).await {
        println!("❌ Anthropic Error: {:?}", e);
    }

    if let Ok(zhipu_key) = env::var("ZHIPU_API_KEY") {
        if !zhipu_key.contains("your_") {
            println!("\n--- Testing Zhipu Native ---");
            if let Err(e) = test_zhipu(&zhipu_key).await {
                println!("❌ Zhipu Error: {:?}", e);
            }
        }
    }

    if let Ok(aliyun_key) = env::var("ALIYUN_API_KEY") {
        if !aliyun_key.contains("your_") {
            let aliyun_url = env::var("ALIYUN_BASE_URL")
                .unwrap_or_else(|_| "https://dashscope.aliyuncs.com".to_string());
            println!("\n--- Testing Aliyun Native ---");
            if let Err(e) = test_aliyun(&aliyun_key, &aliyun_url).await {
                println!("❌ Aliyun Error: {:?}", e);
            }
        }
    }

    if let Ok(tencent_id) = env::var("TENCENT_SECRET_ID") {
        if let Ok(tencent_key) = env::var("TENCENT_SECRET_KEY") {
            if !tencent_id.contains("your_") {
                let tencent_url = env::var("TENCENT_BASE_URL")
                    .unwrap_or_else(|_| "hunyuan.tencentcloudapi.com".to_string());
                println!("\n--- Testing Tencent Native ---");
                if let Err(e) = test_tencent(&tencent_id, &tencent_key, &tencent_url).await {
                    println!("❌ Tencent Error: {:?}", e);
                }
            }
        }
    }

    // DeepSeek via OpenAI compatible client
    if let Ok(deepseek_key) = env::var("DEEPSEEK_API_KEY") {
        let deepseek_url = env::var("DEEPSEEK_BASE_URL")
            .unwrap_or_else(|_| "https://api.deepseek.com".to_string());
        println!("\n--- Testing DeepSeek ---");
        if let Err(e) = test_openai_compatible(&deepseek_url, &deepseek_key, "deepseek-chat").await
        {
            println!("❌ DeepSeek Error: {:?}", e);
        }
    }

    // MiniMax via OpenAI compatible client
    if let Ok(minimax_key) = env::var("MINIMAX_API_KEY") {
        let minimax_url = env::var("MINIMAX_BASE_URL")
            .unwrap_or_else(|_| "https://api.minimax.io/v1".to_string());
        println!("\n--- Testing MiniMax ---");
        if let Err(e) = test_openai_compatible(&minimax_url, &minimax_key, "abab6.5s-chat").await {
            println!("❌ MiniMax Error: {:?}", e);
        }
    }

    Ok(())
}

async fn test_openai(url: &str, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::openai(key, url)?;
    let request =
        llm_connector::ChatRequest::new("gpt-4o").add_message(Message::user("Hello, who are you?"));

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
    let url = env::var("ZHIPU_BASE_URL")
        .unwrap_or_else(|_| "https://open.bigmodel.cn/api/paas/v4".to_string());
    let client = LlmClient::zhipu(key, &url)?;
    let request = llm_connector::ChatRequest::new("glm-4")
        .add_message(Message::user("Hello Zhipu, are you there?"));

    let response = client.chat(&request).await?;
    println!("Response: {}", response.content);
    Ok(())
}

async fn test_aliyun(key: &str, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::aliyun(key, url)?;
    let request = llm_connector::ChatRequest::new("qwen-max")
        .add_message(Message::user("Hello Qwen, say something nice."));

    let response = client.chat(&request).await?;
    println!("Response: {}", response.content);
    Ok(())
}

async fn test_tencent(id: &str, key: &str, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "tencent")]
    {
        let client = LlmClient::tencent(id, key, url)?;
        let request = llm_connector::ChatRequest::new("hunyuan-lite").add_message(Message::user(
            "Hello Tencent, describe the city of Shenzhen.",
        ));

        let response = client.chat(&request).await?;
        println!("Response: {}", response.content);
    }
    #[cfg(not(feature = "tencent"))]
    {
        let _ = (id, key, url);
        println!("Skipping Tencent test (feature not enabled)");
    }
    Ok(())
}

async fn test_openai_compatible(
    url: &str,
    key: &str,
    model: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = LlmClient::openai(key, url)?;
    let request =
        llm_connector::ChatRequest::new(model).add_message(Message::user("Hello, simple test."));

    let response = client.chat(&request).await?;
    println!("{}: Response: {}", model, response.content);
    Ok(())
}
