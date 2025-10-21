//! 可配置的协议适配器 - 配置驱动的抽象
//!
//! 这个模块提供了一个通用的协议适配器，通过配置来定制行为，
//! 避免为每个 Provider 编写重复的样板代码。

use crate::core::Protocol;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse};
use async_trait::async_trait;
use std::sync::Arc;

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// 可配置的协议适配器
///
/// 包装一个基础协议，通过配置来修改其行为（端点路径、认证方式等）。
///
/// # 示例
/// ```rust,no_run
/// use llm_connector::core::{ConfigurableProtocol, ProtocolConfig, EndpointConfig, AuthConfig};
/// use llm_connector::protocols::OpenAIProtocol;
///
/// let config = ProtocolConfig {
///     name: "custom".to_string(),
///     endpoints: EndpointConfig {
///         chat_template: "{base_url}/v1/chat/completions".to_string(),
///         models_template: Some("{base_url}/v1/models".to_string()),
///     },
///     auth: AuthConfig::Bearer,
///     extra_headers: vec![],
/// };
///
/// let protocol = ConfigurableProtocol::new(
///     OpenAIProtocol::new("sk-..."),
///     config
/// );
/// ```
#[derive(Clone)]
pub struct ConfigurableProtocol<P: Protocol> {
    inner: P,
    config: ProtocolConfig,
}

/// 协议配置
///
/// 定义协议的静态配置，包括名称、端点、认证方式等。
#[derive(Clone, Debug)]
pub struct ProtocolConfig {
    /// 协议名称
    pub name: String,

    /// 端点配置
    pub endpoints: EndpointConfig,

    /// 认证配置
    pub auth: AuthConfig,

    /// 额外的静态头部
    pub extra_headers: Vec<(String, String)>,
}

/// 端点配置
///
/// 定义 API 端点的路径模板，支持 `{base_url}` 变量替换。
#[derive(Clone, Debug)]
pub struct EndpointConfig {
    /// 聊天端点模板
    ///
    /// 支持变量: `{base_url}`
    ///
    /// 例如: `"{base_url}/v1/chat/completions"`
    pub chat_template: String,

    /// 模型列表端点模板（可选）
    ///
    /// 例如: `"{base_url}/v1/models"`
    pub models_template: Option<String>,
}

/// 认证配置
///
/// 定义如何处理 API 认证。
#[derive(Clone)]
pub enum AuthConfig {
    /// Bearer token 认证
    ///
    /// 生成: `Authorization: Bearer {token}`
    Bearer,

    /// API Key header 认证
    ///
    /// 生成: `{header_name}: {token}`
    ApiKeyHeader {
        /// Header 名称
        header_name: String,
    },

    /// 无认证
    None,

    /// 自定义认证（通过闭包）
    ///
    /// 闭包接收 token，返回头部列表
    Custom(Arc<dyn Fn(&str) -> Vec<(String, String)> + Send + Sync>),
}

impl std::fmt::Debug for AuthConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthConfig::Bearer => write!(f, "Bearer"),
            AuthConfig::ApiKeyHeader { header_name } => {
                write!(f, "ApiKeyHeader({})", header_name)
            }
            AuthConfig::None => write!(f, "None"),
            AuthConfig::Custom(_) => write!(f, "Custom(...)"),
        }
    }
}

impl<P: Protocol> ConfigurableProtocol<P> {
    /// 创建新的可配置协议适配器
    ///
    /// # 参数
    /// - `inner`: 基础协议实例
    /// - `config`: 协议配置
    pub fn new(inner: P, config: ProtocolConfig) -> Self {
        Self { inner, config }
    }

    /// 便捷构造器 - OpenAI 兼容协议
    ///
    /// 创建一个使用标准 OpenAI 端点和 Bearer 认证的配置。
    ///
    /// # 参数
    /// - `inner`: 基础协议实例
    /// - `name`: 协议名称
    ///
    /// # 示例
    /// ```rust,no_run
    /// use llm_connector::core::ConfigurableProtocol;
    /// use llm_connector::protocols::OpenAIProtocol;
    ///
    /// let protocol = ConfigurableProtocol::openai_compatible(
    ///     OpenAIProtocol::new("sk-..."),
    ///     "custom-openai"
    /// );
    /// ```
    pub fn openai_compatible(inner: P, name: &str) -> Self {
        Self::new(
            inner,
            ProtocolConfig {
                name: name.to_string(),
                endpoints: EndpointConfig {
                    chat_template: "{base_url}/v1/chat/completions".to_string(),
                    models_template: Some("{base_url}/v1/models".to_string()),
                },
                auth: AuthConfig::Bearer,
                extra_headers: vec![],
            },
        )
    }

    /// 从内部协议提取 token
    ///
    /// 这是一个辅助方法，用于从内部协议的认证头中提取 token。
    fn extract_token_from_inner(&self) -> String {
        let headers = self.inner.auth_headers();
        for (key, value) in headers {
            if key.to_lowercase() == "authorization" {
                // 提取 "Bearer xxx" 或 "xxx"
                if let Some(token) = value.strip_prefix("Bearer ") {
                    return token.to_string();
                }
                return value;
            } else if key.to_lowercase() == "x-api-key" {
                return value;
            }
        }
        // 如果找不到，返回空字符串
        String::new()
    }
}

#[async_trait]
impl<P: Protocol> Protocol for ConfigurableProtocol<P> {
    type Request = P::Request;
    type Response = P::Response;

    fn name(&self) -> &str {
        &self.config.name
    }

    fn chat_endpoint(&self, base_url: &str) -> String {
        self.config
            .endpoints
            .chat_template
            .replace("{base_url}", base_url.trim_end_matches('/'))
    }

    fn models_endpoint(&self, base_url: &str) -> Option<String> {
        self.config
            .endpoints
            .models_template
            .as_ref()
            .map(|template| template.replace("{base_url}", base_url.trim_end_matches('/')))
    }

    fn build_request(
        &self,
        request: &ChatRequest,
    ) -> Result<Self::Request, LlmConnectorError> {
        self.inner.build_request(request)
    }

    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        self.inner.parse_response(response)
    }

    fn parse_models(&self, response: &str) -> Result<Vec<String>, LlmConnectorError> {
        self.inner.parse_models(response)
    }

    fn map_error(&self, status: u16, body: &str) -> LlmConnectorError {
        self.inner.map_error(status, body)
    }

    fn auth_headers(&self) -> Vec<(String, String)> {
        let mut headers = match &self.config.auth {
            AuthConfig::Bearer => {
                // 从 inner protocol 获取 token 并转换为 Bearer 格式
                let token = self.extract_token_from_inner();
                if token.is_empty() {
                    vec![]
                } else {
                    vec![("Authorization".to_string(), format!("Bearer {}", token))]
                }
            }
            AuthConfig::ApiKeyHeader { header_name } => {
                // 从 inner protocol 获取 token，使用自定义 header 名称
                let token = self.extract_token_from_inner();
                if token.is_empty() {
                    vec![]
                } else {
                    vec![(header_name.clone(), token)]
                }
            }
            AuthConfig::None => vec![],
            AuthConfig::Custom(f) => {
                let token = self.extract_token_from_inner();
                f(&token)
            }
        };

        // 添加额外的静态头部
        headers.extend(self.config.extra_headers.clone());
        headers
    }

    #[cfg(feature = "streaming")]
    async fn parse_stream_response(
        &self,
        response: reqwest::Response,
    ) -> Result<ChatStream, LlmConnectorError> {
        self.inner.parse_stream_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::OpenAIProtocol;

    #[test]
    fn test_configurable_protocol_basic() {
        let config = ProtocolConfig {
            name: "test".to_string(),
            endpoints: EndpointConfig {
                chat_template: "{base_url}/v1/chat/completions".to_string(),
                models_template: Some("{base_url}/v1/models".to_string()),
            },
            auth: AuthConfig::Bearer,
            extra_headers: vec![],
        };

        let protocol = ConfigurableProtocol::new(OpenAIProtocol::new("sk-test"), config);

        assert_eq!(protocol.name(), "test");
        assert_eq!(
            protocol.chat_endpoint("https://api.example.com"),
            "https://api.example.com/v1/chat/completions"
        );
        assert_eq!(
            protocol.models_endpoint("https://api.example.com"),
            Some("https://api.example.com/v1/models".to_string())
        );
    }

    #[test]
    fn test_openai_compatible() {
        let protocol =
            ConfigurableProtocol::openai_compatible(OpenAIProtocol::new("sk-test"), "custom");

        assert_eq!(protocol.name(), "custom");
        assert_eq!(
            protocol.chat_endpoint("https://api.example.com"),
            "https://api.example.com/v1/chat/completions"
        );
    }

    #[test]
    fn test_custom_endpoint() {
        let config = ProtocolConfig {
            name: "volcengine".to_string(),
            endpoints: EndpointConfig {
                chat_template: "{base_url}/api/v3/chat/completions".to_string(),
                models_template: Some("{base_url}/api/v3/models".to_string()),
            },
            auth: AuthConfig::Bearer,
            extra_headers: vec![],
        };

        let protocol = ConfigurableProtocol::new(OpenAIProtocol::new("sk-test"), config);

        assert_eq!(
            protocol.chat_endpoint("https://api.example.com"),
            "https://api.example.com/api/v3/chat/completions"
        );
    }

    #[test]
    fn test_extra_headers() {
        let config = ProtocolConfig {
            name: "test".to_string(),
            endpoints: EndpointConfig {
                chat_template: "{base_url}/v1/chat/completions".to_string(),
                models_template: None,
            },
            auth: AuthConfig::Bearer,
            extra_headers: vec![
                ("X-Custom-Header".to_string(), "value".to_string()),
                ("X-Another-Header".to_string(), "value2".to_string()),
            ],
        };

        let protocol = ConfigurableProtocol::new(OpenAIProtocol::new("sk-test"), config);
        let headers = protocol.auth_headers();

        assert!(headers
            .iter()
            .any(|(k, v)| k == "X-Custom-Header" && v == "value"));
        assert!(headers
            .iter()
            .any(|(k, v)| k == "X-Another-Header" && v == "value2"));
    }
}

