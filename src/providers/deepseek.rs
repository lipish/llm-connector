//! DeepSeek serviceProviderimplementation
//!
//! DeepSeek uses OpenAI compatible API format，fully compatible with standard OpenAI protocol。
//! Supports reasoning models (reasoning content) and standard chat models。

use crate::core::{ConfigurableProtocol, ProviderBuilder};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;

/// DeepSeek protocoladapter
/// 
/// Use ConfigurableProtocol wrap OpenAI protocol
pub type DeepSeekProtocol = ConfigurableProtocol<OpenAIProtocol>;

/// DeepSeek serviceProvidertype
pub type DeepSeekProvider = crate::core::GenericProvider<DeepSeekProtocol>;

/// Create DeepSeek serviceProvider
/// 
/// # Parameters
/// - `api_key`: DeepSeek API key (format: sk-...)
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

/// Createwithcustomconfiguration DeepSeek serviceProvider
/// 
/// # Parameters
/// - `api_key`: API key
/// - `base_url`: customBase URL (optional，defaultas DeepSeek endpoint)
/// - `timeout_secs`: Timeout (seconds) (optional)
/// - `proxy`: proxy URL (optional)
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::deepseek_with_config;
/// 
/// let provider = deepseek_with_config(
///     "sk-...",
///     None, // Usedefault URL
///     Some(60), // 60 seconds timeout
///     None
/// ).unwrap();
/// ```
pub fn deepseek_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<DeepSeekProvider, LlmConnectorError> {
    // Create configuration-driven protocol
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
        assert_eq!(endpoint, "https://api.deepseek.com/chat/completions");
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

