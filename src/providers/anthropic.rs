//! Anthropic Claude Service Provider Implementation - V2 Architecture
//!
//! This module provides complete Anthropic Claude service implementation，using unified V2 architecture。

use crate::core::{GenericProvider, HttpClient, Protocol};
use crate::error::LlmConnectorError;
use crate::protocols::AnthropicProtocol;
use std::collections::HashMap;

/// Anthropic ClaudeserviceProvidertype
pub type AnthropicProvider = GenericProvider<AnthropicProtocol>;

/// CreateAnthropic ClaudeserviceProvider
///
/// # Parameters
/// - `api_key`: Anthropic API key (Format: sk-ant-...)
///
/// # Returns
/// Configured Anthropic service Provider instance
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

/// CreatewithcustomconfigurationAnthropicserviceProvider
///
/// # Parameters
/// - `api_key`: API key
/// - `base_url`: Custom base URL (optional, defaults to official endpoint)
/// - `timeout_secs`: Timeout (seconds) (optional)
/// - `proxy`: Proxy URL (optional)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::anthropic_with_config;
///
/// let provider = anthropic_with_config(
///     "sk-ant-...",
///     None, // Use default URL
///     Some(60), // 60 seconds timeout
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

    // Add authentication headers
    let auth_headers: HashMap<String, String> = protocol.auth_headers().into_iter().collect();
    let client = client.with_headers(auth_headers);

    // Create generic provider
    Ok(GenericProvider::new(protocol, client))
}

/// CreateforAnthropic Vertex AIserviceProvider
///
/// # Parameters
/// - `project_id`: Google Cloud project ID
/// - `location`: Region (such as "us-central1")
/// - `access_token`: Google Cloud access token
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
    let protocol = AnthropicProtocol::new(""); // Vertex AI does not require Anthropic API key

    let base_url = format!(
        "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/anthropic",
        location, project_id, location
    );

    let client = HttpClient::new(&base_url)?.with_header(
        "Authorization".to_string(),
        format!("Bearer {}", access_token),
    );
    // Note: Content-Type is automatically set by HttpClient::post() .json() method

    Ok(GenericProvider::new(protocol, client))
}

/// CreateforAmazon BedrockAnthropicserviceProvider
///
/// # Parameters
/// - `region`: AWS region (such as "us-east-1")
/// - `access_key_id`: AWS access key ID
/// - `secret_access_key`: AWS secret access key
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
    let protocol = AnthropicProtocol::new(""); // Bedrock does not require Anthropic API key

    let base_url = format!("https://bedrock-runtime.{}.amazonaws.com", region);

    // Note: This simplifies AWS signature process，actual use requires AWS SigV4 signature implementation
    // Content-Type is automatically set by HttpClient::post() .json() method
    let client = HttpClient::new(&base_url)?.with_header(
        "X-Amz-Target".to_string(),
        "BedrockRuntime_20231002.InvokeModel".to_string(),
    );

    Ok(GenericProvider::new(protocol, client))
}

/// CreatewithcustomtimeoutAnthropicserviceProvider
///
/// Some Anthropic models may require longer processing time，this function provides convenient timeout configuration。
///
/// # Parameters
/// - `api_key`: API key
/// - `timeout_secs`: Timeout (seconds)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::anthropic_with_timeout;
///
/// // Set 120 seconds timeout, suitable for long text processing
/// let provider = anthropic_with_timeout("sk-ant-...", 120).unwrap();
/// ```
pub fn anthropic_with_timeout(
    api_key: &str,
    timeout_secs: u64,
) -> Result<AnthropicProvider, LlmConnectorError> {
    anthropic_with_config(api_key, None, Some(timeout_secs), None)
}

/// ValidateAnthropic API keyformat
///
/// # Parameters
/// - `api_key`: API key to validate
///
/// # Returns
/// Returns true if format is correct, otherwise returns false
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
            None,
        );
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.client().base_url(), "https://custom.anthropic.com");
    }

    #[test]
    fn test_anthropic_vertex() {
        let provider = anthropic_vertex("test-project", "us-central1", "test-token");
        assert!(provider.is_ok());
    }

    #[test]
    fn test_anthropic_bedrock() {
        let provider = anthropic_bedrock("us-east-1", "test-key-id", "test-secret");
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
