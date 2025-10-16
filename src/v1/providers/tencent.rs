//! Tencent Cloud Protocol Implementation
//!
//! This module implements the Tencent Cloud native API protocol.
//! Unlike the OpenAI-compatible interface, this uses Tencent Cloud's native API
//! with TC3-HMAC-SHA256 signature authentication.
//!
//! # Supported Models
//!
//! - **Tencent Cloud** - `tencent()` - hunyuan-lite, hunyuan-standard, hunyuan-pro
//!
//! # Protocol Differences
//!
//! Tencent Hunyuan native API differs significantly from OpenAI:
//!
//! ## 1. Endpoint
//! - Hunyuan: `POST https://hunyuan.tencentcloudapi.com/`
//! - OpenAI: `POST /v1/chat/completions`
//!
//! ## 2. Authentication
//! - **TC3-HMAC-SHA256**: Complex signature-based authentication
//! - **Headers**: Authorization, X-TC-Action, X-TC-Version, X-TC-Timestamp, etc.
//!
//! ## 3. Request Structure
//! - **Action-based**: Uses `Action: ChatCompletions` parameter
//! - **Version**: Requires `Version: 2023-09-01`
//! - **Region**: Requires region specification
//!
//! ## 4. Response Structure
//! - **Response wrapper**: Data wrapped in `Response` object
//! - **RequestId**: Tencent-specific request tracking
//! - **Error format**: Tencent Cloud error structure
//!
//! # Request Format
//!
//! ```json
//! {
//!   "Model": "hunyuan-lite",
//!   "Messages": [
//!     {"Role": "user", "Content": "Hello"}
//!   ],
//!   "Stream": false,
//!   "Temperature": 0.7,
//!   "TopP": 0.9
//! }
//! ```
//!
//! # Response Format
//!
//! ```json
//! {
//!   "Response": {
//!     "Choices": [{
//!       "Message": {
//!         "Role": "assistant",
//!         "Content": "Hello! How can I help you?"
//!       },
//!       "FinishReason": "stop"
//!     }],
//!     "Usage": {
//!       "PromptTokens": 10,
//!       "CompletionTokens": 20,
//!       "TotalTokens": 30
//!     },
//!     "RequestId": "req-123456"
//!   }
//! }
//! ```

use crate::error::LlmConnectorError;
use crate::v1::protocols::core::{ErrorMapper, ProviderAdapter, Provider, HttpTransport};
use crate::types::{ChatRequest, ChatResponse, Message, Role, Usage};
use crate::config::ProviderConfig;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::collections::HashMap;

#[cfg(feature = "tencent")]
use {
    hmac::{Hmac, Mac},
    sha2::{Digest, Sha256},
};

#[cfg(feature = "streaming")]
use crate::types::StreamingResponse;

// ============================================================================
// Tencent Cloud API V3 Signature Implementation
// ============================================================================

#[cfg(feature = "tencent")]
#[derive(Debug)]
pub struct TencentCloudSigner {
    secret_id: String,
    secret_key: String,
    service: String,
}

#[cfg(feature = "tencent")]
impl TencentCloudSigner {
    pub fn new(secret_id: String, secret_key: String, service: String) -> Self {
        Self {
            secret_id,
            secret_key,
            service,
        }
    }

    /// Generate TC3-HMAC-SHA256 signature
    pub fn sign_request(
        &self,
        method: &str,
        uri: &str,
        query_string: &str,
        headers: &HashMap<String, String>,
        payload: &str,
        timestamp: i64,
    ) -> Result<String, LlmConnectorError> {
        // Step 1: Create canonical request
        let canonical_request = self.create_canonical_request(method, uri, query_string, headers, payload)?;

        if std::env::var("LLM_DEBUG_REQUEST_RAW").map(|v| v == "1").unwrap_or(false) {
            eprintln!("[signature-debug] Canonical request:\n{}", canonical_request);
        }
        
        // Step 2: Create string to sign
        let date = chrono::DateTime::from_timestamp(timestamp, 0)
            .ok_or_else(|| LlmConnectorError::ConfigError("Invalid timestamp".to_string()))?
            .format("%Y-%m-%d")
            .to_string();
        
        let credential_scope = format!("{}/{}/tc3_request", date, self.service);
        let string_to_sign = format!(
            "TC3-HMAC-SHA256\n{}\n{}\n{}",
            timestamp,
            credential_scope,
            hex::encode(Sha256::digest(canonical_request.as_bytes()))
        );

        if std::env::var("LLM_DEBUG_REQUEST_RAW").map(|v| v == "1").unwrap_or(false) {
            eprintln!("[signature-debug] String to sign:\n{}", string_to_sign);
        }

        // Step 3: Calculate signature
        let signature = self.calculate_signature(&string_to_sign, &date)?;

        // Step 4: Create authorization header
        let authorization = format!(
            "TC3-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            self.secret_id,
            credential_scope,
            self.get_signed_headers(headers),
            signature
        );

        Ok(authorization)
    }

    pub fn create_canonical_request(
        &self,
        method: &str,
        uri: &str,
        query_string: &str,
        headers: &HashMap<String, String>,
        payload: &str,
    ) -> Result<String, LlmConnectorError> {
        let canonical_headers = self.get_canonical_headers(headers);
        let signed_headers = self.get_signed_headers(headers);
        let hashed_payload = hex::encode(Sha256::digest(payload.as_bytes()));

        Ok(format!(
            "{}\n{}\n{}\n{}\n\n{}\n{}",
            method, uri, query_string, canonical_headers, signed_headers, hashed_payload
        ))
    }

    fn get_canonical_headers(&self, headers: &HashMap<String, String>) -> String {
        let mut sorted_headers: Vec<_> = headers.iter().collect();
        sorted_headers.sort_by_key(|(k, _)| k.to_lowercase());
        
        sorted_headers
            .iter()
            .map(|(k, v)| format!("{}:{}", k.to_lowercase(), v.trim()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn get_signed_headers(&self, headers: &HashMap<String, String>) -> String {
        let mut header_names: Vec<_> = headers.keys().map(|k| k.to_lowercase()).collect();
        header_names.sort();
        header_names.join(";")
    }

    fn calculate_signature(&self, string_to_sign: &str, date: &str) -> Result<String, LlmConnectorError> {
        type HmacSha256 = Hmac<Sha256>;

        let k_date = HmacSha256::new_from_slice(format!("TC3{}", self.secret_key).as_bytes())
            .map_err(|e| LlmConnectorError::ConfigError(format!("HMAC key error: {}", e)))?
            .chain_update(date.as_bytes())
            .finalize()
            .into_bytes();

        let k_service = HmacSha256::new_from_slice(&k_date)
            .map_err(|e| LlmConnectorError::ConfigError(format!("HMAC key error: {}", e)))?
            .chain_update(self.service.as_bytes())
            .finalize()
            .into_bytes();

        let k_signing = HmacSha256::new_from_slice(&k_service)
            .map_err(|e| LlmConnectorError::ConfigError(format!("HMAC key error: {}", e)))?
            .chain_update(b"tc3_request")
            .finalize()
            .into_bytes();

        let signature = HmacSha256::new_from_slice(&k_signing)
            .map_err(|e| LlmConnectorError::ConfigError(format!("HMAC key error: {}", e)))?
            .chain_update(string_to_sign.as_bytes())
            .finalize()
            .into_bytes();

        Ok(hex::encode(signature))
    }
}

// ============================================================================
// Hunyuan-specific Request Structures
// ============================================================================

#[derive(Debug, Serialize)]
pub struct TencentRequest {
    #[serde(rename = "Model")]
    model: String,
    #[serde(rename = "Messages")]
    messages: Vec<TencentMessage>,
    #[serde(rename = "Stream", skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(rename = "Temperature", skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(rename = "TopP", skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    // 注意：腾讯混元API不支持MaxTokens参数
    // #[serde(rename = "MaxTokens", skip_serializing_if = "Option::is_none")]
    // max_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct TencentMessage {
    #[serde(rename = "Role")]
    role: String,
    #[serde(rename = "Content")]
    content: String,
}

// ============================================================================
// Hunyuan-specific Response Structures
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct TencentResponse {
    #[serde(rename = "Response")]
    response: HunyuanResponseData,
}

#[derive(Debug, Deserialize)]
pub struct HunyuanResponseData {
    #[serde(rename = "Choices")]
    choices: Vec<HunyuanChoice>,
    #[serde(rename = "Usage")]
    usage: HunyuanUsage,
    #[serde(rename = "RequestId")]
    request_id: String,
}

#[derive(Debug, Deserialize)]
pub struct HunyuanChoice {
    #[serde(rename = "Message")]
    message: HunyuanResponseMessage,
    #[serde(rename = "FinishReason")]
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct HunyuanResponseMessage {
    #[serde(rename = "Role")]
    role: String,
    #[serde(rename = "Content")]
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct HunyuanUsage {
    #[serde(rename = "PromptTokens")]
    prompt_tokens: i32,
    #[serde(rename = "CompletionTokens")]
    completion_tokens: i32,
    #[serde(rename = "TotalTokens")]
    total_tokens: i32,
}

// ============================================================================
// Hunyuan-specific Streaming Response
// ============================================================================

#[cfg(feature = "streaming")]
#[derive(Debug, Deserialize)]
pub struct HunyuanStreamResponse {
    #[serde(rename = "Response")]
    response: HunyuanStreamResponseData,
}

#[cfg(feature = "streaming")]
#[derive(Debug, Deserialize)]
pub struct HunyuanStreamResponseData {
    #[serde(rename = "Choices")]
    choices: Vec<HunyuanStreamChoice>,
    #[serde(rename = "Usage", skip_serializing_if = "Option::is_none")]
    usage: Option<HunyuanUsage>,
    #[serde(rename = "RequestId")]
    request_id: String,
}

#[cfg(feature = "streaming")]
#[derive(Debug, Deserialize)]
pub struct HunyuanStreamChoice {
    #[serde(rename = "Delta")]
    delta: HunyuanDelta,
    #[serde(rename = "FinishReason", skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
}

#[cfg(feature = "streaming")]
#[derive(Debug, Deserialize)]
pub struct HunyuanDelta {
    #[serde(rename = "Role", skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(rename = "Content", skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

// ============================================================================
// Tencent Error Mapper
// ============================================================================

pub struct TencentErrorMapper;

impl ErrorMapper for TencentErrorMapper {
    fn map_http_error(status: u16, body: Value) -> LlmConnectorError {
        let error_message = body["Response"]["Error"]["Message"]
            .as_str()
            .or_else(|| body["Error"]["Message"].as_str())
            .or_else(|| body["message"].as_str())
            .unwrap_or("Unknown Tencent error");

        let error_code = body["Response"]["Error"]["Code"]
            .as_str()
            .or_else(|| body["Error"]["Code"].as_str())
            .or_else(|| body["code"].as_str())
            .unwrap_or("unknown");

        match status {
            400 => LlmConnectorError::InvalidRequest(format!(
                "Hunyuan: {} ({})",
                error_message, error_code
            )),
            401 => LlmConnectorError::AuthenticationError(format!(
                "Hunyuan: {} ({}). Please verify your SecretId and SecretKey are correct.",
                error_message, error_code
            )),
            403 => LlmConnectorError::PermissionError(format!(
                "Hunyuan: {} ({})",
                error_message, error_code
            )),
            429 => LlmConnectorError::RateLimitError(format!(
                "Hunyuan: {} ({})",
                error_message, error_code
            )),
            500..=599 => LlmConnectorError::ServerError(format!(
                "Hunyuan HTTP {}: {} ({})",
                status, error_message, error_code
            )),
            _ => LlmConnectorError::ProviderError(format!(
                "Hunyuan HTTP {}: {} ({})",
                status, error_message, error_code
            )),
        }
    }

    fn map_network_error(error: reqwest::Error) -> LlmConnectorError {
        if error.is_timeout() {
            LlmConnectorError::TimeoutError(format!("Tencent: {}", error))
        } else if error.is_connect() {
            LlmConnectorError::ConnectionError(format!("Tencent: {}", error))
        } else {
            LlmConnectorError::NetworkError(format!("Tencent: {}", error))
        }
    }

    fn is_retriable_error(error: &LlmConnectorError) -> bool {
        matches!(
            error,
            LlmConnectorError::RateLimitError(_)
                | LlmConnectorError::ServerError(_)
                | LlmConnectorError::TimeoutError(_)
                | LlmConnectorError::ConnectionError(_)
        )
    }
}

// ============================================================================
// Hunyuan Native Protocol Adapter Implementation
// ============================================================================

/// Hunyuan Native Protocol adapter for Tencent Cloud API
///
/// Uses TC3-HMAC-SHA256 signature authentication and native Tencent Cloud API format.
#[derive(Debug, Clone)]
pub struct TencentProtocol {
    base_url: Arc<str>,
    #[cfg(feature = "tencent")]
    signer: Option<Arc<TencentCloudSigner>>,
    region: String,
}

impl TencentProtocol {
    /// Create new Hunyuan native protocol with SecretId and SecretKey
    ///
    /// Uses default Tencent Cloud Hunyuan endpoint
    #[cfg(feature = "tencent")]
    pub fn new(secret_id: &str, secret_key: &str, region: Option<&str>) -> Self {
        let region = region.unwrap_or("ap-beijing").to_string();
        let signer = TencentCloudSigner::new(
            secret_id.to_string(),
            secret_key.to_string(),
            "hunyuan".to_string(),
        );

        Self {
            base_url: Arc::from("https://hunyuan.tencentcloudapi.com/"),
            signer: Some(Arc::new(signer)),
            region,
        }
    }

    /// Create new Hunyuan native protocol without signature (for testing)
    #[cfg(not(feature = "tencent"))]
    pub fn new(_secret_id: &str, _secret_key: &str, region: Option<&str>) -> Self {
        Self {
            base_url: Arc::from("https://hunyuan.tencentcloudapi.com/"),
            region: region.unwrap_or("ap-beijing").to_string(),
        }
    }

    /// Create new Hunyuan native protocol with custom endpoint
    #[cfg(feature = "tencent")]
    pub fn with_url(secret_id: &str, secret_key: &str, base_url: &str, region: Option<&str>) -> Self {
        let region = region.unwrap_or("ap-beijing").to_string();
        let signer = TencentCloudSigner::new(
            secret_id.to_string(),
            secret_key.to_string(),
            "hunyuan".to_string(),
        );

        Self {
            base_url: Arc::from(base_url),
            signer: Some(Arc::new(signer)),
            region,
        }
    }

    /// Parse a role string into a Role enum
    fn parse_role(role: &str) -> Role {
        match role {
            "system" => Role::System,
            "user" => Role::User,
            "assistant" => Role::Assistant,
            "tool" => Role::Tool,
            _ => Role::User, // Default to user for unknown roles
        }
    }

    /// Generate authentication headers for Tencent Cloud API
    #[cfg(feature = "tencent")]
    pub fn generate_auth_headers(
        &self,
        payload: &str,
        timestamp: i64,
    ) -> Result<HashMap<String, String>, LlmConnectorError> {
        let signer = self.signer.as_ref()
            .ok_or_else(|| LlmConnectorError::ConfigError("Signer not initialized".to_string()))?;

        // 只对必要的headers进行签名（按照腾讯云官方示例）
        let mut sign_headers = HashMap::new();
        sign_headers.insert("content-type".to_string(), "application/json; charset=utf-8".to_string());
        sign_headers.insert("host".to_string(), "hunyuan.tencentcloudapi.com".to_string());
        sign_headers.insert("x-tc-action".to_string(), "chatcompletions".to_string());

        let authorization = signer.sign_request(
            "POST",
            "/",
            "",
            &sign_headers,
            payload,
            timestamp,
        )?;

        // 构建完整的发送headers
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json; charset=utf-8".to_string());
        headers.insert("host".to_string(), "hunyuan.tencentcloudapi.com".to_string());
        headers.insert("x-tc-action".to_string(), "ChatCompletions".to_string());
        headers.insert("x-tc-version".to_string(), "2023-09-01".to_string());
        headers.insert("x-tc-timestamp".to_string(), timestamp.to_string());
        headers.insert("x-tc-region".to_string(), self.region.clone());

        headers.insert("authorization".to_string(), authorization);

        // Convert to proper HTTP header names for sending
        let mut http_headers = HashMap::new();
        for (key, value) in headers {
            let http_key = match key.as_str() {
                "content-type" => "Content-Type",
                "host" => "Host",
                "x-tc-action" => "X-TC-Action",
                "x-tc-version" => "X-TC-Version",
                "x-tc-timestamp" => "X-TC-Timestamp",
                "x-tc-region" => "X-TC-Region",
                "authorization" => "Authorization",
                _ => &key,
            };
            http_headers.insert(http_key.to_string(), value);
        }

        Ok(http_headers)
    }


}

#[async_trait]
impl ProviderAdapter for TencentProtocol {
    type RequestType = TencentRequest;
    type ResponseType = TencentResponse;
    #[cfg(feature = "streaming")]
    type StreamResponseType = HunyuanStreamResponse;
    type ErrorMapperType = TencentErrorMapper;

    fn name(&self) -> &str {
        "tencent"
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        base_url.as_deref().unwrap_or(&self.base_url).to_string()
    }

    fn build_request_data(&self, request: &ChatRequest, stream: bool) -> Self::RequestType {
        let messages = request
            .messages
            .iter()
            .map(|msg| TencentMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect();

        TencentRequest {
            model: request.model.clone(),
            messages,
            stream: if stream { Some(true) } else { None },
            temperature: request.temperature,
            top_p: request.top_p,
            // 腾讯混元API不支持MaxTokens参数，已移除
        }
    }

    fn parse_response_data(&self, response: Self::ResponseType) -> ChatResponse {
        let response_data = response.response;

        // Convenience: capture first choice content before moving choices
        let first_content = response_data
            .choices
            .get(0)
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        ChatResponse {
            id: response_data.request_id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: "hunyuan".to_string(), // Hunyuan doesn't return model in response
            choices: response_data
                .choices
                .into_iter()
                .enumerate()
                .map(|(index, choice)| crate::types::Choice {
                    index: index as u32,
                    message: Message {
                        role: Self::parse_role(&choice.message.role),
                        content: choice.message.content,
                        name: None,
                        tool_calls: None,
                        tool_call_id: None,
                        ..Default::default()
                    },
                    finish_reason: Some(choice.finish_reason),
                    logprobs: None,
                })
                .collect(),
            content: first_content,
            usage: Some(Usage {
                prompt_tokens: response_data.usage.prompt_tokens as u32,
                completion_tokens: response_data.usage.completion_tokens as u32,
                total_tokens: response_data.usage.total_tokens as u32,
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        }
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> StreamingResponse {
        let response_data = response.response;

        // Convenience: capture first chunk content before moving choices
        let first_chunk_content = response_data
            .choices
            .get(0)
            .and_then(|c| c.delta.content.clone())
            .unwrap_or_default();

        StreamingResponse {
            id: response_data.request_id,
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: "hunyuan".to_string(),
            choices: response_data
                .choices
                .into_iter()
                .enumerate()
                .map(|(index, choice)| crate::types::StreamingChoice {
                    index: index as u32,
                    delta: crate::types::Delta {
                        role: choice.delta.role.map(|r| Self::parse_role(&r)),
                        content: choice.delta.content,
                        tool_calls: None,
                        reasoning_content: None,
                        ..Default::default()
                    },
                    finish_reason: choice.finish_reason,
                    logprobs: None,
                })
                .collect(),
            content: first_chunk_content,
            reasoning_content: None,
            usage: response_data.usage.map(|usage| Usage {
                prompt_tokens: usage.prompt_tokens as u32,
                completion_tokens: usage.completion_tokens as u32,
                total_tokens: usage.total_tokens as u32,
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        }
    }
}

// ============================================================================
// Convenience Functions and Type Aliases
// ============================================================================

/// Create a Tencent provider
pub fn tencent(secret_id: &str, secret_key: &str, region: Option<&str>) -> Result<TencentProvider, LlmConnectorError> {
    let adapter = TencentProtocol::new(secret_id, secret_key, region);
    let config = ProviderConfig::new(secret_id)
        .with_base_url("https://hunyuan.tencentcloudapi.com/")
        .with_timeout_ms(30000);
    TencentProvider::new(config, adapter)
}

/// Create a Tencent provider with custom timeout
pub fn tencent_with_timeout(secret_id: &str, secret_key: &str, region: Option<&str>, timeout_ms: u64) -> Result<TencentProvider, LlmConnectorError> {
    let adapter = TencentProtocol::new(secret_id, secret_key, region);
    let config = ProviderConfig::new(secret_id)
        .with_base_url("https://hunyuan.tencentcloudapi.com/")
        .with_timeout_ms(timeout_ms);
    TencentProvider::new(config, adapter)
}

// ============================================================================
// Custom Tencent Provider Implementation
// ============================================================================

/// Custom Tencent Provider that handles TC3-HMAC-SHA256 authentication
#[derive(Clone)]
pub struct TencentProvider {
    adapter: TencentProtocol,
    transport: HttpTransport,
}

impl TencentProvider {
    pub fn new(config: ProviderConfig, adapter: TencentProtocol) -> Result<Self, LlmConnectorError> {
        let client = HttpTransport::build_client(
            &config.proxy,
            config.timeout_ms,
            config.base_url.as_ref(),
        )?;

        let transport = HttpTransport::new(client, config);

        Ok(Self { adapter, transport })
    }

    /// Send HTTP request with Tencent Cloud authentication headers
    async fn send_authenticated_request(
        &self,
        request_data: &TencentRequest,
        _stream: bool,
    ) -> Result<reqwest::Response, LlmConnectorError> {
        let url = self.adapter.endpoint_url(&self.transport.config.base_url);
        let payload = serde_json::to_string(request_data)
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;

        let timestamp = chrono::Utc::now().timestamp();

        #[cfg(feature = "tencent")]
        let auth_headers = self.adapter.generate_auth_headers(&payload, timestamp)?;

        #[cfg(not(feature = "tencent"))]
        let auth_headers = self.adapter.generate_auth_headers(&payload, timestamp)?;

        if std::env::var("LLM_DEBUG_REQUEST_RAW").map(|v| v == "1").unwrap_or(false) {
            eprintln!("[request-debug] URL: {}", url);
            eprintln!("[request-debug] Payload: {}", payload);
            for (key, value) in &auth_headers {
                eprintln!("[request-debug] Header: {}: {}", key, value);
            }
        }

        let mut request_builder = self.transport.client
            .post(&url)
            .body(payload);

        // Add all authentication headers
        for (key, value) in auth_headers {
            request_builder = request_builder.header(&key, &value);
        }

        let response = request_builder
            .send()
            .await
            .map_err(LlmConnectorError::from)?;

        Ok(response)
    }
}

#[async_trait]
impl Provider for TencentProvider {
    fn name(&self) -> &str {
        self.adapter.name()
    }

    async fn fetch_models(&self) -> Result<Vec<String>, LlmConnectorError> {
        // Hunyuan native API doesn't provide a models endpoint
        Err(LlmConnectorError::UnsupportedOperation(
            "Hunyuan native API does not support model listing".to_string()
        ))
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        let request_data = self.adapter.build_request_data(request, false);
        let response = self.send_authenticated_request(&request_data, false).await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body: Value = response.json().await.unwrap_or_default();
            return Err(TencentErrorMapper::map_http_error(status, body));
        }

        let text = response
            .text()
            .await
            .map_err(|e| LlmConnectorError::ParseError(e.to_string()))?;

        if std::env::var("LLM_DEBUG_RESPONSE_RAW").map(|v| v == "1").unwrap_or(false) {
            eprintln!("[response-raw] {}", text);
        }

        let response_data: TencentResponse = serde_json::from_str(&text)
            .map_err(|e| LlmConnectorError::ParseError(format!("Failed to parse response: {}", e)))?;

        Ok(self.adapter.parse_response_data(response_data))
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, _request: &ChatRequest) -> Result<crate::types::ChatStream, LlmConnectorError> {
        // For now, return an error as streaming requires more complex implementation
        Err(LlmConnectorError::UnsupportedOperation(
            "Streaming not yet implemented for Hunyuan native API".to_string()
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
