//! 验证所有 Providers 的 reasoning_content 支持
//!
//! 这个示例用于验证各个 provider 是否正确提取 reasoning_content

use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 验证 Reasoning Content 支持");
    println!("{}", "=".repeat(80));

    // 测试 DeepSeek
    if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
        println!("\n📝 测试 DeepSeek Reasoner");
        println!("{}", "-".repeat(80));
        
        let client = LlmClient::deepseek(&api_key)?;
        let request = ChatRequest {
            model: "deepseek-reasoner".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: "9.11 和 9.9 哪个更大？".to_string(),
                ..Default::default()
            }],
            max_tokens: Some(500),
            ..Default::default()
        };

        match client.chat(&request).await {
            Ok(response) => {
                println!("✅ DeepSeek 请求成功");
                if let Some(reasoning) = response.reasoning_content {
                    println!("   ✅ reasoning_content 存在");
                    println!("   长度: {} 字符", reasoning.len());
                    println!("   预览: {}...", &reasoning[..reasoning.len().min(100)]);
                } else {
                    println!("   ❌ reasoning_content 为 None");
                }
                println!("   content: {}", response.content);
            }
            Err(e) => {
                println!("❌ DeepSeek 错误: {}", e);
            }
        }
    } else {
        println!("\n⏭️  跳过 DeepSeek (未设置 DEEPSEEK_API_KEY)");
    }

    // 测试 Moonshot
    if let Ok(api_key) = std::env::var("MOONSHOT_API_KEY") {
        println!("\n📝 测试 Moonshot Kimi Thinking");
        println!("{}", "-".repeat(80));
        
        let client = LlmClient::moonshot(&api_key)?;
        let request = ChatRequest {
            model: "kimi-thinking-preview".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: "计算 15 * 23".to_string(),
                ..Default::default()
            }],
            max_tokens: Some(500),
            ..Default::default()
        };

        match client.chat(&request).await {
            Ok(response) => {
                println!("✅ Moonshot 请求成功");
                if let Some(reasoning) = response.reasoning_content {
                    println!("   ✅ reasoning_content 存在");
                    println!("   长度: {} 字符", reasoning.len());
                    println!("   预览: {}...", &reasoning[..reasoning.len().min(100)]);
                } else {
                    println!("   ❌ reasoning_content 为 None");
                }
                println!("   content: {}", response.content);
            }
            Err(e) => {
                println!("❌ Moonshot 错误: {}", e);
            }
        }
    } else {
        println!("\n⏭️  跳过 Moonshot (未设置 MOONSHOT_API_KEY)");
    }

    // 测试 Zhipu GLM-Z1
    if let Ok(api_key) = std::env::var("ZHIPU_API_KEY") {
        println!("\n📝 测试 Zhipu GLM-Z1");
        println!("{}", "-".repeat(80));
        
        let client = LlmClient::zhipu(&api_key)?;
        let request = ChatRequest {
            model: "glm-z1".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: "解释为什么天空是蓝色的".to_string(),
                ..Default::default()
            }],
            max_tokens: Some(500),
            ..Default::default()
        };

        match client.chat(&request).await {
            Ok(response) => {
                println!("✅ Zhipu 请求成功");
                if let Some(reasoning) = response.reasoning_content {
                    println!("   ✅ reasoning_content 存在（从 ###Thinking 提取）");
                    println!("   长度: {} 字符", reasoning.len());
                    println!("   预览: {}...", &reasoning[..reasoning.len().min(100)]);
                } else {
                    println!("   ⚠️  reasoning_content 为 None（可能不是推理模型）");
                }
                println!("   content: {}", response.content);
            }
            Err(e) => {
                println!("❌ Zhipu 错误: {}", e);
            }
        }
    } else {
        println!("\n⏭️  跳过 Zhipu (未设置 ZHIPU_API_KEY)");
    }

    // 测试 Aliyun Qwen
    if let Ok(api_key) = std::env::var("ALIYUN_API_KEY") {
        println!("\n📝 测试 Aliyun Qwen Plus (需要 enable_thinking)");
        println!("{}", "-".repeat(80));
        println!("⚠️  注意: Aliyun 需要在请求中设置 enable_thinking=true");
        println!("   当前实现可能不支持此参数，需要手动测试");
        
        // 注意：当前 llm-connector 可能不支持 enable_thinking 参数
        // 这需要在 Aliyun provider 中添加支持
    } else {
        println!("\n⏭️  跳过 Aliyun (未设置 ALIYUN_API_KEY)");
    }

    // 测试 OpenAI o1
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        println!("\n📝 测试 OpenAI o1");
        println!("{}", "-".repeat(80));
        
        let client = LlmClient::openai(&api_key)?;
        let request = ChatRequest {
            model: "o1-mini".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: "如果一个数的平方是 144，这个数是多少？".to_string(),
                ..Default::default()
            }],
            max_tokens: Some(500),
            ..Default::default()
        };

        match client.chat(&request).await {
            Ok(response) => {
                println!("✅ OpenAI 请求成功");
                if let Some(reasoning) = response.reasoning_content {
                    println!("   ✅ reasoning_content 存在");
                    println!("   长度: {} 字符", reasoning.len());
                    println!("   预览: {}...", &reasoning[..reasoning.len().min(100)]);
                } else {
                    println!("   ⚠️  reasoning_content 为 None");
                    println!("   注意: OpenAI 可能已移除 reasoning_content 字段");
                }
                println!("   content: {}", response.content);
            }
            Err(e) => {
                println!("❌ OpenAI 错误: {}", e);
            }
        }
    } else {
        println!("\n⏭️  跳过 OpenAI (未设置 OPENAI_API_KEY)");
    }

    // 测试非推理模型（应该返回 None）
    println!("\n📝 测试非推理模型（应该返回 reasoning_content = None）");
    println!("{}", "-".repeat(80));
    
    if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
        let client = LlmClient::deepseek(&api_key)?;
        let request = ChatRequest {
            model: "deepseek-chat".to_string(),  // 非推理模型
            messages: vec![Message {
                role: Role::User,
                content: "你好".to_string(),
                ..Default::default()
            }],
            max_tokens: Some(50),
            ..Default::default()
        };

        match client.chat(&request).await {
            Ok(response) => {
                println!("✅ DeepSeek Chat 请求成功");
                if response.reasoning_content.is_none() {
                    println!("   ✅ reasoning_content 正确为 None（非推理模型）");
                } else {
                    println!("   ⚠️  reasoning_content 不为 None（意外）");
                }
            }
            Err(e) => {
                println!("❌ DeepSeek Chat 错误: {}", e);
            }
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("✅ 验证完成！");
    println!("{}", "=".repeat(80));

    println!("\n📊 总结:");
    println!("   - DeepSeek Reasoner: 应该有 reasoning_content");
    println!("   - Moonshot Kimi Thinking: 应该有 reasoning_content");
    println!("   - Zhipu GLM-Z1: 应该有 reasoning_content（从 ###Thinking 提取）");
    println!("   - Aliyun Qwen Plus: 需要 enable_thinking=true");
    println!("   - OpenAI o1: 可能有 reasoning_content（取决于 API 版本）");
    println!("   - 非推理模型: reasoning_content 应该为 None");

    Ok(())
}

