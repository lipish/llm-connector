//! Tencent Cloud Native Protocol Implementation
//!
//! Implements Tencent Cloud API v3 signature (TC3-HMAC-SHA256).
//! Reference: https://cloud.tencent.com/document/api/1729/101837

#![cfg(feature = "tencent")]

use crate::core::Protocol;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, Role, Usage, Choice, Message};
use async_trait::async_trait;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

type HmacSha256 = Hmac<Sha256>;

/// Tencent Cloud Native Protocol
#[derive(Debug, Clone)]
pub struct TencentNativeProtocol {
    secret_id: String,
    secret_key: String,
    region: String,
}

impl TencentNativeProtocol {
    pub fn new(secret_id: &str, secret_key: &str) -> Self {
        Self {
            secret_id: secret_id.to_string(),
            secret_key: secret_key.to_string(),
            region: "ap-guangzhou".to_string(), // Default region
        }
    }

    pub fn with_region(mut self, region: &str) -> Self {
        self.region = region.to_string();
        self
    }

    /// Calculate TC3-HMAC-SHA256 signature
    pub fn calculate_signature(&self, host: &str, payload: &str, timestamp: i64, date: &str) -> Result<String, LlmConnectorError> {
        let service = "hunyuan";
        let algorithm = "TC3-HMAC-SHA256";

        // 1. Concatenate CanonicalRequest
        let http_request_method = "POST";
        let canonical_uri = "/";
        let canonical_query_string = "";
        let canonical_headers = format!("content-type:application/json\nhost:{}\n", host);
        let signed_headers = "content-type;host";
        
        let mut hasher = Sha256::new();
        hasher.update(payload);
        let hashed_request_payload = hex::encode(hasher.finalize());
        
        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            http_request_method,
            canonical_uri,
            canonical_query_string,
            canonical_headers,
            signed_headers,
            hashed_request_payload
        );

        // 2. String to sign
        let credential_scope = format!("{}/{}/tc3_request", date, service);
        
        let mut hasher = Sha256::new();
        hasher.update(canonical_request.as_bytes());
        let hashed_canonical_request = hex::encode(hasher.finalize());
        
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm,
            timestamp,
            credential_scope,
            hashed_canonical_request
        );

        // 3. Calculate Signature
        let k_key = format!("TC3{}", self.secret_key);
        let k_date = hmac_sha256(k_key.as_bytes(), date.as_bytes())?;
        let k_service = hmac_sha256(&k_date, service.as_bytes())?;
        let k_signing = hmac_sha256(&k_service, "tc3_request".as_bytes())?;
        
        let signature = hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes())?);

        // 4. Build Authorization Header
        Ok(format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm,
            self.secret_id,
            credential_scope,
            signed_headers,
            signature
        ))
    }
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> Result<Vec<u8>, LlmConnectorError> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|e| LlmConnectorError::ProviderError(format!("HMAC init failed: {}", e)))?;
    mac.update(message);
    Ok(mac.finalize().into_bytes().to_vec())
}

#[async_trait]
impl Protocol for TencentNativeProtocol {
    type Request = TencentRequest;
    type Response = TencentResponse;

    fn name(&self) -> &str {
        "tencent"
    }

    fn chat_endpoint(&self, _base_url: &str) -> String {
        // Native API uses a fixed endpoint usually, but we respect base_url if provided
        // Official endpoint: https://hunyuan.tencentcloudapi.com
        if _base_url.is_empty() || _base_url == "https://api.openai.com" { 
             "https://hunyuan.tencentcloudapi.com".to_string()
        } else {
             _base_url.to_string()
        }
    }

    fn auth_headers(&self) -> Vec<(String, String)> {
        // Native signature requires body, handled in Provider
        vec![]
    }

    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError> {
        // Convert to Tencent format
        Ok(TencentRequest {
            model: Some(request.model.clone()),
            messages: request.messages.iter().map(|m| TencentMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::System => "system".to_string(),
                    _ => "user".to_string(),
                },
                content: m.content_as_text(),
            }).collect(),
            temperature: request.temperature,
            top_p: request.top_p,
            stream: request.stream,
            enable_thinking: request.enable_thinking,
        })
    }

    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        let resp: TencentResponse = serde_json::from_str(response)
            .map_err(|e| LlmConnectorError::InvalidRequest(format!("Parse error: {}", e)))?;
            
        if let Some(err) = resp.response.error {
            return Err(LlmConnectorError::ApiError(format!("{}: {}", err.code, err.message)));
        }

        let task = resp.response; // Inner wrapper
        
        let content = task.choices.as_ref()
            .and_then(|c| c.first())
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let choice = Choice {
            index: 0,
            message: Message::text(Role::Assistant, &content),
            finish_reason: Some("stop".to_string()),
            logprobs: None,
        };

        Ok(ChatResponse {
            id: task.request_id.unwrap_or_default(),
            object: "chat.completion".to_string(),
            created: Utc::now().timestamp() as u64,
            model: "hunyuan".to_string(), // Model usually not in response
            choices: vec![choice],
            content,
            reasoning_content: None,
            usage: task.usage.map(|u| Usage {
                prompt_tokens: u.prompt_tokens as u32,
                completion_tokens: u.completion_tokens as u32,
                total_tokens: u.total_tokens as u32,
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        })
    }

    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError {
        LlmConnectorError::from_status_code(status, body.to_string())
    }
}


// Internal Types
#[derive(Debug, Serialize, Deserialize)]
pub struct TencentRequest {
    #[serde(rename = "Model")]
    pub model: Option<String>,
    #[serde(rename = "Messages")]
    pub messages: Vec<TencentMessage>,
    #[serde(rename = "Temperature", skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(rename = "TopP", skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(rename = "Stream", skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(rename = "EnableThinking", skip_serializing_if = "Option::is_none")]
    pub enable_thinking: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TencentMessage {
    #[serde(rename = "Role")]
    pub role: String,
    #[serde(rename = "Content")]
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct TencentResponse {
    #[serde(rename = "Response")]
    pub response: TencentResponseInner,
}

#[derive(Debug, Deserialize)]
pub struct TencentResponseInner {
    #[serde(rename = "RequestId")]
    pub request_id: Option<String>,
    #[serde(rename = "Error")]
    pub error: Option<TencentError>,
    #[serde(rename = "Choices")]
    pub choices: Option<Vec<TencentChoice>>,
    #[serde(rename = "Usage")]
    pub usage: Option<TencentUsage>,
}

#[derive(Debug, Deserialize)]
pub struct TencentError {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct TencentChoice {
    #[serde(rename = "Message")]
    pub message: TencentMessage,
    #[serde(rename = "FinishReason")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TencentUsage {
    #[serde(rename = "PromptTokens")]
    pub prompt_tokens: i64,
    #[serde(rename = "CompletionTokens")]
    pub completion_tokens: i64,
    #[serde(rename = "TotalTokens")]
    pub total_tokens: i64,
}
