//! V2统一客户端 - 下一代LLM客户端接口
//!
//! 这个模块提供统一的客户端接口，支持所有LLM服务提供商。

use crate::core::Provider;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse};
use std::sync::Arc;

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// 统一LLM客户端
///
/// 这个客户端提供统一的接口来访问各种LLM服务，
/// 使用V2架构的清晰抽象层。
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // 创建OpenAI客户端
///     let client = LlmClient::openai("sk-...")?;
///
///     // 创建请求
///     let request = ChatRequest {
///         model: "gpt-4".to_string(),
///         messages: vec![Message::text(Role::User, "Hello, how are you?")],
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
        let provider = crate::providers::zhipu(api_key)?;
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
    /// let client = LlmClient::ollama_with_base_url("http://192.168.1.100:11434").unwrap();
    /// ```
    pub fn ollama_with_base_url(base_url: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::ollama_with_base_url(base_url)?;
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
    ///
    /// // LongCat (OpenAI format)
    /// let longcat = LlmClient::openai_compatible(
    ///     "ak_...",
    ///     "https://api.longcat.chat/openai",
    ///     "longcat"
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

    /// 创建LongCat Anthropic格式客户端
    ///
    /// LongCat的Anthropic端点使用Bearer认证而不是标准的x-api-key认证
    ///
    /// # 参数
    /// - `api_key`: LongCat API密钥 (格式: ak_...)
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::longcat_anthropic("ak_...").unwrap();
    /// ```
    pub fn longcat_anthropic(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::longcat_anthropic(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建带有自定义配置的LongCat Anthropic客户端
    pub fn longcat_anthropic_with_config(
        api_key: &str,
        base_url: Option<&str>,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::longcat_anthropic_with_config(
            api_key,
            base_url,
            timeout_secs,
            proxy,
        )?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建火山引擎（Volcengine）客户端
    ///
    /// 火山引擎使用 OpenAI 兼容的 API 格式，但端点路径不同
    ///
    /// # 参数
    /// - `api_key`: 火山引擎 API 密钥 (UUID 格式)
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::volcengine("26f962bd-450e-4876-bc32-a732e6da9cd2").unwrap();
    /// ```
    pub fn volcengine(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::volcengine(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建带有自定义配置的火山引擎客户端
    pub fn volcengine_with_config(
        api_key: &str,
        base_url: Option<&str>,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::volcengine_with_config(
            api_key,
            base_url,
            timeout_secs,
            proxy,
        )?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建腾讯云混元（Tencent Hunyuan）客户端
    ///
    /// 腾讯云混元使用 OpenAI 兼容的 API 格式
    ///
    /// # 参数
    /// - `api_key`: 腾讯云混元 API 密钥 (格式: sk-...)
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::tencent("sk-YMiR2Q7LNWVKVWKivkfPn49geQXT27OZXumFkSS3Ef6FlQ50").unwrap();
    /// ```
    pub fn tencent(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::tencent(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建带有自定义配置的腾讯云混元客户端
    pub fn tencent_with_config(
        api_key: &str,
        base_url: Option<&str>,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::tencent_with_config(
            api_key,
            base_url,
            timeout_secs,
            proxy,
        )?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建 Moonshot（月之暗面）客户端
    ///
    /// Moonshot 使用 OpenAI 兼容的 API 格式
    ///
    /// # 参数
    /// - `api_key`: Moonshot API 密钥 (格式: sk-...)
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::moonshot("sk-...").unwrap();
    /// ```
    pub fn moonshot(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::moonshot(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建带有自定义配置的 Moonshot 客户端
    pub fn moonshot_with_config(
        api_key: &str,
        base_url: Option<&str>,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::moonshot_with_config(
            api_key,
            base_url,
            timeout_secs,
            proxy,
        )?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建 DeepSeek 客户端
    ///
    /// DeepSeek 使用 OpenAI 兼容的 API 格式，支持推理模型
    ///
    /// # 参数
    /// - `api_key`: DeepSeek API 密钥 (格式: sk-...)
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::deepseek("sk-...").unwrap();
    /// ```
    pub fn deepseek(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::deepseek(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建带有自定义配置的 DeepSeek 客户端
    pub fn deepseek_with_config(
        api_key: &str,
        base_url: Option<&str>,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::deepseek_with_config(
            api_key,
            base_url,
            timeout_secs,
            proxy,
        )?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    // ============================================================================
    // 高级构造函数 - 自定义配置
    // ============================================================================

    /// 创建带有自定义配置的OpenAI客户端
    pub fn openai_with_config(
        api_key: &str,
        base_url: Option<&str>,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        let provider =
            crate::providers::openai_with_config(api_key, base_url, timeout_secs, proxy)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建带有自定义配置的Aliyun客户端
    pub fn aliyun_with_config(
        api_key: &str,
        base_url: Option<&str>,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        let provider =
            crate::providers::aliyun_with_config(api_key, base_url, timeout_secs, proxy)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建Aliyun国际版客户端
    pub fn aliyun_international(api_key: &str, region: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::aliyun_international(api_key, region)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建Aliyun专有云客户端
    pub fn aliyun_private(api_key: &str, base_url: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::aliyun_private(api_key, base_url)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建带有自定义超时的Aliyun客户端
    pub fn aliyun_with_timeout(
        api_key: &str,
        timeout_secs: u64,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::aliyun_with_timeout(api_key, timeout_secs)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建带有自定义配置的Anthropic客户端
    pub fn anthropic_with_config(
        api_key: &str,
        base_url: Option<&str>,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        let provider =
            crate::providers::anthropic_with_config(api_key, base_url, timeout_secs, proxy)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建Anthropic Vertex AI客户端
    pub fn anthropic_vertex(
        project_id: &str,
        location: &str,
        access_token: &str,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::anthropic_vertex(project_id, location, access_token)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建Anthropic AWS Bedrock客户端
    pub fn anthropic_bedrock(
        region: &str,
        access_key: &str,
        secret_key: &str,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::anthropic_bedrock(region, access_key, secret_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建带有自定义超时的Anthropic客户端
    pub fn anthropic_with_timeout(
        api_key: &str,
        timeout_secs: u64,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::anthropic_with_timeout(api_key, timeout_secs)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建带有自定义配置的Zhipu客户端
    pub fn zhipu_with_config(
        api_key: &str,
        openai_compatible: bool,
        base_url: Option<&str>,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::zhipu_with_config(
            api_key,
            openai_compatible,
            base_url,
            timeout_secs,
            proxy,
        )?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建带有自定义超时的Zhipu客户端
    pub fn zhipu_with_timeout(api_key: &str, timeout_secs: u64) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::zhipu_with_timeout(api_key, timeout_secs)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建Zhipu企业版客户端
    pub fn zhipu_enterprise(api_key: &str, base_url: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::zhipu_enterprise(api_key, base_url)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 创建带有自定义配置的Ollama客户端
    pub fn ollama_with_config(
        base_url: &str,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::ollama_with_config(base_url, timeout_secs, proxy)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// 获取提供商名称
    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }

    pub fn supported_providers() -> Vec<&'static str> {
        vec![
            "openai",
            "aliyun",
            "anthropic",
            "zhipu",
            "ollama",
            "tencent",
            "volcengine",
            "longcat_anthropic",
            "azure_openai",
            "openai_compatible",
        ]
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
    pub async fn chat_stream(
        &self,
        request: &ChatRequest,
    ) -> Result<ChatStream, LlmConnectorError> {
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

    // ============================================================================
    // 类型安全的Provider转换方法
    // ============================================================================

    /// 尝试将客户端转换为OllamaProvider
    ///
    /// # 返回
    /// 如果底层Provider是OllamaProvider，返回Some引用，否则返回None
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = LlmClient::ollama()?;
    /// if let Some(_ollama) = client.as_ollama() {
    ///     // 可以访问 Ollama 特定的功能
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn as_ollama(&self) -> Option<&crate::providers::OllamaProvider> {
        self.provider
            .as_any()
            .downcast_ref::<crate::providers::OllamaProvider>()
    }

    /// 尝试将客户端转换为OpenAIProvider
    pub fn as_openai(&self) -> Option<&crate::providers::OpenAIProvider> {
        self.provider
            .as_any()
            .downcast_ref::<crate::providers::OpenAIProvider>()
    }

    /// 尝试将客户端转换为AliyunProvider
    pub fn as_aliyun(&self) -> Option<&crate::providers::AliyunProvider> {
        self.provider
            .as_any()
            .downcast_ref::<crate::providers::AliyunProvider>()
    }

    /// 尝试将客户端转换为AnthropicProvider
    pub fn as_anthropic(&self) -> Option<&crate::providers::AnthropicProvider> {
        self.provider
            .as_any()
            .downcast_ref::<crate::providers::AnthropicProvider>()
    }

    /// 尝试将客户端转换为ZhipuProvider
    pub fn as_zhipu(&self) -> Option<&crate::providers::ZhipuProvider> {
        self.provider
            .as_any()
            .downcast_ref::<crate::providers::ZhipuProvider>()
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
