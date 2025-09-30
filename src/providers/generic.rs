use crate::config::ProviderConfig;
use crate::error::LlmConnectorError;
use crate::providers::base::Provider;
use crate::providers::errors::ErrorMapper;
use crate::providers::traits::ProviderAdapter;
use crate::providers::transport::HttpTransport;
use crate::types::{ChatRequest, ChatResponse};
use async_trait::async_trait;
#[cfg(feature = "streaming")]
use {
    crate::types::ChatStream,
    crate::utils::streaming::sse_data_events,
    futures_util::StreamExt,
};

#[derive(Clone, Debug)]
pub struct GenericProvider<T: ProviderAdapter> {
    pub transport: HttpTransport,
    adapter: T,
}

impl<T: ProviderAdapter> GenericProvider<T> {
    pub fn new(config: ProviderConfig, adapter: T) -> Result<Self, LlmConnectorError> {
        let client = HttpTransport::build_client(
            &config.proxy,
            config.timeout_ms,
            config.base_url.as_ref(),
        )?;
        let transport = HttpTransport::new(client, config);
        Ok(Self { transport, adapter })
    }

    pub fn name(&self) -> &str {
        self.adapter.name()
    }

    pub fn supported_models(&self) -> Vec<String> {
        self.adapter.supported_models()
    }

    pub fn supports_model(&self, model: &str) -> bool {
        self.supported_models().contains(&model.to_string())
    }

    pub async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        let url = self.adapter.endpoint_url(&self.transport.config.base_url);
        let request_data = self.adapter.build_request_data(request, false);

        let response = self.transport.post(&url, &request_data).await?;

        if response.status().is_success() {
            let response_data: T::ResponseType = response.json().await.map_err(LlmConnectorError::HttpError)?;
            Ok(self.adapter.parse_response_data(response_data))
        } else {
            let status = response.status().as_u16();
            let response_data: serde_json::Value = response.json().await.map_err(LlmConnectorError::HttpError)?;
            Err(T::ErrorMapperType::map_http_error(status, response_data))
        }
    }

    #[cfg(feature = "streaming")]
    pub async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        let url = self.adapter.endpoint_url(&self.transport.config.base_url);
        let request_data = self.adapter.build_request_data(request, true);

        let response = self.transport.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", &self.transport.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_data)
            .send()
            .await
            .map_err(LlmConnectorError::from)?;

        if !response.status().is_success() {
            return Err(LlmConnectorError::ProviderError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let mapped_stream = sse_data_events(response).filter_map(|event| async move {
            match event {
                Ok(data) => {
                    if data.as_str() == "[DONE]" {
                        None
                    } else {
                        match serde_json::from_str::<T::StreamResponseType>(&data) {
                            Ok(response) => Some(Ok(self.adapter.parse_stream_response_data(response))),
                            Err(e) => Some(Err(LlmConnectorError::JsonError(e))),
                        }
                    }
                }
                Err(e) => Some(Err(LlmConnectorError::StreamingError(e.to_string()))),
            }
        });

        Ok(Box::pin(mapped_stream))
    }
}

// Implement Provider trait for GenericProvider
#[async_trait]
impl<T: ProviderAdapter> Provider for GenericProvider<T> {
    fn name(&self) -> &str {
        self.adapter.name()
    }

    fn supported_models(&self) -> Vec<String> {
        self.adapter.supported_models()
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        self.chat(request).await
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        self.chat_stream(request).await
    }
}