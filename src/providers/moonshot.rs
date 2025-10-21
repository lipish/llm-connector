//! Moonshot（月之暗面）服务提供商实现
//!
//! Moonshot 使用 OpenAI 兼容的 API 格式，完全兼容标准 OpenAI 协议。

use crate::core::{ConfigurableProtocol, ProviderBuilder};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;

/// Moonshot 协议适配器
/// 
/// 使用 ConfigurableProtocol 包装 OpenAI protocol
pub type MoonshotProtocol = ConfigurableProtocol<OpenAIProtocol>;

/// Moonshot 服务提供商类型
pub type MoonshotProvider = crate::core::GenericProvider<MoonshotProtocol>;

/// 创建 Moonshot 服务提供商
/// 
/// # 参数
/// - `api_key`: Moonshot API 密钥 (格式: sk-...)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::moonshot;
/// 
/// let provider = moonshot("sk-...").unwrap();
/// ```
pub fn moonshot(api_key: &str) -> Result<MoonshotProvider, LlmConnectorError> {
    moonshot_with_config(api_key, None, None, None)
}

/// 创建带有自定义配置的 Moonshot 服务提供商
/// 
/// # 参数
/// - `api_key`: API 密钥
/// - `base_url`: 自定义基础 URL (可选，默认为 Moonshot 端点)
/// - `timeout_secs`: 超时时间(秒) (可选)
/// - `proxy`: 代理 URL (可选)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::moonshot_with_config;
/// 
/// let provider = moonshot_with_config(
///     "sk-...",
///     None, // 使用默认 URL
///     Some(60), // 60秒超时
///     None
/// ).unwrap();
/// ```
pub fn moonshot_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<MoonshotProvider, LlmConnectorError> {
    // 创建配置驱动的协议
    let protocol = ConfigurableProtocol::openai_compatible(
        OpenAIProtocol::new(api_key),
        "moonshot"
    );
    
    // 使用 Builder 构建
    let mut builder = ProviderBuilder::new(
        protocol,
        base_url.unwrap_or("https://api.moonshot.cn")
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
    fn test_moonshot() {
        let provider = moonshot("sk-test-key");
        assert!(provider.is_ok());
    }
    
    #[test]
    fn test_moonshot_with_config() {
        let provider = moonshot_with_config(
            "sk-test-key",
            Some("https://custom.url"),
            Some(60),
            None
        );
        assert!(provider.is_ok());
    }
    
    #[test]
    fn test_moonshot_protocol_endpoint() {
        let protocol = ConfigurableProtocol::openai_compatible(
            OpenAIProtocol::new("sk-test-key"),
            "moonshot"
        );
        let endpoint = protocol.chat_endpoint("https://api.moonshot.cn");
        assert_eq!(endpoint, "https://api.moonshot.cn/v1/chat/completions");
    }
    
    #[test]
    fn test_moonshot_protocol_name() {
        let protocol = ConfigurableProtocol::openai_compatible(
            OpenAIProtocol::new("sk-test-key"),
            "moonshot"
        );
        assert_eq!(protocol.name(), "moonshot");
    }
}

