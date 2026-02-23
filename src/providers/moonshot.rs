//! Moonshot（Moonshot）serviceProviderimplementation
//!
//! Moonshot uses OpenAI-compatible API format，fully compatible with standard OpenAI protocol。

use crate::core::{ConfigurableProtocol, ProviderBuilder};
use crate::error::LlmConnectorError;
use crate::protocols::OpenAIProtocol;

/// Moonshot protocoladapter
///
/// Use ConfigurableProtocol wrap OpenAI protocol
pub type MoonshotProtocol = ConfigurableProtocol<OpenAIProtocol>;

/// Moonshot serviceProvidertype
pub type MoonshotProvider = crate::core::GenericProvider<MoonshotProtocol>;

/// Create Moonshot serviceProvider
///
/// # Parameters
/// - `api_key`: Moonshot API key (format: sk-...)
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

/// Createwithcustomconfiguration Moonshot serviceProvider
///
/// # Parameters
/// - `api_key`: API key
/// - `base_url`: customBase URL (optional，defaultas Moonshot endpoint)
/// - `timeout_secs`: Timeout (seconds) (optional)
/// - `proxy`: proxy URL (optional)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::moonshot_with_config;
///
/// let provider = moonshot_with_config(
///     "sk-...",
///     None, // Usedefault URL
///     Some(60), // 60 seconds timeout
///     None
/// ).unwrap();
/// ```
pub fn moonshot_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<MoonshotProvider, LlmConnectorError> {
    // Create configuration-driven protocol
    let protocol =
        ConfigurableProtocol::openai_compatible(OpenAIProtocol::new(api_key), "moonshot");

    // Use Builder Build
    let mut builder = ProviderBuilder::new(protocol, base_url.unwrap_or("https://api.moonshot.cn"));

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
        let provider =
            moonshot_with_config("sk-test-key", Some("https://custom.url"), Some(60), None);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_moonshot_protocol_endpoint() {
        let protocol =
            ConfigurableProtocol::openai_compatible(OpenAIProtocol::new("sk-test-key"), "moonshot");
        let endpoint = protocol.chat_endpoint("https://api.moonshot.cn");
        assert_eq!(endpoint, "https://api.moonshot.cn/chat/completions");
    }

    #[test]
    fn test_moonshot_protocol_name() {
        let protocol =
            ConfigurableProtocol::openai_compatible(OpenAIProtocol::new("sk-test-key"), "moonshot");
        assert_eq!(protocol.name(), "moonshot");
    }
}
