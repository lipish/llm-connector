//! Ollama服务提供商实现 - V2架构
//!
//! Ollama是一个本地LLM服务，具有特殊的模型管理功能，因此需要自定义Provider实现。

use crate::core::{HttpClient, Provider};
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse, Choice, Message, Role};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// Ollama服务提供商
///
/// 由于Ollama具有特殊的模型管理功能，我们使用自定义Provider实现
/// 而不是GenericProvider模式。
#[derive(Clone, Debug)]
pub struct OllamaProvider {
    client: HttpClient,
    base_url: String,
}

impl OllamaProvider {
    /// 创建新的Ollama提供商
    ///
    /// # 参数
    /// - `base_url`: Ollama服务的URL (默认: http://localhost:11434)
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::providers::OllamaProvider;
    ///
    /// let provider = OllamaProvider::new("http://localhost:11434").unwrap();
    /// ```
    pub fn new(base_url: &str) -> Result<Self, LlmConnectorError> {
        // Content-Type 由 HttpClient::post() 的 .json() 方法自动设置
        let client = HttpClient::new(base_url)?;

        Ok(Self {
            client,
            base_url: base_url.to_string(),
        })
    }

    /// 创建带有自定义配置的Ollama提供商
    pub fn with_config(
        base_url: &str,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        // Content-Type 由 HttpClient::post() 的 .json() 方法自动设置
        let client = HttpClient::with_config(base_url, timeout_secs, proxy)?;

        Ok(Self {
            client,
            base_url: base_url.to_string(),
        })
    }

    /// 拉取模型
    ///
    /// # 参数
    /// - `model_name`: 要拉取的模型名称 (如 "llama2", "codellama")
    ///
    /// # 示例
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

    /// 删除模型
    ///
    /// # 参数
    /// - `model_name`: 要删除的模型名称
    ///
    /// # 示例
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

    /// 获取模型信息
    ///
    /// # 参数
    /// - `model_name`: 模型名称
    ///
    /// # 返回
    /// 模型的详细信息
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

    /// 检查模型是否存在
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
                        Role::Tool => "user".to_string(), // Ollama不支持tool角色
                    },
                    content: msg.content.clone(),
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
                content: content.clone(),
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
            usage: None, // Ollama不返回token使用信息
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
                    content: msg.content.clone(),
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

        // Ollama使用JSONL格式而不是SSE
        Ok(crate::sse::sse_to_streaming_response(response))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Ollama请求/响应类型
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

/// 创建Ollama服务提供商 (默认本地地址)
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::ollama;
///
/// let provider = ollama().unwrap();
/// ```
pub fn ollama() -> Result<OllamaProvider, LlmConnectorError> {
    OllamaProvider::new("http://localhost:11434")
}

/// 创建带有自定义URL的Ollama服务提供商
///
/// # 参数
/// - `base_url`: Ollama服务的URL
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::ollama_with_base_url;
///
/// let provider = ollama_with_base_url("http://192.168.1.100:11434").unwrap();
/// ```
pub fn ollama_with_base_url(base_url: &str) -> Result<OllamaProvider, LlmConnectorError> {
    OllamaProvider::new(base_url)
}

/// 创建带有自定义配置的Ollama服务提供商
///
/// # 参数
/// - `base_url`: Ollama服务的URL
/// - `timeout_secs`: 超时时间(秒)
/// - `proxy`: 代理URL (可选)
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::providers::ollama_with_config;
///
/// let provider = ollama_with_config(
///     "http://localhost:11434",
///     Some(120), // 2分钟超时
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
