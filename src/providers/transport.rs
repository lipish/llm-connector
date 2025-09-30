use serde::Serialize;
use reqwest::Client;

use crate::{config::ProviderConfig, error::LlmConnectorError};

#[derive(Clone, Debug)]
pub struct HttpTransport {
    pub client: Client,
    pub config: ProviderConfig,
}

impl HttpTransport {
    pub fn new(client: Client, config: ProviderConfig) -> Self {
        Self { client, config }
    }

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

    pub async fn post<T: Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<reqwest::Response, LlmConnectorError> {
        self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", &self.config.api_key))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(LlmConnectorError::from)
    }

    #[cfg(feature = "streaming")]
    pub async fn stream<T: Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<impl futures_util::Stream<Item = Result<reqwest::Bytes, reqwest::Error>>, LlmConnectorError> {
        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", &self.config.api_key))
            .header("Content-Type", "application/json")
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
