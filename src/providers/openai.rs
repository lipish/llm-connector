//! OpenAI服务提供商实现 - V2架构
//!
//! 这个模块提供OpenAI服务的完整实现，使用统一的V2架构。

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;
use std::collections::HashMap;

/// OpenAI服务提供商类型
pub type OpenAIProvider = GenericProvider<OpenAIProtocol>;

/// 创建OpenAI服务提供商
/// 
/// # 参数
/// - `api_key`: OpenAI API密钥
/// 
/// # 返回
/// 配置好的OpenAI服务提供商实例
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::openai;
/// 
/// let provider = openai("sk-...").unwrap();
/// ```
pub fn openai(api_key: &str) -> Result<OpenAIProvider, LlmConnectorError> {
    openai_with_config(api_key, None, None, None)
}

/// 创建带有自定义基础URL的OpenAI服务提供商
/// 
/// # 参数
/// - `api_key`: API密钥
/// - `base_url`: 自定义基础URL (如用于OpenAI兼容的服务)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::openai_with_base_url;
/// 
/// // 使用自定义端点 (如Azure OpenAI)
/// let provider = openai_with_base_url("sk-...", "https://your-resource.openai.azure.com").unwrap();
/// ```
pub fn openai_with_base_url(api_key: &str, base_url: &str) -> Result<OpenAIProvider, LlmConnectorError> {
    openai_with_config(api_key, Some(base_url), None, None)
}

/// 创建带有自定义配置的OpenAI服务提供商
/// 
/// # 参数
/// - `api_key`: API密钥
/// - `base_url`: 自定义基础URL (可选)
/// - `timeout_secs`: 超时时间(秒) (可选)
/// - `proxy`: 代理URL (可选)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::openai_with_config;
/// 
/// let provider = openai_with_config(
///     "sk-...",
///     Some("https://api.openai.com"),
///     Some(60), // 60秒超时
///     Some("http://proxy:8080")
/// ).unwrap();
/// ```
pub fn openai_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<OpenAIProvider, LlmConnectorError> {
    // 创建协议实例
    let protocol = OpenAIProtocol::new(api_key);
    
    // 创建HTTP客户端
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://api.openai.com"),
        timeout_secs,
        proxy,
    )?;
    
    // 添加认证头
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);
    
    // 创建通用提供商
    Ok(GenericProvider::new(protocol, client))
}

/// 创建用于Azure OpenAI的服务提供商
/// 
/// # 参数
/// - `api_key`: Azure OpenAI API密钥
/// - `endpoint`: Azure OpenAI端点 (如 "https://your-resource.openai.azure.com")
/// - `api_version`: API版本 (如 "2024-02-15-preview")
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::azure_openai;
/// 
/// let provider = azure_openai(
///     "your-api-key",
///     "https://your-resource.openai.azure.com",
///     "2024-02-15-preview"
/// ).unwrap();
/// ```
pub fn azure_openai(
    api_key: &str,
    endpoint: &str,
    api_version: &str,
) -> Result<OpenAIProvider, LlmConnectorError> {
    let protocol = OpenAIProtocol::new(api_key);
    
    let client = HttpClient::new(endpoint)?
        .with_header("api-key".to_string(), api_key.to_string())
        .with_header("api-version".to_string(), api_version.to_string())
        .with_header("Content-Type".to_string(), "application/json".to_string());
    
    Ok(GenericProvider::new(protocol, client))
}

/// 创建用于OpenAI兼容服务的提供商
/// 
/// 这个函数为各种OpenAI兼容的服务提供便利的创建方法，
/// 如DeepSeek、Moonshot、Together AI等。
/// 
/// # 参数
/// - `api_key`: API密钥
/// - `base_url`: 服务的基础URL
/// - `service_name`: 服务名称 (用于错误消息)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::openai_compatible;
/// 
/// // DeepSeek
/// let deepseek = openai_compatible(
///     "sk-...",
///     "https://api.deepseek.com",
///     "deepseek"
/// ).unwrap();
/// 
/// // Moonshot
/// let moonshot = openai_compatible(
///     "sk-...",
///     "https://api.moonshot.cn",
///     "moonshot"
/// ).unwrap();
/// ```
pub fn openai_compatible(
    api_key: &str,
    base_url: &str,
    service_name: &str,
) -> Result<OpenAIProvider, LlmConnectorError> {
    let protocol = OpenAIProtocol::new(api_key);
    
    let client = HttpClient::new(base_url)?
        .with_header("Authorization".to_string(), format!("Bearer {}", api_key))
        .with_header("Content-Type".to_string(), "application/json".to_string())
        .with_header("User-Agent".to_string(), format!("llm-connector/{}", service_name));
    
    Ok(GenericProvider::new(protocol, client))
}

/// 验证OpenAI API密钥格式
pub fn validate_openai_key(api_key: &str) -> bool {
    api_key.starts_with("sk-") && api_key.len() > 20
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_openai_provider_creation() {
        let provider = openai("test-key");
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.protocol().name(), "openai");
        assert_eq!(provider.protocol().api_key(), "test-key");
    }
    
    #[test]
    fn test_openai_with_base_url() {
        let provider = openai_with_base_url("test-key", "https://custom.api.com");
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.client().base_url(), "https://custom.api.com");
    }
    
    #[test]
    fn test_azure_openai() {
        let provider = azure_openai(
            "test-key",
            "https://test.openai.azure.com",
            "2024-02-15-preview"
        );
        assert!(provider.is_ok());
    }
    
    #[test]
    fn test_openai_compatible() {
        let provider = openai_compatible(
            "test-key",
            "https://api.deepseek.com",
            "deepseek"
        );
        assert!(provider.is_ok());
    }
}
