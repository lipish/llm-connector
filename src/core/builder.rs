//! Provider 构建器 - 统一的构建接口
//!
//! 这个模块提供了一个优雅的 Builder 模式 API，用于构建各种 Provider。

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::error::LlmConnectorError;
use std::collections::HashMap;

/// Provider 构建器
///
/// 提供链式调用的 API 来构建 Provider，统一处理所有配置项。
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::core::{ProviderBuilder, Protocol};
/// use llm_connector::protocols::OpenAIProtocol;
///
/// let provider = ProviderBuilder::new(
///     OpenAIProtocol::new("sk-..."),
///     "https://api.openai.com"
/// )
/// .timeout(60)
/// .proxy("http://proxy:8080")
/// .header("X-Custom-Header", "value")
/// .build()
/// .unwrap();
/// ```
pub struct ProviderBuilder<P: Protocol> {
    protocol: P,
    base_url: String,
    timeout_secs: Option<u64>,
    proxy: Option<String>,
    extra_headers: HashMap<String, String>,
}

impl<P: Protocol> ProviderBuilder<P> {
    /// 创建新的 Provider 构建器
    ///
    /// # 参数
    /// - `protocol`: 协议实例
    /// - `base_url`: 基础 URL
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::core::ProviderBuilder;
    /// use llm_connector::protocols::OpenAIProtocol;
    ///
    /// let builder = ProviderBuilder::new(
    ///     OpenAIProtocol::new("sk-..."),
    ///     "https://api.openai.com"
    /// );
    /// ```
    pub fn new(protocol: P, base_url: &str) -> Self {
        Self {
            protocol,
            base_url: base_url.to_string(),
            timeout_secs: None,
            proxy: None,
            extra_headers: HashMap::new(),
        }
    }

    /// 设置超时时间（秒）
    ///
    /// # 示例
    /// ```rust,no_run
    /// # use llm_connector::core::ProviderBuilder;
    /// # use llm_connector::protocols::OpenAIProtocol;
    /// let builder = ProviderBuilder::new(
    ///     OpenAIProtocol::new("sk-..."),
    ///     "https://api.openai.com"
    /// )
    /// .timeout(60);  // 60秒超时
    /// ```
    pub fn timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    /// 设置代理
    ///
    /// # 示例
    /// ```rust,no_run
    /// # use llm_connector::core::ProviderBuilder;
    /// # use llm_connector::protocols::OpenAIProtocol;
    /// let builder = ProviderBuilder::new(
    ///     OpenAIProtocol::new("sk-..."),
    ///     "https://api.openai.com"
    /// )
    /// .proxy("http://proxy:8080");
    /// ```
    pub fn proxy(mut self, proxy: &str) -> Self {
        self.proxy = Some(proxy.to_string());
        self
    }

    /// 添加额外的 HTTP 头部
    ///
    /// 注意：这些头部会与协议的认证头部合并。
    ///
    /// # 示例
    /// ```rust,no_run
    /// # use llm_connector::core::ProviderBuilder;
    /// # use llm_connector::protocols::OpenAIProtocol;
    /// let builder = ProviderBuilder::new(
    ///     OpenAIProtocol::new("sk-..."),
    ///     "https://api.openai.com"
    /// )
    /// .header("X-Custom-Header", "value")
    /// .header("X-Another-Header", "value2");
    /// ```
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.extra_headers.insert(key.to_string(), value.to_string());
        self
    }

    /// 构建 Provider
    ///
    /// # 返回
    /// 配置好的 GenericProvider 实例
    ///
    /// # 错误
    /// 如果 HTTP 客户端创建失败，返回错误
    ///
    /// # 示例
    /// ```rust,no_run
    /// # use llm_connector::core::ProviderBuilder;
    /// # use llm_connector::protocols::OpenAIProtocol;
    /// let provider = ProviderBuilder::new(
    ///     OpenAIProtocol::new("sk-..."),
    ///     "https://api.openai.com"
    /// )
    /// .timeout(60)
    /// .build()
    /// .unwrap();
    /// ```
    pub fn build(self) -> Result<GenericProvider<P>, LlmConnectorError> {
        // 创建 HTTP 客户端
        let client = HttpClient::with_config(
            &self.base_url,
            self.timeout_secs,
            self.proxy.as_deref(),
        )?;

        // 合并认证头和额外头部
        let mut headers: HashMap<String, String> =
            self.protocol.auth_headers().into_iter().collect();
        headers.extend(self.extra_headers);
        let client = client.with_headers(headers);

        // 创建通用提供商
        Ok(GenericProvider::new(self.protocol, client))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::OpenAIProtocol;

    #[test]
    fn test_builder_basic() {
        let provider = ProviderBuilder::new(
            OpenAIProtocol::new("sk-test"),
            "https://api.openai.com"
        )
        .build();

        assert!(provider.is_ok());
    }

    #[test]
    fn test_builder_with_timeout() {
        let provider = ProviderBuilder::new(
            OpenAIProtocol::new("sk-test"),
            "https://api.openai.com"
        )
        .timeout(60)
        .build();

        assert!(provider.is_ok());
    }

    #[test]
    fn test_builder_with_proxy() {
        let provider = ProviderBuilder::new(
            OpenAIProtocol::new("sk-test"),
            "https://api.openai.com"
        )
        .proxy("http://proxy:8080")
        .build();

        assert!(provider.is_ok());
    }

    #[test]
    fn test_builder_with_headers() {
        let provider = ProviderBuilder::new(
            OpenAIProtocol::new("sk-test"),
            "https://api.openai.com"
        )
        .header("X-Custom-Header", "value")
        .header("X-Another-Header", "value2")
        .build();

        assert!(provider.is_ok());
    }

    #[test]
    fn test_builder_chain() {
        let provider = ProviderBuilder::new(
            OpenAIProtocol::new("sk-test"),
            "https://api.openai.com"
        )
        .timeout(60)
        .proxy("http://proxy:8080")
        .header("X-Custom-Header", "value")
        .build();

        assert!(provider.is_ok());
    }
}

