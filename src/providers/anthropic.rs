//! Anthropic Claude服务提供商实现 - V2架构
//!
//! 这个模块提供Anthropic Claude服务的完整实现，使用统一的V2架构。

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::protocols::AnthropicProtocol;
use crate::error::LlmConnectorError;
use std::collections::HashMap;

/// Anthropic Claude服务提供商类型
pub type AnthropicProvider = GenericProvider<AnthropicProtocol>;

/// 创建Anthropic Claude服务提供商
/// 
/// # 参数
/// - `api_key`: Anthropic API密钥 (格式: sk-ant-...)
/// 
/// # 返回
/// 配置好的Anthropic服务提供商实例
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::anthropic;
/// 
/// let provider = anthropic("sk-ant-...").unwrap();
/// ```
pub fn anthropic(api_key: &str) -> Result<AnthropicProvider, LlmConnectorError> {
    anthropic_with_config(api_key, None, None, None)
}

/// 创建带有自定义配置的Anthropic服务提供商
/// 
/// # 参数
/// - `api_key`: API密钥
/// - `base_url`: 自定义基础URL (可选，默认为官方端点)
/// - `timeout_secs`: 超时时间(秒) (可选)
/// - `proxy`: 代理URL (可选)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::anthropic_with_config;
/// 
/// let provider = anthropic_with_config(
///     "sk-ant-...",
///     None, // 使用默认URL
///     Some(60), // 60秒超时
///     Some("http://proxy:8080")
/// ).unwrap();
/// ```
pub fn anthropic_with_config(
    api_key: &str,
    base_url: Option<&str>,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<AnthropicProvider, LlmConnectorError> {
    // 创建协议实例
    let protocol = AnthropicProtocol::new(api_key);
    
    // 创建HTTP客户端
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://api.anthropic.com"),
        timeout_secs,
        proxy,
    )?;
    
    // 添加认证头
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);
    
    // 创建通用提供商
    Ok(GenericProvider::new(protocol, client))
}

/// 创建用于Anthropic Vertex AI的服务提供商
/// 
/// # 参数
/// - `project_id`: Google Cloud项目ID
/// - `location`: 区域 (如 "us-central1")
/// - `access_token`: Google Cloud访问令牌
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::anthropic_vertex;
/// 
/// let provider = anthropic_vertex(
///     "my-project-id",
///     "us-central1",
///     "ya29...."
/// ).unwrap();
/// ```
pub fn anthropic_vertex(
    project_id: &str,
    location: &str,
    access_token: &str,
) -> Result<AnthropicProvider, LlmConnectorError> {
    let protocol = AnthropicProtocol::new(""); // Vertex AI不需要Anthropic API密钥
    
    let base_url = format!(
        "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/anthropic",
        location, project_id, location
    );
    
    let client = HttpClient::new(&base_url)?
        .with_header("Authorization".to_string(), format!("Bearer {}", access_token))
        .with_header("Content-Type".to_string(), "application/json".to_string());
    
    Ok(GenericProvider::new(protocol, client))
}

/// 创建用于Amazon Bedrock的Anthropic服务提供商
/// 
/// # 参数
/// - `region`: AWS区域 (如 "us-east-1")
/// - `access_key_id`: AWS访问密钥ID
/// - `secret_access_key`: AWS秘密访问密钥
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::anthropic_bedrock;
/// 
/// let provider = anthropic_bedrock(
///     "us-east-1",
///     "AKIA...",
///     "..."
/// ).unwrap();
/// ```
pub fn anthropic_bedrock(
    region: &str,
    _access_key_id: &str,
    _secret_access_key: &str,
) -> Result<AnthropicProvider, LlmConnectorError> {
    let protocol = AnthropicProtocol::new(""); // Bedrock不需要Anthropic API密钥
    
    let base_url = format!("https://bedrock-runtime.{}.amazonaws.com", region);
    
    // 注意: 这里简化了AWS签名过程，实际使用中需要实现AWS SigV4签名
    let client = HttpClient::new(&base_url)?
        .with_header("Content-Type".to_string(), "application/json".to_string())
        .with_header("X-Amz-Target".to_string(), "BedrockRuntime_20231002.InvokeModel".to_string());
    
    Ok(GenericProvider::new(protocol, client))
}

/// 创建带有自定义超时的Anthropic服务提供商
/// 
/// Anthropic的某些模型可能需要较长的处理时间，这个函数提供便利的超时配置。
/// 
/// # 参数
/// - `api_key`: API密钥
/// - `timeout_secs`: 超时时间(秒)
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::provider::anthropic_with_timeout;
/// 
/// // 设置120秒超时，适用于长文本处理
/// let provider = anthropic_with_timeout("sk-ant-...", 120).unwrap();
/// ```
pub fn anthropic_with_timeout(
    api_key: &str,
    timeout_secs: u64,
) -> Result<AnthropicProvider, LlmConnectorError> {
    anthropic_with_config(api_key, None, Some(timeout_secs), None)
}

/// 验证Anthropic API密钥格式
/// 
/// # 参数
/// - `api_key`: 要验证的API密钥
/// 
/// # 返回
/// 如果格式正确返回true，否则返回false
/// 
/// # 示例
/// ```rust
/// use llm_connector::provider::validate_anthropic_key;
/// 
/// assert!(validate_anthropic_key("sk-ant-api03-..."));
/// assert!(!validate_anthropic_key("sk-..."));
/// ```
pub fn validate_anthropic_key(api_key: &str) -> bool {
    api_key.starts_with("sk-ant-")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_anthropic_provider_creation() {
        let provider = anthropic("sk-ant-test-key");
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.protocol().name(), "anthropic");
        assert_eq!(provider.protocol().api_key(), "sk-ant-test-key");
    }
    
    #[test]
    fn test_anthropic_with_config() {
        let provider = anthropic_with_config(
            "sk-ant-test-key",
            Some("https://custom.anthropic.com"),
            Some(60),
            None
        );
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.client().base_url(), "https://custom.anthropic.com");
    }
    
    #[test]
    fn test_anthropic_vertex() {
        let provider = anthropic_vertex(
            "test-project",
            "us-central1",
            "test-token"
        );
        assert!(provider.is_ok());
    }
    
    #[test]
    fn test_anthropic_bedrock() {
        let provider = anthropic_bedrock(
            "us-east-1",
            "test-key-id",
            "test-secret"
        );
        assert!(provider.is_ok());
    }
    
    #[test]
    fn test_anthropic_with_timeout() {
        let provider = anthropic_with_timeout("sk-ant-test-key", 120);
        assert!(provider.is_ok());
    }
    
    #[test]
    fn test_validate_anthropic_key() {
        assert!(validate_anthropic_key("sk-ant-api03-test"));
        assert!(validate_anthropic_key("sk-ant-test"));
        assert!(!validate_anthropic_key("sk-test"));
        assert!(!validate_anthropic_key("test"));
        assert!(!validate_anthropic_key(""));
    }
}
