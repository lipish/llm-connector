//! Tencent Cloud Hunyuan Service Provider Implementation
//!
//! This module provides the Tencent Cloud Hunyuan service implementation using the Native API v3 (TC3-HMAC-SHA256).
//! Reference: https://cloud.tencent.com/document/api/1729/101837

#![cfg(feature = "tencent")]

use crate::core::{HttpClient, Protocol};
use crate::error::LlmConnectorError;
use crate::protocols::tencent_native::TencentNativeProtocol;
use crate::types::{ChatRequest, ChatResponse};
#[cfg(feature = "streaming")]
use crate::types::ChatStream;
use async_trait::async_trait;
use chrono::Utc;

/// Tencent Cloud Service Provider Type
pub type TencentProvider = TencentProviderImpl;

/// Tencent Provider Implementation
pub struct TencentProviderImpl {
    protocol: TencentNativeProtocol,
    client: HttpClient,
}

impl TencentProviderImpl {
    pub fn protocol(&self) -> &TencentNativeProtocol {
        &self.protocol
    }

    pub fn client(&self) -> &HttpClient {
        &self.client
    }

    fn sign_request(&self, payload: &str) -> Result<Vec<(String, String)>, LlmConnectorError> {
        // Current UTC time
        let now = Utc::now();
        let timestamp = now.timestamp();
        let date = now.format("%Y-%m-%d").to_string();
        let host = "hunyuan.tencentcloudapi.com";

        self.protocol.calculate_signature(host, payload, timestamp, &date)
            .map(|auth| {
                vec![
                    ("Authorization".to_string(), auth),
                    ("X-TC-Action".to_string(), "ChatCompletions".to_string()),
                    ("X-TC-Version".to_string(), "2023-09-01".to_string()),
                    ("X-TC-Timestamp".to_string(), timestamp.to_string()),
                    ("X-TC-Region".to_string(), "ap-guangzhou".to_string()), // TODO: Make configurable
                ]
            })
    }
}

#[async_trait]
impl crate::core::Provider for TencentProviderImpl {
    fn name(&self) -> &str {
        "tencent"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        let protocol_request = self.protocol.build_request(request)?;
        let url = self.protocol.chat_endpoint(self.client.base_url());

        // Serialize body to string for signing
        let body = serde_json::to_string(&protocol_request)
            .map_err(|e| LlmConnectorError::InvalidRequest(format!("Serialize error: {}", e)))?;

        // Sign request
        let headers = self.sign_request(&body)?;

        // Send request with headers
        let client_with_auth = self.client.clone().with_headers(headers.into_iter().collect());
        
        let response = client_with_auth.post(&url, &protocol_request).await?;
        let status = response.status();
        let text = response.text().await
            .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;

        if !status.is_success() {
             return Err(self.protocol.map_error(status.as_u16(), &text));
        }

        self.protocol.parse_response(&text)
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        let mut streaming_request = request.clone();
        streaming_request.stream = Some(true);

        let protocol_request = self.protocol.build_request(&streaming_request)?;
        let url = self.protocol.chat_endpoint(self.client.base_url());

        // Serialize body to string for signing
        // Note: usage of serde_json::to_string MUST match reqwest .json() serialization
        let body = serde_json::to_string(&protocol_request)
            .map_err(|e| LlmConnectorError::InvalidRequest(format!("Serialize error: {}", e)))?;

        // Sign request
        let headers = self.sign_request(&body)?;

        // Send request with headers
        let client_with_auth = self.client.clone().with_headers(headers.into_iter().collect());
        
        let response = client_with_auth.stream(&url, &protocol_request).await?;
        let status = response.status();

        if !status.is_success() {
             let text = response.text().await
                .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;
             return Err(self.protocol.map_error(status.as_u16(), &text));
        }

        self.protocol.parse_stream_response(response).await
    }

    async fn models(&self) -> Result<Vec<String>, LlmConnectorError> {
        Ok(vec!["hunyuan-lite".to_string(), "hunyuan-standard".to_string(), "hunyuan-pro".to_string()])
    }
}

/// Create Tencent Cloud Hunyuan Service Provider
///
/// # Parameters
/// - `secret_id`: Tencent Cloud SecretID
/// - `secret_key`: Tencent Cloud SecretKey
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::tencent;
///
/// let provider = tencent("AKID...", "Processing...").unwrap();
/// ```
pub fn tencent(secret_id: &str, secret_key: &str) -> Result<TencentProvider, LlmConnectorError> {
    tencent_with_config(secret_id, secret_key, None, None)
}

/// Create Tencent Provider with custom configuration
pub fn tencent_with_config(
    secret_id: &str,
    secret_key: &str,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<TencentProvider, LlmConnectorError> {
    let protocol = TencentNativeProtocol::new(secret_id, secret_key);
    
    // Official endpoint for Hunyuan
    let base_url = "https://hunyuan.tencentcloudapi.com";
    
    let client = HttpClient::with_config(
        base_url,
        timeout_secs,
        proxy,
    )?;

    Ok(TencentProviderImpl {
        protocol,
        client,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tencent_creation() {
        let provider = tencent("AKID_TEST", "KEY_TEST");
        assert!(provider.is_ok());
    }
}

