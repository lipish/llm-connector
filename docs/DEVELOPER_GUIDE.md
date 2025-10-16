# Developer Guide: Adding New Providers

This guide shows you how to add new LLM providers to `llm-connector` using the recommended patterns.

## 🎯 **Quick Start: Adding a Standard Provider**

Most LLM providers can be added using the `GenericProvider` pattern. Here's how:

### **Step 1: Implement ProviderAdapter**

Create a new file in `src/providers/your_provider.rs`:

```rust
use crate::protocols::core::{ProviderAdapter, ErrorMapper};
use crate::types::{ChatRequest, ChatResponse, Role, Usage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct YourProviderAdapter {
    api_key: String,
}

impl YourProviderAdapter {
    pub fn new(api_key: &str) -> Self {
        Self { api_key: api_key.to_string() }
    }
}

#[async_trait]
impl ProviderAdapter for YourProviderAdapter {
    type RequestType = YourRequest;
    type ResponseType = YourResponse;
    type ErrorMapperType = YourErrorMapper;

    fn name(&self) -> &str {
        "your_provider"
    }

    fn endpoint_url(&self, base_url: &str) -> String {
        format!("{}/v1/chat/completions", base_url.trim_end_matches('/'))
    }

    fn build_request(&self, request: &ChatRequest) -> Result<Self::RequestType, LlmConnectorError> {
        // Transform generic request to provider-specific format
        Ok(YourRequest {
            model: request.model.clone(),
            messages: request.messages.iter().map(|msg| YourMessage {
                role: match msg.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::System => "system".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                content: msg.content.clone(),
            }).collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        })
    }

    fn parse_response(&self, response: &str) -> Result<ChatResponse, LlmConnectorError> {
        let your_response: YourResponse = serde_json::from_str(response)?;
        
        Ok(ChatResponse {
            id: your_response.id,
            object: "chat.completion".to_string(),
            created: your_response.created,
            model: your_response.model,
            choices: your_response.choices.into_iter().map(|choice| {
                crate::types::Choice {
                    index: choice.index,
                    message: crate::types::ResponseMessage {
                        role: Role::Assistant,
                        content: choice.message.content,
                        tool_calls: None,
                    },
                    finish_reason: choice.finish_reason,
                }
            }).collect(),
            usage: your_response.usage.map(|u| Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
                completion_tokens_details: None,
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: None,
                prompt_tokens_details: None,
            }),
            system_fingerprint: None,
        })
    }
}

// Define your request/response types
#[derive(Serialize)]
struct YourRequest {
    model: String,
    messages: Vec<YourMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

#[derive(Serialize)]
struct YourMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct YourResponse {
    id: String,
    created: u64,
    model: String,
    choices: Vec<YourChoice>,
    usage: Option<YourUsage>,
}

#[derive(Deserialize)]
struct YourChoice {
    index: u32,
    message: YourResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct YourResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct YourUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// Error mapper
pub struct YourErrorMapper;

impl ErrorMapper for YourErrorMapper {
    fn map_http_error(status: u16, body: serde_json::Value) -> LlmConnectorError {
        let message = body.get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");

        match status {
            400 => LlmConnectorError::InvalidRequest(format!("YourProvider: {}", message)),
            401 => LlmConnectorError::AuthenticationError(format!("YourProvider: {}", message)),
            429 => LlmConnectorError::RateLimitError(format!("YourProvider: {}", message)),
            _ => LlmConnectorError::ApiError(format!("YourProvider: {}", message)),
        }
    }

    fn map_network_error(error: reqwest::Error) -> LlmConnectorError {
        LlmConnectorError::NetworkError(format!("YourProvider: {}", error))
    }
}
```

### **Step 2: Create Provider Type and Functions**

Add to the same file:

```rust
use crate::protocols::core::GenericProvider;
use crate::config::ProviderConfig;

// Type alias for your provider
pub type YourProvider = GenericProvider<YourProviderAdapter>;

// Convenience function
pub fn your_provider(api_key: &str) -> Result<YourProvider, LlmConnectorError> {
    let adapter = YourProviderAdapter::new(api_key);
    let config = ProviderConfig::new(api_key)
        .with_base_url("https://api.yourprovider.com")
        .with_timeout_ms(30000);
    
    GenericProvider::new(config, adapter)
}
```

### **Step 3: Add to Client**

Add to `src/client.rs`:

```rust
impl LlmClient {
    /// Create client with YourProvider
    pub fn your_provider(api_key: &str) -> Self {
        let provider = crate::providers::your_provider::your_provider(api_key)
            .expect("Failed to create YourProvider");
        Self::from_provider(Arc::new(provider))
    }
}
```

### **Step 4: Export in Module**

Add to `src/providers/mod.rs`:

```rust
pub mod your_provider;
pub use your_provider::{YourProvider, your_provider};
```

## 🔧 **Advanced: Custom Provider Implementation**

For providers with special requirements (like Ollama's model management), implement the `Provider` trait directly:

```rust
use crate::core::Provider;
use async_trait::async_trait;

#[derive(Clone)]
pub struct CustomProvider {
    // Your fields
}

#[async_trait]
impl Provider for CustomProvider {
    fn name(&self) -> &str {
        "custom_provider"
    }

    async fn fetch_models(&self) -> Result<Vec<String>, LlmConnectorError> {
        // Custom model fetching logic
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmConnectorError> {
        // Custom chat logic
    }

    #[cfg(feature = "streaming")]
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream, LlmConnectorError> {
        // Custom streaming logic
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
```

## 📝 **Testing Your Provider**

Create tests in `tests/` directory:

```rust
#[tokio::test]
async fn test_your_provider() {
    let client = LlmClient::your_provider("test-key");
    
    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![Message::user("Hello")],
        ..Default::default()
    };

    // Test with mock server or real API
    let response = client.chat(&request).await;
    assert!(response.is_ok());
}
```

## 🎯 **Best Practices**

1. **Use GenericProvider** for standard APIs
2. **Implement proper error mapping** for better user experience
3. **Add comprehensive tests** for your provider
4. **Document API-specific features** in your module
5. **Follow naming conventions** (snake_case for functions, PascalCase for types)

This pattern has been successfully used for OpenAI, Anthropic, Aliyun, and other providers in the codebase.
