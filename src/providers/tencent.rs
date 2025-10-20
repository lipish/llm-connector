//! 腾讯云混元（Tencent Hunyuan）服务提供商实现
//!
//! 腾讯云混元使用 OpenAI 兼容的 API 格式，完全兼容标准 OpenAI 协议。

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;
use std::collections::HashMap;

/// 腾讯云混元协议适配器
/// 
/// 包装 OpenAI protocol，使用腾讯云混元的端点
#[derive(Clone, Debug)]
pub struct TencentProtocol {
    inner: OpenAIProtocol,
}

impl TencentProtocol {
    /// 创建新的腾讯云混元协议实例
    pub fn new(api_key: &str) -> Self {
        Self {
            inner: OpenAIProtocol::new(api_key),
        }
    }
}

#[async_trait::async_trait]
impl Protocol for TencentProtocol {
    type Request = <OpenAIProtocol as Protocol>::Request;
    type Response = <OpenAIProtocol as Protocol>::Response;
    
    fn name(&self) -> &str {
        "tencent"
    }
    
    fn chat_endpoint(&self, base_url: &str) -> String {
        self.inner.chat_endpoint(base_url)
    }
    
    fn models_endpoint(&self, base_url: &str) -> Option<String> {
        self.inner.models_endpoint(base_url)
    }
    
    fn build_request(&self, request: &crate::types::ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        self.inner.build_request(request)
    }
    
    fn parse_response(&self, response: &str) -> Result<crate::types::ChatResponse, LlmConnectorError> {
        self.inner.parse_response(response)
    }
    
    fn map_error(&self, status: u16, message: &str) -> LlmConnectorError {
        self.inner.map_error(status, message)
    }
    
    fn auth_headers(&self) -> Vec<(String, String)> {
        self.inner.auth_headers()
    }
    
    #[cfg(feature = "streaming")]
    async fn parse_stream_response(&self, response: reqwest::Response) -> Result<crate::types::ChatStream, LlmConnectorError> {
        self.inner.parse_stream_response(response).await
    }
}

/// 腾讯云混元服务提供商类型
pub type TencentProvider = GenericProvider<TencentProtocol>;

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
    // 创建协议实例
    let protocol = TencentProtocol::new(api_key);
    
    // 创建 HTTP 客户端
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://api.hunyuan.cloud.tencent.com"),
        timeout_secs,
        proxy,
    )?;
    
    // 添加认证头
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);
    
    // 创建通用提供商
    Ok(GenericProvider::new(protocol, client))
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
        let protocol = TencentProtocol::new("sk-test-key");
        let endpoint = protocol.chat_endpoint("https://api.hunyuan.cloud.tencent.com");
        assert_eq!(endpoint, "https://api.hunyuan.cloud.tencent.com/v1/chat/completions");
    }
    
    #[test]
    fn test_tencent_protocol_name() {
        let protocol = TencentProtocol::new("sk-test-key");
        assert_eq!(protocol.name(), "tencent");
    }
}

