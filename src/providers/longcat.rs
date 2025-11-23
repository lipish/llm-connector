//! LongCat API 服务Provide商实现
//!
//! LongCat Support两种 API 格式：
//! 1. OpenAI 格式 - Use OpenAI 兼容接口
//! 2. Anthropic 格式 - Use Anthropic 兼容接口，但authentication方式as Bearer token
//!
//! Note：LongCat  Anthropic 格式Use `Authorization: Bearer` authentication，
//! 而不is标准 Anthropic  `x-api-key` authentication。

use crate::core::{ConfigurableProtocol, ProviderBuilder, ProtocolConfig, EndpointConfig, AuthConfig};
use crate::protocols::AnthropicProtocol;
use crate::error::LlmConnectorError;

/// LongCat Anthropic 格式protocoladapter
///
/// Use ConfigurableProtocol 包装 Anthropic protocol，Use Bearer authentication
pub type LongCatAnthropicProtocol = ConfigurableProtocol<AnthropicProtocol>;

/// LongCat Anthropic 格式服务Provide商类型
pub type LongCatAnthropicProvider = crate::core::GenericProvider<LongCatAnthropicProtocol>;

/// Create LongCat Anthropic 格式服务Provide商
///
/// # Parameters
/// - `api_key`: LongCat API 密钥 (格式: ak_...)
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

/// Create带有customconfiguration LongCat Anthropic 服务Provide商
///
/// # Parameters
/// - `api_key`: API 密钥
/// - `base_url`: custom基础 URL (optional，默认as LongCat Anthropic endpoint)
/// - `timeout_secs`: 超时时间(秒) (optional)
/// - `proxy`: 代理 URL (optional)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::longcat_anthropic_with_config;
///
/// let provider = longcat_anthropic_with_config(
///     "ak_...",
///     None, // Use默认 URL
///     Some(60), // 60秒超时
///     None
/// ).unwrap();
/// ```
pub fn longcat_anthropic_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<LongCatAnthropicProvider, LlmConnectorError> {
    // Createconfiguration驱动protocol（Use Bearer authentication + Additionalheaders）
    let protocol = ConfigurableProtocol::new(
        AnthropicProtocol::new(api_key),
        ProtocolConfig {
            name: "longcat-anthropic".to_string(),
            endpoints: EndpointConfig {
                chat_template: "{base_url}/v1/messages".to_string(),
                models_template: None,
            },
            auth: AuthConfig::Bearer,  // Use Bearer 而不is x-api-key
            extra_headers: vec![
                ("anthropic-version".to_string(), "2023-06-01".to_string()),
            ],
        }
    );

    // Use Builder Build
    let mut builder = ProviderBuilder::new(
        protocol,
        base_url.unwrap_or("https://api.longcat.chat/anthropic")
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
        let provider = longcat_anthropic_with_config(
            "ak_test",
            Some("https://custom.url"),
            Some(60),
            None
        );
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
                extra_headers: vec![
                    ("anthropic-version".to_string(), "2023-06-01".to_string()),
                ],
            }
        );
        let headers = protocol.auth_headers();

        // 应该Use Bearer authentication
        assert!(headers.iter().any(|(k, v)| k == "Authorization" && v == "Bearer ak_test123"));

        // 应该Contains anthropic-version
        assert!(headers.iter().any(|(k, v)| k == "anthropic-version" && v == "2023-06-01"));

        // 不应该Contains x-api-key
        assert!(!headers.iter().any(|(k, _)| k == "x-api-key"));
    }
}

