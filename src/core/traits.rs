//! Unified Trait Definitions - V2 Architecture Core
//!
//! This module defines core traits for V2 architecture, providing clear and unified abstraction layer。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;

// 重用现有类型，保持兼容性
use crate::types::{ChatRequest, ChatResponse};
use crate::error::LlmConnectorError;

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// Protocol trait - Defines pure API specification
/// 
/// This trait represents an LLM API protocol specification, such as OpenAI API, Anthropic API, etc.
/// 它只关注API格式Convert，不涉及具体网络通信。
#[async_trait]
pub trait Protocol: Send + Sync + Clone + 'static {
    /// Protocol-specific request type
    type Request: Serialize + Send + Sync;
    
    /// Protocol-specific response type  
    type Response: for<'de> Deserialize<'de> + Send + Sync;
    
    /// Protocol name (such as "openai", "anthropic")
    fn name(&self) -> &str;
    
    /// Get聊天完成endpointURL
    fn chat_endpoint(&self, base_url: &str) -> String;
    
    /// Get model list endpoint URL (optional)
    fn models_endpoint(&self, _base_url: &str) -> Option<String> {
        None
    }
    
    /// Build protocol-specific request
    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError>;
    
    /// Parse protocol-specific response
    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError>;
    
    /// Parse model list response
    fn parse_models(&self, _response: &str) -> Result<Vec<String>, LlmConnectorError> {
        Err(LlmConnectorError::UnsupportedOperation(
            format!("{} does not support model listing", self.name())
        ))
    }
    
    /// 映射HTTPErrorsto统一Errors类型
    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError;
    
    /// Getauthentication头 (optional)
    fn auth_headers(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// Parse streaming response (optional)
    #[cfg(feature = "streaming")]
    async fn parse_stream_response(&self, response: reqwest::Response) -> Result<ChatStream, LlmConnectorError> {
        // 默认Use通用SSE流Parse
        Ok(crate::sse::sse_to_streaming_response(response))
    }
}

/// 服务Provide商trait - Define统一服务接口
/// 
/// thistrait代表a具体LLM服务Provide商，Provide完整服务功能。
/// 它is用户直接交互接口。
#[async_trait]
pub trait Provider: Send + Sync {
    /// Provide商name (such as "openai", "aliyun", "ollama")
    fn name(&self) -> &str;
    
    /// 聊天完成
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError>;
    
    /// Streaming chat completion
    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError>;
    
    /// Get available models list
    async fn models(&self) -> Result<Vec<String>, LlmConnectorError>;
    
    /// 类型ConvertSupport (for特殊功能访问)
    fn as_any(&self) -> &dyn Any;
}

/// Generic provider implementation
/// 
/// this结构体as大多数标准LLM APIProvide通用实现。
/// 它UseProtocol traitto处理API特定格式Convert，
/// UseHttpClientto处理网络通信。
pub struct GenericProvider<P: Protocol> {
    protocol: P,
    client: super::HttpClient,
}

impl<P: Protocol> GenericProvider<P> {
    /// Create新通用Provide商
    pub fn new(protocol: P, client: super::HttpClient) -> Self {
        Self { protocol, client }
    }
    
    /// Get protocol reference
    pub fn protocol(&self) -> &P {
        &self.protocol
    }
    
    /// Get client reference
    pub fn client(&self) -> &super::HttpClient {
        &self.client
    }
}

impl<P: Protocol> Clone for GenericProvider<P> {
    fn clone(&self) -> Self {
        Self {
            protocol: self.protocol.clone(),
            client: self.client.clone(),
        }
    }
}

#[async_trait]
impl<P: Protocol> Provider for GenericProvider<P> {
    fn name(&self) -> &str {
        self.protocol.name()
    }
    
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        // Build protocol-specific request
        let protocol_request = self.protocol.build_request(request)?;
        
        // GetendpointURL
        let url = self.protocol.chat_endpoint(self.client.base_url());
        
        // Send request
        let response = self.client.post(&url, &protocol_request).await?;
        let status = response.status();
        let text = response.text().await
            .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;
            
        // CheckHTTP状态
        if !status.is_success() {
            return Err(self.protocol.map_error(status.as_u16(), &text));
        }
        
        // Parse response
        self.protocol.parse_response(&text)
    }
    
    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        let mut streaming_request = request.clone();
        streaming_request.stream = Some(true);

        let protocol_request = self.protocol.build_request(&streaming_request)?;
        let url = self.protocol.chat_endpoint(self.client.base_url());

        let response = self.client.stream(&url, &protocol_request).await?;
        let status = response.status();

        if !status.is_success() {
            let text = response.text().await
                .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;
            return Err(self.protocol.map_error(status.as_u16(), &text));
        }

        self.protocol.parse_stream_response(response).await
    }
    
    async fn models(&self) -> Result<Vec<String>, LlmConnectorError> {
        let endpoint = self.protocol.models_endpoint(self.client.base_url())
            .ok_or_else(|| LlmConnectorError::UnsupportedOperation(
                format!("{} does not support model listing", self.protocol.name())
            ))?;
            
        let response = self.client.get(&endpoint).await?;
        let status = response.status();
        let text = response.text().await
            .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;
            
        if !status.is_success() {
            return Err(self.protocol.map_error(status.as_u16(), &text));
        }
        
        self.protocol.parse_models(&text)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}
