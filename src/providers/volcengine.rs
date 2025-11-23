//! Volcengine（Volcengine）serviceProviderimplementation
//!
//! Volcengine uses OpenAI compatible API format, but with different endpoint paths：
//! - OpenAI: `/v1/chat/completions`
//! - Volcengine: `/api/v3/chat/completions`

use crate::core::{ConfigurableProtocol, ProviderBuilder, ProtocolConfig, EndpointConfig, AuthConfig};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;

/// Volcengineprotocoladapter
///
/// Use ConfigurableProtocol to wrap OpenAI protocol, custom endpoint paths
pub type VolcengineProtocol = ConfigurableProtocol<OpenAIProtocol>;

/// VolcengineserviceProvidertype
pub type VolcengineProvider = crate::core::GenericProvider<VolcengineProtocol>;

/// CreateVolcengineserviceProvider
///
/// # Parameters
/// - `api_key`: Volcengine API key (UUID format)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::volcengine;
///
/// let provider = volcengine("your-volcengine-api-key").unwrap();
/// ```
pub fn volcengine(api_key: &str) -> Result<VolcengineProvider, LlmConnectorError> {
    volcengine_with_config(api_key, None, None, None)
}

/// CreatewithcustomconfigurationVolcengineserviceProvider
///
/// # Parameters
/// - `api_key`: API key
/// - `base_url`: customBase URL (optional，defaultasVolcengineendpoint)
/// - `timeout_secs`: Timeout (seconds) (optional)
/// - `proxy`: proxy URL (optional)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::volcengine_with_config;
///
/// let provider = volcengine_with_config(
///     "your-volcengine-api-key",
///     None, // Usedefault URL
///     Some(60), // 60 seconds timeout
///     None
/// ).unwrap();
/// ```
pub fn volcengine_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<VolcengineProvider, LlmConnectorError> {
    // Create configuration-driven protocol（custom endpoint paths）
    let protocol = ConfigurableProtocol::new(
        OpenAIProtocol::new(api_key),
        ProtocolConfig {
            name: "volcengine".to_string(),
            endpoints: EndpointConfig {
                chat_template: "{base_url}/api/v3/chat/completions".to_string(),
                models_template: Some("{base_url}/api/v3/models".to_string()),
            },
            auth: AuthConfig::Bearer,
            extra_headers: vec![],
        }
    );

    // Use Builder Build
    let mut builder = ProviderBuilder::new(
        protocol,
        base_url.unwrap_or("https://ark.cn-beijing.volces.com")
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
    fn test_volcengine() {
        let provider = volcengine("test-key");
        assert!(provider.is_ok());
    }
    
    #[test]
    fn test_volcengine_with_config() {
        let provider = volcengine_with_config(
            "test-key",
            Some("https://custom.url"),
            Some(60),
            None
        );
        assert!(provider.is_ok());
    }
    
    #[test]
    fn test_volcengine_protocol_endpoint() {
        let protocol = ConfigurableProtocol::new(
            OpenAIProtocol::new("test-key"),
            ProtocolConfig {
                name: "volcengine".to_string(),
                endpoints: EndpointConfig {
                    chat_template: "{base_url}/api/v3/chat/completions".to_string(),
                    models_template: Some("{base_url}/api/v3/models".to_string()),
                },
                auth: AuthConfig::Bearer,
                extra_headers: vec![],
            }
        );
        let endpoint = protocol.chat_endpoint("https://ark.cn-beijing.volces.com");
        assert_eq!(endpoint, "https://ark.cn-beijing.volces.com/api/v3/chat/completions");
    }
}

