//! 统一的trait定义 - V2架构核心
//!
//! 这个模块定义了V2架构的核心trait，提供清晰、统一的抽象层。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;

// 重用现有类型，保持兼容性
use crate::types::{ChatRequest, ChatResponse};
use crate::error::LlmConnectorError;

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// 协议trait - 定义纯API规范
/// 
/// 这个trait代表一个LLM API的协议规范，如OpenAI API、Anthropic API等。
/// 它只关注API的格式转换，不涉及具体的网络通信。
#[async_trait]
pub trait Protocol: Send + Sync + Clone + 'static {
    /// 协议特定的请求类型
    type Request: Serialize + Send + Sync;
    
    /// 协议特定的响应类型  
    type Response: for<'de> Deserialize<'de> + Send + Sync;
    
    /// 协议名称 (如 "openai", "anthropic")
    fn name(&self) -> &str;
    
    /// 获取聊天完成的端点URL
    fn chat_endpoint(&self, base_url: &str) -> String;
    
    /// 获取模型列表的端点URL (可选)
    fn models_endpoint(&self, _base_url: &str) -> Option<String> {
        None
    }
    
    /// 构建协议特定的请求
    fn build_request(&self, request: &ChatRequest) -> Result<Self::Request, LlmConnectorError>;
    
    /// 解析协议特定的响应
    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError>;
    
    /// 解析模型列表响应
    fn parse_models(&self, _response: &str) -> Result<Vec<String>, LlmConnectorError> {
        Err(LlmConnectorError::UnsupportedOperation(
            format!("{} does not support model listing", self.name())
        ))
    }
    
    /// 映射HTTP错误到统一错误类型
    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError;
    
    /// 获取认证头 (可选)
    fn auth_headers(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// 解析流式响应 (可选)
    #[cfg(feature = "streaming")]
    async fn parse_stream_response(&self, response: reqwest::Response) -> Result<ChatStream, LlmConnectorError> {
        // 默认使用通用SSE流解析
        Ok(crate::sse::sse_to_streaming_response(response))
    }
}

/// 服务提供商trait - 定义统一的服务接口
/// 
/// 这个trait代表一个具体的LLM服务提供商，提供完整的服务功能。
/// 它是用户直接交互的接口。
#[async_trait]
pub trait Provider: Send + Sync {
    /// 提供商名称 (如 "openai", "aliyun", "ollama")
    fn name(&self) -> &str;
    
    /// 聊天完成
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError>;
    
    /// 流式聊天完成
    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError>;
    
    /// 获取可用模型列表
    async fn models(&self) -> Result<Vec<String>, LlmConnectorError>;
    
    /// 类型转换支持 (用于特殊功能访问)
    fn as_any(&self) -> &dyn Any;
}

/// 通用提供商实现
/// 
/// 这个结构体为大多数标准LLM API提供通用实现。
/// 它使用Protocol trait来处理API特定的格式转换，
/// 使用HttpClient来处理网络通信。
pub struct GenericProvider<P: Protocol> {
    protocol: P,
    client: super::HttpClient,
}

impl<P: Protocol> GenericProvider<P> {
    /// 创建新的通用提供商
    pub fn new(protocol: P, client: super::HttpClient) -> Self {
        Self { protocol, client }
    }
    
    /// 获取协议引用
    pub fn protocol(&self) -> &P {
        &self.protocol
    }
    
    /// 获取客户端引用
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
        // 构建协议特定的请求
        let protocol_request = self.protocol.build_request(request)?;
        
        // 获取端点URL
        let url = self.protocol.chat_endpoint(self.client.base_url());
        
        // 发送请求
        let response = self.client.post(&url, &protocol_request).await?;
        let status = response.status();
        let text = response.text().await
            .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;
            
        // 检查HTTP状态
        if !status.is_success() {
            return Err(self.protocol.map_error(status.as_u16(), &text));
        }
        
        // 解析响应
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
