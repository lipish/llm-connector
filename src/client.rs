//! V2 Unified Client - Next-generation LLM client interface
//!
//! This module provides a unified client interface supporting all LLM service providers.

use crate::core::Provider;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse};
use std::sync::Arc;

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// Unified LLM Client
///
/// This client provides a unified interface to access various LLM services,
/// using V2 architecture's clean abstraction layer.
///
/// # Example
/// ```rust,no_run
/// use llm_connector::{LlmClient, types::{ChatRequest, Message, Role}};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create OpenAI client
///     let client = LlmClient::openai("sk-...")?;
///
///     // Create request
///     let request = ChatRequest {
///         model: "gpt-4".to_string(),
///         messages: vec![Message::text(Role::User, "Hello, how are you?")],
///         ..Default::default()
///     };
///
///     // Send request
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
    /// Create client from any Provider
    pub fn from_provider(provider: Arc<dyn Provider>) -> Self {
        Self { provider }
    }

    /// Create OpenAI client
    ///
    /// # Parameters
    /// - `api_key`: OpenAI API key
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::openai("sk-...").unwrap();
    /// ```
    pub fn openai(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::openai(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create OpenAI client with custom base URL
    ///
    /// # Parameters
    /// - `api_key`: API key
    /// - `base_url`: Custom base URL
    ///
    /// # Example
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

    /// Create Azure OpenAI client
    ///
    /// # Parameters
    /// - `api_key`: Azure OpenAI API key
    /// - `endpoint`: Azure OpenAI endpoint
    /// - `api_version`: API version
    ///
    /// # Example
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

    /// Create Aliyun DashScope client
    ///
    /// # Parameters
    /// - `api_key`: Aliyun DashScope API key
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::aliyun("sk-...").unwrap();
    /// ```
    pub fn aliyun(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::aliyun(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Anthropic Claude client
    ///
    /// # Parameters
    /// - `api_key`: Anthropic API key (格式: sk-ant-...)
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::anthropic("sk-ant-...").unwrap();
    /// ```
    pub fn anthropic(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::anthropic(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Zhipu GLM client
    ///
    /// # Parameters
    /// - `api_key`: Zhipu GLM API key
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::zhipu("your-api-key").unwrap();
    /// ```
    pub fn zhipu(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::zhipu(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Zhipu GLM client (OpenAI兼容模式)
    ///
    /// # Parameters
    /// - `api_key`: Zhipu GLM API key
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::zhipu_openai_compatible("your-api-key").unwrap();
    /// ```
    pub fn zhipu_openai_compatible(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::zhipu_openai_compatible(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Ollama client (default local address)
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::ollama().unwrap();
    /// ```
    pub fn ollama() -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::ollama()?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Ollama client with custom URL
    ///
    /// # Parameters
    /// - `base_url`: Ollama service URL
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::ollama_with_base_url("http://192.168.1.100:11434").unwrap();
    /// ```
    pub fn ollama_with_base_url(base_url: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::ollama_with_base_url(base_url)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create OpenAI-compatible service client
    ///
    /// # Parameters
    /// - `api_key`: API key
    /// - `base_url`: Service base URL
    /// - `service_name`: Service name
    ///
    /// # Example
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

    /// Create LongCat Anthropic format client
    ///
    /// LongCat's Anthropic endpoint uses Bearer authentication instead of standard x-api-key authentication
    ///
    /// # Parameters
    /// - `api_key`: LongCat API key (格式: ak_...)
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::longcat_anthropic("ak_...").unwrap();
    /// ```
    pub fn longcat_anthropic(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::longcat_anthropic(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create LongCat Anthropic client with custom configuration
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

    /// Create Volcengine client
    ///
    /// VolcengineUse OpenAI 兼容 API 格式，但endpoint路径不同
    ///
    /// # Parameters
    /// - `api_key`: Volcengine API 密钥 (UUID 格式)
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::volcengine("your-volcengine-api-key").unwrap();
    /// ```
    pub fn volcengine(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::volcengine(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Volcengine client with custom configuration
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

    /// Create Tencent Hunyuan client
    ///
    /// Tencent HunyuanUse OpenAI 兼容 API 格式
    ///
    /// # Parameters
    /// - `api_key`: Tencent Hunyuan API 密钥 (格式: sk-...)
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::tencent("sk-YMiR2Q7LNWVKVWKivkfPn49geQXT27OZXumFkSS3Ef6FlQ50").unwrap();
    /// ```
    pub fn tencent(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::tencent(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create带有customconfigurationTencent Hunyuanclient
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

    /// Create Moonshot（Moonshot）client
    ///
    /// Moonshot uses OpenAI-compatible API format
    ///
    /// # Parameters
    /// - `api_key`: Moonshot API 密钥 (格式: sk-...)
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::moonshot("sk-...").unwrap();
    /// ```
    pub fn moonshot(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::moonshot(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Moonshot client with custom configuration
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

    /// Create DeepSeek client
    ///
    /// DeepSeek uses OpenAI-compatible API format, supports reasoning models
    ///
    /// # Parameters
    /// - `api_key`: DeepSeek API 密钥 (格式: sk-...)
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::deepseek("sk-...").unwrap();
    /// ```
    pub fn deepseek(api_key: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::deepseek(api_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create DeepSeek client with custom configuration
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
    // Advanced Constructors - Custom Configuration
    // ============================================================================

    /// Create OpenAI client with custom configuration
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

    /// Create Aliyun client with custom configuration
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

    /// Create Aliyun international client
    pub fn aliyun_international(api_key: &str, region: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::aliyun_international(api_key, region)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Aliyun private cloud client
    pub fn aliyun_private(api_key: &str, base_url: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::aliyun_private(api_key, base_url)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Aliyun client with custom timeout
    pub fn aliyun_with_timeout(
        api_key: &str,
        timeout_secs: u64,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::aliyun_with_timeout(api_key, timeout_secs)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Anthropic client with custom configuration
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

    /// Create Anthropic Vertex AI client
    pub fn anthropic_vertex(
        project_id: &str,
        location: &str,
        access_token: &str,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::anthropic_vertex(project_id, location, access_token)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Anthropic AWS Bedrock client
    pub fn anthropic_bedrock(
        region: &str,
        access_key: &str,
        secret_key: &str,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::anthropic_bedrock(region, access_key, secret_key)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Anthropic client with custom timeout
    pub fn anthropic_with_timeout(
        api_key: &str,
        timeout_secs: u64,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::anthropic_with_timeout(api_key, timeout_secs)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Zhipu client with custom configuration
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

    /// Create Zhipu client with custom timeout
    pub fn zhipu_with_timeout(api_key: &str, timeout_secs: u64) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::zhipu_with_timeout(api_key, timeout_secs)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Zhipu enterprise client
    pub fn zhipu_enterprise(api_key: &str, base_url: &str) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::zhipu_enterprise(api_key, base_url)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Create Ollama client with custom configuration
    pub fn ollama_with_config(
        base_url: &str,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        let provider = crate::providers::ollama_with_config(base_url, timeout_secs, proxy)?;
        Ok(Self::from_provider(Arc::new(provider)))
    }

    /// Get provider name
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

    /// Send chat completion request
    ///
    /// # Parameters
    /// - `request`: Chat request
    ///
    /// # Returns
    /// Chat response
    ///
    /// # Example
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

    /// Send streaming chat completion request
    ///
    /// # Parameters
    /// - `request`: Chat request
    ///
    /// # Returns
    /// Chat stream
    ///
    /// # Example
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

    /// Get available models list
    ///
    /// # Returns
    /// List of model names
    ///
    /// # Example
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

    /// Get reference to underlying provider (for special feature access)
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::openai("sk-...").unwrap();
    /// let provider = client.provider();
    ///
    /// // Can perform type conversion to access provider-specific features
    /// ```
    pub fn provider(&self) -> &dyn Provider {
        self.provider.as_ref()
    }

    // ============================================================================
    // Type-safe Provider conversion methods
    // ============================================================================

    /// Try to convert client to OllamaProvider
    ///
    /// # Returns
    /// if底层ProviderisOllamaProvider，ReturnsSome引用，if则ReturnsNone
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = LlmClient::ollama()?;
    /// if let Some(_ollama) = client.as_ollama() {
    ///     // 可以访问 Ollama 特定功能
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn as_ollama(&self) -> Option<&crate::providers::OllamaProvider> {
        self.provider
            .as_any()
            .downcast_ref::<crate::providers::OllamaProvider>()
    }

    /// 尝试将clientConvertasOpenAIProvider
    pub fn as_openai(&self) -> Option<&crate::providers::OpenAIProvider> {
        self.provider
            .as_any()
            .downcast_ref::<crate::providers::OpenAIProvider>()
    }

    /// 尝试将clientConvertasAliyunProvider
    pub fn as_aliyun(&self) -> Option<&crate::providers::AliyunProvider> {
        self.provider
            .as_any()
            .downcast_ref::<crate::providers::AliyunProvider>()
    }

    /// 尝试将clientConvertasAnthropicProvider
    pub fn as_anthropic(&self) -> Option<&crate::providers::AnthropicProvider> {
        self.provider
            .as_any()
            .downcast_ref::<crate::providers::AnthropicProvider>()
    }

    /// 尝试将clientConvertasZhipuProvider
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
