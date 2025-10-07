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
    /// Create new client with OpenAI protocol
    ///
    /// ```rust
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::openai("sk-...");
    /// ```
    pub fn openai(api_key: &str) -> Self {
        let protocol = OpenAIProtocol::new(api_key);
        let config = ProviderConfig::new(api_key)
            .with_base_url("https://api.openai.com/v1");
        let provider = GenericProvider::new(config, protocol)
            .expect("Failed to create OpenAI provider");
        Self {
            provider: Arc::new(provider),
        }
    }

    /// Create new client with OpenAI-compatible protocol
    ///
    /// Use this for OpenAI-compatible endpoints with custom base URLs.
    ///
    /// ```rust
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::openai_compatible(
    ///     "sk-...",
    ///     "https://api.example.com/v1"
    /// );
    /// ```
    pub fn openai_compatible(api_key: &str, base_url: &str) -> Self {
        let protocol = OpenAIProtocol::with_url(api_key, base_url);
        let config = ProviderConfig::new(api_key)
            .with_base_url(base_url);
        let provider = GenericProvider::new(config, protocol)
            .expect("Failed to create OpenAI-compatible provider");
        Self {
            provider: Arc::new(provider),
        }
    }

    /// Create new client with Anthropic protocol
    ///
    /// ```rust
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

    /// Create new client with Aliyun protocol
    ///
    /// ```rust
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
        Self {
            provider: Arc::new(provider),
        }
    }

    /// Create new client with Ollama protocol (local)
    ///
    /// No API key needed for local Ollama.
    ///
    /// ```rust
    /// use llm_connector::LlmClient;
    ///
    /// // Default: localhost:11434
    /// let client = LlmClient::ollama();
    ///
    /// // Custom URL
    /// let client = LlmClient::ollama_at("http://192.168.1.100:11434");
    /// ```
    pub fn ollama() -> Self {
        let protocol = OllamaProtocol::new();
        let config = ProviderConfig::new("") // Ollama doesn't need API key
            .with_base_url("http://localhost:11434");
        let provider = GenericProvider::new(config, protocol)
            .expect("Failed to create Ollama provider");
        Self {
            provider: Arc::new(provider),
        }
    }

    /// Create new client with Ollama protocol at custom URL
    pub fn ollama_at(base_url: &str) -> Self {
        let protocol = OllamaProtocol::with_url(base_url);
        let config = ProviderConfig::new("") // Ollama doesn't need API key
            .with_base_url(base_url);
        let provider = GenericProvider::new(config, protocol)
            .expect("Failed to create Ollama provider");
        Self {
            provider: Arc::new(provider),
        }
    }

    /// Send a chat completion request
    ///
    /// ```rust
    /// use llm_connector::{LlmClient, types::{ChatRequest, Message}};
    ///
    /// let client = LlmClient::openai("sk-...");
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
    /// let client = LlmClient::openai("sk-...");
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

    // ============================================================================
    // Ollama Model Management Methods
    // ============================================================================

    /// List all available Ollama models
    ///
    /// Only works with Ollama protocol clients.
    ///
    /// ```rust
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::ollama();
    /// let models = client.list_ollama_models().await?;
    /// for model in models {
    ///     println!("Model: {}", model);
    /// }
    /// ```
    pub async fn list_ollama_models(&self) -> Result<Vec<String>, LlmConnectorError> {
        if self.provider.name() != "ollama" {
            return Err(LlmConnectorError::UnsupportedOperation(
                "Model management is only supported for Ollama protocol".to_string()
            ));
        }

        // We need to access the OllamaProtocol directly
        // This is a bit of a hack due to the trait abstraction
        // In a real implementation, we might want to add these methods to the Provider trait
        if let Some(ollama_provider) = self.provider.as_any().downcast_ref::<crate::protocols::core::GenericProvider<crate::protocols::ollama::OllamaProtocol>>() {
            let client = reqwest::Client::new();
            ollama_provider.adapter().list_models(&client).await
        } else {
            Err(LlmConnectorError::UnsupportedOperation(
                "Failed to access Ollama protocol".to_string()
            ))
        }
    }

    /// Pull a model from Ollama registry
    ///
    /// Only works with Ollama protocol clients.
    ///
    /// ```rust
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::ollama();
    /// client.pull_ollama_model("llama3.2").await?;
    /// println!("Model pulled successfully!");
    /// ```
    pub async fn pull_ollama_model(&self, model_name: &str) -> Result<(), LlmConnectorError> {
        if self.provider.name() != "ollama" {
            return Err(LlmConnectorError::UnsupportedOperation(
                "Model management is only supported for Ollama protocol".to_string()
            ));
        }

        if let Some(ollama_provider) = self.provider.as_any().downcast_ref::<crate::protocols::core::GenericProvider<crate::protocols::ollama::OllamaProtocol>>() {
            let client = reqwest::Client::new();
            ollama_provider.adapter().pull_model(&client, model_name).await
        } else {
            Err(LlmConnectorError::UnsupportedOperation(
                "Failed to access Ollama protocol".to_string()
            ))
        }
    }

    /// Push a model to Ollama registry
    ///
    /// Only works with Ollama protocol clients.
    pub async fn push_ollama_model(&self, model_name: &str) -> Result<(), LlmConnectorError> {
        if self.provider.name() != "ollama" {
            return Err(LlmConnectorError::UnsupportedOperation(
                "Model management is only supported for Ollama protocol".to_string()
            ));
        }

        if let Some(ollama_provider) = self.provider.as_any().downcast_ref::<crate::protocols::core::GenericProvider<crate::protocols::ollama::OllamaProtocol>>() {
            let client = reqwest::Client::new();
            ollama_provider.adapter().push_model(&client, model_name).await
        } else {
            Err(LlmConnectorError::UnsupportedOperation(
                "Failed to access Ollama protocol".to_string()
            ))
        }
    }

    /// Delete a model from Ollama
    ///
    /// Only works with Ollama protocol clients.
    ///
    /// ```rust
    /// use llm_connector::LlmClient;
    ///
    /// let client = LlmClient::ollama();
    /// client.delete_ollama_model("llama3.2").await?;
    /// println!("Model deleted successfully!");
    /// ```
    pub async fn delete_ollama_model(&self, model_name: &str) -> Result<(), LlmConnectorError> {
        if self.provider.name() != "ollama" {
            return Err(LlmConnectorError::UnsupportedOperation(
                "Model management is only supported for Ollama protocol".to_string()
            ));
        }

        if let Some(ollama_provider) = self.provider.as_any().downcast_ref::<crate::protocols::core::GenericProvider<crate::protocols::ollama::OllamaProtocol>>() {
            let client = reqwest::Client::new();
            ollama_provider.adapter().delete_model(&client, model_name).await
        } else {
            Err(LlmConnectorError::UnsupportedOperation(
                "Failed to access Ollama protocol".to_string()
            ))
        }
    }

    /// Get detailed information about an Ollama model
    ///
    /// Only works with Ollama protocol clients.
    pub async fn show_ollama_model(&self, model_name: &str) -> Result<crate::protocols::ollama::OllamaModel, LlmConnectorError> {
        if self.provider.name() != "ollama" {
            return Err(LlmConnectorError::UnsupportedOperation(
                "Model management is only supported for Ollama protocol".to_string()
            ));
        }

        if let Some(ollama_provider) = self.provider.as_any().downcast_ref::<crate::protocols::core::GenericProvider<crate::protocols::ollama::OllamaProtocol>>() {
            let client = reqwest::Client::new();
            ollama_provider.adapter().show_model(&client, model_name).await
        } else {
            Err(LlmConnectorError::UnsupportedOperation(
                "Failed to access Ollama protocol".to_string()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        // Test all protocol creation methods
        let _openai = LlmClient::openai("test-key");
        let _anthropic = LlmClient::anthropic("test-key");
        let _aliyun = LlmClient::aliyun("test-key");
        let _ollama = LlmClient::ollama();
    }

    #[test]
    fn test_protocol_names() {
        let openai_client = LlmClient::openai("test-key");
        assert_eq!(openai_client.protocol_name(), "openai");

        let anthropic_client = LlmClient::anthropic("test-key");
        assert_eq!(anthropic_client.protocol_name(), "anthropic");

        let aliyun_client = LlmClient::aliyun("test-key");
        assert_eq!(aliyun_client.protocol_name(), "aliyun");

        let ollama_client = LlmClient::ollama();
        assert_eq!(ollama_client.protocol_name(), "ollama");
    }
}