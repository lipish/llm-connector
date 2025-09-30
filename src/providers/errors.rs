use crate::error::LlmConnectorError;
use serde_json::Value;

pub trait ErrorMapper {
    fn map_http_error(status: u16, body: Value) -> LlmConnectorError;
    fn map_network_error(error: reqwest::Error) -> LlmConnectorError;
    fn is_retriable_error(error: &LlmConnectorError) -> bool;
}
