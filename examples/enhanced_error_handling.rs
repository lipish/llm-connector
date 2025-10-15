//! 增强的错误处理和调试示例
//!
//! 展示如何使用 llm-connector 的增强错误处理、超时配置和调试功能

use llm_connector::{LlmClient, types::{ChatRequest, Message}, error::LlmConnectorError};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 llm-connector 增强错误处理示例\n");

    // 设置调试环境变量（可选）
    std::env::set_var("LLM_DEBUG_REQUEST_RAW", "1");
    std::env::set_var("LLM_DEBUG_RESPONSE_RAW", "1");

    // 示例1: 使用自定义超时的 Zhipu 客户端
    println!("📋 示例1: 自定义超时配置");
    let api_key = std::env::var("ZHIPU_API_KEY")
        .unwrap_or_else(|_| "sk-test-key".to_string());

    // 创建带有5秒超时的客户端（用于演示超时）
    let client = LlmClient::zhipu_with_timeout(&api_key, 5000);
    println!("✅ 创建 Zhipu 客户端，超时设置: 5秒");

    let request = ChatRequest {
        model: "glm-4-flash".to_string(),
        messages: vec![Message::user("Hello!")],
        max_tokens: Some(50),
        ..Default::default()
    };

    // 示例2: 增强的错误处理
    println!("\n📋 示例2: 增强错误处理");
    match client.chat(&request).await {
        Ok(response) => {
            println!("✅ 请求成功!");
            println!("   响应: {}", response.choices[0].message.content);
        }
        Err(e) => {
            println!("❌ 请求失败，详细错误信息:");
            print_detailed_error(&e);
        }
    }

    // 示例3: 模型列表获取（如果支持）
    println!("\n📋 示例3: 模型列表获取");
    match client.fetch_models().await {
        Ok(models) => {
            println!("✅ 获取到 {} 个模型", models.len());
            for model in models.iter().take(5) {
                println!("   - {}", model);
            }
            if models.len() > 5 {
                println!("   ... 还有 {} 个模型", models.len() - 5);
            }
        }
        Err(e) => {
            println!("ℹ️  模型列表获取失败（可能不支持）: {}", e);
        }
    }

    // 示例4: 使用 tokio::timeout 进行额外的超时控制
    println!("\n📋 示例4: 应用层超时控制");
    let timeout_duration = Duration::from_secs(10);
    
    match tokio::time::timeout(timeout_duration, client.chat(&request)).await {
        Ok(Ok(response)) => {
            println!("✅ 在超时时间内完成请求");
            println!("   响应: {}", response.choices[0].message.content);
        }
        Ok(Err(e)) => {
            println!("❌ 请求失败: {}", e);
        }
        Err(_) => {
            println!("⏰ 应用层超时 ({}秒)", timeout_duration.as_secs());
        }
    }

    // 示例5: 不同提供商的超时配置
    println!("\n📋 示例5: 不同提供商的超时配置");
    
    // OpenAI 客户端，60秒超时
    if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
        let openai_client = LlmClient::openai_with_timeout(&openai_key, None, 60000);
        println!("✅ OpenAI 客户端创建成功，超时: 60秒");
        
        match openai_client.fetch_models().await {
            Ok(models) => println!("   OpenAI 模型数量: {}", models.len()),
            Err(e) => println!("   OpenAI 模型获取失败: {}", e),
        }
    } else {
        println!("ℹ️  跳过 OpenAI 测试（未设置 OPENAI_API_KEY）");
    }

    // Anthropic 客户端，45秒超时
    if let Ok(anthropic_key) = std::env::var("ANTHROPIC_API_KEY") {
        let anthropic_client = LlmClient::anthropic_with_timeout(&anthropic_key, 45000);
        println!("✅ Anthropic 客户端创建成功，超时: 45秒");
        
        match anthropic_client.fetch_models().await {
            Ok(models) => println!("   Anthropic 模型数量: {}", models.len()),
            Err(e) => println!("   Anthropic 模型获取失败: {}", e),
        }
    } else {
        println!("ℹ️  跳过 Anthropic 测试（未设置 ANTHROPIC_API_KEY）");
    }

    println!("\n🎯 示例完成！");
    println!("\n💡 调试提示:");
    println!("   - 设置 LLM_DEBUG_REQUEST_RAW=1 查看请求详情");
    println!("   - 设置 LLM_DEBUG_RESPONSE_RAW=1 查看响应详情");
    println!("   - 设置 LLM_DEBUG_STREAM_RAW=1 查看流式响应详情");
    println!("   - 使用 *_with_timeout 方法配置自定义超时");

    Ok(())
}

/// 打印详细的错误信息
fn print_detailed_error(error: &LlmConnectorError) {
    println!("   错误类型: {}", get_error_type_name(error));
    println!("   错误信息: {}", error);
    println!("   HTTP状态码: {}", error.status_code());
    println!("   是否可重试: {}", error.is_retryable());
    
    // 根据错误类型提供具体建议
    match error {
        LlmConnectorError::AuthenticationError(_) => {
            println!("   🔧 解决建议:");
            println!("      1. 检查 API Key 是否正确");
            println!("      2. 确认账户是否有余额");
            println!("      3. 验证 API Key 权限");
        }
        LlmConnectorError::TimeoutError(_) => {
            println!("   🔧 解决建议:");
            println!("      1. 增加超时时间");
            println!("      2. 检查网络连接");
            println!("      3. 尝试使用更快的模型");
        }
        LlmConnectorError::RateLimitError(_) => {
            println!("   🔧 解决建议:");
            println!("      1. 降低请求频率");
            println!("      2. 实现指数退避重试");
            println!("      3. 升级账户限额");
        }
        LlmConnectorError::ConnectionError(_) => {
            println!("   🔧 解决建议:");
            println!("      1. 检查网络连接");
            println!("      2. 验证服务器地址");
            println!("      3. 检查防火墙设置");
        }
        _ => {
            println!("   🔧 解决建议:");
            println!("      1. 查看详细错误信息");
            println!("      2. 启用调试模式");
            println!("      3. 检查请求参数");
        }
    }
}

/// 获取错误类型名称
fn get_error_type_name(error: &LlmConnectorError) -> &'static str {
    match error {
        LlmConnectorError::AuthenticationError(_) => "认证错误",
        LlmConnectorError::RateLimitError(_) => "频率限制",
        LlmConnectorError::NetworkError(_) => "网络错误",
        LlmConnectorError::InvalidRequest(_) => "无效请求",
        LlmConnectorError::UnsupportedModel(_) => "不支持的模型",
        LlmConnectorError::ProviderError(_) => "提供商错误",
        LlmConnectorError::PermissionError(_) => "权限错误",
        LlmConnectorError::NotFoundError(_) => "未找到",
        LlmConnectorError::ServerError(_) => "服务器错误",
        LlmConnectorError::TimeoutError(_) => "超时错误",
        LlmConnectorError::ConnectionError(_) => "连接错误",
        LlmConnectorError::ParseError(_) => "解析错误",
        LlmConnectorError::ConfigError(_) => "配置错误",
        LlmConnectorError::MaxRetriesExceeded(_) => "重试次数超限",
        LlmConnectorError::StreamingNotSupported(_) => "不支持流式",
        LlmConnectorError::UnsupportedOperation(_) => "不支持的操作",
        LlmConnectorError::ApiError(_) => "API错误",
        LlmConnectorError::JsonError(_) => "JSON错误",
        LlmConnectorError::StreamingError(_) => "流式错误",
        #[cfg(feature = "reqwest")]
        LlmConnectorError::HttpError(_) => "HTTP错误",
    }
}
