use crate::types::{ChatRequest, ChatResponse};
#[cfg(feature = "streaming")]
use crate::types::StreamingResponse;
use super::errors::ErrorMapper;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[async_trait]
pub trait ProviderAdapter: Send + Sync + Clone + 'static {
    type RequestType: Serialize + Send + Sync;
    type ResponseType: DeserializeOwned + Send + Sync;
    type StreamResponseType: DeserializeOwned + Send + Sync;
    type ErrorMapperType: ErrorMapper;

    fn name(&self) -> &str;
    fn supported_models(&self) -> Vec<String>;
    fn endpoint_url(&self, base_url: &Option<String>) -> String;

    fn build_request_data(&self, request: &ChatRequest, stream: bool) -> Self::RequestType;

    fn parse_response_data(&self, response: Self::ResponseType) -> ChatResponse;

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> StreamingResponse;
}
