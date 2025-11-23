//! Moonshot（Moonshot）服务Provide商实现
//!
//! Moonshot uses OpenAI-compatible API format，完全兼容标准 OpenAI 协议。

use crate::core::{ConfigurableProtocol, ProviderBuilder};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;

/// Moonshot 协议适配器
/// 
/// Use ConfigurableProtocol 包装 OpenAI protocol
pub type MoonshotProtocol = ConfigurableProtocol<OpenAIProtocol>;

/// Moonshot 服务Provide商类型
pub type MoonshotProvider = crate::core::GenericProvider<MoonshotProtocol>;

/// Create Moonshot 服务Provide商
/// 
/// # Parameters
/// - `api_key`: Moonshot API 密钥 (格式: sk-...)
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::moonshot;
/// 
/// let provider = moonshot("sk-...").unwrap();
/// ```
pub fn moonshot(api_key: &str) -> Result<MoonshotProvider, LlmConnectorError> {
    moonshot_with_config(api_key, None, None, None)
}

/// Create带有自Define配置的 Moonshot 服务Provide商
/// 
/// # Parameters
/// - `api_key`: API 密钥
/// - `base_url`: 自Define基础 URL (可选，默认为 Moonshot 端点)
/// - `timeout_secs`: 超时时间(秒) (可选)
/// - `proxy`: 代理 URL (可选)
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::moonshot_with_config;
/// 
/// let provider = moonshot_with_config(
///     "sk-...",
///     None, // Use默认 URL
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
    // Create配置驱动的协议
    let protocol = ConfigurableProtocol::openai_compatible(
        OpenAIProtocol::new(api_key),
        "moonshot"
    );
    
    // Use Builder Build
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

