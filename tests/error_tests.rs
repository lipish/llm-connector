//! Tests for error handling

use llm_connector::error::LlmConnectorError;

#[test]
fn test_error_display() {
    let err = LlmConnectorError::AuthenticationError("Invalid API key".to_string());
    assert_eq!(err.to_string(), "Authentication failed: Invalid API key");

    let err = LlmConnectorError::RateLimitError("Too many requests".to_string());
    assert_eq!(err.to_string(), "Rate limit exceeded: Too many requests");

    let err = LlmConnectorError::UnsupportedOperation("Not supported".to_string());
    assert_eq!(err.to_string(), "Unsupported operation: Not supported");
}

#[test]
fn test_error_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<LlmConnectorError>();
}

#[test]
fn test_error_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<LlmConnectorError>();
}

#[test]
fn test_error_status_code() {
    let err = LlmConnectorError::AuthenticationError("test".to_string());
    assert_eq!(err.status_code(), 401);

    let err = LlmConnectorError::RateLimitError("test".to_string());
    assert_eq!(err.status_code(), 429);

    let err = LlmConnectorError::NotFoundError("test".to_string());
    assert_eq!(err.status_code(), 404);

    let err = LlmConnectorError::UnsupportedOperation("test".to_string());
    assert_eq!(err.status_code(), 501);

    let err = LlmConnectorError::ParseError("test".to_string());
    assert_eq!(err.status_code(), 400);
}

#[test]
fn test_error_variants() {
    // Test all error variants can be created
    let _auth = LlmConnectorError::AuthenticationError("test".to_string());
    let _rate = LlmConnectorError::RateLimitError("test".to_string());
    let _not_found = LlmConnectorError::NotFoundError("test".to_string());
    let _invalid = LlmConnectorError::InvalidRequest("test".to_string());
    let _server = LlmConnectorError::ServerError("test".to_string());
    let _timeout = LlmConnectorError::TimeoutError("test".to_string());
    let _connection = LlmConnectorError::ConnectionError("test".to_string());
    let _parse = LlmConnectorError::ParseError("test".to_string());
    let _unsupported = LlmConnectorError::UnsupportedOperation("test".to_string());
}

// Removed test_error_from_reqwest as it's difficult to construct reqwest::Error in tests

#[test]
fn test_error_debug() {
    let err = LlmConnectorError::AuthenticationError("test".to_string());
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("AuthenticationError"));
}

#[test]
fn test_unsupported_operation_error() {
    let err = LlmConnectorError::UnsupportedOperation(
        "anthropic does not support model listing".to_string(),
    );

    assert_eq!(err.status_code(), 501);
    assert!(err.to_string().contains("Unsupported operation"));
    assert!(err.to_string().contains("anthropic"));
}
