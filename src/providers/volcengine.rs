//! 火山引擎（Volcengine）服务提供商实现
//!
//! 火山引擎使用 OpenAI 兼容的 API 格式，但端点路径不同：
//! - OpenAI: `/v1/chat/completions`
//! - Volcengine: `/api/v3/chat/completions`

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;
use std::collections::HashMap;

/// 火山引擎协议适配器
/// 
/// 包装 OpenAI protocol，但使用火山引擎的端点路径
#[derive(Clone, Debug)]
pub struct VolcengineProtocol {
    inner: OpenAIProtocol,
}

impl VolcengineProtocol {
    /// 创建新的火山引擎协议实例
    pub fn new(api_key: &str) -> Self {
        Self {
            inner: OpenAIProtocol::new(api_key),
        }
    }
}

#[async_trait::async_trait]
impl Protocol for VolcengineProtocol {
    type Request = <OpenAIProtocol as Protocol>::Request;
    type Response = <OpenAIProtocol as Protocol>::Response;
    
    fn name(&self) -> &str {
        "volcengine"
    }
    
    /// 火山引擎使用 /api/v3/chat/completions 而不是 /v1/chat/completions
    fn chat_endpoint(&self, base_url: &str) -> String {
        format!("{}/api/v3/chat/completions", base_url.trim_end_matches('/'))
    }
    
    fn models_endpoint(&self, base_url: &str) -> Option<String> {
        Some(format!("{}/api/v3/models", base_url.trim_end_matches('/')))
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

/// 火山引擎服务提供商类型
pub type VolcengineProvider = GenericProvider<VolcengineProtocol>;

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
    // 创建协议实例
    let protocol = VolcengineProtocol::new(api_key);
    
    // 创建 HTTP 客户端
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://ark.cn-beijing.volces.com"),
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
        let protocol = VolcengineProtocol::new("test-key");
        let endpoint = protocol.chat_endpoint("https://ark.cn-beijing.volces.com");
        assert_eq!(endpoint, "https://ark.cn-beijing.volces.com/api/v3/chat/completions");
    }
}

