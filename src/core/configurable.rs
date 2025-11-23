//! Configurable Protocol Adapter - Configuration-driven abstraction
//!
//! This module provides a generic protocol adapter that customizes behavior through configuration,
//! 避免as每个 Provider 编写重复样板代码。

use crate::core::Protocol;
use crate::error::LlmConnectorError;
use crate::types::{ChatRequest, ChatResponse};
use async_trait::async_trait;
use std::sync::Arc;

#[cfg(feature = "streaming")]
use crate::types::ChatStream;

/// Configurable Protocol Adapter
///
/// Wraps a base protocol and modifies its behavior through configuration (endpoint paths, authentication methods, etc.).
///
/// # Example
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

/// Protocol Configuration
///
/// Defines static configuration for the protocol, including name, endpoints, authentication methods, etc.
#[derive(Clone, Debug)]
pub struct ProtocolConfig {
    /// Protocol name
    pub name: String,

    /// Endpoint Configuration
    pub endpoints: EndpointConfig,

    /// Authentication Configuration
    pub auth: AuthConfig,

    /// 额外静态headers
    pub extra_headers: Vec<(String, String)>,
}

/// Endpoint Configuration
///
/// Defines API endpoint path templates, supporting `{base_url}` variable substitution.
#[derive(Clone, Debug)]
pub struct EndpointConfig {
    /// 聊天endpoint模板
    ///
    /// Supports variable: `{base_url}`
    ///
    /// 例such as: `"{base_url}/v1/chat/completions"`
    pub chat_template: String,

    /// Model list endpoint template (optional)
    ///
    /// 例such as: `"{base_url}/v1/models"`
    pub models_template: Option<String>,
}

/// Authentication Configuration
///
/// Defines how to handle API authentication.
#[derive(Clone)]
pub enum AuthConfig {
    /// Bearer token authentication
    ///
    /// 生成: `Authorization: Bearer {token}`
    Bearer,

    /// API Key header authentication
    ///
    /// 生成: `{header_name}: {token}`
    ApiKeyHeader {
        /// Header 名称
        header_name: String,
    },

    /// 无authentication
    None,

    /// customauthentication（Through闭包）
    ///
    /// 闭包接收 token，Returnsheaders列表
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
    /// Create new configurable protocol adapter
    ///
    /// # Parameters
    /// - `inner`: Base protocol instance
    /// - `config`: Protocol Configuration
    pub fn new(inner: P, config: ProtocolConfig) -> Self {
        Self { inner, config }
    }

    /// Convenience constructor - OpenAI compatible protocol
    ///
    /// Create a configuration using standard OpenAI endpoints and Bearer authentication.
    ///
    /// # Parameters
    /// - `inner`: Base protocol instance
    /// - `name`: Protocol name
    ///
    /// # Example
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

    /// Extract token from internal protocol
    ///
    /// This is a helper method to extract token from the internal protocol's authentication headers.
    fn extract_token_from_inner(&self) -> String {
        let headers = self.inner.auth_headers();
        for (key, value) in headers {
            if key.to_lowercase() == "authorization" {
                // 提取 "Bearer xxx" or "xxx"
                if let Some(token) = value.strip_prefix("Bearer ") {
                    return token.to_string();
                }
                return value;
            } else if key.to_lowercase() == "x-api-key" {
                return value;
            }
        }
        // if找不to，Returns空字符串
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
                // from inner protocol Get token 并Convertas Bearer 格式
                let token = self.extract_token_from_inner();
                if token.is_empty() {
                    vec![]
                } else {
                    vec![("Authorization".to_string(), format!("Bearer {}", token))]
                }
            }
            AuthConfig::ApiKeyHeader { header_name } => {
                // from inner protocol Get token，Usecustom header 名称
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

        // 添加额外静态headers
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

