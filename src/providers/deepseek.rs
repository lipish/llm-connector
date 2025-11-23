//! DeepSeek 服务Provide商实现
//!
//! DeepSeek Use OpenAI 兼容 API 格式，完全兼容标准 OpenAI protocol。
//! Support推理model（reasoning content）and标准对话model。

use crate::core::{ConfigurableProtocol, ProviderBuilder};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;

/// DeepSeek protocoladapter
/// 
/// Use ConfigurableProtocol 包装 OpenAI protocol
pub type DeepSeekProtocol = ConfigurableProtocol<OpenAIProtocol>;

/// DeepSeek 服务Provide商类型
pub type DeepSeekProvider = crate::core::GenericProvider<DeepSeekProtocol>;

/// Create DeepSeek 服务Provide商
/// 
/// # Parameters
/// - `api_key`: DeepSeek API 密钥 (格式: sk-...)
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::deepseek;
/// 
/// let provider = deepseek("sk-...").unwrap();
/// ```
pub fn deepseek(api_key: &str) -> Result<DeepSeekProvider, LlmConnectorError> {
    deepseek_with_config(api_key, None, None, None)
}

/// Create带有customconfiguration DeepSeek 服务Provide商
/// 
/// # Parameters
/// - `api_key`: API 密钥
/// - `base_url`: custom基础 URL (optional，默认as DeepSeek endpoint)
/// - `timeout_secs`: 超时时间(秒) (optional)
/// - `proxy`: 代理 URL (optional)
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::deepseek_with_config;
/// 
/// let provider = deepseek_with_config(
///     "sk-...",
///     None, // Use默认 URL
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
    // Createconfiguration驱动protocol
    let protocol = ConfigurableProtocol::openai_compatible(
        OpenAIProtocol::new(api_key),
        "deepseek"
    );
    
    // Use Builder Build
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

