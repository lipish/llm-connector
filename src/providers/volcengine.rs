//! 火山引擎（Volcengine）服务提供商实现
//!
//! 火山引擎使用 OpenAI 兼容的 API 格式，但端点路径不同：
//! - OpenAI: `/v1/chat/completions`
//! - Volcengine: `/api/v3/chat/completions`

use crate::core::{ConfigurableProtocol, ProviderBuilder, ProtocolConfig, EndpointConfig, AuthConfig};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;

/// 火山引擎协议适配器
///
/// 使用 ConfigurableProtocol 包装 OpenAI protocol，自定义端点路径
pub type VolcengineProtocol = ConfigurableProtocol<OpenAIProtocol>;

/// 火山引擎服务提供商类型
pub type VolcengineProvider = crate::core::GenericProvider<VolcengineProtocol>;

/// 创建火山引擎服务提供商
///
/// # 参数
/// - `api_key`: 火山引擎 API 密钥 (UUID 格式)
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::volcengine;
///
/// let provider = volcengine("26f962bd-450e-4876-bc32-a732e6da9cd2").unwrap();
/// ```
pub fn volcengine(api_key: &str) -> Result<VolcengineProvider, LlmConnectorError> {
    volcengine_with_config(api_key, None, None, None)
}

/// 创建带有自定义配置的火山引擎服务提供商
///
/// # 参数
/// - `api_key`: API 密钥
/// - `base_url`: 自定义基础 URL (可选，默认为火山引擎端点)
/// - `timeout_secs`: 超时时间(秒) (可选)
/// - `proxy`: 代理 URL (可选)
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::volcengine_with_config;
///
/// let provider = volcengine_with_config(
///     "26f962bd-450e-4876-bc32-a732e6da9cd2",
///     None, // 使用默认 URL
///     Some(60), // 60秒超时
///     None
/// ).unwrap();
/// ```
pub fn volcengine_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<VolcengineProvider, LlmConnectorError> {
    // 创建配置驱动的协议（自定义端点路径）
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

    // 使用 Builder 构建
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

