//! 主架构单元测试
//!
//! 这个文件包含主架构的全面单元测试。

use llm_connector::*;
use llm_connector::types::{ChatRequest, Message, Role};
use llm_connector::error::LlmConnectorError;
use llm_connector::providers::{validate_anthropic_key, validate_zhipu_key};

    #[test]
    fn test_protocol_creation() {
        // 测试OpenAI协议创建
        let openai_protocol = OpenAIProtocol::new("sk-test");
        assert_eq!(openai_protocol.name(), "openai");
        assert_eq!(openai_protocol.api_key(), "sk-test");
        assert_eq!(
            openai_protocol.chat_endpoint("https://api.openai.com"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert!(openai_protocol.models_endpoint("https://api.openai.com").is_some());

        // 测试Aliyun协议创建
        let aliyun_protocol = AliyunProtocol::new("sk-test");
        assert_eq!(aliyun_protocol.name(), "aliyun");
        assert_eq!(aliyun_protocol.api_key(), "sk-test");
        assert_eq!(
            aliyun_protocol.chat_endpoint("https://dashscope.aliyuncs.com"),
            "https://dashscope.aliyuncs.com/services/aigc/text-generation/generation"
        );

        // 测试Anthropic协议创建
        let anthropic_protocol = AnthropicProtocol::new("sk-ant-test");
        assert_eq!(anthropic_protocol.name(), "anthropic");
        assert_eq!(anthropic_protocol.api_key(), "sk-ant-test");
        assert_eq!(
            anthropic_protocol.chat_endpoint("https://api.anthropic.com"),
            "https://api.anthropic.com/v1/messages"
        );

        // 测试Zhipu协议创建
        let zhipu_protocol = ZhipuProtocol::new("test-key");
        assert_eq!(zhipu_protocol.name(), "zhipu");
        assert_eq!(zhipu_protocol.api_key(), "test-key");
        assert!(!zhipu_protocol.is_openai_compatible());

        let zhipu_openai = ZhipuProtocol::new_openai_compatible("test-key");
        assert!(zhipu_openai.is_openai_compatible());
    }

    #[test]
    fn test_provider_creation() {
        // 测试OpenAI提供商创建
        let openai_provider = openai("sk-test");
        assert!(openai_provider.is_ok());
        let provider = openai_provider.unwrap();
        assert_eq!(provider.protocol().name(), "openai");

        // 测试Aliyun提供商创建
        let aliyun_provider = aliyun("sk-test");
        assert!(aliyun_provider.is_ok());
        let provider = aliyun_provider.unwrap();
        assert_eq!(provider.protocol().name(), "aliyun");

        // 测试Anthropic提供商创建
        let anthropic_provider = anthropic("sk-ant-test");
        assert!(anthropic_provider.is_ok());
        let provider = anthropic_provider.unwrap();
        assert_eq!(provider.protocol().name(), "anthropic");

        // 测试Zhipu提供商创建
        let zhipu_provider = zhipu("test-key");
        assert!(zhipu_provider.is_ok());
        let provider = zhipu_provider.unwrap();
        assert_eq!(provider.protocol().name(), "zhipu");

        // 测试Ollama提供商创建
        let ollama_provider = ollama();
        assert!(ollama_provider.is_ok());
        let provider = ollama_provider.unwrap();
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn test_client_creation() {
        // 测试所有客户端创建方法
        assert!(LlmClient::openai("sk-test").is_ok());
        assert!(LlmClient::openai_with_base_url("sk-test", "https://api.deepseek.com").is_ok());
        assert!(LlmClient::azure_openai("test-key", "https://test.openai.azure.com", "2024-02-15-preview").is_ok());
        assert!(LlmClient::openai_compatible("sk-test", "https://api.deepseek.com", "deepseek").is_ok());
        
        assert!(LlmClient::aliyun("sk-test").is_ok());
        assert!(LlmClient::anthropic("sk-ant-test").is_ok());
        assert!(LlmClient::zhipu("test-key").is_ok());
        assert!(LlmClient::zhipu_openai_compatible("test-key").is_ok());
        assert!(LlmClient::ollama().is_ok());
        assert!(LlmClient::ollama_with_url("http://192.168.1.100:11434").is_ok());
    }

    #[test]
    fn test_client_provider_name() {
        let openai_client = LlmClient::openai("sk-test").unwrap();
        assert_eq!(openai_client.provider_name(), "openai");

        let aliyun_client = LlmClient::aliyun("sk-test").unwrap();
        assert_eq!(aliyun_client.provider_name(), "aliyun");

        let anthropic_client = LlmClient::anthropic("sk-ant-test").unwrap();
        assert_eq!(anthropic_client.provider_name(), "anthropic");

        let zhipu_client = LlmClient::zhipu("test-key").unwrap();
        assert_eq!(zhipu_client.provider_name(), "zhipu");

        let ollama_client = LlmClient::ollama().unwrap();
        assert_eq!(ollama_client.provider_name(), "ollama");
    }

    #[test]
    fn test_ollama_special_access() {
        let ollama_client = LlmClient::ollama().unwrap();
        assert!(ollama_client.as_ollama().is_some());

        let openai_client = LlmClient::openai("sk-test").unwrap();
        assert!(openai_client.as_ollama().is_none());
    }

    #[test]
    fn test_client_cloning() {
        let client = LlmClient::openai("sk-test").unwrap();
        let cloned = client.clone();
        
        assert_eq!(client.provider_name(), cloned.provider_name());
    }

    #[test]
    fn test_validation_functions() {
        // 测试Anthropic密钥验证
        assert!(validate_anthropic_key("sk-ant-api03-test"));
        assert!(validate_anthropic_key("sk-ant-test"));
        assert!(!validate_anthropic_key("sk-test"));
        assert!(!validate_anthropic_key(""));

        // 测试Zhipu密钥验证
        assert!(validate_zhipu_key("valid-test-key"));
        assert!(!validate_zhipu_key("short"));
        assert!(!validate_zhipu_key(""));
    }

    #[test]
    fn test_request_building() {
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![
                Message {
                    role: Role::System,
                    content: "You are a helpful assistant.".to_string(),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    reasoning_content: None,
                    reasoning: None,
                    thought: None,
                    thinking: None,
                },
                Message {
                    role: Role::User,
                    content: "Hello!".to_string(),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    reasoning_content: None,
                    reasoning: None,
                    thought: None,
                    thinking: None,
                },
            ],
            temperature: Some(0.7),
            max_tokens: Some(100),
            ..Default::default()
        };

        // 测试OpenAI协议请求构建
        let openai_protocol = OpenAIProtocol::new("sk-test");
        let openai_request = openai_protocol.build_request(&request);
        assert!(openai_request.is_ok());

        // 测试Aliyun协议请求构建
        let aliyun_protocol = AliyunProtocol::new("sk-test");
        let aliyun_request = aliyun_protocol.build_request(&request);
        assert!(aliyun_request.is_ok());

        // 测试Anthropic协议请求构建
        let anthropic_protocol = AnthropicProtocol::new("sk-ant-test");
        let anthropic_request = anthropic_protocol.build_request(&request);
        assert!(anthropic_request.is_ok());

        // 测试Zhipu协议请求构建
        let zhipu_protocol = ZhipuProtocol::new("test-key");
        let zhipu_request = zhipu_protocol.build_request(&request);
        assert!(zhipu_request.is_ok());
    }

    #[test]
    fn test_auth_headers() {
        // 测试OpenAI认证头
        let openai_protocol = OpenAIProtocol::new("sk-test");
        let headers = openai_protocol.auth_headers();
        assert!(!headers.is_empty());
        assert!(headers.iter().any(|(k, v)| k == "Authorization" && v == "Bearer sk-test"));

        // 测试Anthropic认证头
        let anthropic_protocol = AnthropicProtocol::new("sk-ant-test");
        let headers = anthropic_protocol.auth_headers();
        assert!(!headers.is_empty());
        assert!(headers.iter().any(|(k, v)| k == "x-api-key" && v == "sk-ant-test"));

        // 测试Aliyun认证头
        let aliyun_protocol = AliyunProtocol::new("sk-test");
        let headers = aliyun_protocol.auth_headers();
        assert!(!headers.is_empty());
        assert!(headers.iter().any(|(k, v)| k == "Authorization" && v == "Bearer sk-test"));
    }

    #[test]
    fn test_error_mapping() {
        let openai_protocol = OpenAIProtocol::new("sk-test");
        
        // 测试不同HTTP状态码的错误映射
        let auth_error = openai_protocol.map_error(401, "Unauthorized");
        assert!(matches!(auth_error, LlmConnectorError::AuthenticationError(_)));

        let rate_limit_error = openai_protocol.map_error(429, "Rate limit exceeded");
        assert!(matches!(rate_limit_error, LlmConnectorError::RateLimitError(_)));

        let server_error = openai_protocol.map_error(500, "Internal server error");
        assert!(matches!(server_error, LlmConnectorError::ServerError(_)));
    }

    #[test]
    fn test_http_client_creation() {
        // 测试基础HTTP客户端创建
        let client = HttpClient::new("https://api.openai.com");
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.base_url(), "https://api.openai.com");

        // 测试带配置的HTTP客户端创建
        let client = HttpClient::with_config("https://api.openai.com", Some(60), None);
        assert!(client.is_ok());

        // 测试带头部的HTTP客户端
        let mut headers = std::collections::HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer test".to_string());
        let client = HttpClient::new("https://api.openai.com").unwrap()
            .with_headers(headers);
        assert_eq!(client.base_url(), "https://api.openai.com");
    }

    #[test]
    fn test_performance_characteristics() {
        use std::time::Instant;

        // 测试客户端创建性能
        let start = Instant::now();
        for _ in 0..10 {
            let _client = LlmClient::openai("sk-test").unwrap();
        }
        let creation_time = start.elapsed();

        // 10个客户端创建应该在合理时间内完成
        assert!(creation_time.as_millis() < 1000);

        // 测试克隆性能
        let client = LlmClient::openai("sk-test").unwrap();
        let start = Instant::now();
        for _ in 0..100 {
            let _cloned = client.clone();
        }
        let clone_time = start.elapsed();

        // 100次克隆应该在合理时间内完成
        assert!(clone_time.as_millis() < 100);

        // 验证性能比V1更好 (这里只是验证能正常运行)
        println!("V2 客户端创建时间: {:?}", creation_time);
        println!("V2 克隆时间: {:?}", clone_time);
    }

    #[test]
    fn test_memory_usage() {
        let client = LlmClient::openai("sk-test").unwrap();
        let size = std::mem::size_of_val(&client);
        
        // 客户端应该很小 (小于100字节)
        assert!(size < 100);
    }
