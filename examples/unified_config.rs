//! 统一配置接口示例
//!
//! 展示如何使用统一的配置方式创建不同的 LLM 客户端

use llm_connector::{LlmClient, types::{ChatRequest, Message}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 统一的 LLM 后端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum LlmBackendConfig {
    OpenAI {
        api_key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        base_url: Option<String>,
        #[serde(default = "default_timeout")]
        timeout_ms: u64,
    },
    Anthropic {
        api_key: String,
        #[serde(default = "default_timeout")]
        timeout_ms: u64,
    },
    Zhipu {
        api_key: String,
        #[serde(default = "default_timeout")]
        timeout_ms: u64,
    },
    Aliyun {
        api_key: String,
    },
    Ollama {
        #[serde(skip_serializing_if = "Option::is_none")]
        base_url: Option<String>,
    },
}

fn default_timeout() -> u64 {
    30000 // 30秒默认超时
}

impl LlmBackendConfig {
    /// 从配置创建 LLM 客户端
    pub fn create_client(&self) -> LlmClient {
        match self {
            LlmBackendConfig::OpenAI { api_key, base_url, timeout_ms } => {
                LlmClient::openai_with_timeout(api_key, base_url.as_deref(), *timeout_ms)
            }
            LlmBackendConfig::Anthropic { api_key, timeout_ms } => {
                LlmClient::anthropic_with_timeout(api_key, *timeout_ms)
            }
            LlmBackendConfig::Zhipu { api_key, timeout_ms } => {
                LlmClient::zhipu_with_timeout(api_key, *timeout_ms)
            }
            LlmBackendConfig::Aliyun { api_key } => {
                LlmClient::aliyun(api_key)
            }
            LlmBackendConfig::Ollama { base_url } => {
                LlmClient::ollama(base_url.as_deref())
            }
        }
    }

    /// 获取提供商名称
    pub fn provider_name(&self) -> &'static str {
        match self {
            LlmBackendConfig::OpenAI { .. } => "openai",
            LlmBackendConfig::Anthropic { .. } => "anthropic",
            LlmBackendConfig::Zhipu { .. } => "zhipu",
            LlmBackendConfig::Aliyun { .. } => "aliyun",
            LlmBackendConfig::Ollama { .. } => "ollama",
        }
    }

    /// 获取超时配置
    pub fn timeout_ms(&self) -> u64 {
        match self {
            LlmBackendConfig::OpenAI { timeout_ms, .. } => *timeout_ms,
            LlmBackendConfig::Anthropic { timeout_ms, .. } => *timeout_ms,
            LlmBackendConfig::Zhipu { timeout_ms, .. } => *timeout_ms,
            LlmBackendConfig::Aliyun { .. } => 30000, // Aliyun 使用默认超时
            LlmBackendConfig::Ollama { .. } => 30000, // Ollama 使用默认超时
        }
    }
}

/// 多提供商配置管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiProviderConfig {
    pub providers: HashMap<String, LlmBackendConfig>,
    pub default_provider: String,
}

impl MultiProviderConfig {
    /// 获取默认客户端
    pub fn default_client(&self) -> Option<LlmClient> {
        self.providers.get(&self.default_provider)
            .map(|config| config.create_client())
    }

    /// 获取指定提供商的客户端
    pub fn client(&self, provider: &str) -> Option<LlmClient> {
        self.providers.get(provider)
            .map(|config| config.create_client())
    }

    /// 列出所有可用的提供商
    pub fn available_providers(&self) -> Vec<&String> {
        self.providers.keys().collect()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 统一配置接口示例\n");

    // 示例1: 单个提供商配置
    println!("📋 示例1: 单个提供商配置");
    
    let openai_config = LlmBackendConfig::OpenAI {
        api_key: std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "sk-test".to_string()),
        base_url: None,
        timeout_ms: 45000, // 45秒超时
    };

    let client = openai_config.create_client();
    println!("✅ 创建 {} 客户端，超时: {}ms", 
             openai_config.provider_name(), 
             openai_config.timeout_ms());

    // 示例2: 从 YAML 配置文件加载（模拟）
    println!("\n📋 示例2: 多提供商配置");
    
    let yaml_config = r#"
providers:
  primary_openai:
    provider: openai
    api_key: "sk-your-openai-key"
    timeout_ms: 60000
  backup_zhipu:
    provider: zhipu
    api_key: "sk-your-zhipu-key"
    timeout_ms: 30000
  local_ollama:
    provider: ollama
    base_url: "http://localhost:11434"
  anthropic_claude:
    provider: anthropic
    api_key: "sk-ant-your-key"
    timeout_ms: 45000
default_provider: "primary_openai"
"#;

    if let Ok(multi_config) = serde_yaml::from_str::<MultiProviderConfig>(yaml_config) {
        println!("✅ 配置加载成功");
        println!("   默认提供商: {}", multi_config.default_provider);
        println!("   可用提供商: {:?}", multi_config.available_providers());

        // 使用默认客户端
        if let Some(default_client) = multi_config.default_client() {
            println!("   默认客户端协议: {}", default_client.protocol_name());
        }

        // 使用特定提供商
        if let Some(zhipu_client) = multi_config.client("backup_zhipu") {
            println!("   Zhipu 客户端协议: {}", zhipu_client.protocol_name());
        }
    }

    // 示例3: 动态配置切换
    println!("\n📋 示例3: 动态配置切换");
    
    let configs = vec![
        ("OpenAI", LlmBackendConfig::OpenAI {
            api_key: "sk-test".to_string(),
            base_url: Some("https://api.openai.com/v1".to_string()),
            timeout_ms: 30000,
        }),
        ("DeepSeek", LlmBackendConfig::OpenAI {
            api_key: "sk-test".to_string(),
            base_url: Some("https://api.deepseek.com/v1".to_string()),
            timeout_ms: 45000,
        }),
        ("Zhipu", LlmBackendConfig::Zhipu {
            api_key: "sk-test".to_string(),
            timeout_ms: 25000,
        }),
    ];

    for (name, config) in configs {
        let client = config.create_client();
        println!("   {} -> 协议: {}, 超时: {}ms", 
                 name, 
                 client.protocol_name(), 
                 config.timeout_ms());
    }

    // 示例4: 实际测试（如果有有效的 API Key）
    println!("\n📋 示例4: 实际测试");
    
    let test_configs = vec![
        ("ZHIPU_API_KEY", LlmBackendConfig::Zhipu {
            api_key: std::env::var("ZHIPU_API_KEY").unwrap_or_default(),
            timeout_ms: 15000,
        }),
        ("OPENAI_API_KEY", LlmBackendConfig::OpenAI {
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            base_url: None,
            timeout_ms: 20000,
        }),
    ];

    for (env_var, config) in test_configs {
        if std::env::var(env_var).is_ok() {
            let client = config.create_client();
            println!("   测试 {} 客户端...", config.provider_name());
            
            let request = ChatRequest {
                model: match config.provider_name() {
                    "zhipu" => "glm-4-flash".to_string(),
                    "openai" => "gpt-3.5-turbo".to_string(),
                    _ => "default".to_string(),
                },
                messages: vec![Message::user("Hello!")],
                max_tokens: Some(10),
                ..Default::default()
            };

            match client.chat(&request).await {
                Ok(response) => {
                    println!("   ✅ 成功: {}", response.choices[0].message.content.chars().take(50).collect::<String>());
                }
                Err(e) => {
                    println!("   ❌ 失败: {}", e);
                }
            }
        } else {
            println!("   ⏭️  跳过 {} 测试（未设置环境变量）", env_var);
        }
    }

    println!("\n🎯 统一配置示例完成！");
    println!("\n💡 使用建议:");
    println!("   1. 使用 LlmBackendConfig 枚举统一管理不同提供商");
    println!("   2. 通过 YAML/JSON 配置文件管理多个提供商");
    println!("   3. 为不同提供商设置合适的超时时间");
    println!("   4. 实现配置热重载以支持动态切换");

    Ok(())
}
