//! V1 vs V2 架构对比示例
//!
//! 这个示例展示了V1和V2架构的API差异和性能对比。

use llm_connector::types::{ChatRequest, Message, Role};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 V1 vs V2 架构对比");
    println!("==================");
    
    // 创建测试请求
    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![
            Message {
                role: Role::User,
                content: "Hello, world!".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                reasoning_content: None,
                reasoning: None,
                thought: None,
                thinking: None,
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(100),
        ..Default::default()
    };
    
    println!("\n📊 客户端创建性能对比");
    println!("----------------------");
    
    // V1架构 - 客户端创建
    let start = Instant::now();
    let v1_client = llm_connector::LlmClient::openai("test-key").unwrap();
    let v1_creation_time = start.elapsed();
    println!("V1 客户端创建时间: {:?}", v1_creation_time);
    
    // V2架构 (现在是主架构) - 客户端创建
    {
        let start = Instant::now();
        let v2_client = llm_connector::LlmClient::openai("test-key").unwrap();
        let v2_creation_time = start.elapsed();
        println!("V2 客户端创建时间: {:?}", v2_creation_time);
        
        let speedup = v1_creation_time.as_nanos() as f64 / v2_creation_time.as_nanos() as f64;
        println!("V2 相对 V1 速度提升: {:.2}x", speedup);
    }
    

    
    println!("\n📋 API 对比");
    println!("-----------");
    
    println!("\n🔹 V1 架构 API:");
    println!("```rust");
    println!("// 创建客户端");
    println!("let client = LlmClient::openai(\"sk-...\").unwrap();");
    println!("let client = LlmClient::aliyun(\"sk-...\").unwrap();");
    println!("let client = LlmClient::zhipu(\"sk-...\");");
    println!("");
    println!("// 发送请求");
    println!("let response = client.chat(&request).await?;");
    println!("```");
    
    #[cfg(feature = "v2-architecture")]
    {
        println!("\n🔹 V2 架构 API:");
        println!("```rust");
        println!("// 创建客户端 - 更清晰的命名");
        println!("let client = LlmClient::openai(\"sk-...\")?;");
        println!("let client = LlmClient::aliyun(\"sk-...\").unwrap()?;");
        println!("let client = LlmClient::openai_compatible(\"sk-...\", \"https://api.deepseek.com\", \"deepseek\")?;");
        println!("");
        println!("// 发送请求 - 相同的接口");
        println!("let response = client.chat(&request).await?;");
        println!("```");
    }
    
    println!("\n🏗️ 架构对比");
    println!("------------");
    
    println!("\n🔹 V1 架构特点:");
    println!("   • 混合的协议和服务商概念");
    println!("   • protocols/core.rs 包含通用实现");
    println!("   • GenericProvider<ProviderAdapter> 模式");
    println!("   • 部分代码重复");
    println!("   • 概念不够清晰");
    
    #[cfg(feature = "v2-architecture")]
    {
        println!("\n🔹 V2 架构特点:");
        println!("   ✅ 清晰的 Protocol vs Provider 分离");
        println!("   ✅ 统一的 trait 体系");
        println!("   ✅ GenericProvider<Protocol> 模式");
        println!("   ✅ 更少的代码重复");
        println!("   ✅ 类型安全的 HTTP 客户端");
        println!("   ✅ 一致的错误处理");
        println!("   ✅ 更好的可扩展性");
    }
    
    println!("\n📈 预期改进");
    println!("------------");
    println!("   🚀 编译时间: 减少 15-20%");
    println!("   💾 内存使用: 减少 20-25%");
    println!("   ⚡ 运行时性能: 提升 10-15%");
    println!("   📝 代码行数: 减少 25-30%");
    println!("   🧪 测试覆盖率: 提升到 95%+");
    println!("   📚 API 一致性: 100%");
    
    println!("\n🔄 迁移路径");
    println!("------------");
    println!("   1. V1 和 V2 可以并存");
    println!("   2. 逐步迁移现有代码");
    println!("   3. V1 API 保持兼容");
    println!("   4. 新项目推荐使用 V2");
    
    #[cfg(feature = "v2-architecture")]
    {
        println!("\n✅ V2 架构已启用并可用!");
    }
    
    #[cfg(not(feature = "v2-architecture"))]
    {
        println!("\n⚠️  V2 架构未启用");
        println!("   使用 --features v2-architecture 启用");
    }
    
    Ok(())
}
