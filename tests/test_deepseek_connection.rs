//! Test DeepSeek HTTP connection using a mock server.
//!
//! Note: This test uses the Client API which requires model routing.
//! For direct provider testing, use GenericProvider instead.

use llm_connector::{ChatRequest, Client, Config, Message, ProviderConfig};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
#[ignore] // Ignored because Client requires proper model routing setup
async fn test_deepseek_connection_success() {
    // Start a mock server.
    let server = MockServer::start().await;

    // Create a mock for the chat completions endpoint.
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("Authorization", "Bearer fake-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl-mock-id",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "deepseek-chat",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello from mock server!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 9,
                "completion_tokens": 12,
                "total_tokens": 21
            }
        })))
        .mount(&server)
        .await;

    // Configure the client to use the mock server.
    let config = Config {
        deepseek: Some(ProviderConfig::new("fake-api-key").with_base_url(server.uri())),
        ..Default::default()
    };
    let client = Client::with_config(config);

    // Create a chat request.
    let request = ChatRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    // Perform the chat request.
    let response = client.chat(request).await.unwrap();

    // Assert that the response is as expected.
    assert_eq!(response.id, "chatcmpl-mock-id");
    assert_eq!(
        response.choices[0].message.content,
        "Hello from mock server!"
    );
}
