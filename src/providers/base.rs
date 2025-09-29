//! Base provider trait and utilities

use async_trait::async_trait;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse};

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// Trait for LLM providers
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;
    
    /// Get the list of supported models
    fn supported_models(&self) -> Vec<String>;
    
    /// Check if a model is supported
    fn supports_model(&self, model: &str) -> bool {
        self.supported_models().iter().any(|m| m == model)
    }
    
    /// Send a chat completion request
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError>;
    
    /// Send a streaming chat completion request
    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError>;
}

/// Utility functions for providers
pub mod utils {
    use crate::types::ChatRequest;
    
    /// Parse model name to extract provider and model
    pub fn parse_model_name(model: &str) -> (Option<&str>, &str) {
        if let Some(slash_pos) = model.find('/') {
            let (provider, model_name) = model.split_at(slash_pos);
            (Some(provider), &model_name[1..]) // Skip the '/'
        } else {
            (None, model)
        }
    }
    
    /// Detect provider from model name
    pub fn detect_provider_from_model(model: &str) -> Option<&str> {
        // Check for explicit provider prefix
        if let (Some(provider), _) = parse_model_name(model) {
            return Some(provider);
        }
        
        // Auto-detect based on model name patterns
        if model.starts_with("gpt-") || model.starts_with("text-") || model.starts_with("ft:") {
            Some("openai")
        } else if model.starts_with("claude") {
            Some("anthropic")
        } else if model.starts_with("deepseek") {
            Some("deepseek")
        } else if model.starts_with("glm") || model.starts_with("chatglm") {
            Some("zhipu")
        } else if model.starts_with("qwen") {
            Some("aliyun")
        } else if model.starts_with("moonshot") || model == "kimi-chat" {
            Some("kimi")
        } else {
            None
        }
    }
    
    /// Validate chat request
    pub fn validate_chat_request(request: &ChatRequest) -> Result<(), crate::error::LlmConnectorError> {
        if request.model.is_empty() {
            return Err(crate::error::LlmConnectorError::InvalidRequest(
                "Model name cannot be empty".to_string()
            ));
        }
        
        if request.messages.is_empty() {
            return Err(crate::error::LlmConnectorError::InvalidRequest(
                "Messages cannot be empty".to_string()
            ));
        }
        
        // Validate message roles
        for message in &request.messages {
            if message.role.is_empty() {
                return Err(crate::error::LlmConnectorError::InvalidRequest(
                    "Message role cannot be empty".to_string()
                ));
            }
            
            if !matches!(message.role.as_str(), "system" | "user" | "assistant" | "tool") {
                return Err(crate::error::LlmConnectorError::InvalidRequest(
                    format!("Invalid message role: {}", message.role)
                ));
            }
        }
        
        // Validate temperature
        if let Some(temp) = request.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err(crate::error::LlmConnectorError::InvalidRequest(
                    "Temperature must be between 0.0 and 2.0".to_string()
                ));
            }
        }
        
        // Validate top_p
        if let Some(top_p) = request.top_p {
            if top_p < 0.0 || top_p > 1.0 {
                return Err(crate::error::LlmConnectorError::InvalidRequest(
                    "top_p must be between 0.0 and 1.0".to_string()
                ));
            }
        }
        
        // Validate max_tokens
        if let Some(max_tokens) = request.max_tokens {
            if max_tokens == 0 {
                return Err(crate::error::LlmConnectorError::InvalidRequest(
                    "max_tokens must be greater than 0".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Clean model name by removing provider prefix
    pub fn clean_model_name(model: &str) -> &str {
        parse_model_name(model).1
    }
    
    #[cfg(test)]
    mod tests {
        use super::*;
        
        #[test]
        fn test_parse_model_name() {
            assert_eq!(parse_model_name("openai/gpt-4"), (Some("openai"), "gpt-4"));
            assert_eq!(parse_model_name("gpt-4"), (None, "gpt-4"));
            assert_eq!(parse_model_name("anthropic/claude-3-haiku"), (Some("anthropic"), "claude-3-haiku"));
        }
        
        #[test]
        fn test_detect_provider_from_model() {
            assert_eq!(detect_provider_from_model("openai/gpt-4"), Some("openai"));
            assert_eq!(detect_provider_from_model("gpt-4"), Some("openai"));
            assert_eq!(detect_provider_from_model("claude-3-haiku"), Some("anthropic"));
            assert_eq!(detect_provider_from_model("deepseek-chat"), Some("deepseek"));
            assert_eq!(detect_provider_from_model("glm-4"), Some("zhipu"));
            assert_eq!(detect_provider_from_model("qwen-turbo"), Some("aliyun"));
            assert_eq!(detect_provider_from_model("moonshot-v1-8k"), Some("kimi"));
            assert_eq!(detect_provider_from_model("unknown-model"), None);
        }
        
        #[test]
        fn test_clean_model_name() {
            assert_eq!(clean_model_name("openai/gpt-4"), "gpt-4");
            assert_eq!(clean_model_name("gpt-4"), "gpt-4");
        }
    }
}
