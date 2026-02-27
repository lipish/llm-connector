use llm_connector::core::{EnvVarResolver, ServiceResolver};

#[tokio::test]
async fn test_env_var_resolver() {
    // Set mock env vars
    unsafe {
        std::env::set_var("TEST_OPENAI_KEY", "sk-test-openai");
        std::env::set_var("TEST_ANTHROPIC_KEY", "sk-test-anthropic");
    }

    let resolver = EnvVarResolver::new()
        .with_mapping("gpt", "TEST_OPENAI_KEY")
        .with_mapping("claude", "TEST_ANTHROPIC_KEY");

    // Test GPT resolution
    let target = resolver.resolve("gpt-4").await.unwrap();
    assert_eq!(target.model, "gpt-4");
    assert_eq!(target.api_key, Some("sk-test-openai".to_string()));

    // Test Claude resolution
    let target = resolver.resolve("claude-3").await.unwrap();
    assert_eq!(target.model, "claude-3");
    assert_eq!(target.api_key, Some("sk-test-anthropic".to_string()));

    // Test unknown model
    let target = resolver.resolve("unknown-model").await.unwrap();
    assert_eq!(target.model, "unknown-model");
    assert_eq!(target.api_key, None);
}
