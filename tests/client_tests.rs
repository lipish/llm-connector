//! Integration tests for LlmClient

use llm_connector::LlmClient;

#[test]
fn test_openai_client_creation() {
    let client = LlmClient::openai("sk-test", None);
    assert_eq!(client.protocol_name(), "openai");
}

#[test]
fn test_openai_compatible_client_creation() {
    let client = LlmClient::openai("sk-test", Some("https://api.test.com/v1"));
    assert_eq!(client.protocol_name(), "openai");
}

#[test]
fn test_anthropic_client_creation() {
    let client = LlmClient::anthropic("sk-ant-test");
    assert_eq!(client.protocol_name(), "anthropic");
}

#[test]
fn test_aliyun_client_creation() {
    let client = LlmClient::aliyun("sk-test");
    assert_eq!(client.protocol_name(), "aliyun");
}

#[test]
fn test_ollama_client_creation() {
    let client = LlmClient::ollama(None);
    assert_eq!(client.protocol_name(), "ollama");
}

#[test]
fn test_ollama_with_custom_url() {
    let client = LlmClient::ollama(Some("http://localhost:11434"));
    assert_eq!(client.protocol_name(), "ollama");
}

#[test]
fn test_multiple_clients_can_coexist() {
    let openai = LlmClient::openai("sk-1", None);
    let anthropic = LlmClient::anthropic("sk-2");
    let aliyun = LlmClient::aliyun("sk-3");
    let ollama = LlmClient::ollama(None);

    assert_eq!(openai.protocol_name(), "openai");
    assert_eq!(anthropic.protocol_name(), "anthropic");
    assert_eq!(aliyun.protocol_name(), "aliyun");
    assert_eq!(ollama.protocol_name(), "ollama");
}

#[test]
fn test_client_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<LlmClient>();
}

#[test]
fn test_client_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<LlmClient>();
}

#[tokio::test]
async fn test_openai_fetch_models_unsupported_with_invalid_key() {
    let client = LlmClient::openai("invalid-key", None);
    let result = client.fetch_models().await;
    // Should fail (either auth error or connection error)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_anthropic_fetch_models_unsupported() {
    let client = LlmClient::anthropic("test-key");
    let result = client.fetch_models().await;
    // May fail due to unsupported listing or API error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_aliyun_fetch_models_unsupported() {
    let client = LlmClient::aliyun("test-key");
    let result = client.fetch_models().await;
    // Should return UnsupportedOperation error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("does not support model listing"));
}

#[tokio::test]
async fn test_ollama_fetch_models_unsupported() {
    let client = LlmClient::ollama(None);
    let result = client.fetch_models().await;
    // Fails when local Ollama server is not running
    assert!(result.is_err());
}

