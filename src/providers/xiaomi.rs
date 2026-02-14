//! Xiaomi MiMo service Provider implementation
//!
//! Xiaomi MiMo uses OpenAI-compatible API format, fully compatible with standard OpenAI protocol.
//! Main model: mimo-v2-flash

use crate::core::{ConfigurableProtocol, ProviderBuilder};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;

/// Xiaomi MiMo protocol adapter
/// 
/// Use ConfigurableProtocol wrap OpenAI protocol
pub type XiaomiProtocol = ConfigurableProtocol<OpenAIProtocol>;

/// Xiaomi MiMo service Provider type
pub type XiaomiProvider = crate::core::GenericProvider<XiaomiProtocol>;

/// Create Xiaomi MiMo service Provider
/// 
/// # Parameters
/// - `api_key`: Xiaomi MiMo API key
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::xiaomi;
/// 
/// let provider = xiaomi("your-api-key").unwrap();
/// ```
pub fn xiaomi(api_key: &str) -> Result<XiaomiProvider, LlmConnectorError> {
    xiaomi_with_config(api_key, None, None, None)
}

/// Create with custom configuration Xiaomi MiMo service Provider
/// 
/// # Parameters
/// - `api_key`: API key
/// - `base_url`: custom Base URL (optional, default as Xiaomi MiMo endpoint)
/// - `timeout_secs`: Timeout (seconds) (optional)
/// - `proxy`: proxy URL (optional)
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::xiaomi_with_config;
/// 
/// let provider = xiaomi_with_config(
///     "your-api-key",
///     None, // Use default URL
///     Some(60), // 60 seconds timeout
///     None
/// ).unwrap();
/// ```
pub fn xiaomi_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<XiaomiProvider, LlmConnectorError> {
    // Create configuration-driven protocol
    let protocol = ConfigurableProtocol::openai_compatible(
        OpenAIProtocol::new(api_key),
        "xiaomi"
    );
    
    // Use Builder Build
    let mut builder = ProviderBuilder::new(
        protocol,
        base_url.unwrap_or("https://api.xiaomimimo.com/v1")
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
    fn test_xiaomi() {
        let provider = xiaomi("test-api-key");
        assert!(provider.is_ok());
    }
    
    #[test]
    fn test_xiaomi_with_config() {
        let provider = xiaomi_with_config(
            "test-api-key",
            Some("https://custom.url"),
            Some(60),
            None
        );
        assert!(provider.is_ok());
    }
    
    #[test]
    fn test_xiaomi_protocol_endpoint() {
        let protocol = ConfigurableProtocol::openai_compatible(
            OpenAIProtocol::new("test-api-key"),
            "xiaomi"
        );
        let endpoint = protocol.chat_endpoint("https://api.xiaomimimo.com/v1");
        assert_eq!(endpoint, "https://api.xiaomimimo.com/v1/chat/completions");
    }
    
    #[test]
    fn test_xiaomi_protocol_name() {
        let protocol = ConfigurableProtocol::openai_compatible(
            OpenAIProtocol::new("test-api-key"),
            "xiaomi"
        );
        assert_eq!(protocol.name(), "xiaomi");
    }
}

