//! Tencent Hunyuan（Tencent Hunyuan）服务Provide商实现
//!
//! Tencent HunyuanUse OpenAI 兼容的 API 格式，完全兼容标准 OpenAI 协议。

use crate::core::{ConfigurableProtocol, ProviderBuilder};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;

/// Tencent Hunyuan协议适配器
///
/// Use ConfigurableProtocol 包装 OpenAI protocol
pub type TencentProtocol = ConfigurableProtocol<OpenAIProtocol>;

/// Tencent Hunyuan服务Provide商类型
pub type TencentProvider = crate::core::GenericProvider<TencentProtocol>;

/// CreateTencent Hunyuan服务Provide商
///
/// # Parameters
/// - `api_key`: Tencent Hunyuan API 密钥 (格式: sk-...)
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

/// Create带有自Define配置的Tencent Hunyuan服务Provide商
///
/// # Parameters
/// - `api_key`: API 密钥
/// - `base_url`: 自Define基础 URL (可选，默认为Tencent Hunyuan端点)
/// - `timeout_secs`: 超时时间(秒) (可选)
/// - `proxy`: 代理 URL (可选)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::tencent_with_config;
///
/// let provider = tencent_with_config(
///     "sk-YMiR2Q7LNWVKVWKivkfPn49geQXT27OZXumFkSS3Ef6FlQ50",
///     None, // Use默认 URL
///     Some(60), // 60秒超时
///     None
/// ).unwrap();
/// ```
pub fn tencent_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<TencentProvider, LlmConnectorError> {
    // Create配置驱动的协议
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

