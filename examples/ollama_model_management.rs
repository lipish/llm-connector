//! Ollama 模型管理示例
//!
//! 展示如何使用新的 Ollama 模型管理功能

use llm_connector::{LlmClient, Provider, types::{ChatRequest, Message, Role}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦙 Ollama 模型管理示例\n");

    // 创建 Ollama 客户端（默认本地地址）
    let client = LlmClient::ollama()?;

    // 获取 Ollama 特殊接口
    let ollama = match client.as_ollama() {
        Some(ollama) => ollama,
        None => {
            println!("❌ 无法获取 Ollama 特殊接口");
            return Ok(());
        }
    };

    // 1. 列出所有可用模型
    println!("📋 列出所有可用模型:");
    match ollama.models().await {
        Ok(models) => {
            if models.is_empty() {
                println!("   没有找到已安装的模型");
            } else {
                for (i, model) in models.iter().enumerate() {
                    println!("   {}. {}", i + 1, model);
                }
            }
        }
        Err(e) => {
            println!("   ❌ 错误: {}", e);
            println!("   💡 请确保 Ollama 正在运行在 localhost:11434");
        }
    }

    println!();

    // 2. 获取模型详细信息
    println!("🔍 获取模型详细信息:");
    let model_name = "llama3.2"; // 可以根据你实际拥有的模型修改
    match ollama.show_model(model_name).await {
        Ok(model_info) => {
            println!("   模型详情:");
            println!("     格式: {}", model_info.details.format);
            println!("     系列: {}", model_info.details.family);
            println!("     参数规模: {}", model_info.details.parameter_size);
            println!("     量化级别: {}", model_info.details.quantization_level);
            if let Some(families) = &model_info.details.families {
                println!("     支持的系列: {:?}", families);
            }
            println!("     模板长度: {} 字符", model_info.template.len());
            println!("     参数长度: {} 字符", model_info.parameters.len());
        }
        Err(e) => {
            println!("   ❌ 错误: {}", e);
            println!("   💡 确保模型 '{}' 已安装", model_name);
        }
    }

    println!();

    // 3. 拉取新模型（注释掉以避免实际下载）
    println!("📥 拉取新模型:");
    println!("   // 下面的代码展示了如何拉取新模型");
    println!("   // ollama.pull_model(\"llama3.2:1b\").await?;");
    println!("   // println!(\"模型拉取成功!\");");

    println!();

    // 4. 使用 models() 方法（通用接口）
    println!("🌐 使用通用接口获取模型列表:");
    match client.models().await {
        Ok(models) => {
            if models.is_empty() {
                println!("   没有找到模型");
            } else {
                for (i, model) in models.iter().enumerate() {
                    println!("   {}. {}", i + 1, model);
                }
            }
        }
        Err(e) => {
            println!("   ❌ 错误: {}", e);
        }
    }

    println!();

    // 5. 简单的聊天测试
    println!("💬 聊天测试:");
    let chat_request = ChatRequest {
        model: "llama3.2".to_string(), // 使用你实际拥有的模型
        messages: vec![
            Message {
                role: Role::User,
                content: "你好！请用中文回答。".to_string(),
                ..Default::default()
            }
        ],
        ..Default::default()
    };

    match client.chat(&chat_request).await {
        Ok(response) => {
            println!("   模型回复: {}", response.content);
        }
        Err(e) => {
            println!("   ❌ 聊天错误: {}", e);
            println!("   💡 确保模型 '{}' 已安装且可用", chat_request.model);
        }
    }

    println!("\n✅ 示例完成！");
    println!("\n💡 提示:");
    println!("   - 使用 'ollama list' 命令查看已安装的模型");
    println!("   - 使用 'ollama pull <模型名>' 下载新模型");
    println!("   - 使用 'ollama rm <模型名>' 删除模型");

    Ok(())
}