//! Ollama model management trait
//!
//! Extension trait providing simple model management operations for
//! `LlmClient` when using the Ollama protocol.

use crate::error::LlmConnectorError;
use async_trait::async_trait;

/// Model management operations for Ollama
#[async_trait]
pub trait OllamaModelOps {
    fn is_ollama(&self) -> bool;
    fn ensure_ollama(&self) -> Result<(), LlmConnectorError>;

    async fn list_models(&self) -> Result<Vec<String>, LlmConnectorError>;
    async fn pull_model(&self, model_name: &str) -> Result<(), LlmConnectorError>;
    async fn push_model(&self, model_name: &str) -> Result<(), LlmConnectorError>;
    async fn delete_model(&self, model_name: &str) -> Result<(), LlmConnectorError>;
    async fn show_model(&self, model_name: &str) -> Result<crate::v1::providers::ollama::OllamaModel, LlmConnectorError>;
}

#[async_trait]
impl OllamaModelOps for crate::LlmClient {
    fn is_ollama(&self) -> bool {
        self.protocol_name() == "ollama"
    }

    fn ensure_ollama(&self) -> Result<(), LlmConnectorError> {
        if self.is_ollama() { Ok(()) } else { Err(LlmConnectorError::UnsupportedOperation(
            "Model management is only supported for Ollama protocol".to_string()
        )) }
    }

    async fn list_models(&self) -> Result<Vec<String>, LlmConnectorError> {
        self.ensure_ollama()?;
        if let Some(ollama_provider) = self.provider_dyn().as_any().downcast_ref::<crate::v1::providers::ollama::OllamaProvider>() {
            ollama_provider.list_models().await
        } else {
            Err(LlmConnectorError::UnsupportedOperation(
                "Failed to access Ollama provider".to_string()
            ))
        }
    }

    async fn pull_model(&self, model_name: &str) -> Result<(), LlmConnectorError> {
        self.ensure_ollama()?;
        if let Some(ollama_provider) = self.provider_dyn().as_any().downcast_ref::<crate::v1::providers::ollama::OllamaProvider>() {
            ollama_provider.pull_model(model_name).await
        } else {
            Err(LlmConnectorError::UnsupportedOperation(
                "Failed to access Ollama provider".to_string()
            ))
        }
    }

    async fn push_model(&self, model_name: &str) -> Result<(), LlmConnectorError> {
        self.ensure_ollama()?;
        if let Some(ollama_provider) = self.provider_dyn().as_any().downcast_ref::<crate::v1::providers::ollama::OllamaProvider>() {
            ollama_provider.push_model(model_name).await
        } else {
            Err(LlmConnectorError::UnsupportedOperation(
                "Failed to access Ollama provider".to_string()
            ))
        }
    }

    async fn delete_model(&self, model_name: &str) -> Result<(), LlmConnectorError> {
        self.ensure_ollama()?;
        if let Some(ollama_provider) = self.provider_dyn().as_any().downcast_ref::<crate::v1::providers::ollama::OllamaProvider>() {
            ollama_provider.delete_model(model_name).await
        } else {
            Err(LlmConnectorError::UnsupportedOperation(
                "Failed to access Ollama provider".to_string()
            ))
        }
    }

    async fn show_model(&self, model_name: &str) -> Result<crate::v1::providers::ollama::OllamaModel, LlmConnectorError> {
        self.ensure_ollama()?;
        if let Some(ollama_provider) = self.provider_dyn().as_any().downcast_ref::<crate::v1::providers::ollama::OllamaProvider>() {
            ollama_provider.show_model(model_name).await
        } else {
            Err(LlmConnectorError::UnsupportedOperation(
                "Failed to access Ollama provider".to_string()
            ))
        }
    }
}