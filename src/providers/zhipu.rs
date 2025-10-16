//! 智谱GLM服务提供商实现 - V2架构
//!
//! 这个模块提供智谱GLM服务的完整实现，支持原生格式和OpenAI兼容格式。

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::protocols::ZhipuProtocol;
use crate::error::LlmConnectorError;
use std::collections::HashMap;

/// 智谱GLM服务提供商类型
pub type ZhipuProvider = GenericProvider<ZhipuProtocol>;

/// 创建智谱GLM服务提供商 (使用原生格式)
/// 
/// # 参数
/// - `api_key`: 智谱GLM API密钥
/// 
/// # 返回
/// 配置好的智谱服务提供商实例
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::zhipu;
/// 
/// let provider = zhipu("your-api-key").unwrap();
/// ```
pub fn zhipu(api_key: &str) -> Result<ZhipuProvider, LlmConnectorError> {
    zhipu_with_config(api_key, false, None, None, None)
}

/// 创建智谱GLM服务提供商 (使用OpenAI兼容格式)
/// 
/// # 参数
/// - `api_key`: 智谱GLM API密钥
/// 
/// # 返回
/// 配置好的智谱服务提供商实例 (OpenAI兼容模式)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::zhipu_openai_compatible;
/// 
/// let provider = zhipu_openai_compatible("your-api-key").unwrap();
/// ```
pub fn zhipu_openai_compatible(api_key: &str) -> Result<ZhipuProvider, LlmConnectorError> {
    zhipu_with_config(api_key, true, None, None, None)
}

/// 创建带有自定义配置的智谱GLM服务提供商
/// 
/// # 参数
/// - `api_key`: API密钥
/// - `openai_compatible`: 是否使用OpenAI兼容格式
/// - `base_url`: 自定义基础URL (可选)
/// - `timeout_secs`: 超时时间(秒) (可选)
/// - `proxy`: 代理URL (可选)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::zhipu_with_config;
/// 
/// let provider = zhipu_with_config(
///     "your-api-key",
///     true, // 使用OpenAI兼容格式
///     None, // 使用默认URL
///     Some(60), // 60秒超时
///     None
/// ).unwrap();
/// ```
pub fn zhipu_with_config(
    api_key: &str,
    openai_compatible: bool,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<ZhipuProvider, LlmConnectorError> {
    // 创建协议实例
    let protocol = if openai_compatible {
        ZhipuProtocol::new_openai_compatible(api_key)
    } else {
        ZhipuProtocol::new(api_key)
    };
    
    // 创建HTTP客户端
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://open.bigmodel.cn"),
        timeout_secs,
        proxy,
    )?;
    
    // 添加认证头
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);
    
    // 创建通用提供商
    Ok(GenericProvider::new(protocol, client))
}

/// 创建智谱GLM服务提供商 (默认配置)
/// 
/// 这是一个便利函数，使用推荐的默认配置。
/// 
/// # 参数
/// - `api_key`: API密钥
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::zhipu_default;
/// 
/// let provider = zhipu_default("your-api-key").unwrap();
/// ```
pub fn zhipu_default(api_key: &str) -> Result<ZhipuProvider, LlmConnectorError> {
    // 默认使用OpenAI兼容格式，因为它更标准
    zhipu_openai_compatible(api_key)
}

/// 创建带有自定义超时的智谱GLM服务提供商
/// 
/// # 参数
/// - `api_key`: API密钥
/// - `timeout_secs`: 超时时间(秒)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::zhipu_with_timeout;
/// 
/// // 设置120秒超时
/// let provider = zhipu_with_timeout("your-api-key", 120).unwrap();
/// ```
pub fn zhipu_with_timeout(
    api_key: &str,
    timeout_secs: u64,
) -> Result<ZhipuProvider, LlmConnectorError> {
    zhipu_with_config(api_key, true, None, Some(timeout_secs), None)
}

/// 创建用于智谱GLM企业版的服务提供商
/// 
/// # 参数
/// - `api_key`: 企业版API密钥
/// - `enterprise_endpoint`: 企业版端点URL
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::zhipu_enterprise;
/// 
/// let provider = zhipu_enterprise(
///     "your-enterprise-key",
///     "https://enterprise.bigmodel.cn"
/// ).unwrap();
/// ```
pub fn zhipu_enterprise(
    api_key: &str,
    enterprise_endpoint: &str,
) -> Result<ZhipuProvider, LlmConnectorError> {
    zhipu_with_config(api_key, true, Some(enterprise_endpoint), None, None)
}

/// 验证智谱GLM API密钥格式
/// 
/// # 参数
/// - `api_key`: 要验证的API密钥
/// 
/// # 返回
/// 如果格式看起来正确返回true，否则返回false
/// 
/// # 示例
/// ```rust
/// use llm_connector::provider::validate_zhipu_key;
/// 
/// assert!(validate_zhipu_key("your-valid-key"));
/// assert!(!validate_zhipu_key(""));
/// ```
pub fn validate_zhipu_key(api_key: &str) -> bool {
    !api_key.is_empty() && api_key.len() > 10
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_zhipu_provider_creation() {
        let provider = zhipu("test-key");
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.protocol().name(), "zhipu");
        assert_eq!(provider.protocol().api_key(), "test-key");
        assert!(!provider.protocol().is_openai_compatible());
    }
    
    #[test]
    fn test_zhipu_openai_compatible() {
        let provider = zhipu_openai_compatible("test-key");
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.protocol().name(), "zhipu");
        assert!(provider.protocol().is_openai_compatible());
    }
    
    #[test]
    fn test_zhipu_with_config() {
        let provider = zhipu_with_config(
            "test-key",
            true,
            Some("https://custom.bigmodel.cn"),
            Some(60),
            None
        );
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.client().base_url(), "https://custom.bigmodel.cn");
        assert!(provider.protocol().is_openai_compatible());
    }
    
    #[test]
    fn test_zhipu_default() {
        let provider = zhipu_default("test-key");
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert!(provider.protocol().is_openai_compatible());
    }
    
    #[test]
    fn test_zhipu_with_timeout() {
        let provider = zhipu_with_timeout("test-key", 120);
        assert!(provider.is_ok());
    }
    
    #[test]
    fn test_zhipu_enterprise() {
        let provider = zhipu_enterprise("test-key", "https://enterprise.bigmodel.cn");
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.client().base_url(), "https://enterprise.bigmodel.cn");
    }
    
    #[test]
    fn test_validate_zhipu_key() {
        assert!(validate_zhipu_key("valid-test-key"));
        assert!(validate_zhipu_key("another-valid-key-12345"));
        assert!(!validate_zhipu_key("short"));
        assert!(!validate_zhipu_key(""));
    }
}
