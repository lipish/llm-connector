//! OllamaserviceProviderimplementation
//!
//! Ollama is a local LLM service，with special model management features，therefore requires custom Provider implementation。

use crate::core::{HttpClient, Provider};
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, Choice, Message, Role};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// OllamaserviceProvider
///
/// Since Ollama has special model management features, we use custom Provider implementation
/// instead of GenericProvider pattern。
#[derive(Clone, Debug)]
pub struct OllamaProvider {
    client: HttpClient,
    base_url: String,
}

impl OllamaProvider {
    /// Create new OllamaProvider
    ///
    /// # Parameters
    /// - `base_url`: Ollama service URL (default: http://localhost:11434)
    ///
    /// # Example
    /// ```rust,no_run
    /// use llm_connector::providers::OllamaProvider;
    ///
    /// let provider = OllamaProvider::new("http://localhost:11434").unwrap();
    /// ```
    pub fn new(base_url: &str) -> Result<Self, LlmConnectorError> {
        // Content-Type is automatically set by HttpClient::post() .json() method
        let client = HttpClient::new(base_url)?;

        Ok(Self {
            client,
            base_url: base_url.to_string(),
        })
    }

    /// CreatewithcustomconfigurationOllamaProvider
    pub fn with_config(
        base_url: &str,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        // Content-Type is automatically set by HttpClient::post() .json() method
        let client = HttpClient::with_config(base_url, timeout_secs, proxy)?;

        Ok(Self {
            client,
            base_url: base_url.to_string(),
        })
    }

    /// Pull model
    ///
    /// # Parameters
    /// - `model_name`: Model name to pull (such as "llama2", "codellama")
    ///
    /// # Example
    /// ```rust,no_run
    /// # use llm_connector::providers::OllamaProvider;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = OllamaProvider::new("http://localhost:11434")?;
    /// provider.pull_model("llama2").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn pull_model(&self, model_name: &str) -> Result<(), LlmConnectorError> {
        let request = OllamaPullRequest {
            name: model_name.to_string(),
            stream: Some(false),
        };

        let url = format!("{}/api/pull", self.base_url);
        let response = self.client.post(&url, &request).await?;

        if !response.status().is_success() {
            let text = response
                .text()
                .await
                .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;
            return Err(LlmConnectorError::ApiError(format!(
                "Failed to pull model: {}",
                text
            )));
        }

        Ok(())
    }

    /// Delete model
    ///
    /// # Parameters
    /// - `model_name`: Model name to delete
    ///
    /// # Example
    /// ```rust,no_run
    /// # use llm_connector::providers::OllamaProvider;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = OllamaProvider::new("http://localhost:11434")?;
    /// provider.delete_model("llama2").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_model(&self, model_name: &str) -> Result<(), LlmConnectorError> {
        let request = OllamaDeleteRequest {
            name: model_name.to_string(),
        };

        let url = format!("{}/api/delete", self.base_url);
        let response = self.client.post(&url, &request).await?;

        if !response.status().is_success() {
            let text = response
                .text()
                .await
                .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;
            return Err(LlmConnectorError::ApiError(format!(
                "Failed to delete model: {}",
                text
            )));
        }

        Ok(())
    }

    /// Get model information
    ///
    /// # Parameters
    /// - `model_name`: modelname
    ///
    /// # Returns
    /// Model details
    pub async fn show_model(&self, model_name: &str) -> Result<OllamaModelInfo, LlmConnectorError> {
        let request = OllamaShowRequest {
            name: model_name.to_string(),
        };

        let url = format!("{}/api/show", self.base_url);
        let response = self.client.post(&url, &request).await?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;

        if !status.is_success() {
            return Err(LlmConnectorError::ApiError(format!(
                "Failed to show model: {}",
                text
            )));
        }

        serde_json::from_str(&text).map_err(|e| {
            LlmConnectorError::ParseError(format!("Failed to parse model info: {}", e))
        })
    }

    /// Check if model exists
    pub async fn model_exists(&self, model_name: &str) -> Result<bool, LlmConnectorError> {
        match self.show_model(model_name).await {
            Ok(_) => Ok(true),
            Err(LlmConnectorError::ApiError(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn models(&self) -> Result<Vec<String>, LlmConnectorError> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).await?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;

        if !status.is_success() {
            return Err(LlmConnectorError::ApiError(format!(
                "Failed to get models: {}",
                text
            )));
        }

        let models_response: OllamaModelsResponse = serde_json::from_str(&text)
            .map_err(|e| LlmConnectorError::ParseError(format!("Failed to parse models: {}", e)))?;

        Ok(models_response.models.into_iter().map(|m| m.name).collect())
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|msg| OllamaMessage {
                    role: match msg.role {
                        Role::User => "user".to_string(),
                        Role::Assistant => "assistant".to_string(),
                        Role::System => "system".to_string(),
                        Role::Tool => "user".to_string(), // Ollama does not support tool role
                    },
                    // Ollama uses plain text format
                    content: msg.content_as_text(),
                })
                .collect(),
            stream: Some(false),
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens.map(|t| t as i32),
                top_p: request.top_p,
            }),
        };

        let url = format!("{}/api/chat", self.base_url);
        let response = self.client.post(&url, &ollama_request).await?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;

        if !status.is_success() {
            return Err(LlmConnectorError::ApiError(format!(
                "Ollama chat failed: {}",
                text
            )));
        }

        let ollama_response: OllamaChatResponse = serde_json::from_str(&text).map_err(|e| {
            LlmConnectorError::ParseError(format!("Failed to parse Ollama response: {}", e))
        })?;

        let content = ollama_response.message.content.clone();

        let choices = vec![Choice {
            index: 0,
            message: Message {
                role: Role::Assistant,
                content: vec![crate::types::MessageBlock::text(&content)],
                name: None,
                tool_calls: None,
                tool_call_id: None,
                reasoning_content: None,
                reasoning: None,
                thought: None,
                thinking: None,
            },
            finish_reason: Some("stop".to_string()),
            logprobs: None,
        }];

        Ok(ChatResponse {
            id: "ollama-response".to_string(),
            object: "chat.completion".to_string(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            model: ollama_response.model,
            choices,
            content,
            reasoning_content: None,
            usage: None, // Ollama does not return token usage information
            system_fingerprint: None,
        })
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|msg| OllamaMessage {
                    role: match msg.role {
                        Role::User => "user".to_string(),
                        Role::Assistant => "assistant".to_string(),
                        Role::System => "system".to_string(),
                        Role::Tool => "user".to_string(),
                    },
                    content: msg.content_as_text(),
                })
                .collect(),
            stream: Some(true),
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens.map(|t| t as i32),
                top_p: request.top_p,
            }),
        };

        let url = format!("{}/api/chat", self.base_url);
        let response = self.client.stream(&url, &ollama_request).await?;
        let status = response.status();

        if !status.is_success() {
            let text = response
                .text()
                .await
                .map_err(|e| LlmConnectorError::NetworkError(e.to_string()))?;
            return Err(LlmConnectorError::ApiError(format!(
                "Ollama stream failed: {}",
                text
            )));
        }

        // Ollama uses JSONL format instead of SSE
        Ok(crate::sse::sse_to_streaming_response(response))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Ollamarequest/responsetype
#[derive(Serialize, Debug)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: Option<bool>,
    options: Option<OllamaOptions>,
}

#[derive(Serialize, Debug)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Debug)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
}

#[derive(Deserialize, Debug)]
struct OllamaChatResponse {
    model: String,
    message: OllamaResponseMessage,
    #[allow(dead_code)]
    done: bool,
}

#[derive(Deserialize, Debug)]
struct OllamaResponseMessage {
    #[allow(dead_code)]
    role: String,
    content: String,
}

#[derive(Serialize, Debug)]
struct OllamaPullRequest {
    name: String,
    stream: Option<bool>,
}

#[derive(Serialize, Debug)]
struct OllamaDeleteRequest {
    name: String,
}

#[derive(Serialize, Debug)]
struct OllamaShowRequest {
    name: String,
}

#[derive(Deserialize, Debug)]
pub struct OllamaModelInfo {
    pub modelfile: String,
    pub parameters: String,
    pub template: String,
    pub details: OllamaModelDetails,
}

#[derive(Deserialize, Debug)]
pub struct OllamaModelDetails {
    pub format: String,
    pub family: String,
    pub families: Option<Vec<String>>,
    pub parameter_size: String,
    pub quantization_level: String,
}

#[derive(Deserialize, Debug)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize, Debug)]
struct OllamaModel {
    name: String,
    #[allow(dead_code)]
    modified_at: String,
    #[allow(dead_code)]
    size: u64,
}

/// Create Ollama service Provider (default local address)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::ollama;
///
/// let provider = ollama().unwrap();
/// ```
pub fn ollama() -> Result<OllamaProvider, LlmConnectorError> {
    OllamaProvider::new("http://localhost:11434")
}

/// CreatewithcustomURLOllamaserviceProvider
///
/// # Parameters
/// - `base_url`: Ollama service URL
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::ollama_with_base_url;
///
/// let provider = ollama_with_base_url("http://192.168.1.100:11434").unwrap();
/// ```
pub fn ollama_with_base_url(base_url: &str) -> Result<OllamaProvider, LlmConnectorError> {
    OllamaProvider::new(base_url)
}

/// CreatewithcustomconfigurationOllamaserviceProvider
///
/// # Parameters
/// - `base_url`: Ollama service URL
/// - `timeout_secs`: Timeout (seconds)
/// - `proxy`: Proxy URL (optional)
///
/// # Example
/// ```rust,no_run
/// use llm_connector::providers::ollama_with_config;
///
/// let provider = ollama_with_config(
///     "http://localhost:11434",
///     Some(120), // 2 minutes timeout
///     None
/// ).unwrap();
/// ```
pub fn ollama_with_config(
    base_url: &str,
    timeout_secs: Option<u64>,
    proxy: Option<&str>,
) -> Result<OllamaProvider, LlmConnectorError> {
    OllamaProvider::with_config(base_url, timeout_secs, proxy)
}
