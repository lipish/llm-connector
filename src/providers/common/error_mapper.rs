use crate::error::LlmConnectorError;

pub trait ErrorMapper {
    fn map_error(response: serde_json::Value) -> LlmConnectorError;
}