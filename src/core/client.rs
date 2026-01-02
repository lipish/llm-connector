//! HTTP Client Implementation - V2 Architecture
//!
//! Provides unified HTTP communication layer, supporting standard and streaming requests.

use crate::error::LlmConnectorError;
use reqwest::Client;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

/// HTTP Client
///
/// Encapsulates all HTTP communication details, including authentication, timeout, proxy configuration, etc.
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    base_url: String,
    headers: HashMap<String, String>,
}

impl HttpClient {
    /// Create new HTTP client
    ///
    /// Default timeout: 60 seconds (suitable for most requests including streaming)
    ///
    /// **Important**: System proxy is **disabled** by default to avoid unexpected timeout issues.
    /// If you need to use a proxy, use `with_config()` and explicitly set the proxy parameter.
    pub fn new(base_url: &str) -> Result<Self, LlmConnectorError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60)) // Increased from 30 to 60 seconds
            .no_proxy() // Disable system proxy by default to avoid timeout issues
            .build()
            .map_err(|e| {
                LlmConnectorError::ConfigError(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            headers: HashMap::new(),
        })
    }

    /// Create HTTP client with custom configuration
    ///
    /// # Parameters
    /// - `base_url`: Base URL for the API
    /// - `timeout_secs`: Optional timeout in seconds (default: 60 seconds)
    /// - `proxy`: Optional proxy URL
    ///
    /// # Proxy Behavior
    /// - If `proxy` is `None`: System proxy is **disabled** (no proxy used)
    /// - If `proxy` is `Some(url)`: The specified proxy is used for all protocols (HTTP/HTTPS)
    ///
    /// **Note**: System proxy is disabled by default to avoid unexpected timeout issues.
    /// This is different from reqwest's default behavior which enables system proxy.
    pub fn with_config(
        base_url: &str,
        timeout_secs: Option<u64>,
        proxy: Option<&str>,
    ) -> Result<Self, LlmConnectorError> {
        let mut builder = Client::builder();

        // Set timeout (default 60 seconds for streaming compatibility)
        if let Some(timeout) = timeout_secs {
            builder = builder.timeout(Duration::from_secs(timeout));
        } else {
            builder = builder.timeout(Duration::from_secs(60)); // Increased from 30 to 60 seconds
        }

        // Set proxy or disable system proxy
        if let Some(proxy_url) = proxy {
            // Use explicit proxy
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| LlmConnectorError::ConfigError(format!("Invalid proxy URL: {}", e)))?;
            builder = builder.proxy(proxy);
        } else {
            // Disable system proxy to avoid timeout issues
            builder = builder.no_proxy();
        }

        let client = builder.build().map_err(|e| {
            LlmConnectorError::ConfigError(format!("Failed to create HTTP client: {}", e))
        })?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            headers: HashMap::new(),
        })
    }

    /// Add request headers
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers.extend(headers);
        self
    }

    /// Add single request header
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Get base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Send GET request
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, LlmConnectorError> {
        let mut request = self.client.get(url);

        // Add all configured request headers
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        request.send().await.map_err(|e| {
            if e.is_timeout() {
                LlmConnectorError::TimeoutError(format!("GET request timeout: {}", e))
            } else if e.is_connect() {
                LlmConnectorError::ConnectionError(format!("GET connection failed: {}", e))
            } else {
                LlmConnectorError::NetworkError(format!("GET request failed: {}", e))
            }
        })
    }

    /// Send POST request
    pub async fn post<T: Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<reqwest::Response, LlmConnectorError> {
        let mut request = self.client.post(url).json(body);

        // Add all configured request headers
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        request.send().await.map_err(|e| {
            if e.is_timeout() {
                LlmConnectorError::TimeoutError(format!("POST request timeout: {}", e))
            } else if e.is_connect() {
                LlmConnectorError::ConnectionError(format!("POST connection failed: {}", e))
            } else {
                LlmConnectorError::NetworkError(format!("POST request failed: {}", e))
            }
        })
    }

    /// Send streaming POST request
    ///
    /// Note: Streaming requests use the same timeout as configured in the client.
    /// For long-running streams, consider using `with_config()` to set a longer timeout.
    ///
    /// Recommended timeout for streaming: 60-300 seconds depending on expected response length.
    #[cfg(feature = "streaming")]
    pub async fn stream<T: Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<reqwest::Response, LlmConnectorError> {
        let mut request = self.client.post(url).json(body);

        // Add streaming-specific headers
        request = request.header("Accept", "text/event-stream");
        request = request.header("Cache-Control", "no-cache");
        request = request.header("Connection", "keep-alive");

        // Add all configured request headers
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        request.send().await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmConnectorError::TimeoutError(format!("Stream request timeout: {}. Consider increasing timeout for long-running streams.", e))
                } else if e.is_connect() {
                    LlmConnectorError::ConnectionError(format!("Stream connection failed: {}", e))
                } else {
                    LlmConnectorError::NetworkError(format!("Stream request failed: {}", e))
                }
            })
    }

    /// Send POST request with custom headers
    pub async fn post_with_custom_headers<T: Serialize>(
        &self,
        url: &str,
        body: &T,
        custom_headers: &HashMap<String, String>,
    ) -> Result<reqwest::Response, LlmConnectorError> {
        let mut request = self.client.post(url).json(body);

        // Add custom headers first
        for (key, value) in custom_headers {
            request = request.header(key, value);
        }

        // Then add configured headers (may override custom headers)
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        request.send().await.map_err(|e| {
            if e.is_timeout() {
                LlmConnectorError::TimeoutError(format!("POST request timeout: {}", e))
            } else if e.is_connect() {
                LlmConnectorError::ConnectionError(format!("POST connection failed: {}", e))
            } else {
                LlmConnectorError::NetworkError(format!("POST request failed: {}", e))
            }
        })
    }
}

impl std::fmt::Debug for HttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClient")
            .field("base_url", &self.base_url)
            .field("headers_count", &self.headers.len())
            .finish()
    }
}
