//! Provider Build器 - 统一Build接口
//!
//! this模块Providea优雅 Builder 模式 API，forBuild各种 Provider。

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::error::LlmConnectorError;
use std::collections::HashMap;

/// Provider Build器
///
/// Provides fluent API to build Provider, handling all configuration items uniformly.
///
/// # Example
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
    /// Create新 Provider Build器
    ///
    /// # Parameters
    /// - `protocol`: Protocol instance
    /// - `base_url`: 基础 URL
    ///
    /// # Example
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

    /// Set timeout时间（秒）
    ///
    /// # Example
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

    /// Set proxy
    ///
    /// # Example
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

    /// 添加Additional HTTP headers
    ///
    /// Note: These headers will be merged with the protocol's authentication headers.
    ///
    /// # Example
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

    /// Build Provider
    ///
    /// # Returns
    /// Configured GenericProvider instance
    ///
    /// # Errors
    /// Returns error if HTTP client creation fails
    ///
    /// # Example
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
        // Create HTTP client
        let client = HttpClient::with_config(
            &self.base_url,
            self.timeout_secs,
            self.proxy.as_deref(),
        )?;

        // 合并authentication头andAdditionalheaders
        let mut headers: HashMap<String, String> =
            self.protocol.auth_headers().into_iter().collect();
        headers.extend(self.extra_headers);
        let client = client.with_headers(headers);

        // Create通用Provide商
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

