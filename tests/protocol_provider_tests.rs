//! Unit tests for ProtocolProvider streaming functionality
//!
//! These tests focus on the ProtocolProvider implementation without making real API calls.

use llm_connector::{
    core::Protocol,
    protocols::{AnthropicProtocol, OpenAIProtocol, ProtocolProvider, ZhipuProtocol},
    types::{ChatRequest, ChatResponse, Message},
};

// Import the Provider trait for ProtocolProvider
use llm_connector::Provider;

#[cfg(feature = "streaming")]

/// Mock protocol for testing
#[derive(Debug, Clone)]
struct MockProtocol {
    name: String,
    uses_sse: bool,
}

impl MockProtocol {
    fn new(name: &str, uses_sse: bool) -> Self {
        Self {
            name: name.to_string(),
            uses_sse,
        }
    }
}

impl Default for MockProtocol {
    fn default() -> Self {
        Self::new("mock", false)
    }
}

#[async_trait::async_trait]
impl Protocol for MockProtocol {
    type Request = serde_json::Value;
    type Response = serde_json::Value;
    type Error = MockError;

    #[cfg(feature = "streaming")]
    type StreamResponse = serde_json::Value;

    fn name(&self) -> &str {
        &self.name
    }

    fn endpoint(&self, base_url: &str) -> String {
        format!("{}/mock/chat", base_url)
    }

    fn models_endpoint(&self, _base_url: &str) -> Option<String> {
        Some(format!("{}/mock/models", "https://example.com"))
    }

    fn build_request(&self, request: &ChatRequest, stream: bool) -> Self::Request {
        serde_json::json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        llm_connector::types::Role::User => "user",
                        llm_connector::types::Role::Assistant => "assistant",
                        llm_connector::types::Role::System => "system",
                        llm_connector::types::Role::Tool => "tool",
                    },
                    "content": m.content
                })
            }).collect::<Vec<_>>(),
            "stream": stream,
            "max_tokens": request.max_tokens,
        })
    }

    fn parse_response(&self, response: Self::Response) -> ChatResponse {
        // Mock parsing - just return a simple response
        ChatResponse {
            id: "mock-id".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: response["model"]
                .as_str()
                .unwrap_or("mock-model")
                .to_string(),
            choices: vec![llm_connector::types::Choice {
                index: 0,
                message: Message::user("mock response"),
                finish_reason: Some("stop".to_string()),
                logprobs: None,
            }],
            content: "mock response".to_string(),
            usage: None,
            system_fingerprint: None,
        }
    }

    #[cfg(feature = "streaming")]
    fn parse_stream_response(
        &self,
        _response: Self::StreamResponse,
    ) -> llm_connector::types::ChatStream {
        use futures_util::stream;

        // Mock streaming response
        let mock_response = llm_connector::types::StreamingResponse {
            id: "mock-id".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: 1234567890,
            model: "mock-model".to_string(),
            choices: vec![llm_connector::types::StreamingChoice {
                index: 0,
                delta: llm_connector::types::Delta {
                    role: Some(llm_connector::types::Role::Assistant),
                    content: Some("mock content".to_string()),
                    ..Default::default()
                },
                finish_reason: Some("stop".to_string()),
                logprobs: None,
            }],
            content: "mock content".to_string(),
            reasoning_content: None,
            usage: None,
            system_fingerprint: None,
        };

        let stream = stream::once(async { Ok(mock_response) });
        Box::pin(stream)
    }

    #[cfg(feature = "streaming")]
    fn uses_sse_stream(&self) -> bool {
        self.uses_sse
    }
}

#[derive(Debug)]
struct MockError;

impl llm_connector::core::protocol::ProtocolError for MockError {
    fn map_http_error(
        status: u16,
        _body: serde_json::Value,
    ) -> llm_connector::error::LlmConnectorError {
        llm_connector::error::LlmConnectorError::ProviderError(format!("Mock HTTP {}", status))
    }

    fn map_network_error(_error: reqwest::Error) -> llm_connector::error::LlmConnectorError {
        llm_connector::error::LlmConnectorError::NetworkError("Mock network error".to_string())
    }

    fn is_retriable_error(_error: &llm_connector::error::LlmConnectorError) -> bool {
        false
    }
}

#[test]
fn test_protocol_provider_creation() -> Result<(), Box<dyn std::error::Error>> {
    // Test with OpenAI protocol
    let openai_protocol = OpenAIProtocol::new();
    let _openai_provider =
        ProtocolProvider::new(openai_protocol, "https://api.openai.com/v1", "test-key")?;

    // Test with Anthropic protocol
    let anthropic_protocol = AnthropicProtocol::new();
    let _anthropic_provider =
        ProtocolProvider::new(anthropic_protocol, "https://api.anthropic.com", "test-key")?;

    // Test with Zhipu protocol
    let zhipu_protocol = ZhipuProtocol::new();
    let _zhipu_provider = ProtocolProvider::new(
        zhipu_protocol,
        "https://open.bigmodel.cn/api/paas/v4",
        "test-key",
    )?;

    // Test with mock protocol
    let mock_protocol = MockProtocol::new("mock", false);
    let _mock_provider = ProtocolProvider::new(mock_protocol, "https://example.com", "test-key")?;

    Ok(())
}

#[test]
fn test_protocol_provider_properties() -> Result<(), Box<dyn std::error::Error>> {
    let openai_protocol = OpenAIProtocol::new();
    let provider = ProtocolProvider::new(openai_protocol, "https://api.openai.com/v1", "test-key")?;

    // Test protocol access
    let protocol = provider.protocol();
    assert_eq!(protocol.name(), "openai");

    // Test name delegation
    assert_eq!(provider.name(), "openai");

    Ok(())
}

#[cfg(feature = "streaming")]
#[tokio::test]
async fn test_protocol_provider_streaming_fallback() -> Result<(), Box<dyn std::error::Error>> {
    let mock_protocol = MockProtocol::new("mock-fallback", false);
    let provider = ProtocolProvider::new(mock_protocol, "https://example.com", "test-key")?;

    let _request = ChatRequest {
        model: "mock-model".to_string(),
        messages: vec![Message::user("test")],
        max_tokens: Some(10),
        ..Default::default()
    };

    // Test that the streaming logic correctly detects SSE vs non-SSE
    // We'll test the logic path without actually making HTTP calls
    assert!(
        !provider.protocol().uses_sse_stream(),
        "Should detect as non-SSE protocol"
    );

    // Note: We can't test the actual streaming without a real HTTP server
    // But we can verify the logic detection works

    Ok(())
}

#[cfg(feature = "streaming")]
#[tokio::test]
async fn test_protocol_provider_sse_detection() -> Result<(), Box<dyn std::error::Error>> {
    let mock_protocol = MockProtocol::new("mock-sse", true);
    let provider = ProtocolProvider::new(mock_protocol, "https://example.com", "test-key")?;

    let _request = ChatRequest {
        model: "mock-model".to_string(),
        messages: vec![Message::user("test")],
        max_tokens: Some(10),
        ..Default::default()
    };

    // Test that the streaming logic correctly detects SSE
    assert!(
        provider.protocol().uses_sse_stream(),
        "Should detect as SSE protocol"
    );

    // Note: We can't test the actual streaming without a real HTTP server
    // But we can verify the logic detection works

    Ok(())
}

#[test]
fn test_real_protocols_sse_settings() {
    // Test that real protocols have correct SSE settings
    let openai = OpenAIProtocol::new();
    assert!(openai.uses_sse_stream(), "OpenAI should use SSE streaming");

    let anthropic = AnthropicProtocol::new();
    assert!(
        anthropic.uses_sse_stream(),
        "Anthropic should use SSE streaming"
    );

    let zhipu = ZhipuProtocol::new();
    assert!(zhipu.uses_sse_stream(), "Zhipu should use SSE streaming");
}

#[test]
fn test_protocol_provider_request_building() -> Result<(), Box<dyn std::error::Error>> {
    let mock_protocol = MockProtocol::new("mock", false);
    let provider = ProtocolProvider::new(mock_protocol, "https://example.com", "test-key")?;

    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![
            Message::system("You are a helpful assistant"),
            Message::user("Hello"),
        ],
        max_tokens: Some(50),
        temperature: Some(0.7),
        ..Default::default()
    };

    // Test that we can build requests (this doesn't actually send them)
    let _stream_request = provider.protocol().build_request(&request, true);
    let _regular_request = provider.protocol().build_request(&request, false);

    Ok(())
}

#[test]
fn test_endpoint_urls() -> Result<(), Box<dyn std::error::Error>> {
    let openai = OpenAIProtocol::new();
    assert_eq!(
        openai.endpoint("https://api.openai.com/v1"),
        "https://api.openai.com/v1/chat/completions"
    );

    let anthropic = AnthropicProtocol::new();
    assert_eq!(
        anthropic.endpoint("https://api.anthropic.com"),
        "https://api.anthropic.com/v1/messages"
    );

    let zhipu = ZhipuProtocol::new();
    assert_eq!(
        zhipu.endpoint("https://open.bigmodel.cn/api/paas/v4"),
        "https://open.bigmodel.cn/api/paas/v4/chat/completions"
    );

    Ok(())
}
