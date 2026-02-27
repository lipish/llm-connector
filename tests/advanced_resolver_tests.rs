use async_trait::async_trait;
use llm_connector::core::{ServiceResolver, ServiceTarget};
use llm_connector::error::LlmConnectorError;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// ============================================================================
// 1. Exact Match Resolver (精确匹配配置表)
// ============================================================================
// 场景：精确控制每个模型去哪里，不依赖前缀，完全配置驱动。
// 比如：
// - "gpt-4" -> Azure US East
// - "gpt-4-32k" -> Azure Europe
// - "llama3" -> Localhost
struct ConfigTableResolver {
    // Map<ModelName, Target>
    table: HashMap<String, ServiceTarget>,
    // Fallback if not found
    fallback: Option<ServiceTarget>,
}

impl ConfigTableResolver {
    fn new() -> Self {
        Self {
            table: HashMap::new(),
            fallback: None,
        }
    }

    fn add_route(mut self, model: &str, target: ServiceTarget) -> Self {
        self.table.insert(model.to_string(), target);
        self
    }
}

#[async_trait]
impl ServiceResolver for ConfigTableResolver {
    async fn resolve(&self, model: &str) -> Result<ServiceTarget, LlmConnectorError> {
        if let Some(target) = self.table.get(model) {
            Ok(target.clone())
        } else if let Some(fallback) = &self.fallback {
            let mut t = fallback.clone();
            t.model = model.to_string(); // Keep original model name for fallback
            Ok(t)
        } else {
            // Default behavior: just pass through
            Ok(ServiceTarget::new(model))
        }
    }
}

// ============================================================================
// 2. Load Balancer Resolver (多Key轮询/负载均衡)
// ============================================================================
// 场景：我有 3 个 OpenAI Key，为了避免 Rate Limit，我想轮询使用。
// 业务代码只管请求 "gpt-4"，Resolver 自动分配 Key。
struct RoundRobinKeyResolver {
    keys: Vec<String>,
    counter: AtomicUsize,
}

impl RoundRobinKeyResolver {
    fn new(keys: Vec<String>) -> Self {
        Self {
            keys,
            counter: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl ServiceResolver for RoundRobinKeyResolver {
    async fn resolve(&self, model: &str) -> Result<ServiceTarget, LlmConnectorError> {
        let mut target = ServiceTarget::new(model);

        if !self.keys.is_empty() {
            // Simple Round-Robin
            let current = self.counter.fetch_add(1, Ordering::Relaxed);
            let key = &self.keys[current % self.keys.len()];
            target.api_key = Some(key.clone());
        }

        Ok(target)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_exact_match_routing() {
    let mut azure_target = ServiceTarget::new("gpt-4-turbo"); // Map to real model name
    azure_target.endpoint = Some("https://azure-us-east.com".to_string());
    azure_target.api_key = Some("azure-key".to_string());

    let mut local_target = ServiceTarget::new("llama3:8b");
    local_target.endpoint = Some("http://localhost:11434".to_string());

    let resolver = ConfigTableResolver::new()
        .add_route("gpt-4", azure_target)
        .add_route("local-chat", local_target);

    // Case 1: Request "gpt-4" -> Routed to Azure
    let t1 = resolver.resolve("gpt-4").await.unwrap();
    assert_eq!(t1.model, "gpt-4-turbo"); // Model rewrite
    assert_eq!(t1.endpoint.unwrap(), "https://azure-us-east.com");

    // Case 2: Request "local-chat" -> Routed to Ollama
    let t2 = resolver.resolve("local-chat").await.unwrap();
    assert_eq!(t2.model, "llama3:8b");
    assert_eq!(t2.endpoint.unwrap(), "http://localhost:11434");

    // Case 3: Unknown model -> Pass through
    let t3 = resolver.resolve("unknown").await.unwrap();
    assert_eq!(t3.model, "unknown");
    assert!(t3.endpoint.is_none());
}

#[tokio::test]
async fn test_load_balancing() {
    let resolver = RoundRobinKeyResolver::new(vec![
        "key-1".to_string(),
        "key-2".to_string(),
        "key-3".to_string(),
    ]);

    // Request 1 -> key-1
    let t1 = resolver.resolve("gpt-4").await.unwrap();
    assert_eq!(t1.api_key.unwrap(), "key-1");

    // Request 2 -> key-2
    let t2 = resolver.resolve("gpt-4").await.unwrap();
    assert_eq!(t2.api_key.unwrap(), "key-2");

    // Request 3 -> key-3
    let t3 = resolver.resolve("gpt-4").await.unwrap();
    assert_eq!(t3.api_key.unwrap(), "key-3");

    // Request 4 -> key-1 (Loop back)
    let t4 = resolver.resolve("gpt-4").await.unwrap();
    assert_eq!(t4.api_key.unwrap(), "key-1");
}
