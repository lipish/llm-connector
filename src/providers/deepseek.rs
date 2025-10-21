//! DeepSeek 服务提供商实现
//!
//! DeepSeek 使用 OpenAI 兼容的 API 格式，完全兼容标准 OpenAI 协议。
//! 支持推理模型（reasoning content）和标准对话模型。

use crate::core::{ConfigurableProtocol, ProviderBuilder};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;

/// DeepSeek 协议适配器
/// 
/// 使用 ConfigurableProtocol 包装 OpenAI protocol
pub type DeepSeekProtocol = ConfigurableProtocol<OpenAIProtocol>;

/// DeepSeek 服务提供商类型
pub type DeepSeekProvider = crate::core::GenericProvider<DeepSeekProtocol>;

/// 创建 DeepSeek 服务提供商
/// 
/// # 参数
/// - `api_key`: DeepSeek API 密钥 (格式: sk-...)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::deepseek;
/// 
/// let provider = deepseek("sk-...").unwrap();
/// ```
pub fn deepseek(api_key: &str) -> Result<DeepSeekProvider, LlmConnectorError> {
    deepseek_with_config(api_key, None, None, None)
}

/// 创建带有自定义配置的 DeepSeek 服务提供商
/// 
/// # 参数
/// - `api_key`: API 密钥
/// - `base_url`: 自定义基础 URL (可选，默认为 DeepSeek 端点)
/// - `timeout_secs`: 超时时间(秒) (可选)
/// - `proxy`: 代理 URL (可选)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::deepseek_with_config;
/// 
/// let provider = deepseek_with_config(
///     "sk-...",
///     None, // 使用默认 URL
///     Some(60), // 60秒超时
///     None
/// ).unwrap();
/// ```
pub fn deepseek_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<DeepSeekProvider, LlmConnectorError> {
    // 创建配置驱动的协议
    let protocol = ConfigurableProtocol::openai_compatible(
        OpenAIProtocol::new(api_key),
        "deepseek"
    );
    
    // 使用 Builder 构建
    let mut builder = ProviderBuilder::new(
        protocol,
        base_url.unwrap_or("https://api.deepseek.com")
    );
    
    if let Some(timeout) = timeout_secs {
        builder = builder.timeout(timeout);
    }
    
    if let Some(proxy_url) = proxy {
        builder = builder.proxy(proxy_url);
    }
    
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Protocol;
    
    #[test]
    fn test_deepseek() {
        let provider = deepseek("sk-test-key");
        assert!(provider.is_ok());
    }
    
    #[test]
    fn test_deepseek_with_config() {
        let provider = deepseek_with_config(
            "sk-test-key",
            Some("https://custom.url"),
            Some(60),
            None
        );
        assert!(provider.is_ok());
    }
    
    #[test]
    fn test_deepseek_protocol_endpoint() {
        let protocol = ConfigurableProtocol::openai_compatible(
            OpenAIProtocol::new("sk-test-key"),
            "deepseek"
        );
        let endpoint = protocol.chat_endpoint("https://api.deepseek.com");
        assert_eq!(endpoint, "https://api.deepseek.com/v1/chat/completions");
    }
    
    #[test]
    fn test_deepseek_protocol_name() {
        let protocol = ConfigurableProtocol::openai_compatible(
            OpenAIProtocol::new("sk-test-key"),
            "deepseek"
        );
        assert_eq!(protocol.name(), "deepseek");
    }
}

