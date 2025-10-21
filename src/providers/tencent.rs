//! 腾讯云混元（Tencent Hunyuan）服务提供商实现
//!
//! 腾讯云混元使用 OpenAI 兼容的 API 格式，完全兼容标准 OpenAI 协议。

use crate::core::{ConfigurableProtocol, ProviderBuilder};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;

/// 腾讯云混元协议适配器
///
/// 使用 ConfigurableProtocol 包装 OpenAI protocol
pub type TencentProtocol = ConfigurableProtocol<OpenAIProtocol>;

/// 腾讯云混元服务提供商类型
pub type TencentProvider = crate::core::GenericProvider<TencentProtocol>;

/// 创建腾讯云混元服务提供商
///
/// # 参数
/// - `api_key`: 腾讯云混元 API 密钥 (格式: sk-...)
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::tencent;
///
/// let provider = tencent("sk-YMiR2Q7LNWVKVWKivkfPn49geQXT27OZXumFkSS3Ef6FlQ50").unwrap();
/// ```
pub fn tencent(api_key: &str) -> Result<TencentProvider, LlmConnectorError> {
    tencent_with_config(api_key, None, None, None)
}

/// 创建带有自定义配置的腾讯云混元服务提供商
///
/// # 参数
/// - `api_key`: API 密钥
/// - `base_url`: 自定义基础 URL (可选，默认为腾讯云混元端点)
/// - `timeout_secs`: 超时时间(秒) (可选)
/// - `proxy`: 代理 URL (可选)
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::tencent_with_config;
///
/// let provider = tencent_with_config(
///     "sk-YMiR2Q7LNWVKVWKivkfPn49geQXT27OZXumFkSS3Ef6FlQ50",
///     None, // 使用默认 URL
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
    // 创建配置驱动的协议
    let protocol = ConfigurableProtocol::openai_compatible(
        OpenAIProtocol::new(api_key),
        "tencent"
    );

    // 使用 Builder 构建
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

