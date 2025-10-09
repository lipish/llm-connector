//! Minimal LLM Client
//!
//! This client provides a simple interface to use any of the 4 supported protocols.
//! No complex configuration, just pick a protocol and start chatting.

use std::sync::Arc;

use crate::error::LlmConnectorError;
use crate::protocols::{GenericProvider, OpenAIProtocol, AnthropicProtocol, AliyunProtocol, OllamaProtocol};
use crate::config::ProviderConfig;
use crate::types::{ChatRequest, ChatResponse};

/// Minimal LLM Client
///
/// Supports 4 protocols: OpenAI, Anthropic, Aliyun, Ollama
/// No complex configuration needed - just pick a protocol and start.
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
            .with_base_url(base);
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
        let protocol = AliyunProtocol::new(api_key);
        let config = ProviderConfig::new(api_key)
            .with_base_url("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation");
        let provider = GenericProvider::new(config, protocol)
            .expect("Failed to create Aliyun provider");
        Self { provider: Arc::new(provider) }
    }

    /// Create new client with Ollama protocol (single constructor)
    ///
    /// Default base URL: `http://localhost:11434`. Pass `Some(url)` to override.
    pub fn ollama(base_url: Option<&str>) -> Self {
        let base = base_url.unwrap_or("http://localhost:11434");
        let protocol = OllamaProtocol::with_url(base);
        let config = ProviderConfig::new("") // Ollama doesn't need API key
            .with_base_url(base);
        let provider = GenericProvider::new(config, protocol)
            .expect("Failed to create Ollama provider");
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
            .with_base_url("https://api.anthropic.com");
        let provider = GenericProvider::new(config, protocol)
            .expect("Failed to create Anthropic provider");
        Self {
            provider: Arc::new(provider),
        }
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

        let ollama_client = LlmClient::ollama(None);
        assert_eq!(ollama_client.protocol_name(), "ollama");
    }
}