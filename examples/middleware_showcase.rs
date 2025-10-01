//! Middleware Showcase Example
//!
//! This example demonstrates how to use and compose the various middleware provided
//! by the `llm-connector` library. It covers:
//!
//! 1.  `RetryMiddleware` for automatic retries on failure.
//! 2.  `LoggingMiddleware` for detailed request/response logging.
//! 3.  `MetricsMiddleware` for collecting performance and cost metrics.
//! 4.  `InterceptorChain` for request/response validation and modification.
//! 5.  Composing multiple middleware together using the "onion model."
//!
//! To run this example, execute:
//! `cargo run --example middleware_showcase`

use async_trait::async_trait;
use llm_connector::config::RetryConfig;
use llm_connector::error::LlmConnectorError;
use llm_connector::middleware::interceptor::{
    Interceptor, InterceptorChain, SanitizationInterceptor, ValidationInterceptor,
};
use llm_connector::middleware::{LoggingMiddleware, MetricsMiddleware, RetryMiddleware};
use llm_connector::types::{ChatRequest, ChatResponse};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// A mock API client that simulates network requests.
/// It can be configured to fail a certain number of times before succeeding.
#[derive(Clone)]
struct MockApiClient {
    failures_remaining: Arc<AtomicUsize>,
}

impl MockApiClient {
    fn new(initial_failures: usize) -> Self {
        Self {
            failures_remaining: Arc::new(AtomicUsize::new(initial_failures)),
        }
    }

    /// Simulates making an API call.
    async fn make_request(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        println!("\nAttempting API call...");
        if self.failures_remaining.load(Ordering::SeqCst) > 0 {
            self.failures_remaining.fetch_sub(1, Ordering::SeqCst);
            println!("API call failed. Simulating a transient error.");
            // Simulate a retriable error
            Err(LlmConnectorError::NetworkError(
                "Simulated network failure".to_string(),
            ))
        } else {
            println!("API call successful.");
            let response = ChatResponse {
                model: request.model.clone(),
                system_fingerprint: Some("mock-fingerprint".to_string()),
                ..Default::default()
            };
            Ok(response)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), LlmConnectorError> {
    // --- 1. RetryMiddleware Showcase ---
    println!("--- 1. RetryMiddleware Showcase ---");
    let retry_config = RetryConfig {
        max_retries: 3,
        initial_backoff_ms: 100,
        backoff_multiplier: 2.0,
        max_backoff_ms: 5000,
    };
    let retry_policy = RetryMiddleware::new(retry_config);
    let mock_client_retry = MockApiClient::new(2); // Fails 2 times, succeeds on the 3rd
    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![],
        ..Default::default()
    };

    let response = retry_policy
        .execute(|| async {
            sleep(Duration::from_millis(50)).await; // Simulate work
            mock_client_retry.make_request(&request).await
        })
        .await?;
    assert_eq!(response.model, "test-model");
    println!("RetryMiddleware successfully handled 2 failures and returned a response.");

    // --- 2. LoggingMiddleware Showcase ---
    println!("\n--- 2. LoggingMiddleware Showcase ---");
    let logging_middleware = LoggingMiddleware::new()
        .with_request_body(true)
        .with_response_body(true)
        .with_timing(true)
        .with_usage(true);
    let mock_client_logging = MockApiClient::new(0); // Succeeds on the first try

    let _ = logging_middleware
        .execute("mock-provider", &request, || {
            mock_client_logging.make_request(&request)
        })
        .await?;
    println!("LoggingMiddleware captured and displayed request/response details.");

    // --- 3. MetricsMiddleware Showcase ---
    println!("\n--- 3. MetricsMiddleware Showcase ---");
    let metrics_middleware = MetricsMiddleware::new();
    let mock_client_metrics_1 = MockApiClient::new(0);
    let mock_client_metrics_2 = MockApiClient::new(1); // Fails once

    // First call (success)
    metrics_middleware
        .execute(|| mock_client_metrics_1.make_request(&request))
        .await?;
    // Second call (fail then success)
    let retry_for_metrics = RetryMiddleware::default();
    retry_for_metrics
        .execute(|| {
            metrics_middleware.execute(|| mock_client_metrics_2.make_request(&request))
        })
        .await?;

    let snapshot = metrics_middleware.snapshot();
    println!("MetricsMiddleware captured the following stats:");
    println!("{:#?}", snapshot);
    assert_eq!(snapshot.requests_total, 3); // 1 success + 1 failure + 1 success
    assert_eq!(snapshot.requests_failed, 1);

    // --- 4. InterceptorChain Showcase ---
    println!("\n--- 4. InterceptorChain Showcase ---");
    let interceptor_chain = InterceptorChain::new()
        .with_interceptor(Arc::new(
            ValidationInterceptor::new().with_max_tokens(1000),
        ))
        .with_interceptor(Arc::new(
            SanitizationInterceptor::new().with_remove_system_fingerprint(true),
        ));

    // This request will pass validation
    let valid_request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![],
        max_tokens: Some(500),
        ..Default::default()
    };
    // This request will fail validation
    let invalid_request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![],
        max_tokens: Some(2000),
        ..Default::default()
    };
    let mock_client_interceptor = MockApiClient::new(0);

    // Test successful validation and sanitization
    let client_clone = mock_client_interceptor.clone();
    let response = interceptor_chain
        .execute(valid_request, |req| async move {
            client_clone.make_request(&req).await
        })
        .await?;
    assert!(response.system_fingerprint.is_none());
    println!("InterceptorChain validated the request and sanitized the response.");

    // Test failed validation
    let client_clone = mock_client_interceptor.clone();
    let result = interceptor_chain
        .execute(invalid_request, |req| async move {
            client_clone.make_request(&req).await
        })
        .await;
    assert!(matches!(result, Err(LlmConnectorError::InvalidRequest(_))));
    println!("InterceptorChain correctly blocked an invalid request.");

    // --- 5. Composing Middleware Showcase ---
    println!("\n--- 5. Composing Middleware (Onion Model) ---");
    println!("In a real application, you would compose middleware by:");
    println!("1. Wrapping your API client with RetryMiddleware");
    println!("2. Adding LoggingMiddleware for observability");
    println!("3. Using MetricsMiddleware for monitoring");
    println!("4. Applying InterceptorChain for validation/sanitization");
    println!("\nEach middleware layer adds its functionality in an 'onion' pattern,");
    println!("where the request passes through each layer before reaching the API,");
    println!("and the response passes back through each layer in reverse order.");

    let final_snapshot = metrics_middleware.snapshot();
    println!("\nFinal metrics snapshot: {:#?}", final_snapshot);
    println!("\nMiddleware showcase completed successfully!");

    Ok(())
}

/// A simple custom interceptor for demonstration.
#[derive(Debug)]
#[allow(dead_code)]
struct CustomHeaderInterceptor {
    header: String,
}

#[async_trait]
impl Interceptor for CustomHeaderInterceptor {
    async fn before_request(&self, request: &mut ChatRequest) -> Result<(), LlmConnectorError> {
        println!(
            "CustomHeaderInterceptor: Adding header `{}` to request for model `{}`",
            self.header, request.model
        );
        // In a real scenario, you might modify request properties.
        // For this example, we just print.
        Ok(())
    }
}