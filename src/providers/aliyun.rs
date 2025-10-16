//! 阿里云DashScope服务提供商实现 - V2架构
//!
//! 这个模块提供阿里云DashScope服务的完整实现，使用统一的V2架构。

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::protocols::AliyunProtocol;
use crate::error::LlmConnectorError;
use std::collections::HashMap;

/// 阿里云DashScope服务提供商类型
pub type AliyunProvider = GenericProvider<AliyunProtocol>;

/// 创建阿里云DashScope服务提供商
/// 
/// # 参数
/// - `api_key`: 阿里云DashScope API密钥
/// 
/// # 返回
/// 配置好的阿里云服务提供商实例
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::aliyun;
/// 
/// let provider = aliyun("sk-...").unwrap();
/// ```
pub fn aliyun(api_key: &str) -> Result<AliyunProvider, LlmConnectorError> {
    aliyun_with_config(api_key, None, None, None)
}

/// 创建带有自定义配置的阿里云服务提供商
/// 
/// # 参数
/// - `api_key`: API密钥
/// - `base_url`: 自定义基础URL (可选，默认为官方端点)
/// - `timeout_secs`: 超时时间(秒) (可选)
/// - `proxy`: 代理URL (可选)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::aliyun_with_config;
/// 
/// let provider = aliyun_with_config(
///     "sk-...",
///     None, // 使用默认URL
///     Some(60), // 60秒超时
///     Some("http://proxy:8080")
/// ).unwrap();
/// ```
pub fn aliyun_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<AliyunProvider, LlmConnectorError> {
    // 创建协议实例
    let protocol = AliyunProtocol::new(api_key);
    
    // 创建HTTP客户端
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://dashscope.aliyuncs.com"),
        timeout_secs,
        proxy,
    )?;
    
    // 添加认证头
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);
    
    // 创建通用提供商
    Ok(GenericProvider::new(protocol, client))
}

/// 创建用于阿里云国际版的服务提供商
/// 
/// # 参数
/// - `api_key`: 阿里云国际版API密钥
/// - `region`: 区域 (如 "us-west-1", "ap-southeast-1")
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::aliyun_international;
/// 
/// let provider = aliyun_international("sk-...", "us-west-1").unwrap();
/// ```
pub fn aliyun_international(
    api_key: &str,
    region: &str,
) -> Result<AliyunProvider, LlmConnectorError> {
    let base_url = format!("https://dashscope.{}.aliyuncs.com", region);
    aliyun_with_config(api_key, Some(&base_url), None, None)
}

/// 创建用于阿里云专有云的服务提供商
/// 
/// # 参数
/// - `api_key`: API密钥
/// - `endpoint`: 专有云端点URL
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::aliyun_private;
/// 
/// let provider = aliyun_private(
///     "sk-...",
///     "https://dashscope.your-private-cloud.com"
/// ).unwrap();
/// ```
pub fn aliyun_private(
    api_key: &str,
    endpoint: &str,
) -> Result<AliyunProvider, LlmConnectorError> {
    aliyun_with_config(api_key, Some(endpoint), None, None)
}

/// 创建带有自定义超时的阿里云服务提供商
/// 
/// 阿里云的某些模型可能需要较长的处理时间，这个函数提供便利的超时配置。
/// 
/// # 参数
/// - `api_key`: API密钥
/// - `timeout_secs`: 超时时间(秒)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::aliyun_with_timeout;
/// 
/// // 设置120秒超时，适用于长文本处理
/// let provider = aliyun_with_timeout("sk-...", 120).unwrap();
/// ```
pub fn aliyun_with_timeout(
    api_key: &str,
    timeout_secs: u64,
) -> Result<AliyunProvider, LlmConnectorError> {
    aliyun_with_config(api_key, None, Some(timeout_secs), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aliyun_provider_creation() {
        let provider = aliyun("test-key");
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.protocol().name(), "aliyun");
        assert_eq!(provider.protocol().api_key(), "test-key");
    }
    
    #[test]
    fn test_aliyun_with_config() {
        let provider = aliyun_with_config(
            "test-key",
            Some("https://custom.dashscope.com"),
            Some(60),
            None
        );
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.client().base_url(), "https://custom.dashscope.com");
    }
    
    #[test]
    fn test_aliyun_international() {
        let provider = aliyun_international("test-key", "us-west-1");
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.client().base_url(), "https://dashscope.us-west-1.aliyuncs.com");
    }
    
    #[test]
    fn test_aliyun_private() {
        let provider = aliyun_private("test-key", "https://private.dashscope.com");
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.client().base_url(), "https://private.dashscope.com");
    }
    
    #[test]
    fn test_aliyun_with_timeout() {
        let provider = aliyun_with_timeout("test-key", 120);
        assert!(provider.is_ok());
    }
}
