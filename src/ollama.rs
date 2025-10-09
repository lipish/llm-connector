//! Ollama model management trait
//!
//! Extension trait providing simple model management operations for
//! `LlmClient` when using the Ollama protocol.

use crate::error::LlmConnectorError;
use std::future::Future;

/// Model management operations for Ollama
pub trait OllamaModelOps {
    fn is_ollama(&self) -> bool;
    fn ensure_ollama(&self) -> Result<(), LlmConnectorError>;

    fn list_models(&self) -> impl Future<Output = Result<Vec<String>, LlmConnectorError>> + Send;
    fn pull_model(&self, model_name: &str) -> impl Future<Output = Result<(), LlmConnectorError>> + Send;
    fn push_model(&self, model_name: &str) -> impl Future<Output = Result<(), LlmConnectorError>> + Send;
    fn delete_model(&self, model_name: &str) -> impl Future<Output = Result<(), LlmConnectorError>> + Send;
    fn show_model(&self, model_name: &str) -> impl Future<Output = Result<crate::protocols::ollama::OllamaModel, LlmConnectorError>> + Send;
}

impl OllamaModelOps for crate::LlmClient {
    fn is_ollama(&self) -> bool {
        self.protocol_name() == "ollama"
    }

    fn ensure_ollama(&self) -> Result<(), LlmConnectorError> {
        if self.is_ollama() { Ok(()) } else { Err(LlmConnectorError::UnsupportedOperation(
            "Model management is only supported for Ollama protocol".to_string()
        )) }
    }

    fn list_models(&self) -> impl Future<Output = Result<Vec<String>, LlmConnectorError>> + Send {
        let this = self;
        async move {
            this.ensure_ollama()?;
            if let Some(ollama_provider) = this.provider_dyn().as_any().downcast_ref::<crate::protocols::core::GenericProvider<crate::protocols::ollama::OllamaProtocol>>() {
                let client = reqwest::Client::new();
                ollama_provider.adapter().list_models(&client).await
            } else {
                Err(LlmConnectorError::UnsupportedOperation(
                    "Failed to access Ollama protocol".to_string()
                ))
            }
        }
    }

    fn pull_model(&self, model_name: &str) -> impl Future<Output = Result<(), LlmConnectorError>> + Send {
        let this = self;
        let model_name = model_name.to_string();
        async move {
            this.ensure_ollama()?;
            if let Some(ollama_provider) = this.provider_dyn().as_any().downcast_ref::<crate::protocols::core::GenericProvider<crate::protocols::ollama::OllamaProtocol>>() {
                let client = reqwest::Client::new();
                ollama_provider.adapter().pull_model(&client, &model_name).await
            } else {
                Err(LlmConnectorError::UnsupportedOperation(
                    "Failed to access Ollama protocol".to_string()
                ))
            }
        }
    }

    fn push_model(&self, model_name: &str) -> impl Future<Output = Result<(), LlmConnectorError>> + Send {
        let this = self;
        let model_name = model_name.to_string();
        async move {
            this.ensure_ollama()?;
            if let Some(ollama_provider) = this.provider_dyn().as_any().downcast_ref::<crate::protocols::core::GenericProvider<crate::protocols::ollama::OllamaProtocol>>() {
                let client = reqwest::Client::new();
                ollama_provider.adapter().push_model(&client, &model_name).await
            } else {
                Err(LlmConnectorError::UnsupportedOperation(
                    "Failed to access Ollama protocol".to_string()
                ))
            }
        }
    }

    fn delete_model(&self, model_name: &str) -> impl Future<Output = Result<(), LlmConnectorError>> + Send {
        let this = self;
        let model_name = model_name.to_string();
        async move {
            this.ensure_ollama()?;
            if let Some(ollama_provider) = this.provider_dyn().as_any().downcast_ref::<crate::protocols::core::GenericProvider<crate::protocols::ollama::OllamaProtocol>>() {
                let client = reqwest::Client::new();
                ollama_provider.adapter().delete_model(&client, &model_name).await
            } else {
                Err(LlmConnectorError::UnsupportedOperation(
                    "Failed to access Ollama protocol".to_string()
                ))
            }
        }
    }

    fn show_model(&self, model_name: &str) -> impl Future<Output = Result<crate::protocols::ollama::OllamaModel, LlmConnectorError>> + Send {
        let this = self;
        let model_name = model_name.to_string();
        async move {
            this.ensure_ollama()?;
            if let Some(ollama_provider) = this.provider_dyn().as_any().downcast_ref::<crate::protocols::core::GenericProvider<crate::protocols::ollama::OllamaProtocol>>() {
                let client = reqwest::Client::new();
                ollama_provider.adapter().show_model(&client, &model_name).await
            } else {
                Err(LlmConnectorError::UnsupportedOperation(
                    "Failed to access Ollama protocol".to_string()
                ))
            }
        }
    }
}