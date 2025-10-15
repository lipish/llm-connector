//! Zhipu 稳定性测试示例
//!
//! 专门用于测试和诊断 Zhipu API 的稳定性问题

use llm_connector::{LlmClient, types::{ChatRequest, Message}, error::LlmConnectorError};
use std::time::{Duration, Instant};
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Zhipu 稳定性测试工具\n");

    let api_key = std::env::var("ZHIPU_API_KEY")
        .expect("请设置环境变量 ZHIPU_API_KEY");

    println!("API Key: {}...", &api_key[..8.min(api_key.len())]);

    // 启用调试模式
    std::env::set_var("LLM_DEBUG_REQUEST_RAW", "1");
    std::env::set_var("LLM_DEBUG_RESPONSE_RAW", "1");

    // 测试1: 基本连接测试
    println!("\n📋 测试1: 基本连接测试");
    test_basic_connection(&api_key).await;

    // 测试2: 不同超时设置测试
    println!("\n📋 测试2: 不同超时设置测试");
    test_different_timeouts(&api_key).await;

    // 测试3: 并发请求测试
    println!("\n📋 测试3: 并发请求测试");
    test_concurrent_requests(&api_key).await;

    // 测试4: 长时间运行测试
    println!("\n📋 测试4: 长时间运行测试");
    test_long_running(&api_key).await;

    // 测试5: 流式响应稳定性测试
    #[cfg(feature = "streaming")]
    {
        println!("\n📋 测试5: 流式响应稳定性测试");
        test_streaming_stability(&api_key).await;
    }

    println!("\n🎯 所有测试完成！");
    Ok(())
}

/// 基本连接测试
async fn test_basic_connection(api_key: &str) {
    let client = LlmClient::zhipu_with_timeout(api_key, 10000); // 10秒超时
    
    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: vec![Message::user("测试连接")],
        max_tokens: Some(10),
        ..Default::default()
    };

    let start = Instant::now();
    match client.chat(&request).await {
        Ok(response) => {
            println!("   ✅ 基本连接成功 ({:?})", start.elapsed());
            println!("   响应: {}", response.choices[0].message.content);
        }
        Err(e) => {
            println!("   ❌ 基本连接失败 ({:?}): {}", start.elapsed(), e);
            analyze_error(&e);
        }
    }
}

/// 测试不同超时设置
async fn test_different_timeouts(api_key: &str) {
    let timeout_configs = vec![
        ("5秒", 5000),
        ("15秒", 15000),
        ("30秒", 30000),
        ("60秒", 60000),
    ];

    for (name, timeout_ms) in timeout_configs {
        println!("   测试超时设置: {}", name);
        let client = LlmClient::zhipu_with_timeout(api_key, timeout_ms);
        
        let request = ChatRequest {
            model: "glm-4-flash".to_string(),
            messages: vec![Message::user("简单回答：你好")],
            max_tokens: Some(20),
            ..Default::default()
        };

        let start = Instant::now();
        match client.chat(&request).await {
            Ok(_) => {
                println!("     ✅ {} 超时测试成功 ({:?})", name, start.elapsed());
            }
            Err(e) => {
                println!("     ❌ {} 超时测试失败 ({:?}): {}", name, start.elapsed(), e);
                if matches!(e, LlmConnectorError::TimeoutError(_)) {
                    println!("     ⏰ 确认为超时错误");
                }
            }
        }
    }
}

/// 并发请求测试
async fn test_concurrent_requests(api_key: &str) {
    let client = LlmClient::zhipu_with_timeout(api_key, 20000);
    let concurrent_count = 3;

    println!("   启动 {} 个并发请求...", concurrent_count);

    let mut handles = Vec::new();
    for i in 0..concurrent_count {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            let request = ChatRequest {
                model: "glm-4-flash".to_string(),
                messages: vec![Message::user(&format!("并发测试 #{}", i + 1))],
                max_tokens: Some(15),
                ..Default::default()
            };

            let start = Instant::now();
            let result = client.chat(&request).await;
            (i + 1, start.elapsed(), result)
        });
        handles.push(handle);
    }

    let mut success_count = 0;
    let mut failure_count = 0;

    for handle in handles {
        match handle.await {
            Ok((id, duration, Ok(_))) => {
                println!("     ✅ 并发请求 #{} 成功 ({:?})", id, duration);
                success_count += 1;
            }
            Ok((id, duration, Err(e))) => {
                println!("     ❌ 并发请求 #{} 失败 ({:?}): {}", id, duration, e);
                failure_count += 1;
            }
            Err(e) => {
                println!("     💥 并发请求任务失败: {}", e);
                failure_count += 1;
            }
        }
    }

    println!("   并发测试结果: 成功 {}, 失败 {}", success_count, failure_count);
}

/// 长时间运行测试
async fn test_long_running(api_key: &str) {
    let client = LlmClient::zhipu_with_timeout(api_key, 15000);
    let test_duration = Duration::from_secs(60); // 1分钟测试
    let interval = Duration::from_secs(10); // 每10秒一次请求

    println!("   开始长时间运行测试 ({}秒)...", test_duration.as_secs());

    let start_time = Instant::now();
    let mut request_count = 0;
    let mut success_count = 0;
    let mut failure_count = 0;

    while start_time.elapsed() < test_duration {
        request_count += 1;
        
        let request = ChatRequest {
            model: "glm-4-flash".to_string(),
            messages: vec![Message::user(&format!("长时间测试 #{}", request_count))],
            max_tokens: Some(10),
            ..Default::default()
        };

        let request_start = Instant::now();
        match timeout(Duration::from_secs(20), client.chat(&request)).await {
            Ok(Ok(_)) => {
                println!("     ✅ 请求 #{} 成功 ({:?})", request_count, request_start.elapsed());
                success_count += 1;
            }
            Ok(Err(e)) => {
                println!("     ❌ 请求 #{} 失败 ({:?}): {}", request_count, request_start.elapsed(), e);
                failure_count += 1;
            }
            Err(_) => {
                println!("     ⏰ 请求 #{} 超时 ({:?})", request_count, request_start.elapsed());
                failure_count += 1;
            }
        }

        // 等待下一次请求
        if start_time.elapsed() < test_duration {
            tokio::time::sleep(interval).await;
        }
    }

    println!("   长时间测试结果:");
    println!("     总请求数: {}", request_count);
    println!("     成功: {} ({:.1}%)", success_count, (success_count as f64 / request_count as f64) * 100.0);
    println!("     失败: {} ({:.1}%)", failure_count, (failure_count as f64 / request_count as f64) * 100.0);
}

/// 流式响应稳定性测试
#[cfg(feature = "streaming")]
async fn test_streaming_stability(api_key: &str) {
    use futures_util::StreamExt;

    let client = LlmClient::zhipu_with_timeout(api_key, 20000);
    
    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: vec![Message::user("请写一首关于春天的短诗")],
        max_tokens: Some(100),
        ..Default::default()
    };

    println!("   开始流式响应测试...");
    let start = Instant::now();
    
    match client.chat_stream(&request).await {
        Ok(mut stream) => {
            let mut chunk_count = 0;
            let mut total_content = String::new();
            
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        chunk_count += 1;
                        if let Some(content) = chunk.get_content() {
                            total_content.push_str(content);
                            print!("{}", content);
                        }
                    }
                    Err(e) => {
                        println!("\n     ❌ 流式响应错误: {}", e);
                        break;
                    }
                }
            }
            
            println!("\n   ✅ 流式响应完成 ({:?})", start.elapsed());
            println!("     接收到 {} 个数据块", chunk_count);
            println!("     总内容长度: {} 字符", total_content.len());
        }
        Err(e) => {
            println!("   ❌ 流式响应启动失败 ({:?}): {}", start.elapsed(), e);
            analyze_error(&e);
        }
    }
}

/// 分析错误并提供建议
fn analyze_error(error: &LlmConnectorError) {
    println!("     🔍 错误分析:");
    match error {
        LlmConnectorError::TimeoutError(_) => {
            println!("       - 这是超时错误，可能的原因:");
            println!("         1. 网络延迟过高");
            println!("         2. Zhipu 服务器响应慢");
            println!("         3. 超时设置过短");
            println!("       - 建议: 增加超时时间或检查网络连接");
        }
        LlmConnectorError::ConnectionError(_) => {
            println!("       - 这是连接错误，可能的原因:");
            println!("         1. 网络连接问题");
            println!("         2. Zhipu 服务器不可达");
            println!("         3. DNS 解析问题");
            println!("       - 建议: 检查网络连接和防火墙设置");
        }
        LlmConnectorError::AuthenticationError(_) => {
            println!("       - 这是认证错误，可能的原因:");
            println!("         1. API Key 无效或过期");
            println!("         2. 账户余额不足");
            println!("         3. API Key 权限不足");
            println!("       - 建议: 检查 API Key 和账户状态");
        }
        LlmConnectorError::RateLimitError(_) => {
            println!("       - 这是频率限制错误");
            println!("       - 建议: 降低请求频率或升级账户");
        }
        _ => {
            println!("       - 其他错误类型: {}", error);
            println!("       - 建议: 启用详细调试日志查看更多信息");
        }
    }
}
