//! Test proxy configuration issue
//!
//! This example tests whether proxy configuration causes timeout issues.
//! 
//! Run with:
//! ```bash
//! cargo run --example test_proxy_issue --features streaming
//! ```

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Proxy Configuration Issue\n");
    
    let api_key = "6b4c24a7a3df47a8898b006f9f5c23b6.PXpYUIvTdUU9uKPS";
    
    // Test 1: Without proxy (default)
    println!("📝 Test 1: Without proxy (default)");
    println!("===================================");
    test_without_proxy(api_key).await?;
    
    println!("\n");
    
    // Test 2: With invalid proxy (should fail or timeout)
    println!("📝 Test 2: With invalid proxy");
    println!("==============================");
    test_with_invalid_proxy(api_key).await;
    
    println!("\n");
    
    // Test 3: Check if system proxy is being used
    println!("📝 Test 3: System proxy check");
    println!("==============================");
    check_system_proxy();
    
    Ok(())
}

async fn test_without_proxy(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create client without explicit proxy
    let client = LlmClient::zhipu_openai_compatible(api_key)?;
    
    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: vec![Message::text(Role::User, "你好")],
        max_tokens: Some(50),
        stream: Some(false),
        ..Default::default()
    };
    
    let start = Instant::now();
    
    match client.chat(&request).await {
        Ok(response) => {
            let elapsed = start.elapsed();
            println!("✅ Request succeeded without proxy");
            println!("⏱️  Time: {:?}", elapsed);
            println!("📊 Response length: {} chars", response.content.len());
        }
        Err(e) => {
            let elapsed = start.elapsed();
            println!("❌ Request failed without proxy");
            println!("⏱️  Time: {:?}", elapsed);
            println!("🔴 Error: {}", e);
            
            // Check if it's a proxy-related error
            let error_str = e.to_string();
            if error_str.contains("proxy") || error_str.contains("Proxy") {
                println!("⚠️  This appears to be a PROXY-related error!");
            }
            
            return Err(e.into());
        }
    }
    
    Ok(())
}

async fn test_with_invalid_proxy(api_key: &str) {
    // Create client with invalid proxy
    let result = LlmClient::zhipu_with_config(
        api_key,
        true,  // OpenAI compatible
        None,  // Default base URL
        Some(10),  // Short timeout to fail fast
        Some("http://invalid-proxy:9999"),  // Invalid proxy
    );
    
    match result {
        Ok(client) => {
            println!("✅ Client created with invalid proxy");
            
            let request = ChatRequest {
                model: "glm-4-flash".to_string(),
                messages: vec![Message::text(Role::User, "你好")],
                max_tokens: Some(50),
                stream: Some(false),
                ..Default::default()
            };
            
            let start = Instant::now();
            
            match client.chat(&request).await {
                Ok(_) => {
                    println!("⚠️  Unexpected: Request succeeded with invalid proxy!");
                }
                Err(e) => {
                    let elapsed = start.elapsed();
                    println!("❌ Request failed with invalid proxy (expected)");
                    println!("⏱️  Time: {:?}", elapsed);
                    println!("🔴 Error: {}", e);
                    
                    // Check error type
                    let error_str = e.to_string();
                    if error_str.contains("timeout") || error_str.contains("Timeout") {
                        println!("⚠️  This is a TIMEOUT error!");
                    } else if error_str.contains("proxy") || error_str.contains("Proxy") {
                        println!("⚠️  This is a PROXY error!");
                    } else if error_str.contains("connection") || error_str.contains("Connection") {
                        println!("⚠️  This is a CONNECTION error!");
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create client with invalid proxy");
            println!("🔴 Error: {}", e);
        }
    }
}

fn check_system_proxy() {
    // Check environment variables for proxy settings
    let http_proxy = std::env::var("HTTP_PROXY").or_else(|_| std::env::var("http_proxy"));
    let https_proxy = std::env::var("HTTPS_PROXY").or_else(|_| std::env::var("https_proxy"));
    let all_proxy = std::env::var("ALL_PROXY").or_else(|_| std::env::var("all_proxy"));
    let no_proxy = std::env::var("NO_PROXY").or_else(|_| std::env::var("no_proxy"));
    
    println!("Environment proxy settings:");
    
    match http_proxy {
        Ok(proxy) => println!("  HTTP_PROXY: {}", proxy),
        Err(_) => println!("  HTTP_PROXY: (not set)"),
    }
    
    match https_proxy {
        Ok(proxy) => println!("  HTTPS_PROXY: {}", proxy),
        Err(_) => println!("  HTTPS_PROXY: (not set)"),
    }
    
    match all_proxy {
        Ok(proxy) => println!("  ALL_PROXY: {}", proxy),
        Err(_) => println!("  ALL_PROXY: (not set)"),
    }
    
    match no_proxy {
        Ok(proxy) => println!("  NO_PROXY: {}", proxy),
        Err(_) => println!("  NO_PROXY: (not set)"),
    }
    
    println!("\n⚠️  Note: reqwest may use system proxy settings by default!");
    println!("⚠️  This could cause timeout issues if the proxy is slow or unreachable.");
}

