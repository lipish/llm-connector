//! Anthropic Claude服务Provide商实现 - V2架构
//!
//! this模块ProvideAnthropic Claude服务完整实现，Use统一V2架构。

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::protocols::AnthropicProtocol;
use crate::error::LlmConnectorError;
use std::collections::HashMap;

/// Anthropic Claude服务Provide商类型
pub type AnthropicProvider = GenericProvider<AnthropicProtocol>;

/// CreateAnthropic Claude服务Provide商
/// 
/// # Parameters
/// - `api_key`: Anthropic API key (格式: sk-ant-...)
/// 
/// # Returns
/// configuration好Anthropic服务Provide商instance
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::anthropic;
/// 
/// let provider = anthropic("sk-ant-...").unwrap();
/// ```
pub fn anthropic(api_key: &str) -> Result<AnthropicProvider, LlmConnectorError> {
    anthropic_with_config(api_key, None, None, None)
}

/// Create带有customconfigurationAnthropic服务Provide商
/// 
/// # Parameters
/// - `api_key`: API key
/// - `base_url`: Custom base URL (optional，默认as官方endpoint)
/// - `timeout_secs`: 超时时间(秒) (optional)
/// - `proxy`: 代理URL (optional)
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::anthropic_with_config;
/// 
/// let provider = anthropic_with_config(
///     "sk-ant-...",
///     None, // Use默认URL
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
    // CreateProtocol instance
    let protocol = AnthropicProtocol::new(api_key);
    
    // CreateHTTP Client
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://api.anthropic.com"),
        timeout_secs,
        proxy,
    )?;
    
    // 添加authentication头
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);
    
    // Create通用Provide商
    Ok(GenericProvider::new(protocol, client))
}

/// CreateforAnthropic Vertex AI服务Provide商
/// 
/// # Parameters
/// - `project_id`: Google Cloud项目ID
/// - `location`: 区域 (such as "us-central1")
/// - `access_token`: Google Cloud访问令牌
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::anthropic_vertex;
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
    let protocol = AnthropicProtocol::new(""); // Vertex AI不需要Anthropic API key
    
    let base_url = format!(
        "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/anthropic",
        location, project_id, location
    );
    
    let client = HttpClient::new(&base_url)?
        .with_header("Authorization".to_string(), format!("Bearer {}", access_token));
        // Note: Content-Type 由 HttpClient::post()  .json() method自动Set

    Ok(GenericProvider::new(protocol, client))
}

/// CreateforAmazon BedrockAnthropic服务Provide商
/// 
/// # Parameters
/// - `region`: AWS区域 (such as "us-east-1")
/// - `access_key_id`: AWS访问密钥ID
/// - `secret_access_key`: AWS秘密访问密钥
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::anthropic_bedrock;
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
    let protocol = AnthropicProtocol::new(""); // Bedrock不需要Anthropic API key
    
    let base_url = format!("https://bedrock-runtime.{}.amazonaws.com", region);
    
    // Note: 这里简化AWS签名过程，实际Use中需要实现AWS SigV4签名
    // Content-Type 由 HttpClient::post()  .json() method自动Set
    let client = HttpClient::new(&base_url)?
        .with_header("X-Amz-Target".to_string(), "BedrockRuntime_20231002.InvokeModel".to_string());

    Ok(GenericProvider::new(protocol, client))
}

/// Create带有custom超时Anthropic服务Provide商
/// 
/// Anthropic某些modelmay需要较长处理时间，thisfunctionProvide便利超时configuration。
/// 
/// # Parameters
/// - `api_key`: API key
/// - `timeout_secs`: 超时时间(秒)
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::anthropic_with_timeout;
/// 
/// // Set120秒超时，适for长文本处理
/// let provider = anthropic_with_timeout("sk-ant-...", 120).unwrap();
/// ```
pub fn anthropic_with_timeout(
    api_key: &str,
    timeout_secs: u64,
) -> Result<AnthropicProvider, LlmConnectorError> {
    anthropic_with_config(api_key, None, Some(timeout_secs), None)
}

/// ValidateAnthropic API key格式
/// 
/// # Parameters
/// - `api_key`: 要ValidateAPI key
/// 
/// # Returns
/// if格式正确Returnstrue，否则Returnsfalse
/// 
/// # Example
/// ```rust
/// use llm_connector::providers::validate_anthropic_key;
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
