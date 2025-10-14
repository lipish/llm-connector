//! HTTP transport layer for making requests to provider APIs
//!
//! Provides a shared HTTP client with configuration support.
//! Uses Arc for efficient sharing across providers.

use reqwest::Client;
use serde::Serialize;
use std::sync::Arc;

use crate::config::{ProviderConfig, SharedProviderConfig};
use crate::error::LlmConnectorError;

/// HTTP transport layer for making requests to provider APIs
///
/// Uses Arc for efficient sharing of client and config across clones.
#[derive(Clone, Debug)]
pub struct HttpTransport {
    pub client: Arc<Client>,
    pub config: SharedProviderConfig,
}

impl HttpTransport {
    /// Create new HTTP transport with client and config
    pub fn new(client: Client, config: ProviderConfig) -> Self {
        Self {
            client: Arc::new(client),
            config: SharedProviderConfig::new(config),
        }
    }

    /// Create from shared components (zero-copy)
    pub fn from_shared(client: Arc<Client>, config: SharedProviderConfig) -> Self {
        Self { client, config }
    }

    /// Build HTTP client with proxy and timeout configuration
    pub fn build_client(
        proxy: &Option<String>,
        timeout_ms: Option<u64>,
        base_url: Option<&String>,
    ) -> Result<Client, LlmConnectorError> {
        let mut client_builder = Client::builder();

        if let Some(proxy) = proxy {
            client_builder = client_builder.proxy(reqwest::Proxy::all(proxy)?);
        }

        if let Some(timeout) = timeout_ms {
            client_builder = client_builder.timeout(std::time::Duration::from_millis(timeout));
        }

        // If the base_url points to localhost, disable proxy to avoid 502 from system proxies
        if let Some(base) = base_url {
            if let Ok(url) = reqwest::Url::parse(base) {
                if matches!(url.host_str(), Some("localhost") | Some("127.0.0.1")) {
                    client_builder = client_builder.no_proxy();
                }
            }
        }

        client_builder
            .build()
            .map_err(|e| LlmConnectorError::ConfigError(e.to_string()))
    }

    /// Send GET request
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, LlmConnectorError> {
        let mut request = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", &self.config.api_key));

        // Apply custom headers if configured
        if let Some(headers) = &self.config.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        request
            .send()
            .await
            .map_err(LlmConnectorError::from)
    }

    /// Send POST request with JSON body
    pub async fn post<T: Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<reqwest::Response, LlmConnectorError> {
        let mut request = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", &self.config.api_key))
            .header("Content-Type", "application/json");

        // Apply custom headers if configured
        if let Some(headers) = &self.config.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        request
            .json(body)
            .send()
            .await
            .map_err(LlmConnectorError::from)
    }

    /// Send streaming POST request
    #[cfg(feature = "streaming")]
    pub async fn stream<T: Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<
        impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>>,
        LlmConnectorError,
    > {
        let mut request = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", &self.config.api_key))
            .header("Content-Type", "application/json");

        // Apply custom headers if configured
        if let Some(headers) = &self.config.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        let response = request
            .json(body)
            .send()
            .await
            .map_err(LlmConnectorError::from)?;

        if !response.status().is_success() {
            return Err(LlmConnectorError::ProviderError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        Ok(response.bytes_stream())
    }
}