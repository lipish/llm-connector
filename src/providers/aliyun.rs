use async_trait::async_trait;
use crate::config::ProviderConfig;
use crate::error::LlmConnectorError;
use crate::providers::base::Provider;
use crate::providers::common::error_mapper::ErrorMapper;
use crate::types::{ChatRequest, ChatResponse, Choice, Message, Usage};
#[cfg(feature = "streaming")]
use crate::types::{ChatStream, StreamingChoice, StreamingResponse, Delta};
#[cfg(feature = "streaming")]
use crate::providers::common::sse::sse_data_events;

use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Debug, Clone)]
pub struct AliyunProvider {
    pub(crate) generic_provider: crate::providers::generic::GenericProvider<AliyunAdapter>,
}

#[derive(Debug, Serialize)]
pub struct AliyunRequest {
    model: String,
    input: AliyunInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<AliyunParameters>,
}

#[derive(Debug, Serialize)]
pub struct AliyunInput {
    messages: Vec<AliyunMessage>,
}

#[derive(Debug, Serialize)]
pub struct AliyunMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Default)]
pub struct AliyunParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    result_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    incremental_output: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AliyunResponse {
    request_id: String,
    output: AliyunOutput,
    usage: AliyunUsage,
}

#[derive(Debug, Deserialize)]
pub struct AliyunOutput {
    choices: Vec<AliyunChoice>,
}

#[derive(Debug, Deserialize)]
pub struct AliyunChoice {
    message: AliyunResponseMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct AliyunResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct AliyunUsage {
    input_tokens: i32,
    output_tokens: i32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AliyunStreamResponse {
    request_id: String,
    output: AliyunOutput,
    usage: AliyunUsage,
}

impl AliyunProvider {
    pub fn new(config: ProviderConfig) -> Result<Self, LlmConnectorError> {
        let adapter = AliyunAdapter;
        let generic_provider = crate::providers::generic::GenericProvider::new(config, adapter)?;
        Ok(Self { generic_provider })
    }
}

pub struct AliyunErrorMapper;

impl ErrorMapper for AliyunErrorMapper {
    fn map_error(response: Value) -> LlmConnectorError {
        let error_message = response["error"]["message"].as_str().unwrap_or("Unknown Aliyun error");
        LlmConnectorError::ProviderError(error_message.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct AliyunAdapter;

#[async_trait]
impl crate::providers::common::adapter::ProviderAdapter for AliyunAdapter {
    type RequestType = AliyunRequest;
    type ResponseType = AliyunResponse;
    type StreamResponseType = AliyunStreamResponse;
    type ErrorMapperType = AliyunErrorMapper;

    fn name(&self) -> &str {
        "aliyun"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "qwen-turbo".to_string(),
            "qwen-plus".to_string(),
        ]
    }

    fn endpoint_url(&self, base_url: &Option<String>) -> String {
        base_url.as_deref().unwrap_or("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation").to_string()
    }

    fn build_request_data(&self, request: &ChatRequest, stream: bool) -> Self::RequestType {
        let messages = request.messages.iter().map(|msg| AliyunMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        }).collect();

        let parameters = AliyunParameters {
            temperature: request.temperature,
            top_p: request.top_p,
            max_tokens: request.max_tokens,
            seed: request.seed,
            result_format: Some("message".to_string()),
            incremental_output: if stream { Some(true) } else { None },
            ..Default::default()
        };

        AliyunRequest {
            model: request.model.clone(),
            input: AliyunInput { messages },
            parameters: Some(parameters),
        }
    }

    fn parse_response_data(&self, response: Self::ResponseType) -> ChatResponse {
        ChatResponse {
            id: response.request_id,
            object: "chat.completion".to_string(),
            created: 0, // Aliyun does not provide this
            model: "".to_string(), // Aliyun does not provide this
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: response.output.choices[0].message.role.clone(),
                    content: response.output.choices[0].message.content.clone(),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                logprobs: None,
                finish_reason: Some(response.output.choices[0].finish_reason.clone()),
            }],
            usage: Some(Usage {
                prompt_tokens: response.usage.input_tokens as u32,
                completion_tokens: response.usage.output_tokens as u32,
                total_tokens: (response.usage.input_tokens + response.usage.output_tokens) as u32,
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        }
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> StreamingResponse {
        StreamingResponse {
            id: response.request_id,
            object: "chat.completion.chunk".to_string(),
            created: 0, // Aliyun does not provide this
            model: "".to_string(), // Aliyun does not provide this
            choices: vec![StreamingChoice {
                index: 0,
                delta: Delta {
                    role: Some(response.output.choices[0].message.role.clone()),
                    content: Some(response.output.choices[0].message.content.clone()),
                    tool_calls: None,
                    reasoning_content: None,
                },
                finish_reason: Some(response.output.choices[0].finish_reason.clone()),
                logprobs: None,
            }],
            usage: Some(Usage {
                prompt_tokens: response.usage.input_tokens as u32,
                completion_tokens: response.usage.output_tokens as u32,
                total_tokens: (response.usage.input_tokens + response.usage.output_tokens) as u32,
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: None,
        }
    }
}



#[async_trait]
impl Provider for AliyunProvider {
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
    #[cfg(feature = "streaming")]
    use futures_util::StreamExt;
    #[cfg(feature = "streaming")]
    use wiremock::{matchers::{method, path, header}, Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_provider_name() {
        let provider = create_test_provider();
        assert_eq!(provider.name(), "aliyun");
    }

    #[test]
    fn test_supported_models() {
        let provider = create_test_provider();
        let models = provider.supported_models();
        assert!(models.contains(&"qwen-turbo".to_string()));
        assert!(models.contains(&"qwen-plus".to_string()));
    }

    #[test]
    fn test_model_support() {
        let provider = create_test_provider();
        assert!(provider.supports_model("qwen-turbo"));
        assert!(provider.supports_model("qwen-plus"));
        assert!(!provider.supports_model("unsupported-model"));
    }

    fn create_test_provider() -> AliyunProvider {
        let config = ProviderConfig {
            api_key: "test-key".to_string(),
            base_url: None,
            timeout_ms: Some(5000),
            proxy: None,
        };
        AliyunProvider::new(config).unwrap()
    }

    #[cfg(feature = "streaming")]
    #[tokio::test]
    async fn test_chat_stream_mocked() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/services/aigc/text-generation/generation"))
            .and(header("authorization", "Bearer test-api-key"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string("id: 1\nevent:result\ndata:{\"request_id\":\"test-request-id\",\"output\":{\"choices\":[{\"message\":{\"role\":\"assistant\",\"content\":\"Hello World\"},\"finish_reason\":\"stop\"}]},\"usage\":{\"input_tokens\":10,\"output_tokens\":20}}\n\n")
                .insert_header("content-type", "text/event-stream"))
            .mount(&mock_server)
            .await;

        let config = ProviderConfig {
            api_key: "test-api-key".to_string(),
            base_url: Some(mock_server.uri()),
            timeout_ms: Some(5000),
            proxy: None,
        };
        let provider = AliyunProvider::new(config).unwrap();

        let request = ChatRequest {
            model: "qwen-turbo".to_string(),
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