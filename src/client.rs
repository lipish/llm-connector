//! Minimal LLM Client
//!
//! This client provides a simple interface to use any of the 4 supported protocols.
//! No complex configuration, just pick a protocol and start chatting.

use std::sync::Arc;

use crate::error::LlmConnectorError;
use crate::protocols::{OpenAIProtocol, AnthropicProtocol, zhipu_with_timeout};
use crate::protocols::core::GenericProvider;
use crate::config::ProviderConfig;
use crate::types::{ChatRequest, ChatResponse};

/// Minimal LLM Client
///
/// Supports 6 protocols: OpenAI, Anthropic, Aliyun, Zhipu, Ollama, Hunyuan
/// No complex configuration needed - just pick a protocol and start.
#[derive(Clone)]
pub struct LlmClient {
    provider: Arc<dyn crate::protocols::Provider + Send + Sync>,
}

impl LlmClient {
    /// Internal constructor from a provider instance
    #[allow(dead_code)]
    pub(crate) fn from_provider(provider: Arc<dyn crate::protocols::Provider + Send + Sync>) -> Self {
        Self { provider }
    }

    /// Internal accessor to the underlying provider trait object
    pub(crate) fn provider_dyn(&self) -> &dyn crate::protocols::Provider {
        &*self.provider
    }

    /// Create new client with OpenAI protocol (supports custom base URL)
    ///
    /// Default base URL: `https://api.openai.com/v1`. Pass `Some(url)` to use
    /// any OpenAI-compatible endpoint.
    ///
    /// ```rust,ignore
    /// use llm_connector::LlmClient;
    /// // Standard OpenAI
    /// let client = LlmClient::openai("sk-...", None);
    /// // OpenAI-compatible
    /// let client = LlmClient::openai("sk-...", Some("https://api.example.com/v1"));
    /// ```
    pub fn openai(api_key: &str, base_url: Option<&str>) -> Self {
        let base = base_url.unwrap_or("https://api.openai.com/v1");
        let protocol = OpenAIProtocol::with_url(api_key, base);
        let config = ProviderConfig::new(api_key)
            .with_base_url(base)
            .with_timeout_ms(30000); // 默认30秒超时
        let provider = GenericProvider::new(config, protocol)
            .expect("Failed to create OpenAI provider");
        Self { provider: Arc::new(provider) }
    }

    /// Create new client with OpenAI protocol and custom timeout
    ///
    /// ```rust,ignore
    /// use llm_connector::LlmClient;
    /// // With custom timeout (60 seconds)
    /// let client = LlmClient::openai_with_timeout("sk-...", None, 60000);
    /// ```
    pub fn openai_with_timeout(api_key: &str, base_url: Option<&str>, timeout_ms: u64) -> Self {
        let base = base_url.unwrap_or("https://api.openai.com/v1");
        let protocol = OpenAIProtocol::with_url(api_key, base);
        let config = ProviderConfig::new(api_key)
            .with_base_url(base)
            .with_timeout_ms(timeout_ms);
        let provider = GenericProvider::new(config, protocol)
            .expect("Failed to create OpenAI provider");
        Self { provider: Arc::new(provider) }
    }

    /// Create new client with Aliyun protocol
    ///
    /// ```rust,ignore
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::aliyun("sk-...");
    /// ```
    pub fn aliyun(api_key: &str) -> Self {
        let protocol = crate::protocols::aliyun::AliyunProtocol::new(api_key);
        let config = ProviderConfig::new(api_key)
            .with_base_url("https://dashscope.aliyuncs.com/api/v1")
            .with_timeout_ms(30000);
        let provider = GenericProvider::new(config, protocol)
            .expect("Failed to create Aliyun provider");
        Self { provider: Arc::new(provider) }
    }

    /// Create new client with Ollama protocol
    ///
    /// Default base URL: `http://localhost:11434`. Pass `Some(url)` to override.
    pub fn ollama(base_url: Option<&str>) -> Self {
        let provider = if let Some(base) = base_url {
            crate::protocols::ollama::ollama_with_url(base)
        } else {
            crate::protocols::ollama::ollama()
        };
        Self { provider: Arc::new(provider) }
    }

    /// Create new client for LongCat provider (OpenAI or Anthropic compatible)
    ///
    /// Default is OpenAI-compatible. Pass `anthropic = true` to use Anthropic-compatible.
    pub fn longcat(api_key: &str, anthropic: bool) -> Self {
        let base_url = if anthropic {
            "https://api.longcat.chat/anthropic"
        } else {
            "https://api.longcat.chat/openai/v1"
        };

        let config = ProviderConfig::new(api_key).with_base_url(base_url);

        if anthropic {
            let protocol = AnthropicProtocol::with_url(api_key, base_url);
            let provider = GenericProvider::new(config, protocol)
                .expect("Failed to create LongCat Anthropic-compatible provider");
            Self { provider: Arc::new(provider) }
        } else {
            let protocol = OpenAIProtocol::with_url(api_key, base_url);
            let provider = GenericProvider::new(config, protocol)
                .expect("Failed to create LongCat OpenAI-compatible provider");
            Self { provider: Arc::new(provider) }
        }
    }


    /// Create new client with Anthropic protocol
    ///
    /// ```rust,ignore
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::anthropic("sk-ant-...");
    /// ```
    pub fn anthropic(api_key: &str) -> Self {
        let protocol = AnthropicProtocol::new(api_key);
        let config = ProviderConfig::new(api_key)
            .with_base_url("https://api.anthropic.com")
            .with_timeout_ms(30000); // 默认30秒超时
        let provider = GenericProvider::new(config, protocol)
            .expect("Failed to create Anthropic provider");
        Self {
            provider: Arc::new(provider),
        }
    }

    /// Create new client with Anthropic protocol and custom timeout
    ///
    /// ```rust,ignore
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::anthropic_with_timeout("sk-ant-...", 60000);
    /// ```
    pub fn anthropic_with_timeout(api_key: &str, timeout_ms: u64) -> Self {
        let protocol = AnthropicProtocol::new(api_key);
        let config = ProviderConfig::new(api_key)
            .with_base_url("https://api.anthropic.com")
            .with_timeout_ms(timeout_ms);
        let provider = GenericProvider::new(config, protocol)
            .expect("Failed to create Anthropic provider");
        Self {
            provider: Arc::new(provider),
        }
    }

    /// Create new client with Zhipu protocol
    ///
    /// Uses the default PaaS v4 endpoint: `https://open.bigmodel.cn/api/paas/v4`
    ///
    /// ```rust,ignore
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::zhipu("sk-...");
    /// ```
    pub fn zhipu(api_key: &str) -> Self {
        let provider = zhipu_with_timeout(api_key, 30000) // 默认30秒超时
            .expect("Failed to create Zhipu provider");
        Self { provider: Arc::new(provider) }
    }

    /// Create new client with Zhipu protocol and custom timeout
    ///
    /// ```rust,ignore
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::zhipu_with_timeout("sk-...", 60000);
    /// ```
    pub fn zhipu_with_timeout(api_key: &str, timeout_ms: u64) -> Self {
        let provider = crate::protocols::zhipu_with_timeout(api_key, timeout_ms)
            .expect("Failed to create Zhipu provider");
        Self { provider: Arc::new(provider) }
    }

    /// Create new client with Tencent Hunyuan protocol
    ///
    /// Uses the OpenAI-compatible endpoint: `https://api.hunyuan.cloud.tencent.com/v1`
    ///
    /// ```rust,ignore
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::hunyuan("sk-...");
    /// ```
    pub fn hunyuan(api_key: &str) -> Self {
        let base_url = "https://api.hunyuan.cloud.tencent.com/v1";
        let protocol = OpenAIProtocol::with_url(api_key, base_url);
        let config = ProviderConfig::new(api_key)
            .with_base_url(base_url)
            .with_timeout_ms(30000); // 默认30秒超时
        let provider = GenericProvider::new(config, protocol)
            .expect("Failed to create Hunyuan provider");
        Self { provider: Arc::new(provider) }
    }

    /// Create new client with Tencent Hunyuan protocol and custom timeout
    ///
    /// ```rust,ignore
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::hunyuan_with_timeout("sk-...", 60000);
    /// ```
    pub fn hunyuan_with_timeout(api_key: &str, timeout_ms: u64) -> Self {
        let base_url = "https://api.hunyuan.cloud.tencent.com/v1";
        let protocol = OpenAIProtocol::with_url(api_key, base_url);
        let config = ProviderConfig::new(api_key)
            .with_base_url(base_url)
            .with_timeout_ms(timeout_ms);
        let provider = GenericProvider::new(config, protocol)
            .expect("Failed to create Hunyuan provider");
        Self { provider: Arc::new(provider) }
    }

    /// Create new client with Tencent Hunyuan native protocol
    ///
    /// Uses Tencent Cloud's native API with TC3-HMAC-SHA256 signature authentication.
    /// Requires the "tencent-native" feature to be enabled.
    ///
    /// ```rust,ignore
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::hunyuan_native("your-secret-id", "your-secret-key", Some("ap-beijing"));
    /// ```
    #[cfg(feature = "tencent-native")]
    pub fn hunyuan_native(secret_id: &str, secret_key: &str, region: Option<&str>) -> Self {
        let provider = crate::protocols::hunyuan_native::hunyuan_native(secret_id, secret_key, region)
            .expect("Failed to create Hunyuan native provider");
        Self { provider: Arc::new(provider) }
    }

    /// Create new client with Tencent Hunyuan native protocol and custom timeout
    ///
    /// ```rust,ignore
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::hunyuan_native_with_timeout("secret-id", "secret-key", Some("ap-beijing"), 60000);
    /// ```
    #[cfg(feature = "tencent-native")]
    pub fn hunyuan_native_with_timeout(secret_id: &str, secret_key: &str, region: Option<&str>, timeout_ms: u64) -> Self {
        let provider = crate::protocols::hunyuan_native::hunyuan_native_with_timeout(secret_id, secret_key, region, timeout_ms)
            .expect("Failed to create Hunyuan native provider");
        Self { provider: Arc::new(provider) }
    }

    /// Send a chat completion request
    ///
    /// ```rust,ignore
    /// use llm_connector::{LlmClient, types::{ChatRequest, Message}};
    ///
    /// let client = LlmClient::openai("sk-...", None);
    /// let request = ChatRequest {
    ///     model: "gpt-4".to_string(),
    ///     messages: vec![Message::user("Hello!")],
    ///     ..Default::default()
    /// };
    ///
    /// let response = client.chat(&request).await?;
    /// println!("Response: {}", response.choices[0].message.content);
    /// ```
    pub async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        self.provider.chat(request).await
    }

    /// Send a streaming chat completion request
    ///
    /// Requires the "streaming" feature to be enabled.
    #[cfg(feature = "streaming")]
    pub async fn chat_stream(&self, request: &ChatRequest) -> Result<crate::types::ChatStream, LlmConnectorError> {
        self.provider.chat_stream(request).await
    }

    /// Send a streaming chat completion request with format configuration
    ///
    /// Allows specifying the output format (OpenAI or Ollama) and other streaming options.
    /// Requires the "streaming" feature to be enabled.
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::{LlmClient, types::{ChatRequest, Message, StreamingConfig, StreamingFormat}};
    /// use futures_util::StreamExt;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = LlmClient::openai("sk-...", None);
    /// let request = ChatRequest {
    ///     model: "gpt-4".to_string(),
    ///     messages: vec![Message::user("Hello!")],
    ///     ..Default::default()
    /// };
    ///
    /// let config = StreamingConfig {
    ///     format: StreamingFormat::Ollama,
    ///     include_usage: true,
    ///     include_reasoning: false,
    /// };
    ///
    /// let mut stream = client.chat_stream_with_format(&request, &config).await?;
    /// while let Some(chunk) = stream.next().await {
    ///     let chunk = chunk?;
    ///     println!("Chunk: {}", chunk.content);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "streaming")]
    pub async fn chat_stream_with_format(
        &self,
        request: &ChatRequest,
        config: &crate::types::StreamingConfig,
    ) -> Result<crate::types::ChatStream, LlmConnectorError> {
        self.provider.chat_stream_with_format(request, config).await
    }

    /// Send a streaming chat completion request in Ollama format (legacy)
    ///
    /// This method returns Ollama format embedded in OpenAI format for backward compatibility.
    /// For pure Ollama format, use `chat_stream_ollama()` instead.
    /// Requires the "streaming" feature to be enabled.
    #[cfg(feature = "streaming")]
    pub async fn chat_stream_ollama_embedded(&self, request: &ChatRequest) -> Result<crate::types::ChatStream, LlmConnectorError> {
        let config = crate::types::StreamingConfig {
            format: crate::types::StreamingFormat::Ollama,
            include_usage: true,
            include_reasoning: false,
        };
        self.chat_stream_with_format(request, &config).await
    }

    /// Send a streaming chat completion request in pure Ollama format
    ///
    /// Returns a stream of pure Ollama format chunks, not wrapped in OpenAI format.
    /// This is the correct method to use for Ollama-compatible tools like Zed.dev.
    /// Requires the "streaming" feature to be enabled.
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::{LlmClient, types::{ChatRequest, Message}};
    /// use futures_util::StreamExt;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = LlmClient::openai("sk-...", None);
    /// let request = ChatRequest {
    ///     model: "gpt-4".to_string(),
    ///     messages: vec![Message::user("Hello!")],
    ///     ..Default::default()
    /// };
    ///
    /// let mut stream = client.chat_stream_ollama(&request).await?;
    /// while let Some(chunk) = stream.next().await {
    ///     let chunk = chunk?;
    ///     // chunk is now a pure OllamaStreamChunk
    ///     println!("Content: {}", chunk.message.content);
    ///     if chunk.done {
    ///         println!("Stream completed!");
    ///         break;
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "streaming")]
    pub async fn chat_stream_ollama(&self, request: &ChatRequest) -> Result<crate::types::OllamaChatStream, LlmConnectorError> {
        self.provider.chat_stream_ollama_pure(request).await
    }

    /// Fetch available models from the API (online)
    ///
    /// This makes an API call to retrieve the list of available models.
    /// Returns an error if the provider doesn't support model listing or if the API call fails.
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::LlmClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = LlmClient::openai("sk-...", None);
    /// let models = client.fetch_models().await?;
    /// println!("Available models: {:?}", models);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_models(&self) -> Result<Vec<String>, LlmConnectorError> {
        self.provider.fetch_models().await
    }

    /// Get protocol name
    pub fn protocol_name(&self) -> &str {
        self.provider.name()
    }

    // Ollama model management methods moved to `ollama` trait module
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        // Test all protocol creation methods
        let _openai = LlmClient::openai("test-key", None);
        let _anthropic = LlmClient::anthropic("test-key");
        let _aliyun = LlmClient::aliyun("test-key");
        let _zhipu = LlmClient::zhipu("test-key");
        let _ollama = LlmClient::ollama(None);
    }

    #[test]
    fn test_protocol_names() {
        let openai_client = LlmClient::openai("test-key", None);
        assert_eq!(openai_client.protocol_name(), "openai");

        let anthropic_client = LlmClient::anthropic("test-key");
        assert_eq!(anthropic_client.protocol_name(), "anthropic");

        let aliyun_client = LlmClient::aliyun("test-key");
        assert_eq!(aliyun_client.protocol_name(), "aliyun");

        let zhipu_client = LlmClient::zhipu("test-key");
        assert_eq!(zhipu_client.protocol_name(), "zhipu");

        let ollama_client = LlmClient::ollama(None);
        assert_eq!(ollama_client.protocol_name(), "ollama");
    }
}