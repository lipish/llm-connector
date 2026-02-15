//! Builder pattern for LlmClient
//!
//! Provides a fluent API for constructing `LlmClient` instances with optional configuration.
//!
//! # Examples
//!
//! ```rust,no_run
//! use llm_connector::LlmClient;
//!
//! // Simple usage
//! let client = LlmClient::builder()
//!     .openai("sk-...")
//!     .build()
//!     .unwrap();
//!
//! // With custom configuration
//! let client = LlmClient::builder()
//!     .openai("sk-...")
//!     .base_url("https://api.deepseek.com")
//!     .timeout(60)
//!     .build()
//!     .unwrap();
//!
//! // DeepSeek shorthand
//! let client = LlmClient::builder()
//!     .deepseek("sk-...")
//!     .timeout(120)
//!     .build()
//!     .unwrap();
//!
//! // Ollama (no API key needed)
//! let client = LlmClient::builder()
//!     .ollama()
//!     .base_url("http://192.168.1.100:11434")
//!     .build()
//!     .unwrap();
//! ```

use crate::error::LlmConnectorError;
use crate::client::LlmClient;

/// Target provider for the builder
#[derive(Debug, Clone)]
enum ProviderKind {
    OpenAI,
    Anthropic,
    Aliyun,
    Zhipu,
    ZhipuOpenAI,
    Ollama,
    DeepSeek,
    Moonshot,
    Volcengine,
    Google,
    Xiaomi,
    LongcatAnthropic,
    OpenAICompatible { service_name: String },
    AzureOpenAI { endpoint: String, api_version: String },
    #[cfg(feature = "tencent")]
    Tencent { secret_key: String },
}

/// Fluent builder for `LlmClient`
///
/// # Example
/// ```rust,no_run
/// use llm_connector::LlmClient;
///
/// let client = LlmClient::builder()
///     .openai("sk-...")
///     .base_url("https://custom-endpoint.com")
///     .timeout(30)
///     .build()
///     .unwrap();
/// ```
pub struct LlmClientBuilder {
    provider: Option<ProviderKind>,
    api_key: Option<String>,
    base_url: Option<String>,
    timeout_secs: Option<u64>,
    proxy: Option<String>,
}

impl LlmClientBuilder {
    pub(crate) fn new() -> Self {
        Self {
            provider: None,
            api_key: None,
            base_url: None,
            timeout_secs: None,
            proxy: None,
        }
    }

    // ========================================================================
    // Provider selection methods
    // ========================================================================

    /// Use OpenAI as the provider
    pub fn openai(mut self, api_key: &str) -> Self {
        self.provider = Some(ProviderKind::OpenAI);
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Use Anthropic as the provider
    pub fn anthropic(mut self, api_key: &str) -> Self {
        self.provider = Some(ProviderKind::Anthropic);
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Use Aliyun DashScope as the provider
    pub fn aliyun(mut self, api_key: &str) -> Self {
        self.provider = Some(ProviderKind::Aliyun);
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Use Zhipu GLM as the provider
    pub fn zhipu(mut self, api_key: &str) -> Self {
        self.provider = Some(ProviderKind::Zhipu);
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Use Zhipu GLM (OpenAI compatible mode) as the provider
    pub fn zhipu_openai(mut self, api_key: &str) -> Self {
        self.provider = Some(ProviderKind::ZhipuOpenAI);
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Use Ollama as the provider (no API key needed)
    pub fn ollama(mut self) -> Self {
        self.provider = Some(ProviderKind::Ollama);
        self
    }

    /// Use DeepSeek as the provider
    pub fn deepseek(mut self, api_key: &str) -> Self {
        self.provider = Some(ProviderKind::DeepSeek);
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Use Moonshot as the provider
    pub fn moonshot(mut self, api_key: &str) -> Self {
        self.provider = Some(ProviderKind::Moonshot);
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Use Volcengine as the provider
    pub fn volcengine(mut self, api_key: &str) -> Self {
        self.provider = Some(ProviderKind::Volcengine);
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Use Google as the provider
    pub fn google(mut self, api_key: &str) -> Self {
        self.provider = Some(ProviderKind::Google);
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Use Xiaomi MiMo as the provider
    pub fn xiaomi(mut self, api_key: &str) -> Self {
        self.provider = Some(ProviderKind::Xiaomi);
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Use LongCat Anthropic as the provider
    pub fn longcat_anthropic(mut self, api_key: &str) -> Self {
        self.provider = Some(ProviderKind::LongcatAnthropic);
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Use any OpenAI-compatible service as the provider
    ///
    /// # Parameters
    /// - `api_key`: API key
    /// - `service_name`: Name of the service (for logging/identification)
    pub fn openai_compatible(mut self, api_key: &str, service_name: &str) -> Self {
        self.provider = Some(ProviderKind::OpenAICompatible {
            service_name: service_name.to_string(),
        });
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Use Azure OpenAI as the provider
    ///
    /// # Parameters
    /// - `api_key`: Azure OpenAI API key
    /// - `endpoint`: Azure endpoint URL
    /// - `api_version`: API version string
    pub fn azure_openai(mut self, api_key: &str, endpoint: &str, api_version: &str) -> Self {
        self.provider = Some(ProviderKind::AzureOpenAI {
            endpoint: endpoint.to_string(),
            api_version: api_version.to_string(),
        });
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Use Tencent Hunyuan as the provider
    ///
    /// # Parameters
    /// - `secret_id`: Tencent Cloud SecretID
    /// - `secret_key`: Tencent Cloud SecretKey
    #[cfg(feature = "tencent")]
    pub fn tencent(mut self, secret_id: &str, secret_key: &str) -> Self {
        self.provider = Some(ProviderKind::Tencent {
            secret_key: secret_key.to_string(),
        });
        self.api_key = Some(secret_id.to_string());
        self
    }

    // ========================================================================
    // Configuration methods
    // ========================================================================

    /// Set custom base URL
    pub fn base_url(mut self, url: &str) -> Self {
        self.base_url = Some(url.to_string());
        self
    }

    /// Set request timeout in seconds
    pub fn timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    /// Set HTTP proxy
    pub fn proxy(mut self, proxy_url: &str) -> Self {
        self.proxy = Some(proxy_url.to_string());
        self
    }

    // ========================================================================
    // Build
    // ========================================================================

    /// Build the `LlmClient`
    pub fn build(self) -> Result<LlmClient, LlmConnectorError> {
        let provider = self.provider.ok_or_else(|| {
            LlmConnectorError::InvalidRequest("No provider specified. Call .openai(), .deepseek(), etc. before .build()".into())
        })?;

        let api_key = self.api_key.as_deref().unwrap_or("");
        let base_url = self.base_url.as_deref();
        let timeout = self.timeout_secs;
        let proxy = self.proxy.as_deref();

        let has_config = base_url.is_some() || timeout.is_some() || proxy.is_some();

        match provider {
            ProviderKind::OpenAI => {
                if has_config {
                    LlmClient::openai_with_config(api_key, base_url, timeout, proxy)
                } else {
                    LlmClient::openai(api_key)
                }
            }
            ProviderKind::Anthropic => {
                if has_config {
                    LlmClient::anthropic_with_config(api_key, base_url, timeout, proxy)
                } else {
                    LlmClient::anthropic(api_key)
                }
            }
            ProviderKind::Aliyun => {
                if has_config {
                    LlmClient::aliyun_with_config(api_key, base_url, timeout, proxy)
                } else {
                    LlmClient::aliyun(api_key)
                }
            }
            ProviderKind::Zhipu => {
                if has_config {
                    LlmClient::zhipu_with_config(api_key, false, base_url, timeout, proxy)
                } else {
                    LlmClient::zhipu(api_key)
                }
            }
            ProviderKind::ZhipuOpenAI => {
                if has_config {
                    LlmClient::zhipu_with_config(api_key, true, base_url, timeout, proxy)
                } else {
                    LlmClient::zhipu_openai_compatible(api_key)
                }
            }
            ProviderKind::Ollama => {
                if has_config {
                    LlmClient::ollama_with_config(
                        base_url.unwrap_or("http://localhost:11434"),
                        timeout,
                        proxy,
                    )
                } else {
                    LlmClient::ollama()
                }
            }
            ProviderKind::DeepSeek => {
                if has_config {
                    LlmClient::deepseek_with_config(api_key, base_url, timeout, proxy)
                } else {
                    LlmClient::deepseek(api_key)
                }
            }
            ProviderKind::Moonshot => {
                if has_config {
                    LlmClient::moonshot_with_config(api_key, base_url, timeout, proxy)
                } else {
                    LlmClient::moonshot(api_key)
                }
            }
            ProviderKind::Volcengine => {
                if has_config {
                    LlmClient::volcengine_with_config(api_key, base_url, timeout, proxy)
                } else {
                    LlmClient::volcengine(api_key)
                }
            }
            ProviderKind::Google => {
                if has_config {
                    LlmClient::google_with_config(api_key, base_url, timeout, proxy)
                } else {
                    LlmClient::google(api_key)
                }
            }
            ProviderKind::Xiaomi => {
                if has_config {
                    LlmClient::xiaomi_with_config(api_key, base_url, timeout, proxy)
                } else {
                    LlmClient::xiaomi(api_key)
                }
            }
            ProviderKind::LongcatAnthropic => {
                if has_config {
                    LlmClient::longcat_anthropic_with_config(api_key, base_url, timeout, proxy)
                } else {
                    LlmClient::longcat_anthropic(api_key)
                }
            }
            ProviderKind::OpenAICompatible { service_name } => {
                let url = base_url.ok_or_else(|| {
                    LlmConnectorError::InvalidRequest(
                        "OpenAI-compatible provider requires .base_url()".into(),
                    )
                })?;
                LlmClient::openai_compatible(api_key, url, &service_name)
            }
            ProviderKind::AzureOpenAI { endpoint, api_version } => {
                LlmClient::azure_openai(api_key, &endpoint, &api_version)
            }
            #[cfg(feature = "tencent")]
            ProviderKind::Tencent { secret_key } => {
                if has_config {
                    LlmClient::tencent_with_config(api_key, &secret_key, base_url, timeout, proxy)
                } else {
                    LlmClient::tencent(api_key, &secret_key)
                }
            }
        }
    }
}
