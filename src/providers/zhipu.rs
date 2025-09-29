// src/providers/zhipu.rs

use async_trait::async_trait;
use crate::config::ProviderConfig;
use crate::error::LlmConnectorError;
use crate::providers::base::Provider;
use crate::providers::common::error_mapper::ErrorMapper;
use crate::types::{ChatRequest, ChatResponse, Choice, Message, ToolCall};
#[cfg(feature = "streaming")]
use crate::types::{StreamingChoice, StreamingResponse, Delta};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct ZhipuProvider {
    pub(crate) generic_provider: crate::providers::generic::GenericProvider<ZhipuAdapter>,
}

impl ZhipuProvider {
    pub fn new(config: ProviderConfig) -> Result<Self, LlmConnectorError> {
        let adapter = ZhipuAdapter;
        let generic_provider = crate::providers::generic::GenericProvider::new(config, adapter)?;
        Ok(Self { generic_provider })
    }
}

pub struct ZhipuErrorMapper;

impl ErrorMapper for ZhipuErrorMapper {
    fn map_error(response: Value) -> LlmConnectorError {
        let error_message = response["error"]["message"].as_str().unwrap_or("Unknown Zhipu error");
        LlmConnectorError::ProviderError(error_message.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ZhipuAdapter;

#[async_trait]
impl crate::providers::common::adapter::ProviderAdapter for ZhipuAdapter {
    type RequestType = ZhipuRequest;
    type ResponseType = ZhipuResponse;
    type StreamResponseType = ZhipuStreamResponse;
    type ErrorMapperType = ZhipuErrorMapper;

    fn name(&self) -> &str {
        "zhipu"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "glm-4".to_string(),
            "glm-3-turbo".to_string(),
        ]
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        base_url.as_deref().unwrap_or("https://open.bigmodel.cn/api/paas/v4").to_string()
    }

    fn build_request_data(&self, request: &ChatRequest, stream: bool) -> Self::RequestType {
        let messages = request.messages.iter().map(|msg| ZhipuMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        }).collect();

        ZhipuRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: Some(stream),
            stop: request.stop.clone(),
        }
    }

    fn parse_response_data(&self, response: Self::ResponseType) -> ChatResponse {
        ChatResponse {
            id: response.id,
            object: "chat.completion".to_string(),
            created: response.created,
            model: response.model,
            choices: response.choices.into_iter().map(|choice| {
                Choice {
                    index: choice.index,
                    message: Message {
                        role: choice.message.role,
                        content: choice.message.content.unwrap_or_default(),
                        name: None,
                        tool_calls: choice.message.tool_calls,
                        tool_call_id: None,
                    },
                    logprobs: None,
                    finish_reason: Some(choice.finish_reason),
                }
            }).collect(),
            usage: Some(response.usage),
            system_fingerprint: None,
        }
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> StreamingResponse {
        let choices = response
            .choices
            .iter()
            .map(|choice| StreamingChoice {
                index: 0, // ZhipuStreamChoice doesn't have index, so hardcode to 0
                delta: Delta {
                    role: None, // ZhipuStreamDelta doesn't have role
                    content: Some(choice.delta.content.clone()),
                    tool_calls: None, // ZhipuStreamDelta doesn't have tool_calls
                    reasoning_content: None,
                },
                finish_reason: choice.finish_reason.clone(),
                logprobs: None,
            })
            .collect();

        StreamingResponse {
            id: response.id,
            object: "chat.completion.chunk".to_string(),
            created: response.created,
            model: response.model.clone(),
            choices,
            usage: response.usage.clone(),
            system_fingerprint: None,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ZhipuRequest {
    model: String,
    messages: Vec<ZhipuMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Serialize, Debug)]
pub struct ZhipuMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
pub struct ZhipuResponse {
    id: String,
    created: u64,
    model: String,
    choices: Vec<ZhipuChoice>,
    usage: crate::types::Usage,
}

#[derive(Deserialize, Debug)]
pub struct ZhipuChoice {
    index: u32,
    finish_reason: String,
    message: ZhipuResponseMessage,
}

#[derive(Deserialize, Debug)]
pub struct ZhipuResponseMessage {
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct ZhipuStreamResponse {
    id: String,
    created: u64,
    model: String,
    choices: Vec<ZhipuStreamChoice>,
    usage: Option<crate::types::Usage>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct ZhipuStreamChoice {
    delta: ZhipuStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct ZhipuStreamDelta {
    content: String,
}

#[async_trait]
impl Provider for ZhipuProvider {
    fn name(&self) -> &str {
        self.generic_provider.name()
    }

    fn supported_models(&self) -> Vec<String> {
        self.generic_provider.supported_models()
    }

    fn supports_model(&self, model: &str) -> bool {
        self.generic_provider.supports_model(model)
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        self.generic_provider.chat(request).await
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        self.generic_provider.chat_stream(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = create_test_provider();
        assert_eq!(provider.name(), "zhipu");
    }

    #[test]
    fn test_supported_models() {
        let provider = create_test_provider();
        let models = provider.supported_models();
        assert!(models.contains(&"glm-4".to_string()));
        assert!(models.contains(&"glm-3-turbo".to_string()));
    }

    #[test]
    fn test_model_support() {
        let provider = create_test_provider();
        assert!(provider.supports_model("glm-4"));
        assert!(provider.supports_model("glm-3-turbo"));
        assert!(!provider.supports_model("unsupported-model"));
    }

    fn create_test_provider() -> crate::providers::generic::GenericProvider<ZhipuAdapter> {
        let config = ProviderConfig {
            api_key: "test-key".to_string(),
            base_url: None,
            timeout_ms: Some(5000),
            proxy: None,
        };
        let provider = ZhipuProvider::new(config).unwrap();
        provider.generic_provider
    }

    #[cfg(feature = "streaming")]
    #[tokio::test]
    async fn test_chat_stream_mocked() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer test-api-key"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string("data: {\"id\":\"chatcmpl-test\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"glm-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello World\"},\"finish_reason\":null}]}\n\ndata: [DONE]\n\n")
                .insert_header("content-type", "text/event-stream"))
            .mount(&mock_server)
            .await;

        let config = ProviderConfig {
            api_key: "test-api-key".to_string(),
            base_url: Some(mock_server.uri()),
            timeout_ms: Some(5000),
            proxy: None,
        };
        let provider = ZhipuProvider::new(config).unwrap();

        let request = ChatRequest {
            model: "glm-4".to_string(),
            messages: vec![crate::types::Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }],
            max_tokens: Some(100),
            temperature: Some(0.7),
            top_p: Some(0.9),
            stream: Some(true),
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            seed: None,
            tools: None,
            tool_choice: None,
            response_format: None,
        };

        let mut stream = provider.chat_stream(&request).await.unwrap();
        let first_chunk = stream.next().await.unwrap().unwrap();
        assert_eq!(first_chunk.choices[0].delta.content, Some("Hello World".to_string()));
    }
}