//! V2统一客户端 - 下一代LLM客户端接口
//!
//! 这个模块提供统一的客户端接口，支持所有LLM服务提供商。

use crate::core::Provider;
use crate::types::{ChatRequest, ChatResponse};
use crate::error::LlmConnectorError;
use std::sync::Arc;

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// V2统一LLM客户端
/// 
/// 这个客户端提供统一的接口来访问各种LLM服务，
/// 使用V2架构的清晰抽象层。
/// 
/// # 示例
/// ```rust,no_run
/// use llm_connector::{LlmClient, provider};
/// use llm_connector::types::{ChatRequest, Message, Role};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // 创建OpenAI客户端
///     let client = LlmClient::openai("sk-...")?;
///     
///     // 创建请求
///     let request = ChatRequest {
///         model: "gpt-4".to_string(),
///         messages: vec![
///             Message::user("Hello, how are you?")
///         ],
///         ..Default::default()
///     };
///     
///     // 发送请求
///     let response = client.chat(&request).await?;
///     println!("Response: {}", response.content);
///     
///     Ok(())
/// }
/// ```
pub struct LlmClient {
    provider: Arc<dyn Provider>,
}

impl LlmClient {
    /// 从任何Provider创建客户端
    pub fn from_provider(provider: Arc<dyn Provider>) -> Self {
        Self { provider }
    }
    
    /// 创建OpenAI客户端
    /// 
    /// # 参数
    /// - `api_key`: OpenAI API密钥
    /// 
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    /// 
    /// let client = LlmClient::openai("sk-...").unwrap();
    /// ```
    pub fn openai(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::openai(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }
    
    /// 创建带有自定义基础URL的OpenAI客户端
    /// 
    /// # 参数
    /// - `api_key`: API密钥
    /// - `base_url`: 自定义基础URL
    /// 
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    /// 
    /// let client = LlmClient::openai_with_base_url(
    ///     "sk-...", 
    ///     "https://api.deepseek.com"
    /// ).unwrap();
    /// ```
    pub fn openai_with_base_url(api_key: &str, base_url: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::openai_with_base_url(api_key, base_url)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }
    
    /// 创建Azure OpenAI客户端
    /// 
    /// # 参数
    /// - `api_key`: Azure OpenAI API密钥
    /// - `endpoint`: Azure OpenAI端点
    /// - `api_version`: API版本
    /// 
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    /// 
    /// let client = LlmClient::azure_openai(
    ///     "your-api-key",
    ///     "https://your-resource.openai.azure.com",
    ///     "2024-02-15-preview"
    /// ).unwrap();
    /// ```
    pub fn azure_openai(
        api_key: &str,
        endpoint: &str,
        api_version: &str,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::azure_openai(api_key, endpoint, api_version)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }
    
    /// 创建阿里云DashScope客户端
    ///
    /// # 参数
    /// - `api_key`: 阿里云DashScope API密钥
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::aliyun("sk-...").unwrap();
    /// ```
    pub fn aliyun(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::aliyun(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建Anthropic Claude客户端
    ///
    /// # 参数
    /// - `api_key`: Anthropic API密钥 (格式: sk-ant-...)
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::anthropic("sk-ant-...").unwrap();
    /// ```
    pub fn anthropic(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::anthropic(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建智谱GLM客户端
    ///
    /// # 参数
    /// - `api_key`: 智谱GLM API密钥
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::zhipu("your-api-key").unwrap();
    /// ```
    pub fn zhipu(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::zhipu_default(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建智谱GLM客户端 (OpenAI兼容模式)
    ///
    /// # 参数
    /// - `api_key`: 智谱GLM API密钥
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::zhipu_openai_compatible("your-api-key").unwrap();
    /// ```
    pub fn zhipu_openai_compatible(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::zhipu_openai_compatible(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建Ollama客户端 (默认本地地址)
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::ollama().unwrap();
    /// ```
    pub fn ollama() -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::ollama()?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建带有自定义URL的Ollama客户端
    ///
    /// # 参数
    /// - `base_url`: Ollama服务的URL
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::ollama_with_url("http://192.168.1.100:11434").unwrap();
    /// ```
    pub fn ollama_with_url(base_url: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::ollama_with_url(base_url)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }
    
    /// 创建OpenAI兼容服务客户端
    /// 
    /// # 参数
    /// - `api_key`: API密钥
    /// - `base_url`: 服务基础URL
    /// - `service_name`: 服务名称
    /// 
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    /// 
    /// // DeepSeek
    /// let deepseek = LlmClient::openai_compatible(
    ///     "sk-...",
    ///     "https://api.deepseek.com",
    ///     "deepseek"
    /// ).unwrap();
    /// 
    /// // Moonshot
    /// let moonshot = LlmClient::openai_compatible(
    ///     "sk-...",
    ///     "https://api.moonshot.cn",
    ///     "moonshot"
    /// ).unwrap();
    /// ```
    pub fn openai_compatible(
        api_key: &str,
        base_url: &str,
        service_name: &str,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::openai_compatible(api_key, base_url, service_name)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }
    
    /// 获取提供商名称
    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }
    
    /// 发送聊天完成请求
    /// 
    /// # 参数
    /// - `request`: 聊天请求
    /// 
    /// # 返回
    /// 聊天响应
    /// 
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    /// use llm_connector::types::{ChatRequest, Message};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = LlmClient::openai("sk-...")?;
    ///     
    ///     let request = ChatRequest {
    ///         model: "gpt-4".to_string(),
    ///         messages: vec![Message::user("Hello!")],
    ///         ..Default::default()
    ///     };
    ///     
    ///     let response = client.chat(&request).await?;
    ///     println!("Response: {}", response.content);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        self.provider.chat(request).await
    }
    
    /// 发送流式聊天完成请求
    /// 
    /// # 参数
    /// - `request`: 聊天请求
    /// 
    /// # 返回
    /// 聊天流
    /// 
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    /// use llm_connector::types::{ChatRequest, Message};
    /// use futures_util::StreamExt;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = LlmClient::openai("sk-...")?;
    ///     
    ///     let request = ChatRequest {
    ///         model: "gpt-4".to_string(),
    ///         messages: vec![Message::user("Hello!")],
    ///         stream: Some(true),
    ///         ..Default::default()
    ///     };
    ///     
    ///     let mut stream = client.chat_stream(&request).await?;
    ///     while let Some(chunk) = stream.next().await {
    ///         let chunk = chunk?;
    ///         if let Some(content) = chunk.get_content() {
    ///             print!("{}", content);
    ///         }
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    #[cfg(feature = "streaming")]
    pub async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        self.provider.chat_stream(request).await
    }
    
    /// 获取可用模型列表
    /// 
    /// # 返回
    /// 模型名称列表
    /// 
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = LlmClient::openai("sk-...")?;
    ///     
    ///     let models = client.models().await?;
    ///     for model in models {
    ///         println!("Available model: {}", model);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn models(&self) -> Result<Vec<String>, LlmConnectorError> {
        self.provider.models().await
    }
    
    /// 获取底层提供商的引用 (用于特殊功能访问)
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::openai("sk-...").unwrap();
    /// let provider = client.provider();
    ///
    /// // 可以进行类型转换以访问特定提供商的功能
    /// ```
    pub fn provider(&self) -> &dyn Provider {
        self.provider.as_ref()
    }

    /// 获取Ollama特殊功能访问
    ///
    /// 如果当前客户端是Ollama提供商，返回Ollama特殊功能的访问接口。
    ///
    /// # 返回
    /// 如果是Ollama提供商，返回Some(OllamaProvider)，否则返回None
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::ollama().unwrap();
    /// if let Some(ollama) = client.as_ollama() {
    ///     // 使用Ollama特殊功能
    ///     let models = ollama.models().await.unwrap();
    ///     ollama.pull_model("llama2").await.unwrap();
    /// }
    /// ```
    pub fn as_ollama(&self) -> Option<&crate::providers::OllamaProvider> {
        self.provider.as_any().downcast_ref::<crate::providers::OllamaProvider>()
    }
}

impl Clone for LlmClient {
    fn clone(&self) -> Self {
        Self {
            provider: Arc::clone(&self.provider),
        }
    }
}

impl std::fmt::Debug for LlmClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmClient")
            .field("provider", &self.provider.name())
            .finish()
    }
}
