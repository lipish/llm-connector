//! 统一的traitDefine - V2架构核心
//!
//! 这个模块Define了V2架构的核心trait，Provide清晰、统一的抽象层。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;

// 重用现有类型，保持兼容性
use crate::types::{ChatRequest, ChatResponse};
use crate::error::LlmConnectorError;

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// 协议trait - Define纯API规范
/// 
/// 这个trait代表一个LLM API的协议规范，如OpenAI API、Anthropic APIetc.。
/// 它只关注API的格式Convert，不涉及具体的网络通信。
#[async_trait]
pub trait Protocol: Send + Sync + Clone + 'static {
    /// 协议特定的请求类型
    type Request: Serialize + Send + Sync;
    
    /// 协议特定的响应类型  
    type Response: for<'de> Deserialize<'de> + Send + Sync;
    
    /// 协议名称 (如 "openai", "anthropic")
    fn name(&self) -> &str;
    
    /// Get聊天完成的端点URL
    fn chat_endpoint(&self, base_url: &str) -> String;
    
    /// Get模型列表的端点URL (可选)
    fn models_endpoint(&self, _base_url: &str) -> Option<String> {
        None
    }
    
    /// Build协议特定的请求
    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError>;
    
    /// Parse协议特定的响应
    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError>;
    
    /// Parse模型列表响应
    fn parse_models(&self, _response: &str) -> Result<Vec<String>, LlmConnectorError> {
        Err(LlmConnectorError::UnsupportedOperation(
            format!("{} does not support model listing", self.name())
        ))
    }
    
    /// 映射HTTPErrors到统一Errors类型
    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError;
    
    /// Get认证头 (可选)
    fn auth_headers(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// Parse流式响应 (可选)
    #[cfg(feature = "streaming")]
    async fn parse_stream_response(&self, response: reqwest::Response) -> Result<ChatStream, LlmConnectorError> {
        // 默认Use通用SSE流Parse
        Ok(crate::sse::sse_to_streaming_response(response))
    }
}

/// 服务Provide商trait - Define统一的服务接口
/// 
/// 这个trait代表一个具体的LLM服务Provide商，Provide完整的服务功能。
/// 它是用户直接交互的接口。
#[async_trait]
pub trait Provider: Send + Sync {
    /// Provide商名称 (如 "openai", "aliyun", "ollama")
    fn name(&self) -> &str;
    
    /// 聊天完成
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError>;
    
    /// 流式聊天完成
    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError>;
    
    /// Get available models list
    async fn models(&self) -> Result<Vec<String>, LlmConnectorError>;
    
    /// 类型ConvertSupport (用于特殊功能访问)
    fn as_any(&self) -> &dyn Any;
}

/// 通用Provide商实现
/// 
/// 这个结构体为大多数标准LLM APIProvide通用实现。
/// 它UseProtocol trait来处理API特定的格式Convert，
/// UseHttpClient来处理网络通信。
pub struct GenericProvider<P: Protocol> {
    protocol: P,
    client: super::HttpClient,
}

impl<P: Protocol> GenericProvider<P> {
    /// Create新的通用Provide商
    pub fn new(protocol: P, client: super::HttpClient) -> Self {
        Self { protocol, client }
    }
    
    /// Get协议引用
    pub fn protocol(&self) -> &P {
        &self.protocol
    }
    
    /// Get客户端引用
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
        // Build协议特定的请求
        let protocol_request = self.protocol.build_request(request)?;
        
        // Get端点URL
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
        
        // Parse响应
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
