// src/providers/deepseek.rs

use async_trait::async_trait;
use crate::config::ProviderConfig;
use crate::error::LlmConnectorError;
use crate::providers::base::Provider;
use crate::providers::common::error_mapper::ErrorMapper;
use crate::types::{ChatRequest, ChatResponse, Choice, Message, Usage};
#[cfg(feature = "streaming")]
use crate::types::{
    ChatStream, StreamingChoice, StreamingResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct DeepSeekProvider {
    pub(crate) generic_provider: crate::providers::generic::GenericProvider<DeepSeekAdapter>,
}

impl DeepSeekProvider {
    pub fn new(config: ProviderConfig) -> Result<Self, LlmConnectorError> {
        let adapter = DeepSeekAdapter;
        let generic_provider = crate::providers::generic::GenericProvider::new(config, adapter)?;
        Ok(Self { generic_provider })
    }
}

pub struct DeepSeekErrorMapper;

impl ErrorMapper for DeepSeekErrorMapper {
    fn map_error(response: Value) -> LlmConnectorError {
        let error_message = response["error"]["message"].as_str().unwrap_or("Unknown DeepSeek error");
        LlmConnectorError::ProviderError(error_message.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct DeepSeekAdapter;

#[async_trait]
impl crate::providers::common::adapter::ProviderAdapter for DeepSeekAdapter {
    type RequestType = DeepSeekRequest;
    type ResponseType = DeepSeekResponse;
    type StreamResponseType = DeepSeekStreamResponse;
    type ErrorMapperType = DeepSeekErrorMapper;

    fn name(&self) -> &str {
        "deepseek"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "deepseek-chat".to_string(),
            "deepseek-coder".to_string(),
        ]
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        base_url.as_deref().unwrap_or("https://api.deepseek.com/v1").to_string()
    }

    fn build_request_data(&self, request: &ChatRequest, stream: bool) -> Self::RequestType {
        let messages = request.messages.iter().map(|msg| DeepSeekMessage {
            role: msg.role.clone(),
            content: Some(msg.content.clone()),
            name: msg.name.clone(),
            tool_calls: msg.tool_calls.clone(),
            tool_call_id: msg.tool_call_id.clone(),
        }).collect();

        let tools = request.tools.as_ref().map(|tools| {
            tools.iter().map(|t| DeepSeekTool {
                r#type: t.tool_type.clone(),
                function: DeepSeekFunction {
                    name: t.function.name.clone(),
                    description: t.function.description.clone().unwrap_or_default(),
                    parameters: Some(t.function.parameters.clone()),
                    strict: None,
                },
            }).collect()
        });

        let tool_choice = request.tool_choice.as_ref().map(|tc| {
            serde_json::to_value(tc).unwrap_or(Value::Null)
        });

        DeepSeekRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: Some(stream),
            tools,
            tool_choice,
        }
    }

    fn parse_response_data(&self, response: Self::ResponseType) -> ChatResponse {
        ChatResponse {
            id: response.id,
            model: response.model,
            created: response.created,
            object: response.object,
            choices: response.choices.into_iter().map(|choice| {
                Choice {
                    index: choice.index,
                    message: Message {
                        role: choice.message.role,
                        content: choice.message.content.unwrap_or_default(),
                        name: choice.message.name,
                        tool_calls: choice.message.tool_calls,
                        tool_call_id: choice.message.tool_call_id,
                    },
                    logprobs: None,
                    finish_reason: choice.finish_reason,
                }
            }).collect(),
            usage: Some(response.usage),
            system_fingerprint: response.system_fingerprint,
        }
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> StreamingResponse {
        StreamingResponse {
            id: response.id,
            object: response.object,
            created: response.created,
            model: response.model,
            choices: response.choices.into_iter().map(|choice| StreamingChoice {
                index: choice.index,
                delta: Delta {
                    role: choice.delta.role,
                    content: choice.delta.content,
                    tool_calls: choice.delta.tool_calls,
                    reasoning_content: None,
                },
                finish_reason: choice.finish_reason,
                logprobs: None,
            }).collect(),
            usage: response.usage.clone(),
            system_fingerprint: response.system_fingerprint,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct DeepSeekRequest {
    model: String,
    messages: Vec<DeepSeekMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<DeepSeekTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<Value>,
}

#[derive(Serialize, Debug)]
struct DeepSeekMessage {
    role: String,
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<crate::types::ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Serialize, Debug)]
struct DeepSeekTool {
    r#type: String,
    function: DeepSeekFunction,
}

#[derive(Serialize, Debug)]
struct DeepSeekFunction {
    name: String,
    description: String,
    parameters: Option<Value>,
    strict: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct DeepSeekResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<DeepSeekChoice>,
    usage: Usage,
    system_fingerprint: Option<String>,
}

#[derive(Deserialize, Debug)]
struct DeepSeekChoice {
    index: u32,
    message: DeepSeekResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct DeepSeekResponseMessage {
    role: String,
    content: Option<String>,
    name: Option<String>,
    tool_calls: Option<Vec<crate::types::ToolCall>>,
    tool_call_id: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct DeepSeekStreamResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<DeepSeekStreamChoice>,
    usage: Option<Usage>,
    system_fingerprint: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
struct DeepSeekStreamChoice {
    index: u32,
    delta: DeepSeekStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
struct DeepSeekStreamDelta {
    role: Option<String>,
    content: Option<String>,
    tool_calls: Option<Vec<crate::types::ToolCall>>,
}

#[async_trait]
impl Provider for DeepSeekProvider {
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
    use futures_util::stream::StreamExt;
    use wiremock::{
        matchers::{header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[test]
    fn test_provider_name() {
        let provider = create_test_provider();
        assert_eq!(provider.name(), "deepseek");
    }

    #[test]
    fn test_supported_models() {
        let provider = create_test_provider();
        let models = provider.supported_models();
        assert!(models.contains(&"deepseek-chat".to_string()));
        assert!(models.contains(&"deepseek-coder".to_string()));
    }

    #[test]
    fn test_model_support() {
        let provider = create_test_provider();
        assert!(provider.supports_model("deepseek-chat"));
        assert!(provider.supports_model("deepseek-coder"));
        assert!(!provider.supports_model("unsupported-model"));
    }

    fn create_test_provider() -> crate::providers::generic::GenericProvider<DeepSeekAdapter> {
        let config = ProviderConfig {
            api_key: "test-key".to_string(),
            base_url: None,
            timeout_ms: Some(5000),
            proxy: None,
        };
        let provider = DeepSeekProvider::new(config).unwrap();
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
                .set_body_string("data: {\"id\":\"chatcmpl-test\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"deepseek-chat\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"Hello World\"},\"finish_reason\":null}]}\n\ndata: [DONE]\n\n")
                .insert_header("content-type", "text/event-stream"))
            .mount(&mock_server)
            .await;

        let config = ProviderConfig {
            api_key: "test-api-key".to_string(),
            base_url: Some(mock_server.uri()),
            timeout_ms: Some(5000),
            proxy: None,
        };
        let provider = DeepSeekProvider::new(config).unwrap();

        let request = ChatRequest {
            model: "deepseek-chat".to_string(),
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
