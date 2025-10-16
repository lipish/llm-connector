//! HTTP客户端实现 - V2架构
//!
//! 提供统一的HTTP通信层，支持标准请求和流式请求。

use crate::error::LlmConnectorError;
use reqwest::Client;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

/// HTTP客户端
/// 
/// 封装了HTTP通信的所有细节，包括认证、超时、代理等配置。
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    base_url: String,
    headers: HashMap<String, String>,
}

impl HttpClient {
    /// 创建新的HTTP客户端
    pub fn new(base_url: &str) -> Result<Self, LlmConnectorError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| LlmConnectorError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;
            
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            headers: HashMap::new(),
        })
    }
    
    /// 创建带有自定义配置的HTTP客户端
    pub fn with_config(
        base_url: &str,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        let mut builder = Client::builder();
        
        // 设置超时
        if let Some(timeout) = timeout_secs {
            builder = builder.timeout(Duration::from_secs(timeout));
        } else {
            builder = builder.timeout(Duration::from_secs(30));
        }
        
        // 设置代理
        if let Some(proxy_url) = proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| LlmConnectorError::ConfigError(format!("Invalid proxy URL: {}", e)))?;
            builder = builder.proxy(proxy);
        }
        
        let client = builder.build()
            .map_err(|e| LlmConnectorError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;
            
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            headers: HashMap::new(),
        })
    }
    
    /// 添加请求头
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers.extend(headers);
        self
    }
    
    /// 添加单个请求头
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
    
    /// 获取基础URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    
    /// 发送GET请求
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, LlmConnectorError> {
        let mut request = self.client.get(url);
        
        // 添加所有配置的请求头
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }
        
        request.send().await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmConnectorError::TimeoutError(format!("GET request timeout: {}", e))
                } else if e.is_connect() {
                    LlmConnectorError::ConnectionError(format!("GET connection failed: {}", e))
                } else {
                    LlmConnectorError::NetworkError(format!("GET request failed: {}", e))
                }
            })
    }
    
    /// 发送POST请求
    pub async fn post<T: Serialize>(
        &self, 
        url: &str, 
        body: &T
    ) -> Result<reqwest::Response, LlmConnectorError> {
        let mut request = self.client.post(url).json(body);
        
        // 添加所有配置的请求头
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }
        
        request.send().await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmConnectorError::TimeoutError(format!("POST request timeout: {}", e))
                } else if e.is_connect() {
                    LlmConnectorError::ConnectionError(format!("POST connection failed: {}", e))
                } else {
                    LlmConnectorError::NetworkError(format!("POST request failed: {}", e))
                }
            })
    }
    
    /// 发送流式POST请求
    #[cfg(feature = "streaming")]
    pub async fn stream<T: Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<reqwest::Response, LlmConnectorError> {
        let mut request = self.client.post(url).json(body);
        
        // 添加所有配置的请求头
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }
        
        request.send().await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmConnectorError::TimeoutError(format!("Stream request timeout: {}", e))
                } else if e.is_connect() {
                    LlmConnectorError::ConnectionError(format!("Stream connection failed: {}", e))
                } else {
                    LlmConnectorError::NetworkError(format!("Stream request failed: {}", e))
                }
            })
    }
    
    /// 发送带有自定义头的POST请求
    pub async fn post_with_custom_headers<T: Serialize>(
        &self,
        url: &str,
        body: &T,
        custom_headers: &HashMap<String, String>,
    ) -> Result<reqwest::Response, LlmConnectorError> {
        let mut request = self.client.post(url).json(body);
        
        // 先添加自定义头
        for (key, value) in custom_headers {
            request = request.header(key, value);
        }
        
        // 再添加配置的请求头 (可能会覆盖自定义头)
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }
        
        request.send().await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmConnectorError::TimeoutError(format!("POST request timeout: {}", e))
                } else if e.is_connect() {
                    LlmConnectorError::ConnectionError(format!("POST connection failed: {}", e))
                } else {
                    LlmConnectorError::NetworkError(format!("POST request failed: {}", e))
                }
            })
    }
}

impl std::fmt::Debug for HttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClient")
            .field("base_url", &self.base_url)
            .field("headers_count", &self.headers.len())
            .finish()
    }
}
