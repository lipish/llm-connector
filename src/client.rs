//! Main client for llm-connector

use std::collections::HashMap;
use std::sync::Arc;

use crate::config::Config;
use crate::error::LlmConnectorError;
use crate::providers::Provider;
use crate::providers::base::utils;
use crate::types::{ChatRequest, ChatResponse};

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// Main client for interacting with LLM providers
pub struct Client {
    providers: HashMap<String, Arc<dyn Provider>>,
    config: Config,
}

impl Client {
    /// Create a new client with the given configuration
    pub fn with_config(config: Config) -> Self {
        let mut client = Self {
            providers: HashMap::new(),
            config: config.clone(),
        };
        
        // Initialize providers based on configuration
        #[cfg(feature = "reqwest")]
        client.initialize_providers();
        
        client
    }
    
    /// Create a client from environment variables
    pub fn from_env() -> Self {
        Self::with_config(Config::from_env())
    }
    
    /// Initialize providers based on configuration
    #[cfg(feature = "reqwest")]
    fn initialize_providers(&mut self) {
        use crate::providers::{GenericProvider, DeepSeekAdapter, AliyunAdapter, ZhipuAdapter};

        // Initialize DeepSeek provider
        if let Some(deepseek_config) = &self.config.deepseek {
            if let Ok(provider) = GenericProvider::new(deepseek_config.clone(), DeepSeekAdapter) {
                self.providers.insert("deepseek".to_string(), Arc::new(provider));
            }
        }

        // Initialize Aliyun provider
        if let Some(aliyun_config) = &self.config.aliyun {
            if let Ok(provider) = GenericProvider::new(aliyun_config.clone(), AliyunAdapter) {
                self.providers.insert("aliyun".to_string(), Arc::new(provider));
            }
        }

        // Initialize Zhipu provider
        if let Some(zhipu_config) = &self.config.zhipu {
            if let Ok(provider) = GenericProvider::new(zhipu_config.clone(), ZhipuAdapter) {
                self.providers.insert("zhipu".to_string(), Arc::new(provider));
            }
        }

        // TODO: Initialize other providers when they are implemented
        // Example for adding a new provider:
        // if let Some(new_config) = &self.config.new_provider {
        //     if let Ok(provider) = GenericProvider::new(new_config.clone(), NewProviderAdapter) {
        //         self.providers.insert("new_provider".to_string(), Arc::new(provider));
        //     }
        // }
    }
    
    /// Send a chat completion request
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        // Validate the request
        utils::validate_chat_request(&request)?;
        
        // Determine the provider
        let provider = self.get_provider_for_model(&request.model)?;
        
        // Clean the model name (remove provider prefix if present)
        let mut cleaned_request = request;
        cleaned_request.model = utils::clean_model_name(&cleaned_request.model).to_string();
        
        // Send the request
        provider.chat(&cleaned_request).await
    }
    
    /// Send a streaming chat completion request
    #[cfg(feature = "streaming")]
    pub async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        // Validate the request
        utils::validate_chat_request(&request)?;
        
        // Determine the provider
        let provider = self.get_provider_for_model(&request.model)?;
        
        // Clean the model name (remove provider prefix if present)
        let mut cleaned_request = request;
        cleaned_request.model = utils::clean_model_name(&cleaned_request.model).to_string();
        cleaned_request.stream = Some(true); // Ensure streaming is enabled
        
        // Send the streaming request
        provider.chat_stream(&cleaned_request).await
    }
    
    /// Get the appropriate provider for a model
    fn get_provider_for_model(&self, model: &str) -> Result<Arc<dyn Provider>, LlmConnectorError> {
        // First try to detect provider from model name
        let provider_name = utils::detect_provider_from_model(model)
            .ok_or_else(|| LlmConnectorError::UnsupportedModel(model.to_string()))?;
        
        // Get the provider
        self.providers
            .get(provider_name)
            .cloned()
            .ok_or_else(|| {
                LlmConnectorError::ConfigError(
                    format!("Provider '{}' not configured", provider_name)
                )
            })
    }
    
    /// List all supported models across all configured providers
    pub fn list_models(&self) -> Vec<String> {
        let mut models = Vec::new();
        
        for (provider_name, provider) in &self.providers {
            for model in provider.supported_models() {
                // Add both prefixed and non-prefixed versions
                models.push(format!("{}/{}", provider_name, model));
                models.push(model);
            }
        }
        
        // Remove duplicates and sort
        models.sort();
        models.dedup();
        models
    }
    
    /// List all configured providers
    pub fn list_providers(&self) -> Vec<String> {
        self.config.list_providers()
    }
    
    /// Check if a model is supported
    pub fn supports_model(&self, model: &str) -> bool {
        if let Ok(provider) = self.get_provider_for_model(model) {
            let clean_model = utils::clean_model_name(model);
            provider.supports_model(clean_model)
        } else {
            false
        }
    }
    
    /// Get provider information for a model
    pub fn get_provider_info(&self, model: &str) -> Option<String> {
        utils::detect_provider_from_model(model).map(|s| s.to_string())
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Message;
    
    #[test]
    fn test_client_creation() {
        let config = Config::default();
        let client = Client::with_config(config);
        assert_eq!(client.list_providers().len(), 0);
    }
    
    #[test]
    fn test_model_support_detection() {
        let client = Client::default();
        
        // These should work if providers are configured
        assert!(client.get_provider_info("gpt-4").is_some());
        assert!(client.get_provider_info("claude-3-haiku").is_some());
        assert!(client.get_provider_info("deepseek-chat").is_some());
        
        // This should not work
        assert!(client.get_provider_info("unknown-model").is_none());
    }
    
    #[tokio::test]
    async fn test_request_validation() {
        let client = Client::default();
        
        // Empty model should fail
        let request = ChatRequest {
            model: "".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        
        let result = client.chat(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmConnectorError::InvalidRequest(_)));
    }
}
