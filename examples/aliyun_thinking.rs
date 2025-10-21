//! Aliyun enable_thinking 参数测试示例
//!
//! 测试 Aliyun 混合推理模式的 enable_thinking 参数

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量获取 API key
    let api_key = std::env::var("ALIYUN_API_KEY")
        .expect("ALIYUN_API_KEY environment variable not set");

    println!("🧪 测试 Aliyun enable_thinking 参数");
    println!("{}", "=".repeat(80));

    // 创建客户端
    let client = LlmClient::aliyun(&api_key)?;

    println!("\n📝 测试 1: 混合推理模型 + 显式启用");
    println!("{}", "-".repeat(80));
    println!("模型: qwen-plus");
    println!("enable_thinking: Some(true)（显式启用）");
    println!("预期: 返回 reasoning_content");

    let request = ChatRequest {
        model: "qwen-plus".to_string(),
        messages: vec![Message::text(Role::User, "9.11 和 9.9 哪个更大？请详细解释你的推理过程。")],
        enable_thinking: Some(true),  // 显式启用
        max_tokens: Some(500),
        ..Default::default()
    };

    println!("\n📤 发送请求...");

    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ 请求成功！");
            
            if let Some(reasoning) = response.reasoning_content {
                println!("\n🧠 推理过程:");
                println!("{}", "-".repeat(80));
                println!("{}", reasoning);
                println!("{}", "-".repeat(80));
                println!("✅ 成功返回 reasoning_content（显式启用生效）");
            } else {
                println!("\n⚠️  未返回 reasoning_content");
                println!("   可能原因:");
                println!("   1. 模型不支持推理模式");
                println!("   2. API 配置问题");
            }
            
            println!("\n💡 最终答案:");
            println!("{}", response.content);

            if let Some(usage) = response.usage {
                println!("\n📊 Usage:");
                println!("   prompt_tokens: {}", usage.prompt_tokens);
                println!("   completion_tokens: {}", usage.completion_tokens);
                println!("   total_tokens: {}", usage.total_tokens);
            }
        }
        Err(e) => {
            eprintln!("\n❌ 错误: {}", e);
            return Err(e.into());
        }
    }

    println!("\n\n📝 测试 2: 混合推理模型 + 未指定（默认不启用）");
    println!("{}", "-".repeat(80));
    println!("模型: qwen-plus");
    println!("enable_thinking: None（未指定）");
    println!("预期: 不返回 reasoning_content");

    let request = ChatRequest {
        model: "qwen-plus".to_string(),
        messages: vec![Message::text(Role::User, "如果一个数的平方是 144，这个数是多少？")],
        // enable_thinking 未指定
        max_tokens: Some(500),
        ..Default::default()
    };

    println!("\n📤 发送请求...");

    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ 请求成功！");

            if response.reasoning_content.is_none() {
                println!("\n✅ 正确：未返回 reasoning_content（默认不启用）");
            } else {
                println!("\n⚠️  意外：返回了 reasoning_content");
            }

            println!("\n💡 答案:");
            println!("{}", response.content);
        }
        Err(e) => {
            eprintln!("\n❌ 错误: {}", e);
            return Err(e.into());
        }
    }

    println!("\n\n📝 测试 3: 混合推理模型 + 显式禁用");
    println!("{}", "-".repeat(80));
    println!("模型: qwen-plus");
    println!("enable_thinking: Some(false)（显式禁用）");
    println!("预期: 不返回 reasoning_content");

    let request = ChatRequest {
        model: "qwen-plus".to_string(),
        messages: vec![Message::text(Role::User, "你好，请介绍一下你自己")],
        enable_thinking: Some(false),  // 手动禁用
        max_tokens: Some(100),
        ..Default::default()
    };

    println!("\n📤 发送请求...");

    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ 请求成功！");
            
            if response.reasoning_content.is_none() {
                println!("\n✅ 正确：未返回 reasoning_content（显式禁用生效）");
            } else {
                println!("\n⚠️  意外：返回了 reasoning_content");
            }
            
            println!("\n💡 答案:");
            println!("{}", response.content);
        }
        Err(e) => {
            eprintln!("\n❌ 错误: {}", e);
            return Err(e.into());
        }
    }

    println!("\n\n📝 测试 4: 纯推理模型（无需配置）");
    println!("{}", "-".repeat(80));
    println!("模型: qwq-plus");
    println!("enable_thinking: None（纯推理模型默认启用）");
    println!("预期: 返回 reasoning_content");

    let request = ChatRequest {
        model: "qwq-plus".to_string(),
        messages: vec![Message::text(Role::User, "解释为什么天空是蓝色的")],
        max_tokens: Some(500),
        ..Default::default()
    };

    println!("\n📤 发送请求...");

    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ 请求成功！");
            
            if let Some(reasoning) = response.reasoning_content {
                println!("\n🧠 推理过程:");
                println!("{}", "-".repeat(80));
                println!("{}...", &reasoning[..reasoning.len().min(200)]);
                println!("{}", "-".repeat(80));
                println!("✅ 成功返回 reasoning_content（纯推理模型）");
            } else {
                println!("\n⚠️  未返回 reasoning_content");
            }
            
            println!("\n💡 最终答案:");
            println!("{}", response.content);
        }
        Err(e) => {
            eprintln!("\n❌ 错误: {}", e);
            return Err(e.into());
        }
    }

    println!("\n\n📝 测试 5: 非推理模型");
    println!("{}", "-".repeat(80));
    println!("模型: qwen-max");
    println!("enable_thinking: None（非推理模型）");
    println!("预期: 不返回 reasoning_content");

    let request = ChatRequest {
        model: "qwen-max".to_string(),
        messages: vec![Message::text(Role::User, "你好")],
        max_tokens: Some(50),
        ..Default::default()
    };

    println!("\n📤 发送请求...");

    match client.chat(&request).await {
        Ok(response) => {
            println!("\n✅ 请求成功！");
            
            if response.reasoning_content.is_none() {
                println!("\n✅ 正确：未返回 reasoning_content（非推理模型）");
            } else {
                println!("\n⚠️  意外：返回了 reasoning_content");
            }
            
            println!("\n💡 答案:");
            println!("{}", response.content);
        }
        Err(e) => {
            eprintln!("\n❌ 错误: {}", e);
            return Err(e.into());
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("✅ Aliyun enable_thinking 参数测试完成！");
    println!("{}", "=".repeat(80));

    println!("\n📝 总结:");
    println!("   1. 混合推理模型（qwen-plus 等）:");
    println!("      - 需要显式设置 enable_thinking: Some(true)");
    println!("      - 未设置时默认不启用推理模式");
    println!("   2. 纯推理模型（qwq-plus 等）:");
    println!("      - 默认启用，无需配置");
    println!("   3. 非推理模型（qwen-max 等）:");
    println!("      - 不支持 enable_thinking");
    println!("   4. 统一的 API:");
    println!("      - response.reasoning_content - 推理过程");
    println!("      - response.content - 最终答案");
    println!("   5. 显式控制:");
    println!("      - 用户完全控制是否启用推理模式");
    println!("      - 无自动检测，行为明确可预测");

    Ok(())
}

