//! Tencent Hunyuan（Tencent Hunyuan）serviceProviderimplementation
//!
//! Tencent Hunyuan uses OpenAI compatible API format，fully compatible with standard OpenAI protocol。

use crate::core::{ConfigurableProtocol, ProviderBuilder};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;

/// Tencent Hunyuanprotocoladapter
///
/// Use ConfigurableProtocol wrap OpenAI protocol
pub type TencentProtocol = ConfigurableProtocol<OpenAIProtocol>;

/// Tencent HunyuanserviceProvidertype
pub type TencentProvider = crate::core::GenericProvider<TencentProtocol>;

/// CreateTencent HunyuanserviceProvider
///
/// # Parameters
/// - `api_key`: Tencent Hunyuan API key (format: sk-...)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::tencent;
///
/// let provider = tencent("sk-YMiR2Q7LNWVKVWKivkfPn49geQXT27OZXumFkSS3Ef6FlQ50").unwrap();
/// ```
pub fn tencent(api_key: &str) -> Result<TencentProvider, LlmConnectorError> {
    tencent_with_config(api_key, None, None, None)
}

/// CreatewithcustomconfigurationTencent HunyuanserviceProvider
///
/// # Parameters
/// - `api_key`: API key
/// - `base_url`: customBase URL (optional，defaultasTencent Hunyuanendpoint)
/// - `timeout_secs`: Timeout (seconds) (optional)
/// - `proxy`: proxy URL (optional)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::tencent_with_config;
///
/// let provider = tencent_with_config(
///     "sk-YMiR2Q7LNWVKVWKivkfPn49geQXT27OZXumFkSS3Ef6FlQ50",
///     None, // Usedefault URL
///     Some(60), // 60 seconds timeout
///     None
/// ).unwrap();
/// ```
pub fn tencent_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<TencentProvider, LlmConnectorError> {
    // Create configuration-driven protocol
    let protocol = ConfigurableProtocol::openai_compatible(
        OpenAIProtocol::new(api_key),
        "tencent"
    );

    // Use Builder Build
    let mut builder = ProviderBuilder::new(
        protocol,
        base_url.unwrap_or("https://api.hunyuan.cloud.tencent.com")
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
    fn test_tencent() {
        let provider = tencent("sk-test-key");
        assert!(provider.is_ok());
    }

    #[test]
    fn test_tencent_with_config() {
        let provider = tencent_with_config(
            "sk-test-key",
            Some("https://custom.url"),
            Some(60),
            None
        );
        assert!(provider.is_ok());
    }

    #[test]
    fn test_tencent_protocol_endpoint() {
        let protocol = ConfigurableProtocol::openai_compatible(
            OpenAIProtocol::new("sk-test-key"),
            "tencent"
        );
        let endpoint = protocol.chat_endpoint("https://api.hunyuan.cloud.tencent.com");
        assert_eq!(endpoint, "https://api.hunyuan.cloud.tencent.com/v1/chat/completions");
    }

    #[test]
    fn test_tencent_protocol_name() {
        let protocol = ConfigurableProtocol::openai_compatible(
            OpenAIProtocol::new("sk-test-key"),
            "tencent"
        );
        assert_eq!(protocol.name(), "tencent");
    }
}

