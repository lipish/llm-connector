//! OpenAI服务Provide商实现 - V2架构
//!
//! this模块ProvideOpenAI服务完整实现，Use统一V2架构。

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::protocols::OpenAIProtocol;
use crate::error::LlmConnectorError;
use std::collections::HashMap;

/// OpenAI服务Provide商类型
pub type OpenAIProvider = GenericProvider<OpenAIProtocol>;

/// CreateOpenAI服务Provide商
/// 
/// # Parameters
/// - `api_key`: OpenAI API key
/// 
/// # Returns
/// configuration好OpenAI服务Provide商instance
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::openai;
/// 
/// let provider = openai("sk-...").unwrap();
/// ```
pub fn openai(api_key: &str) -> Result<OpenAIProvider, LlmConnectorError> {
    openai_with_config(api_key, None, None, None)
}

/// Create带有Custom base URLOpenAI服务Provide商
/// 
/// # Parameters
/// - `api_key`: API key
/// - `base_url`: Custom base URL (such asforOpenAI兼容服务)
/// 
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::openai_with_base_url;
/// 
/// // Usecustomendpoint (such asAzure OpenAI)
/// let provider = openai_with_base_url("sk-...", "https://your-resource.openai.azure.com").unwrap();
/// ```
pub fn openai_with_base_url(api_key: &str, base_url: &str) -> Result<OpenAIProvider, LlmConnectorError> {
    openai_with_config(api_key, Some(base_url), None, None)
}

/// Create带有customconfigurationOpenAI服务Provide商
/// 
/// # Parameters
/// - `api_key`: API key
/// - `base_url`: Custom base URL (optional)
/// - `timeout_secs`: 超时时间(秒) (optional)
/// - `proxy`: 代理URL (optional)
/// 
/// # Example
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
    // CreateProtocol instance
    let protocol = OpenAIProtocol::new(api_key);
    
    // CreateHTTP Client
    let client = HttpClient::with_config(
        base_url.unwrap_or("https://api.openai.com"),
        timeout_secs,
        proxy,
    )?;
    
    // 添加authentication头
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);
    
    // Create通用Provide商
    Ok(GenericProvider::new(protocol, client))
}

/// CreateforAzure OpenAI服务Provide商
/// 
/// # Parameters
/// - `api_key`: Azure OpenAI API key
/// - `endpoint`: Azure OpenAI endpoint (such as "https://your-resource.openai.azure.com")
/// - `api_version`: API version (such as "2024-02-15-preview")
/// 
/// # Example
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

    // Content-Type 由 HttpClient::post()  .json() method自动Set
    let client = HttpClient::new(endpoint)?
        .with_header("api-key".to_string(), api_key.to_string())
        .with_header("api-version".to_string(), api_version.to_string());

    Ok(GenericProvider::new(protocol, client))
}

/// CreateforOpenAI兼容服务Provide商
/// 
/// thisfunctionas各种OpenAI兼容服务Provide便利Createmethod，
/// such asDeepSeek、Moonshot、Together AIetc.。
/// 
/// # Parameters
/// - `api_key`: API key
/// - `base_url`: 服务基础URL
/// - `service_name`: Service name (forErrors消息)
/// 
/// # Example
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

    // Content-Type 由 HttpClient::post()  .json() method自动Set
    let client = HttpClient::new(base_url)?
        .with_header("Authorization".to_string(), format!("Bearer {}", api_key))
        .with_header("User-Agent".to_string(), format!("llm-connector/{}", service_name));

    Ok(GenericProvider::new(protocol, client))
}

/// ValidateOpenAI API key格式
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
