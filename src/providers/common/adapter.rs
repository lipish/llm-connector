use crate::types::{ChatRequest, ChatResponse};
#[cfg(feature = "streaming")]
use crate::types::StreamingResponse;
use super::error_mapper::ErrorMapper;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[async_trait]
pub trait ProviderAdapter: Send + Sync + Clone + 'static {
    type RequestType: Serialize + Send;
    type ResponseType: DeserializeOwned + Send;
    type StreamResponseType: DeserializeOwned + Send;
    type ErrorMapperType: ErrorMapper;

    fn name(&self) -> &str;
    fn supported_models(&self) -> Vec<String>;
    fn endpoint_url(&self, base_url: &Option<String>) -> String;

    fn build_request_data(&self, request: &ChatRequest, stream: bool) -> Self::RequestType;

    fn parse_response_data(&self, response: Self::ResponseType) -> ChatResponse;

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> StreamingResponse;
}