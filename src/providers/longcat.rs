//! LongCat API 服务提供商实现
//!
//! LongCat 支持两种 API 格式：
//! 1. OpenAI 格式 - 使用 OpenAI 兼容接口
//! 2. Anthropic 格式 - 使用 Anthropic 兼容接口，但认证方式为 Bearer token
//!
//! 注意：LongCat 的 Anthropic 格式使用 `Authorization: Bearer` 认证，
//! 而不是标准 Anthropic 的 `x-api-key` 认证。

use crate::core::{HttpClient, Protocol};
use crate::protocols::AnthropicProtocol;
use crate::error::LlmConnectorError;
use std::collections::HashMap;

/// LongCat Anthropic 格式协议适配器
/// 
/// 这个适配器包装了标准的 AnthropicProtocol，但使用 Bearer 认证而不是 x-api-key
#[derive(Clone, Debug)]
pub struct LongCatAnthropicProtocol {
    inner: AnthropicProtocol,
    api_key: String,
}

impl LongCatAnthropicProtocol {
    /// 创建新的 LongCat Anthropic 协议实例
    pub fn new(api_key: &str) -> Self {
        Self {
            inner: AnthropicProtocol::new(api_key),
            api_key: api_key.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Protocol for LongCatAnthropicProtocol {
    type Request = <AnthropicProtocol as Protocol>::Request;
    type Response = <AnthropicProtocol as Protocol>::Response;
    
    fn name(&self) -> &str {
        "longcat-anthropic"
    }
    
    fn chat_endpoint(&self, base_url: &str) -> String {
        self.inner.chat_endpoint(base_url)
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
    
    /// LongCat 使用 Bearer 认证而不是 x-api-key
    fn auth_headers(&self) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", self.api_key)),
            // Content-Type 由 HttpClient::post() 的 .json() 方法自动设置
            ("anthropic-version".to_string(), "2023-06-01".to_string()),
        ]
    }
    
    #[cfg(feature = "streaming")]
    async fn parse_stream_response(&self, response: reqwest::Response) -> Result<crate::types::ChatStream, LlmConnectorError> {
        self.inner.parse_stream_response(response).await
    }
}

/// LongCat Anthropic 格式服务提供商类型
pub type LongCatAnthropicProvider = crate::core::GenericProvider<LongCatAnthropicProtocol>;

/// 创建 LongCat Anthropic 格式服务提供商
/// 
/// # 参数
/// - `api_key`: LongCat API 密钥 (格式: ak_...)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::longcat_anthropic;
/// 
/// let provider = longcat_anthropic("ak_...").unwrap();
/// ```
pub fn longcat_anthropic(api_key: &str) -> Result<LongCatAnthropicProvider, LlmConnectorError> {
    longcat_anthropic_with_config(api_key, None, None, None)
}

/// 创建带有自定义配置的 LongCat Anthropic 服务提供商
/// 
/// # 参数
/// - `api_key`: API 密钥
/// - `base_url`: 自定义基础 URL (可选，默认为 LongCat Anthropic 端点)
/// - `timeout_secs`: 超时时间(秒) (可选)
/// - `proxy`: 代理 URL (可选)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::longcat_anthropic_with_config;
/// 
/// let provider = longcat_anthropic_with_config(
///     "ak_...",
///     None, // 使用默认 URL
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
    // 创建协议实例
    let protocol = LongCatAnthropicProtocol::new(api_key);
    
    // 创建 HTTP 客户端
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://api.longcat.chat/anthropic"),
        timeout_secs,
        proxy,
    )?;
    
    // 添加认证头
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);
    
    // 创建通用提供商
    Ok(crate::core::GenericProvider::new(protocol, client))
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
        let protocol = LongCatAnthropicProtocol::new("ak_test123");
        let headers = protocol.auth_headers();
        
        // 应该使用 Bearer 认证
        assert!(headers.iter().any(|(k, v)| k == "Authorization" && v == "Bearer ak_test123"));
        
        // 应该包含 anthropic-version
        assert!(headers.iter().any(|(k, v)| k == "anthropic-version" && v == "2023-06-01"));
        
        // 不应该包含 x-api-key
        assert!(!headers.iter().any(|(k, _)| k == "x-api-key"));
    }
}

