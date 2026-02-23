//! LongCat API Service Provider Implementation
//!
//! LongCat supports two API formats:
//! 1. OpenAI format - Uses OpenAI compatible interface
//! 2. Anthropic format - Uses Anthropic compatible interface, but with Bearer token authentication
//!
//! Note: LongCat Anthropic format uses `Authorization: Bearer` authentication,
//! instead of standard Anthropic `x-api-key` authentication.

use crate::core::{
    AuthConfig, ConfigurableProtocol, EndpointConfig, ProtocolConfig, ProviderBuilder,
};
use crate::error::LlmConnectorError;
use crate::protocols::AnthropicProtocol;

/// LongCat Anthropic format protocol adapter
///
/// Uses ConfigurableProtocol to wrap Anthropic protocol, using Bearer authentication
pub type LongCatAnthropicProtocol = ConfigurableProtocol<AnthropicProtocol>;

/// LongCat Anthropic format service provider type
pub type LongCatAnthropicProvider = crate::core::GenericProvider<LongCatAnthropicProtocol>;

/// Create LongCat Anthropic format service provider
///
/// # Parameters
/// - `api_key`: LongCat API key (Format: ak_...)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::longcat_anthropic;
///
/// let provider = longcat_anthropic("ak_...").unwrap();
/// ```
pub fn longcat_anthropic(api_key: &str) -> Result<LongCatAnthropicProvider, LlmConnectorError> {
    longcat_anthropic_with_config(api_key, None, None, None)
}

/// Create LongCat Anthropic service provider with custom configuration
///
/// # Parameters
/// - `api_key`: API key
/// - `base_url`: Custom base URL (optional, defaults to LongCat Anthropic endpoint)
/// - `timeout_secs`: Timeout (seconds) (optional)
/// - `proxy`: proxy URL (optional)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::longcat_anthropic_with_config;
///
/// let provider = longcat_anthropic_with_config(
///     "ak_...",
///     None, // Usedefault URL
///     Some(60), // 60 seconds timeout
///     None
/// ).unwrap();
/// ```
pub fn longcat_anthropic_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<LongCatAnthropicProvider, LlmConnectorError> {
    // Create configuration-driven protocol（Use Bearer authentication + Additional headers）
    let protocol = ConfigurableProtocol::new(
        AnthropicProtocol::new(api_key),
        ProtocolConfig {
            name: "longcat-anthropic".to_string(),
            endpoints: EndpointConfig {
                chat_template: "{base_url}/v1/messages".to_string(),
                models_template: None,
            },
            auth: AuthConfig::Bearer, // Use Bearer instead of x-api-key
            extra_headers: vec![("anthropic-version".to_string(), "2023-06-01".to_string())],
        },
    );

    // Use Builder Build
    let mut builder = ProviderBuilder::new(
        protocol,
        base_url.unwrap_or("https://api.longcat.chat/anthropic"),
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
    fn test_longcat_anthropic() {
        let provider = longcat_anthropic("ak_test");
        assert!(provider.is_ok());
    }

    #[test]
    fn test_longcat_anthropic_with_config() {
        let provider =
            longcat_anthropic_with_config("ak_test", Some("https://custom.url"), Some(60), None);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_longcat_anthropic_protocol_auth_headers() {
        let protocol = ConfigurableProtocol::new(
            AnthropicProtocol::new("ak_test123"),
            ProtocolConfig {
                name: "longcat-anthropic".to_string(),
                endpoints: EndpointConfig {
                    chat_template: "{base_url}/v1/messages".to_string(),
                    models_template: None,
                },
                auth: AuthConfig::Bearer,
                extra_headers: vec![("anthropic-version".to_string(), "2023-06-01".to_string())],
            },
        );
        let headers = protocol.auth_headers();

        // Should use Bearer authentication
        assert!(
            headers
                .iter()
                .any(|(k, v)| k == "Authorization" && v == "Bearer ak_test123")
        );

        // Should contain anthropic-version
        assert!(
            headers
                .iter()
                .any(|(k, v)| k == "anthropic-version" && v == "2023-06-01")
        );

        // Should not contain x-api-key
        assert!(!headers.iter().any(|(k, _)| k == "x-api-key"));
    }
}
