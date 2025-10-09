//! Ollama 模型管理示例
//!
//! 展示如何使用新的 Ollama 模型管理功能

use llm_connector::LlmClient;
use llm_connector::ollama::OllamaModelOps;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦙 Ollama 模型管理示例\n");

    // 创建 Ollama 客户端（默认本地地址）
    let client = LlmClient::ollama(None);

    // 1. 列出所有可用模型
    println!("📋 列出所有可用模型:");
    match client.list_models().await {
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
    match client.show_model(model_name).await {
        Ok(model_info) => {
            println!("   模型名称: {}", model_info.name);
            println!("   模型 ID: {}", model_info.model);
            println!("   修改时间: {}", model_info.modified_at);
            if let Some(size) = model_info.size {
                println!("   模型大小: {} bytes", size);
            }
            if let Some(details) = model_info.details {
                println!("   模型详情:");
                if let Some(format) = details.format {
                    println!("     格式: {}", format);
                }
                if let Some(family) = details.family {
                    println!("     系列: {}", family);
                }
                if let Some(param_size) = details.parameter_size {
                    println!("     参数规模: {}", param_size);
                }
                if let Some(quant) = details.quantization_level {
                    println!("     量化级别: {}", quant);
                }
            }
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
    println!("   // client.pull_model(\"llama3.2:1b\").await?;");
    println!("   // println!(\"模型拉取成功!\");");

    println!();

    // 4. 使用 fetch_models() 方法（通用接口）
    println!("🌐 使用通用接口获取模型列表:");
    match client.fetch_models().await {
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
    let chat_request = llm_connector::types::ChatRequest {
        model: "llama3.2".to_string(), // 使用你实际拥有的模型
        messages: vec![
            llm_connector::types::Message::user("你好！请用中文回答。")
        ],
        ..Default::default()
    };

    match client.chat(&chat_request).await {
        Ok(response) => {
            println!("   模型回复: {}", response.choices[0].message.content);
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